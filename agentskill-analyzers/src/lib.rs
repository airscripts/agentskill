mod common;
pub mod config;
mod evidence;
pub mod git;
pub mod graph;
pub mod measure;
pub mod scan;
pub mod symbols;
pub mod tests;

use agentskill_core::error::error_payload;
use agentskill_core::output::ANALYZER_NAMES;
use rayon::prelude::*;
use serde_json::{Map, Value};

pub fn run_one(name: &str, repo: &str, lang: Option<&str>) -> Value {
    let result = match name {
        "scan" => scan::run(repo, lang),
        "measure" => measure::run(repo, lang),
        "config" => config::run(repo),
        "git" => git::run(repo),
        "graph" => graph::run(repo, lang),
        "symbols" => symbols::run(repo, lang),
        "tests" => tests::run(repo),
        _ => Err(agentskill_core::AgentskillError::InvalidArgument(format!(
            "unknown analyzer: {name}"
        ))),
    };

    result.unwrap_or_else(|error| error_payload(error, name))
}

pub fn run_all(repo: &str, lang: Option<&str>) -> Value {
    let snapshot = match common::RepoSnapshot::load(repo) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let mut map = Map::new();
            for name in ANALYZER_NAMES {
                map.insert(name.to_string(), error_payload(&error, name));
            }
            return Value::Object(map);
        }
    };

    run_all_snapshot(&snapshot, lang)
}

pub(crate) fn run_all_snapshot(snapshot: &common::RepoSnapshot, lang: Option<&str>) -> Value {
    let language_snapshot = snapshot.filtered(lang);
    let values: Vec<(&str, Value)> = ANALYZER_NAMES
        .par_iter()
        .map(|name| {
            let result = match *name {
                "scan" => scan::run_with_snapshot(&language_snapshot),
                "measure" => measure::run_with_snapshot(&language_snapshot),
                "config" => config::run_with_snapshot(snapshot),
                "git" => git::run(&snapshot.root.to_string_lossy()),
                "graph" => graph::run_with_snapshot(&language_snapshot, lang),
                "symbols" => symbols::run_with_snapshot(&language_snapshot),
                "tests" => tests::run_with_snapshot(snapshot),
                _ => Err(agentskill_core::AgentskillError::InvalidArgument(format!(
                    "unknown analyzer: {name}"
                ))),
            };
            (
                *name,
                result.unwrap_or_else(|error| error_payload(error, name)),
            )
        })
        .collect();

    let mut map = Map::new();
    for (name, value) in values {
        map.insert(name.to_string(), value);
    }

    Value::Object(map)
}

pub fn run_many(repos: &[String], lang: Option<&str>) -> Value {
    if repos.len() == 1 {
        return run_all(&repos[0], lang);
    }

    let mut map = Map::new();

    for repo in repos {
        map.insert(repo.clone(), run_all(repo, lang));
    }

    Value::Object(map)
}

/// Builds the normalized evidence bundle used by the LLM skill.
pub fn run_evidence(repo: &str, lang: Option<&str>) -> agentskill_core::Result<Value> {
    let snapshot = common::RepoSnapshot::load(repo)?.filtered(lang);
    evidence::run_snapshot(&snapshot, lang)
}
