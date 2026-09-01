use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use agentskill_analyzers::run_evidence;
use agentskill_core::config::{RepositoryConfig, SignatureMode, load as load_config};
use agentskill_core::document::{normalize_section_name, parse};
use agentskill_core::error::{AgentskillError, Result, validate_repo};
use serde_json::{Value, json};

mod signature;

pub use signature::reconcile_signature;

const OPERATIONAL_TOKEN_TARGET: usize = 1_000;
const OPERATIONAL_TOKEN_HARD_LIMIT: usize = 1_500;
const PROVENANCE_FIELDS: &[(&str, &str, &str, &str)] = &[
    (
        "agentskill version:",
        "missing_provenance_version",
        "agentskill.version",
        "Agentskill version is missing from provenance",
    ),
    (
        "evidence schema version:",
        "missing_provenance_schema",
        "evidence.schema_version",
        "evidence schema version is missing from provenance",
    ),
    (
        "repository revision:",
        "missing_provenance_revision",
        "repository.revision",
        "repository revision is missing from provenance",
    ),
    (
        "configuration:",
        "missing_provenance_configuration",
        "configuration.signature",
        "configuration source is missing from provenance",
    ),
    (
        "maintainer-confirmed decisions:",
        "missing_provenance_decisions",
        "maintainer.decisions",
        "maintainer-confirmed decisions are missing from provenance",
    ),
    (
        "unresolved uncertainty:",
        "missing_provenance_uncertainty",
        "repository.uncertainty",
        "unresolved uncertainty is missing from provenance",
    ),
];

const FACT_PREFIXES: &[&str] = &[
    "architecture.",
    "configuration.",
    "language.",
    "test.",
    "tool.",
];

/// Validates the repository's LLM-authored AGENTS documents without writing.
pub fn validate(repo: &str) -> Result<Value> {
    validate_with_mode(repo, SignatureMode::Auto)
}

/// Validates documents using an ephemeral signature override.
pub fn validate_with_mode(repo: &str, mode: SignatureMode) -> Result<Value> {
    validate_with_mode_and_scopes(repo, mode, None)
}

/// Validates root and selected managed scoped documents.
pub fn validate_with_mode_and_scopes(
    repo: &str,
    mode: SignatureMode,
    selected: Option<&[String]>,
) -> Result<Value> {
    let mut report = validate_root_with_mode(repo, mode)?;
    append_scoped_validation(&mut report, repo, mode, selected)?;
    Ok(report)
}

