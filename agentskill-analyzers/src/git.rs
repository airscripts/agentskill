use std::process::Command;

use agentskill_core::{Result, error::validate_repo};
use regex::Regex;
use serde_json::json;

pub fn run(repo: &str) -> Result<serde_json::Value> {
    let root = validate_repo(repo)?;

    let output = Command::new("git")
        .args(["log", "--format=%s"])
        .current_dir(&root)
        .output();

    let Ok(output) = output else {
        return Ok(json!({"error": "git executable not found", "script": "git"}));
    };

    if !output.status.success() {
        return Ok(json!({"error": "not a git repository", "script": "git"}));
    }

    let text = String::from_utf8_lossy(&output.stdout);

    let regex =
        Regex::new(r"^([a-z][a-z0-9_-]*)(\([^)]+\))?(!)?\s*:\s*(.+)$").expect("valid regex");

    let mut prefixes = serde_json::Map::new();
    let mut examples = serde_json::Map::new();

    let mut total = 0;
    for subject in text.lines() {
        total += 1;

        let captures = regex.captures(subject);
        let key = captures
            .as_ref()
            .and_then(|c| c.get(1))
            .map_or("unprefixed", |m| m.as_str());
        *prefixes.entry(key.to_string()).or_insert(json!(0)) =
            json!(prefixes.get(key).and_then(|v| v.as_u64()).unwrap_or(0) + 1);
        examples.entry(key.to_string()).or_insert(json!(subject));
    }

    Ok(
        json!({"commits": {"total": total, "prefixes": prefixes, "examples": examples}, "branches": {}, "merge_strategy": {"strategy": "unknown", "evidence": "insufficient data"}}),
    )
}
