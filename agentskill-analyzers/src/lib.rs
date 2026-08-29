mod common;
pub mod config;
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
    let values: Vec<(&str, Value)> = ANALYZER_NAMES
        .par_iter()
        .map(|name| (*name, run_one(name, repo, lang)))
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