fn validate_root_with_mode(repo: &str, mode: SignatureMode) -> Result<Value> {
    let root = validate_repo(repo)?;
    let configuration = load_config(&root);
    let operational = root.join("AGENTS.md");
    let reference = root.join("AGENTS.reference.md");
    let evidence = reference
        .is_file()
        .then(|| run_evidence(root.to_string_lossy().as_ref(), None))
        .transpose()?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut findings = Vec::new();

    if !configuration.valid {
        let message = configuration
            .error
            .as_deref()
            .unwrap_or("invalid agentskill.toml");
        errors.push(format!("agentskill.toml: {message}"));
        findings.push(finding(
            "invalid_configuration",
            "error",
            "agentskill.toml",
            message,
            None,
        ));
    }

    let signature_enabled = configuration.resolved_signature(mode);

    if !operational.is_file() {
        errors.push("AGENTS.md not found".to_string());
        findings.push(finding(
            "missing_document",
            "error",
            "AGENTS.md",
            "AGENTS.md not found",
            None,
        ));
    } else {
        validate_markdown(
            &root,
            &operational,
            true,
            signature_enabled,
            &mut errors,
            &mut warnings,
            &mut findings,
        )?;

        let content = read_document(&operational)?;
        let tokens = approximate_tokens(&content);

        if tokens > OPERATIONAL_TOKEN_HARD_LIMIT {
            let message = format!(
                "AGENTS.md is approximately {tokens} tokens; hard limit is {OPERATIONAL_TOKEN_HARD_LIMIT}"
            );

            errors.push(message.clone());
            findings.push(finding(
                "operational_token_limit",
                "error",
                "AGENTS.md",
                &message,
                None,
            ));
        } else if tokens > OPERATIONAL_TOKEN_TARGET {
            let message = format!(
                "AGENTS.md is approximately {tokens} tokens; target is {OPERATIONAL_TOKEN_TARGET}"
            );

            warnings.push(message.clone());
            findings.push(finding(
                "operational_token_target",
                "warning",
                "AGENTS.md",
                &message,
                None,
            ));
        }

        if content.contains("AGENTS.reference.md") && !reference.is_file() {
            let message = "AGENTS.md references missing AGENTS.reference.md";
            errors.push(message.into());

            findings.push(finding(
                "missing_reference_document",
                "error",
                "AGENTS.md",
                message,
                Some("AGENTS.reference.md"),
            ));
        }
    }

    if reference.is_file() {
        validate_markdown(
            &root,
            &reference,
            false,
            signature_enabled,
            &mut errors,
            &mut warnings,
            &mut findings,
        )?;

        validate_provenance(&root, &reference, &mut errors, &mut findings)?;
    }

    for path in [&operational, &reference]
        .into_iter()
        .filter(|path| path.is_file())
    {
        let content = read_document(path)?;
        let display = display_path(&root, path);

        if let Some(evidence) = evidence.as_ref() {
            for issue in guidance_issues(&content, evidence) {
                let message = format!("{}: {}", display.display(), issue.message);
                match issue.severity {
                    "error" => errors.push(message),
                    _ => warnings.push(message),
                }

                findings.push(finding(
                    issue.kind,
                    issue.severity,
                    &display.to_string_lossy(),
                    &issue.message,
                    Some(&issue.fact),
                ));
            }
        }

        for issue in command_issues(&root, &content) {
            let message = format!("{}: {}", display.display(), issue.message);
            warnings.push(message);

            findings.push(finding(
                issue.kind,
                issue.severity,
                &display.to_string_lossy(),
                &issue.message,
                Some(&issue.fact),
            ));
        }
    }

    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "warnings": warnings,
        "findings": findings,
        "configuration": configuration_value(&configuration, mode),
        "files": {
            "operational": operational.strip_prefix(&root).unwrap_or(&operational),
            "reference": reference.is_file().then(|| reference.strip_prefix(&root).unwrap_or(&reference)),
        }
    }))
}

/// Reports broken local references and the current evidence revision without writing.
pub fn drift(repo: &str) -> Result<Value> {
    drift_with_mode(repo, SignatureMode::Auto)
}

/// Reports drift using an ephemeral signature override.
pub fn drift_with_mode(repo: &str, mode: SignatureMode) -> Result<Value> {
    drift_with_mode_and_scopes(repo, mode, None)
}

/// Reports drift for root and selected managed scoped documents.
pub fn drift_with_mode_and_scopes(
    repo: &str,
    mode: SignatureMode,
    selected: Option<&[String]>,
) -> Result<Value> {
    let mut report = drift_root_with_mode(repo, mode)?;
    append_scoped_drift(&mut report, repo, mode, selected)?;
    Ok(report)
}

