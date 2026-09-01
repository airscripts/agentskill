use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use agentskill_core::config::{ConfigSource, RepositoryConfig, load as load_config};
use agentskill_core::fs::FileRole;
use serde_json::{Value, json};

use crate::common::RepoSnapshot;

use agentskill_core::Result;

pub(crate) fn run_snapshot(
    snapshot: &RepoSnapshot,
    lang: Option<&str>,
    selected: Option<&[String]>,
    budget: Option<&str>,
) -> Result<Value> {
    let root = &snapshot.root;
    let files = &snapshot.files;
    let mut analysis = crate::run_all_snapshot(snapshot, lang);
    let configuration = load_config(root);

    let mut facts = Vec::new();
    add_inventory_facts(&mut facts, files);
    add_config_facts(&mut facts, &analysis);
    add_test_facts(&mut facts, &analysis, root);
    add_boundary_facts(&mut facts, &analysis);
    add_repository_config_fact(&mut facts, &configuration);
    facts.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));

    let scopes = crate::scope::run(root.to_string_lossy().as_ref(), selected)?;
    let budget = budget_profile(budget)?;
    apply_budget(&mut analysis, budget["mode"].as_str().unwrap_or("standard"));
    let scope_evidence = scoped_evidence(snapshot, &scopes["scopes"], &analysis);

    Ok(json!({
        "schema_version": 4,
        "agentskill_version": env!("CARGO_PKG_VERSION"),
        "repository": repository_metadata(root, &configuration),
        "budget": budget,
        "scopes": scopes["scopes"],
        "scope_evidence": scope_evidence,
        "facts": facts,
        "analyzers": analysis,
    }))
}

fn budget_profile(mode: Option<&str>) -> Result<Value> {
    let mode = mode.unwrap_or("standard");
    let profile = match mode {
        "compact" => (4_000, 512, 1),
        "standard" => (8_000, 1_000, 2),
        "deep" => (16_000, 2_000, 4),
        other => {
            return Err(agentskill_core::AgentskillError::InvalidArgument(format!(
                "unknown budget mode: {other}"
            )));
        }
    };

    Ok(json!({
        "mode": mode,
        "input_tokens": profile.0,
        "output_tokens": profile.1,
        "follow_up_rounds": profile.2,
    }))
}

fn apply_budget(analysis: &mut Value, mode: &str) {
    if mode != "compact" {
        return;
    }

    for (key, limit) in [("tree", 32), ("read_order", 16)] {
        if let Some(values) = analysis["scan"][key].as_array_mut() {
            values.truncate(limit);
        }
    }

    if let Some(languages) = analysis["graph"].as_object_mut() {
        for result in languages.values_mut() {
            if let Some(edges) = result["edges"].as_array_mut() {
                edges.truncate(32);
            }
        }
    }

    truncate_arrays(analysis, 32);
}

fn truncate_arrays(value: &mut Value, limit: usize) {
    match value {
        Value::Array(values) => {
            values.truncate(limit);
            for value in values {
                truncate_arrays(value, limit);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                truncate_arrays(value, limit);
            }
        }
        _ => {}
    }
}

fn scoped_evidence(snapshot: &RepoSnapshot, scopes: &Value, analysis: &Value) -> Vec<Value> {
    scopes
        .as_array()
        .into_iter()
        .flatten()
        .map(|scope| {
            let path = scope["path"].as_str().unwrap_or(".");
            let local_prefix = if path == "." {
                String::new()
            } else {
                format!("{path}/")
            };

            let local = snapshot
                .files
                .iter()
                .filter(|file| {
                    path == "." || file.relative == path || file.relative.starts_with(&local_prefix)
                })
                .map(|file| file.relative.clone())
                .collect::<Vec<_>>();

            let inherited = snapshot
                .files
                .iter()
                .filter(|file| {
                    !local.contains(&file.relative)
                        && inherited_support_file(&file.relative, path, file.role)
                })
                .map(|file| file.relative.clone())
                .collect::<Vec<_>>();

            let excluded_siblings = snapshot
                .files
                .iter()
                .filter(|file| {
                    !local.contains(&file.relative)
                        && !inherited.contains(&file.relative)
                        && path != "."
                        && !file.relative.starts_with(&local_prefix)
                })
                .map(|file| file.relative.clone())
                .take(32)
                .collect::<Vec<_>>();

            let graph_files = graph_related_files(analysis, &local, path);
            json!({
                "path": path,
                "parent": scope["parent"],
                "ancestors": scope["resolution"]["ancestors"],
                "fallback": scope["resolution"]["fallback"],
                "local_files": local,
                "inherited_files": inherited,
                "graph_files": graph_files,
                "excluded_siblings": excluded_siblings,
            })
        })
        .collect()
}

