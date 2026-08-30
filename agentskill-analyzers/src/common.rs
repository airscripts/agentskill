use std::path::{Path, PathBuf};

use agentskill_core::fs::{RepoFile, collect_files, read_text};
use agentskill_core::language::{LanguageRole, language_role};

pub fn repo_files(
    repo: &str,
    lang: Option<&str>,
) -> agentskill_core::Result<(PathBuf, Vec<RepoFile>)> {
    let snapshot = RepoSnapshot::load(repo)?.filtered(lang);
    Ok((snapshot.root, snapshot.files))
}

pub struct RepoSnapshot {
    pub root: PathBuf,
    pub files: Vec<RepoFile>,
}

impl RepoSnapshot {
    pub fn load(repo: &str) -> agentskill_core::Result<Self> {
        let root = agentskill_core::error::validate_repo(repo)?;
        let files = collect_files(&root)
            .into_iter()
            .filter(|file| file.language.is_some())
            .collect();

        Ok(Self { root, files })
    }

    pub fn filtered(&self, lang: Option<&str>) -> Self {
        let files = self
            .files
            .iter()
            .filter(|file| {
                lang.is_none_or(|filter| {
                    file.language.is_some_and(|language| language.id == filter)
                })
            })
            .cloned()
            .collect();

        Self {
            root: self.root.clone(),
            files,
        }
    }
}

pub fn text(path: &Path) -> String {
    read_text(path)
}

pub fn is_auxiliary(language: &str) -> bool {
    language_role(language) == Some(LanguageRole::Auxiliary)
}

pub fn insert_language_result(
    result: &mut serde_json::Map<String, serde_json::Value>,
    language: &str,
    value: serde_json::Value,
) {
    if is_auxiliary(language) {
        result
            .entry("auxiliary")
            .or_insert_with(|| serde_json::json!({}))[language] = value;
    } else {
        result.insert(language.into(), value);
    }
}

pub fn percentile(values: &mut [usize], percent: usize) -> usize {
    if values.is_empty() {
        return 0;
    }

    values.sort_unstable();

    let index = ((values.len() * percent) / 100)
        .saturating_sub(1)
        .min(values.len() - 1);
    values[index]
}