fn drift_root_with_mode(repo: &str, mode: SignatureMode) -> Result<Value> {
    let root = validate_repo(repo)?;
    let configuration = load_config(&root);

    if !configuration.valid {
        return Err(AgentskillError::InvalidArgument(
            configuration
                .error
                .unwrap_or_else(|| "invalid agentskill.toml".into()),
        ));
    }

    let operational = root.join("AGENTS.md");
    if !operational.is_file() {
        return Err(AgentskillError::InvalidPath("AGENTS.md not found".into()));
    }

    let evidence = run_evidence(root.to_string_lossy().as_ref(), None)?;

    let mut issues = Vec::new();
    let mut referenced = BTreeSet::new();
    let signature_enabled = configuration.resolved_signature(mode);

    let documents = [operational, root.join("AGENTS.reference.md")];
    for document in documents.iter().filter(|path| path.is_file()) {
        let content = read_document(document)?;
        let display = display_path(&root, document);

        for issue in guidance_issues(&content, &evidence) {
            issues.push(json!({
                "kind": issue.kind,
                "severity": issue.severity,
                "document": display,
                "fact": issue.fact,
                "message": issue.message,
            }));
        }

        for issue in command_issues(&root, &content) {
            issues.push(json!({
                "kind": issue.kind,
                "severity": issue.severity,
                "document": display,
                "command": issue.fact,
                "message": issue.message,
            }));
        }

        for issue in signature::issues(&content, signature_enabled) {
            issues.push(json!({
                "kind": issue.kind(),
                "severity": "error",
                "document": display,
                "message": issue.message(),
            }));
        }

        for path in referenced_paths(&content) {
            referenced.insert(path.clone());
            if path != "." && !is_optional_reference(&root, &path) && !root.join(&path).exists() {
                issues.push(json!({
                    "kind": "missing_path",
                    "severity": "error",
                    "document": display_path(&root, document),
                    "path": path,
                    "message": "referenced path does not exist",
                }));
            }
        }

        if document.file_name().and_then(|name| name.to_str()) == Some("AGENTS.reference.md") {
            for issue in provenance_issues(&content) {
                issues.push(json!({
                    "kind": issue.kind,
                    "severity": "error",
                    "document": display,
                    "fact": issue.fact,
                    "message": issue.message,
                }));
            }
        }
    }

    let reference = root.join("AGENTS.reference.md");
    let mut reference_version = None;
    let mut reference_revision = None;

    if reference.is_file() {
        let reference_content = read_document(&reference)?;
        reference_version = provenance_field(&reference_content, "agentskill version:");
        reference_revision = provenance_field(&reference_content, "repository revision:");

        if let Some(version) = reference_version.as_deref()
            && Some(version) != evidence["agentskill_version"].as_str()
        {
            issues.push(json!({
                "kind": "stale_version",
                "severity": "warning",
                "document": "AGENTS.reference.md",
                "fact": "agentskill.version",
                "message": format!(
                    "reference was generated with Agentskill version {version}, current version is {}",
                    evidence["agentskill_version"].as_str().unwrap_or("unknown")
                ),
            }));
        }

        if let Some(revision) = reference_revision.as_deref()
            && let Some(current) = evidence["repository"]["revision"].as_str()
            && revision != current
        {
            issues.push(json!({
                "kind": "changed_revision",
                "severity": "info",
                "document": "AGENTS.reference.md",
                "fact": "repository.revision",
                "message": format!(concat!(
                    "reference revision {} differs from current repository revision; ",
                    "refresh provenance when guidance changes"
                ), revision),
            }));
        }
    }

    let stale = issues.iter().any(|issue| issue["severity"] != "info");
    let revision_changed = reference_revision
        .as_deref()
        .zip(evidence["repository"]["revision"].as_str())
        .is_some_and(|(reference, current)| reference != current);

    Ok(json!({
        "analysis_complete": true,
        "stale": stale,
        "issues": issues,
        "configuration": configuration_value(&configuration, mode),
        "agentskill_version": evidence["agentskill_version"],
        "repository_revision": evidence["repository"]["revision"],
        "reference_version": reference_version,
        "reference_revision": reference_revision,
        "revision_changed": revision_changed,
        "referenced_paths": referenced,
    }))
}

