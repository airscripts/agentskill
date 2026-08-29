use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use agentskill_analyzers::run_evidence;
use agentskill_core::document::{normalize_section_name, parse};
use agentskill_core::error::{AgentskillError, Result, validate_repo};
use serde_json::{Value, json};

const OPERATIONAL_TOKEN_TARGET: usize = 1_000;
const OPERATIONAL_TOKEN_HARD_LIMIT: usize = 1_500;

/// Validates the repository's LLM-authored AGENTS documents without writing.
pub fn validate(repo: &str) -> Result<Value> {
    let root = validate_repo(repo)?;
    let operational = root.join("AGENTS.md");
    let reference = root.join("AGENTS.reference.md");
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !operational.is_file() {
        errors.push("AGENTS.md not found".to_string());
    } else {
        validate_markdown(&root, &operational, &mut errors, &mut warnings)?;
        let content = read_document(&operational)?;
        let tokens = approximate_tokens(&content);
        if tokens > OPERATIONAL_TOKEN_HARD_LIMIT {
            errors.push(format!(
                "AGENTS.md is approximately {tokens} tokens; hard limit is {OPERATIONAL_TOKEN_HARD_LIMIT}"
            ));
        } else if tokens > OPERATIONAL_TOKEN_TARGET {
            warnings.push(format!(
                "AGENTS.md is approximately {tokens} tokens; target is {OPERATIONAL_TOKEN_TARGET}"
            ));
        }
        if content.contains("AGENTS.reference.md") && !reference.is_file() {
            errors.push("AGENTS.md references missing AGENTS.reference.md".into());
        }
    }

    if reference.is_file() {
        validate_markdown(&root, &reference, &mut errors, &mut warnings)?;
    }

    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "warnings": warnings,
        "files": {
            "operational": operational.strip_prefix(&root).unwrap_or(&operational),
            "reference": reference.is_file().then(|| reference.strip_prefix(&root).unwrap_or(&reference)),
        }
    }))
}

/// Reports broken local references and the current evidence revision without writing.
pub fn drift(repo: &str) -> Result<Value> {
    let root = validate_repo(repo)?;
    let operational = root.join("AGENTS.md");
    if !operational.is_file() {
        return Err(AgentskillError::InvalidPath("AGENTS.md not found".into()));
    }

    let evidence = run_evidence(root.to_string_lossy().as_ref(), None)?;
    let mut issues = Vec::new();
    let mut referenced = BTreeSet::new();
    let documents = [operational, root.join("AGENTS.reference.md")];
    for document in documents.iter().filter(|path| path.is_file()) {
        for path in referenced_paths(&read_document(document)?) {
            referenced.insert(path.clone());
            if path != "." && !root.join(&path).exists() {
                issues.push(json!({
                    "kind": "missing_path",
                    "document": display_path(&root, document),
                    "path": path
                }));
            }
        }
    }

    Ok(json!({
        "stale": !issues.is_empty(),
        "issues": issues,
        "repository_revision": evidence["repository"]["revision"],
        "referenced_paths": referenced,
    }))
}

fn validate_markdown(
    root: &Path,
    path: &Path,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let content = read_document(path)?;
    let document = parse(&content);
    let mut headings = BTreeSet::new();
    for section in document.sections {
        let key = normalize_section_name(&section.heading);
        if !headings.insert(key.clone()) {
            errors.push(format!(
                "{}: duplicate heading {key:?}",
                display_path(root, path).display()
            ));
        }
    }
    for referenced in referenced_paths(&content) {
        if referenced != "." && !root.join(&referenced).exists() {
            errors.push(format!(
                "{}: referenced path does not exist: {referenced}",
                display_path(root, path).display()
            ));
        }
    }
    if !content.ends_with('\n') {
        warnings.push(format!(
            "{}: missing trailing newline",
            display_path(root, path).display()
        ));
    }
    Ok(())
}

fn read_document(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

fn referenced_paths(content: &str) -> Vec<String> {
    content
        .split('`')
        .enumerate()
        .filter_map(|(index, value)| (index % 2 == 1).then_some(value))
        .flat_map(str::split_whitespace)
        .map(|value| value.trim_matches(|character: char| ",;:()[]{}<>".contains(character)))
        .filter(|value| {
            value.contains('/')
                || value.ends_with(".rs")
                || value.ends_with(".toml")
                || value.ends_with(".yml")
                || value.ends_with(".yaml")
                || value.ends_with(".md")
        })
        .map(|value| value.trim_start_matches("./").to_string())
        .filter(|value| !value.starts_with("http://") && !value.starts_with("https://"))
        .collect()
}

fn approximate_tokens(content: &str) -> usize {
    content.split_whitespace().count() * 4 / 3
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}
