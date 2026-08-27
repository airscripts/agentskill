use std::path::{Path, PathBuf};

use agentskill_core::fs::{RepoFile, collect_files, read_text};

pub fn repo_files(
    repo: &str,
    lang: Option<&str>,
) -> agentskill_core::Result<(PathBuf, Vec<RepoFile>)> {
    let root = agentskill_core::error::validate_repo(repo)?;

    let files = collect_files(&root)
        .into_iter()
        .filter(|file| {
            let Some(language) = file.language else {
                return false;
            };
            lang.is_none_or(|filter| language.id == filter)
        })
        .collect();

    Ok((root, files))
}

pub fn text(path: &Path) -> String {
    read_text(path)
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