fn append_scoped_validation(
    report: &mut Value,
    repo: &str,
    mode: SignatureMode,
    selected: Option<&[String]>,
) -> Result<()> {
    let root = validate_repo(repo)?;
    let scopes = agentskill_analyzers::run_scopes(repo, selected)?;
    let all_scopes = selected
        .is_some()
        .then(|| agentskill_analyzers::run_scopes(repo, None))
        .transpose()?
        .unwrap_or_else(|| scopes.clone());

    report["scopes"] = scopes["scopes"].clone();

    let mut errors = report["errors"]
        .take()
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();

    let mut warnings = report["warnings"]
        .take()
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();

    let mut findings = report["findings"]
        .take()
        .as_array()
        .cloned()
        .unwrap_or_default();

    let configured_signature = load_config(&root).resolved_signature(mode);
    let mut validated = Vec::new();

    for scope in scopes["scopes"].as_array().into_iter().flatten() {
        let path = scope["path"].as_str().unwrap_or(".");
        if path == "." {
            continue;
        }

        let scope_root = root.join(path);
        let operational = scope_root.join("AGENTS.md");
        let reference = scope_root.join("AGENTS.reference.md");
        if !operational.is_file() {
            findings.push(finding(
                "missing_scoped_document",
                "info",
                path,
                "scope candidate has no AGENTS.md; creation is explicit",
                Some(path),
            ));
            continue;
        }

        let content = read_document(&operational)?;
        let canonical = has_scope_metadata(&content);
        if !canonical {
            warnings.push(format!(
                "{path}/AGENTS.md is a legacy scoped document and needs review"
            ));

            findings.push(finding(
                "legacy_scoped_document",
                "warning",
                path,
                "scoped document lacks managed ## Scope metadata; review before adoption",
                Some(path),
            ));

            continue;
        }

        let parent = scope["parent"].as_str().unwrap_or(".");
        for issue in scope_metadata_issues(&content, path, parent) {
            errors.push(format!("{path}/AGENTS.md: {issue}"));
            findings.push(finding(
                "invalid_scope_metadata",
                "error",
                path,
                &issue,
                Some(path),
            ));
        }

        validate_markdown(
            &scope_root,
            &operational,
            true,
            configured_signature,
            &mut errors,
            &mut warnings,
            &mut findings,
        )?;

        let tokens = approximate_tokens(&content);
        if tokens > OPERATIONAL_TOKEN_HARD_LIMIT {
            let message = format!(
                "{path}/AGENTS.md is approximately {tokens} tokens; hard limit is {OPERATIONAL_TOKEN_HARD_LIMIT}"
            );

            errors.push(message.clone());
            findings.push(finding(
                "operational_token_limit",
                "error",
                path,
                &message,
                None,
            ));
        } else if tokens > OPERATIONAL_TOKEN_TARGET {
            let message = format!(
                "{path}/AGENTS.md is approximately {tokens} tokens; target is {OPERATIONAL_TOKEN_TARGET}"
            );

            warnings.push(message.clone());
            findings.push(finding(
                "operational_token_target",
                "warning",
                path,
                &message,
                None,
            ));
        }

        if content.contains("AGENTS.reference.md") && !reference.is_file() {
            let message = format!("{path}/AGENTS.md references missing AGENTS.reference.md");
            errors.push(message.clone());

            findings.push(finding(
                "missing_reference_document",
                "error",
                path,
                &message,
                Some("AGENTS.reference.md"),
            ));
        }

        if reference.is_file() {
            validate_markdown(
                &scope_root,
                &reference,
                false,
                configured_signature,
                &mut errors,
                &mut warnings,
                &mut findings,
            )?;
            validate_provenance(&scope_root, &reference, &mut errors, &mut findings)?;
        }

        if reference.is_file() {
            let evidence = agentskill_analyzers::run_evidence_scoped(
                repo,
                None,
                Some(&[path.to_string()]),
                Some("standard"),
            )?;

            for document in [operational, reference]
                .into_iter()
                .filter(|document| document.is_file())
            {
                let content = read_document(&document)?;
                let display = display_path(&root, &document);
                for issue in guidance_issues(&content, &evidence) {
                    let message = format!("{}: {}", display.display(), issue.message);
                    match issue.severity {
                        "error" => errors.push(message),
                        _ => warnings.push(message),
                    }

                    findings.push(finding(
                        issue.kind,
                        issue.severity,
                        &display.to_string_lossy(),
                        &issue.message,
                        Some(&issue.fact),
                    ));
                }

                for issue in command_issues(&scope_root, &content) {
                    let message = format!("{}: {}", display.display(), issue.message);
                    warnings.push(message);
                    findings.push(finding(
                        issue.kind,
                        issue.severity,
                        &display.to_string_lossy(),
                        &issue.message,
                        Some(&issue.fact),
                    ));
                }
            }
        }

        validated.push(path);
    }

    append_scope_relationship_findings(
        &root,
        &scopes["scopes"],
        &all_scopes["scopes"],
        &mut warnings,
        &mut findings,
    )?;

    report["errors"] = json!(errors);
    report["warnings"] = json!(warnings);
    report["findings"] = json!(findings);
    report["validated_scopes"] = json!(validated);
    report["valid"] = json!(errors.is_empty());
    Ok(())
}