fn inherited_support_file(path: &str, scope: &str, role: agentskill_core::fs::FileRole) -> bool {
    if scope == "." {
        return true;
    }

    let depth = Path::new(scope).components().count();
    let file_depth = Path::new(path).components().count();

    let is_ancestor = file_depth <= depth
        && Path::new(scope).starts_with(Path::new(path).parent().unwrap_or_else(|| Path::new(".")));

    let top_level = file_depth == 1;
    let shared_ci = path.starts_with(".github/");
    shared_ci
        || is_ancestor
            && (top_level
                || matches!(
                    role,
                    FileRole::Configuration | FileRole::Documentation | FileRole::Auxiliary
                ))
}

fn graph_related_files(analysis: &Value, local: &[String], scope: &str) -> Vec<String> {
    let mut related = BTreeSet::new();
    let Some(languages) = analysis["graph"].as_object() else {
        return Vec::new();
    };

    for result in languages.values() {
        for edge in result["edges"].as_array().into_iter().flatten() {
            let from = edge["from"].as_str().unwrap_or_default();
            let to = edge["to"].as_str().unwrap_or_default();

            let local_edge = local
                .iter()
                .any(|path| path.ends_with(from) || path.ends_with(to));

            if local_edge || scope == "." {
                related.insert(from.to_string());
                related.insert(to.to_string());
            }
        }
    }
    related.into_iter().collect()
}

fn repository_metadata(root: &Path, configuration: &RepositoryConfig) -> Value {
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .is_some_and(|output| !output.stdout.is_empty());

    json!({
        "root": root,
        "revision": revision,
        "dirty": dirty,
        "configuration": {
            "valid": configuration.valid,
            "signature": configuration.signature,
            "source": configuration.source.as_str(),
            "error": configuration.error,
        },
    })
}

fn add_repository_config_fact(facts: &mut Vec<Value>, configuration: &RepositoryConfig) {
    let confidence = if configuration.valid {
        "verified"
    } else {
        "uncertain"
    };

    let evidence = if configuration.source == ConfigSource::File {
        vec![evidence_path("agentskill.toml")]
    } else {
        Vec::new()
    };

    facts.push(fact(
        "configuration.signature",
        "configuration",
        "repository",
        json!({
            "enabled": configuration.signature,
            "source": configuration.source.as_str(),
            "valid": configuration.valid,
            "error": configuration.error,
        }),
        confidence,
        evidence,
    ));
}

fn add_inventory_facts(facts: &mut Vec<Value>, files: &[agentskill_core::fs::RepoFile]) {
    let mut languages = BTreeMap::<String, Vec<String>>::new();
    let mut roles = BTreeMap::<&str, usize>::new();

    for file in files {
        *roles.entry(file.role.as_str()).or_default() += 1;
        if file.role == FileRole::Source
            && let Some(language) = file.language
        {
            languages
                .entry(language.id.to_string())
                .or_default()
                .push(file.relative.clone());
        }
    }

    for (language, paths) in languages {
        facts.push(fact(
            format!("language.{language}"),
            "language",
            "repository",
            json!(language),
            "verified",
            paths.into_iter().take(3).map(evidence_path).collect(),
        ));
    }

    facts.push(fact(
        "inventory.roles",
        "inventory",
        "repository",
        json!(roles),
        "verified",
        vec![evidence_path(".")],
    ));
}

