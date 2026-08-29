use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use agentskill_core::fs::FileRole;
use serde_json::{Value, json};

use crate::common::RepoSnapshot;

use agentskill_core::Result;

pub(crate) fn run_snapshot(snapshot: &RepoSnapshot, lang: Option<&str>) -> Result<Value> {
    let root = &snapshot.root;
    let files = &snapshot.files;
    let analysis = crate::run_all_snapshot(snapshot, lang);

    let mut facts = Vec::new();
    add_inventory_facts(&mut facts, files);
    add_config_facts(&mut facts, &analysis);
    add_test_facts(&mut facts, &analysis, root);
    add_boundary_facts(&mut facts, &analysis);

    Ok(json!({
        "schema_version": 3,
        "repository": repository_metadata(root),
        "facts": facts,
        "analyzers": analysis,
    }))
}

fn repository_metadata(root: &Path) -> Value {
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
    })
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