fn append_scoped_drift(
    report: &mut Value,
    repo: &str,
    mode: SignatureMode,
    selected: Option<&[String]>,
) -> Result<()> {
    let root = validate_repo(repo)?;
    let scopes = agentskill_analyzers::run_scopes(repo, selected)?;
    let all_scopes = selected
        .is_some()
        .then(|| agentskill_analyzers::run_scopes(repo, None))
        .transpose()?
        .unwrap_or_else(|| scopes.clone());

    report["scopes"] = scopes["scopes"].clone();
    let mut issues = report["issues"]
        .take()
        .as_array()
        .cloned()
        .unwrap_or_default();

    let signature_enabled = load_config(&root).resolved_signature(mode);
    let current_version = report["agentskill_version"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let current_revision = report["repository_revision"].as_str().map(str::to_string);
    let mut managed = Vec::new();

    for scope in scopes["scopes"].as_array().into_iter().flatten() {
        let path = scope["path"].as_str().unwrap_or(".");
        if path == "." {
            continue;
        }
        let scope_root = root.join(path);
        let operational = scope_root.join("AGENTS.md");
        let reference = scope_root.join("AGENTS.reference.md");
        if !operational.is_file() {
            issues.push(json!({
                "kind": "missing_scoped_document",
                "severity": "info",
                "document": path,
                "path": path,
                "message": "scope candidate has no AGENTS.md; creation is explicit",
            }));
            continue;
        }

        let operational_content = read_document(&operational)?;
        if !has_scope_metadata(&operational_content) {
            issues.push(json!({
                "kind": "legacy_scoped_document",
                "severity": "warning",
                "document": path,
                "path": path,
                "message": "scoped document lacks managed ## Scope metadata; review before adoption",
            }));
            continue;
        }

        let evidence = agentskill_analyzers::run_evidence_scoped(
            repo,
            None,
            Some(&[path.to_string()]),
            Some("standard"),
        )?;
        for document in [operational, reference]
            .into_iter()
            .filter(|document| document.is_file())
        {
            let content = read_document(&document)?;
            let display = display_path(&root, &document);
            for issue in guidance_issues(&content, &evidence) {
                issues.push(json!({
                    "kind": issue.kind,
                    "severity": issue.severity,
                    "document": display,
                    "fact": issue.fact,
                    "message": issue.message,
                }));
            }

            for issue in command_issues(&scope_root, &content) {
                issues.push(json!({
                    "kind": issue.kind,
                    "severity": issue.severity,
                    "document": display,
                    "command": issue.fact,
                    "message": issue.message,
                }));
            }

            for issue in signature::issues(&content, signature_enabled) {
                issues.push(json!({
                    "kind": issue.kind(),
                    "severity": "error",
                    "document": display,
                    "message": issue.message(),
                }));
            }

            if document.file_name().and_then(|name| name.to_str()) == Some("AGENTS.reference.md") {
                let version = provenance_field(&content, "agentskill version:");
                if version.as_deref() != Some(current_version.as_str()) {
                    issues.push(json!({
                        "kind": "stale_version",
                        "severity": "warning",
                        "document": display,
                        "fact": "agentskill.version",
                        "message": format!("reference was generated with Agentskill version {}, current version is {current_version}", version.unwrap_or_else(|| "unknown".into())),
                    }));
                }

                if let Some(revision) = provenance_field(&content, "repository revision:")
                    && Some(revision.as_str()) != current_revision.as_deref()
                {
                    issues.push(json!({
                        "kind": "changed_revision",
                        "severity": "info",
                        "document": display,
                        "fact": "repository.revision",
                        "message": "reference revision differs from current repository revision; refresh provenance when guidance changes",
                    }));
                }
            }
        }

        managed.push(path);
    }

    append_scope_drift_relationships(&root, &scopes["scopes"], &all_scopes["scopes"], &mut issues)?;

    let stale = issues.iter().any(|issue| issue["severity"] != "info");
    report["issues"] = json!(issues);
    report["managed_scopes"] = json!(managed);
    report["stale"] = json!(stale);
    Ok(())
}

fn has_scope_metadata(content: &str) -> bool {
    parse(content)
        .sections
        .iter()
        .any(|section| section.level == 2 && normalize_section_name(&section.heading) == "scope")
}

fn scope_metadata_issues(content: &str, path: &str, parent: &str) -> Vec<String> {
    let body = parse(content)
        .sections
        .into_iter()
        .find(|section| section.level == 2 && normalize_section_name(&section.heading) == "scope")
        .map(|section| section.body)
        .unwrap_or_default();

    let mut issues = Vec::new();
    if scope_metadata_value(&body, "path").as_deref() != Some(path) {
        issues.push(format!("scope metadata must declare path: {path}"));
    }

    if scope_metadata_value(&body, "parent").as_deref() != Some(parent) {
        issues.push(format!("scope metadata must declare parent: {parent}"));
    }

    if scope_metadata_value(&body, "inheritance")
        .is_none_or(|value| !value.to_ascii_lowercase().contains("additive"))
    {
        issues.push("scope metadata must declare additive inheritance".into());
    }
    issues
}

fn scope_metadata_value(body: &str, key: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let line = line.trim().trim_start_matches('-').trim();
        let (line_key, value) = line.split_once(':')?;
        (line_key.trim().eq_ignore_ascii_case(key) && !value.trim().is_empty())
            .then(|| value.trim().to_string())
    })
}