fn add_config_facts(facts: &mut Vec<Value>, analysis: &Value) {
    let Some(config) = analysis["config"].as_object() else {
        return;
    };

    for (language, values) in config {
        if language == "auxiliary" || language == "editorconfig" {
            continue;
        }

        for kind in ["formatter", "linter", "type_checker"] {
            let Some(tool) = values[kind].as_object() else {
                continue;
            };

            let Some(name) = tool["name"].as_str() else {
                continue;
            };

            let (confidence, evidence) = tool["config_file"]
                .as_str()
                .map(|path| ("verified", vec![evidence_path(path)]))
                .unwrap_or(("inferred", Vec::new()));

            facts.push(fact(
                format!("tool.{language}.{kind}"),
                "tool",
                format!("language:{language}"),
                json!(name),
                confidence,
                evidence,
            ));
        }
    }
}

fn add_test_facts(facts: &mut Vec<Value>, analysis: &Value, root: &Path) {
    let Some(tests) = analysis["tests"].as_object() else {
        return;
    };

    let mut commands = BTreeSet::new();

    for (language, values) in tests {
        if language == "auxiliary" {
            continue;
        }

        if let Some(command) = values["run_command"].as_str()
            && !command.is_empty()
            && command != "unknown"
            && commands.insert(command.to_string())
        {
            facts.push(fact(
                format!("test.command.{}", commands.len()),
                "command",
                "repository",
                json!(command),
                command_confidence(root, command),
                command_evidence(root, command),
            ));
        }

        if let Some(test) = values["representative_test"].as_str() {
            facts.push(fact(
                format!("test.representative.{language}"),
                "testing",
                format!("language:{language}"),
                json!({
                    "framework": values["framework"],
                    "representative_test": test,
                    "test_dir": values["structure"]["test_dir"],
                }),
                "strong",
                vec![evidence_path(test)],
            ));
        }
    }
}

fn add_boundary_facts(facts: &mut Vec<Value>, analysis: &Value) {
    let boundary = &analysis["graph"]["monorepo_boundaries"];
    if !boundary["detected"].as_bool().unwrap_or(false) {
        return;
    }

    facts.push(fact(
        "architecture.monorepo_boundaries",
        "boundary",
        "repository",
        json!({
            "directory": boundary["boundary_dir"],
            "services": boundary["services"],
            "cross_service_imports": boundary["cross_service_imports"],
        }),
        "strong",
        boundary["boundary_dir"]
            .as_str()
            .map(|path| vec![evidence_path(path)])
            .unwrap_or_default(),
    ));
}

fn command_confidence(root: &Path, command: &str) -> &'static str {
    if command_evidence(root, command).is_empty() {
        "inferred"
    } else {
        "verified"
    }
}

fn command_evidence(root: &Path, command: &str) -> Vec<Value> {
    for name in ["Makefile", "makefile", "GNUmakefile"] {
        let path = root.join(name);
        if std::fs::read_to_string(&path)
            .ok()
            .is_some_and(|content| content.contains(command))
        {
            return vec![evidence_path(name)];
        }
    }

    if std::fs::read_to_string(root.join("package.json"))
        .ok()
        .is_some_and(|content| content.contains(command))
    {
        return vec![evidence_path("package.json")];
    }

    Vec::new()
}

fn evidence_path(path: impl Into<String>) -> Value {
    json!({"path": path.into()})
}

fn fact(
    id: impl Into<String>,
    category: &str,
    scope: impl Into<String>,
    value: Value,
    confidence: &str,
    evidence: Vec<Value>,
) -> Value {
    json!({
        "id": id.into(),
        "category": category,
        "scope": scope.into(),
        "value": value,
        "confidence": confidence,
        "evidence": evidence,
    })
}
