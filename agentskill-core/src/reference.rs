use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::error::{AgentskillError, Result};
use serde::Serialize;

const REMOTE_REFERENCE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Serialize)]
pub struct ReferenceSource {
    pub kind: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReferenceDocument {
    pub source: ReferenceSource,
    pub content: String,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

pub fn validate_references(references: &[String]) -> Result<()> {
    let mut seen = HashSet::new();

    for reference in references {
        let is_remote = reference.starts_with("file://")
            || reference.starts_with("http://")
            || reference.starts_with("https://")
            || reference.starts_with("ssh://")
            || reference.starts_with("git@");

        let identity = if is_remote {
            reference.clone()
        } else {
            let path = Path::new(reference).canonicalize().map_err(|_| {
                AgentskillError::InvalidPath(format!("reference path does not exist: {reference}"))
            })?;
            path.to_string_lossy().into_owned()
        };

        if !seen.insert(identity) {
            return Err(AgentskillError::InvalidArgument(format!(
                "duplicate reference source: {reference}"
            )));
        }

        if !is_remote {
            let root = Path::new(reference);

            if !root.is_dir() {
                return Err(AgentskillError::InvalidPath(format!(
                    "reference path is not a directory: {reference}"
                )));
            }

            let document = root.join("AGENTS.md");

            if !document.is_file() {
                return Err(AgentskillError::InvalidPath(format!(
                    "AGENTS.md not found in reference repository: {reference}"
                )));
            }

            if std::fs::read_to_string(document)
                .map(|text| text.trim().is_empty())
                .unwrap_or(true)
            {
                return Err(AgentskillError::InvalidPath(format!(
                    "AGENTS.md is empty in reference repository: {reference}"
                )));
            }
        }
    }

    Ok(())
}

pub fn load_reference_documents(references: &[String]) -> Result<Vec<ReferenceDocument>> {
    validate_references(references)?;
    references
        .iter()
        .map(|reference| {
            if is_remote(reference) {
                load_remote_reference(reference)
            } else {
                let path = Path::new(reference).join("AGENTS.md");

                let content = std::fs::read_to_string(&path).map_err(|_| {
                    AgentskillError::InvalidPath(format!(
                        "AGENTS.md not found in reference repository: {reference}"
                    ))
                })?;

                Ok(ReferenceDocument {
                    source: ReferenceSource {
                        kind: "local".into(),
                        value: reference.clone(),
                    },
                    content,
                    source_path: "AGENTS.md".into(),
                    commit_sha: None,
                })
            }
        })
        .collect()
}

fn is_remote(reference: &str) -> bool {
    ["file://", "http://", "https://", "ssh://", "git@"]
        .iter()
        .any(|prefix| reference.starts_with(prefix))
}

fn load_remote_reference(reference: &str) -> Result<ReferenceDocument> {
    let directory = tempfile::tempdir()?;

    let checkout = directory.path().join("reference");
    let mut child = Command::new("git")
        .args(["clone", "--depth", "1", reference])
        .arg(&checkout)
        .spawn()
        .map_err(|error| {
            AgentskillError::InvalidPath(format!(
                "failed to clone remote reference repository: {reference}: {error}"
            ))
        })?;

    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }

        if started.elapsed() >= REMOTE_REFERENCE_TIMEOUT {
            let _ = child.kill();

            let _ = child.wait();
            return Err(AgentskillError::InvalidPath(format!(
                "failed to clone remote reference repository: {reference}"
            )));
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let status = child.wait_with_output()?;

    if !status.status.success() {
        return Err(AgentskillError::InvalidPath(format!(
            "failed to clone remote reference repository: {reference}"
        )));
    }

    let path = checkout.join("AGENTS.md");

    let content = std::fs::read_to_string(&path).map_err(|_| {
        AgentskillError::InvalidPath(format!(
            "AGENTS.md not found in remote reference repository: {reference}"
        ))
    })?;

    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&checkout)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(ReferenceDocument {
        source: ReferenceSource {
            kind: "remote".into(),
            value: reference.into(),
        },
        content,
        source_path: "AGENTS.md".into(),
        commit_sha: sha,
    })
}