fn append_scope_relationship_findings(
    root: &Path,
    scopes: &Value,
    all_scopes: &Value,
    warnings: &mut Vec<String>,
    findings: &mut Vec<Value>,
) -> Result<()> {
    for scope in scopes.as_array().into_iter().flatten() {
        let path = scope["path"].as_str().unwrap_or(".");
        if path == "." || scope["status"] != "managed" {
            continue;
        }

        let local_path = root.join(path).join("AGENTS.md");
        let local_rules = managed_rules(&read_document(&local_path)?);
        let Some(ancestor) = scope["resolution"]["ancestors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .find(|ancestor| {
                if *ancestor == "." {
                    root.join("AGENTS.md").is_file()
                } else {
                    all_scopes
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|candidate| {
                            candidate["path"] == *ancestor
                                && candidate["status"] == "managed"
                                && root.join(ancestor).join("AGENTS.md").is_file()
                        })
                }
            })
        else {
            continue;
        };

        let ancestor_path = if ancestor == "." {
            root.join("AGENTS.md")
        } else {
            root.join(ancestor).join("AGENTS.md")
        };

        let ancestor_rules = managed_rules(&read_document(&ancestor_path)?);
        for rule in local_rules.intersection(&ancestor_rules) {
            let mut issue = finding(
                "duplicated_inherited_rule",
                "info",
                path,
                "managed rule duplicates an inherited rule; keep the rule at the nearest owner",
                None,
            );

            issue["ancestor"] = json!(ancestor);
            issue["rule"] = json!(rule);
            findings.push(issue);
        }
        for local_rule in &local_rules {
            for ancestor_rule in &ancestor_rules {
                if contradictory_rules(local_rule, ancestor_rule) {
                    let message =
                        format!("managed rule conflicts with inherited guidance from {ancestor}");

                    warnings.push(format!("{path}/AGENTS.md: {message}"));
                    let mut issue = finding(
                        "conflicting_inherited_rule",
                        "warning",
                        path,
                        &message,
                        None,
                    );

                    issue["ancestor"] = json!(ancestor);
                    issue["rule"] = json!(local_rule);
                    issue["inherited_rule"] = json!(ancestor_rule);
                    findings.push(issue);
                }
            }
        }
    }
    Ok(())
}

fn append_scope_drift_relationships(
    root: &Path,
    scopes: &Value,
    all_scopes: &Value,
    issues: &mut Vec<Value>,
) -> Result<()> {
    let mut warnings = Vec::new();
    let mut findings = Vec::new();

    append_scope_relationship_findings(root, scopes, all_scopes, &mut warnings, &mut findings)?;
    issues.extend(findings);
    Ok(())
}

fn managed_rules(content: &str) -> BTreeSet<String> {
    parse(content)
        .sections
        .into_iter()
        .filter(|section| {
            !matches!(
                normalize_section_name(&section.heading).as_str(),
                "scope" | "free region"
            )
        })
        .flat_map(|section| {
            section
                .body
                .lines()
                .map(str::trim)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter_map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .map(normalize_rule)
        })
        .filter(|line| !line.is_empty())
        .collect()
}

