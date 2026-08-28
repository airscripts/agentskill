use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::{AgentskillError, Result};

pub const ANALYZER_NAMES: &[&str] = &[
    "scan", "measure", "config", "git", "graph", "symbols", "tests",
];

pub fn pretty_json(value: &Value, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
    } else {
        serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
    }
}

pub fn validate_out_path(out: &str) -> Result<PathBuf> {
    let path = Path::new(out);

    if path.is_absolute() {
        return Err(AgentskillError::InvalidArgument(format!(
            "invalid output path: absolute paths are not allowed: {out}"
        )));
    }

    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(value) => relative.push(value),
            std::path::Component::ParentDir => {
                if !relative.pop() {
                    return Err(AgentskillError::InvalidArgument(format!(
                        "invalid output path: escaping the working directory is not allowed: {out}"
                    )));
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(AgentskillError::InvalidArgument(format!(
                    "invalid output path: absolute paths are not allowed: {out}"
                )));
            }
        }
    }

    Ok(std::env::current_dir()?.join(relative))
}

pub fn write_value(value: &Value, pretty: bool, out: Option<&str>) -> Result<()> {
    let text = pretty_json(value, pretty) + "\n";

    match out {
        Some(path) => {
            let path = validate_out_path(path)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, text)?;

            Ok(())
        }
        None => {
            print!("{text}");

            Ok(())
        }
    }
}