fn normalize_rule(rule: &str) -> String {
    rule.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn contradictory_rules(left: &str, right: &str) -> bool {
    let polarities = [
        ("must not ", false),
        ("do not ", false),
        ("never ", false),
        ("must ", true),
        ("do ", true),
        ("always ", true),
    ];
    polarities.iter().any(|(prefix, polarity)| {
        let Some(left_rest) = left.strip_prefix(prefix) else {
            return false;
        };

        polarities.iter().any(|(other_prefix, other_polarity)| {
            *polarity != *other_polarity
                && right
                    .strip_prefix(other_prefix)
                    .is_some_and(|right_rest| right_rest == left_rest)
        })
    })
}

fn validate_markdown(
    root: &Path,
    path: &Path,
    require_free_region: bool,
    signature_enabled: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    findings: &mut Vec<Value>,
) -> Result<()> {
    let content = read_document(path)?;
    let document = parse(&content);
    let mut headings = BTreeSet::new();

    for section in &document.sections {
        let key = normalize_section_name(&section.heading);
        if !headings.insert(key.clone()) {
            let message = format!("duplicate heading {key:?}");
            errors.push(format!("{}: {message}", display_path(root, path).display()));
            findings.push(finding(
                "duplicate_heading",
                "error",
                &display_path(root, path).to_string_lossy(),
                &message,
                None,
            ));
        }
    }

    let free_regions = document
        .sections
        .iter()
        .filter(|section| {
            section.level == 2 && normalize_section_name(&section.heading) == "free region"
        })
        .count();

    if require_free_region && free_regions == 0 {
        let message = "missing required ## Free Region section";
        errors.push(format!("{}: {message}", display_path(root, path).display()));

        findings.push(finding(
            "missing_free_region",
            "error",
            &display_path(root, path).to_string_lossy(),
            message,
            Some("Free Region"),
        ));
    } else if free_regions > 1 {
        let message = "duplicate ## Free Region sections";
        errors.push(format!("{}: {message}", display_path(root, path).display()));

        findings.push(finding(
            "duplicate_free_region",
            "error",
            &display_path(root, path).to_string_lossy(),
            message,
            Some("Free Region"),
        ));
    }

    for referenced in referenced_paths(&content) {
        if referenced != "."
            && !is_optional_reference(root, &referenced)
            && !root.join(&referenced).exists()
        {
            let message = format!("referenced path does not exist: {referenced}");
            errors.push(format!("{}: {message}", display_path(root, path).display()));

            findings.push(finding(
                "missing_path",
                "error",
                &display_path(root, path).to_string_lossy(),
                &message,
                Some(&referenced),
            ));
        }
    }

    for issue in signature::issues(&content, signature_enabled) {
        let message = issue.message();
        errors.push(format!("{}: {message}", display_path(root, path).display()));

        findings.push(finding(
            issue.kind(),
            "error",
            &display_path(root, path).to_string_lossy(),
            message,
            None,
        ));
    }

    if !content.ends_with('\n') {
        let message = "missing trailing newline";
        warnings.push(format!("{}: {message}", display_path(root, path).display()));

        findings.push(finding(
            "missing_trailing_newline",
            "warning",
            &display_path(root, path).to_string_lossy(),
            message,
            None,
        ));
    }

    Ok(())
}

fn validate_provenance(
    root: &Path,
    path: &Path,
    errors: &mut Vec<String>,
    findings: &mut Vec<Value>,
) -> Result<()> {
    for issue in provenance_issues(&read_document(path)?) {
        let display = display_path(root, path);
        errors.push(format!("{}: {}", display.display(), issue.message));

        findings.push(finding(
            issue.kind,
            "error",
            &display.to_string_lossy(),
            issue.message,
            Some(issue.fact),
        ));
    }

    Ok(())
}

struct ProvenanceIssue {
    kind: &'static str,
    fact: &'static str,
    message: &'static str,
}

fn provenance_issues(content: &str) -> Vec<ProvenanceIssue> {
    let document = parse(content);

    let Some(section) = document.sections.iter().find(|section| {
        section.level == 2 && normalize_section_name(&section.heading) == "provenance and decisions"
    }) else {
        return vec![ProvenanceIssue {
            kind: "missing_provenance_section",
            fact: "repository.provenance",
            message: "reference document has no ## Provenance And Decisions section",
        }];
    };

    PROVENANCE_FIELDS
        .iter()
        .filter(|(label, _, _, _)| provenance_field(&section.body, label).is_none())
        .map(|(_, kind, fact, message)| ProvenanceIssue {
            kind,
            fact,
            message,
        })
        .collect()
}

fn provenance_field(body: &str, label: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let lower = line.to_ascii_lowercase();
        let (_, value) = lower.split_once(label)?;
        let start = line.len() - value.len();
        let value = line[start..]
            .trim()
            .trim_matches(|character: char| "`.,;:-".contains(character));
        (!value.is_empty()).then(|| value.to_string())
    })
}

struct GuidanceIssue {
    kind: &'static str,
    severity: &'static str,
    fact: String,
    message: String,
}

fn guidance_issues(content: &str, evidence: &Value) -> Vec<GuidanceIssue> {
    let known = evidence["facts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|fact| Some((fact["id"].as_str()?, fact["confidence"].as_str()?)))
        .collect::<BTreeMap<_, _>>();

    referenced_fact_ids(content)
        .into_iter()
        .filter_map(|fact| match known.get(fact.as_str()) {
            Some(confidence) if matches!(*confidence, "inferred" | "uncertain") => {
                Some(GuidanceIssue {
                    kind: "low_confidence_fact",
                    severity: "warning",
                    fact,
                    message: format!("fact is {confidence} and should not become a firm rule"),
                })
            }
            Some(_) => None,
            None => Some(GuidanceIssue {
                kind: "unsupported_fact",
                severity: "error",
                message: "fact is not present in the current evidence".into(),
                fact,
            }),
        })
        .collect()
}

fn referenced_fact_ids(content: &str) -> BTreeSet<String> {
    code_spans(content)
        .flat_map(str::split_whitespace)
        .map(|value| value.trim_matches(|character: char| ",;:()[]{}<>".contains(character)))
        .filter(|value| {
            FACT_PREFIXES.iter().any(|prefix| value.starts_with(prefix))
                && value
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
        })
        .map(str::to_string)
        .collect()
}

fn command_issues(root: &Path, content: &str) -> Vec<GuidanceIssue> {
    code_spans(content)
        .map(str::trim)
        .filter(|command| command.split_whitespace().count() > 1)
        .filter_map(|command| {
            let executable = command.split_whitespace().next()?;
            let supported = match executable {
                "make" => command
                    .split_whitespace()
                    .nth(1)
                    .is_some_and(|target| make_target_exists(root, target)),
                "cargo" => root.join("Cargo.toml").is_file(),
                "agentskill" | "agsk" => root.join("Cargo.toml").is_file(),
                _ => true,
            };
            (!supported).then(|| GuidanceIssue {
                kind: "unverified_command",
                severity: "warning",
                fact: command.to_string(),
                message: "command has no matching repository support".into(),
            })
        })
        .collect()
}

fn make_target_exists(root: &Path, target: &str) -> bool {
    ["Makefile", "makefile", "GNUmakefile"].iter().any(|name| {
        std::fs::read_to_string(root.join(name))
            .ok()
            .is_some_and(|content| {
                content.lines().any(|line| {
                    line.split_once(':')
                        .is_some_and(|(name, _)| name.split_whitespace().any(|name| name == target))
                })
            })
    })
}

fn configuration_value(configuration: &RepositoryConfig, mode: SignatureMode) -> Value {
    json!({
        "valid": configuration.valid,
        "signature": configuration.resolved_signature(mode),
        "configured_signature": configuration.signature,
        "source": configuration.source.as_str(),
        "mode": mode.as_str(),
        "error": configuration.error,
    })
}

fn finding(
    kind: &str,
    severity: &str,
    document: &str,
    message: &str,
    path_or_fact: Option<&str>,
) -> Value {
    let mut value = serde_json::Map::new();
    value.insert("kind".into(), json!(kind));
    value.insert("severity".into(), json!(severity));
    value.insert("document".into(), json!(document));
    value.insert("message".into(), json!(message));
    if let Some(value_name) = path_or_fact {
        let key = if kind.contains("revision")
            || kind.contains("provenance")
            || kind.ends_with("_fact")
        {
            "fact"
        } else if kind.contains("command") {
            "command"
        } else {
            "path"
        };
        value.insert(key.into(), json!(value_name));
    }

    Value::Object(value)
}

fn is_optional_reference(root: &Path, path: &str) -> bool {
    path == "agentskill.toml" && !root.join(path).exists()
}

fn read_document(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

fn referenced_paths(content: &str) -> Vec<String> {
    code_spans(content)
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

fn code_spans(content: &str) -> impl Iterator<Item = &str> {
    content.split('`').skip(1).step_by(2)
}

fn approximate_tokens(content: &str) -> usize {
    content.split_whitespace().count() * 4 / 3
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}
