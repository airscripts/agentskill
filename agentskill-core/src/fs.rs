use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::language::{LanguageSpec, language_by_id, language_for_path};

const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "vendor",
    "third_party",
    ".tox",
    ".nox",
    ".eggs",
    "site-packages",
    "htmlcov",
    "coverage",
    "out",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    ".agentskill",
];

const MAX_FILES_TO_PARSE: usize = 10_000;
const MAX_FILE_BYTES: usize = 1_000_000;

#[derive(Clone, Debug)]
pub struct RepoFile {
    pub path: PathBuf,
    pub relative: String,
    pub language: Option<&'static LanguageSpec>,
    pub bytes: u64,
    pub lines: usize,
}

pub fn read_text(path: &Path) -> String {
    fs::File::open(path)
        .and_then(|file| {
            let mut bytes = Vec::new();
            file.take(MAX_FILE_BYTES as u64).read_to_end(&mut bytes)?;
            Ok(bytes)
        })
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

pub fn collect_files(repo: &Path) -> Vec<RepoFile> {
    let mut files = Vec::new();
    collect_into(repo, repo, &mut files);
    files.sort_by(|left, right| left.relative.cmp(&right.relative));
    files
}

fn collect_into(repo: &Path, current: &Path, files: &mut Vec<RepoFile>) {
    if files.len() >= MAX_FILES_TO_PARSE {
        return;
    }

    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        if files.len() >= MAX_FILES_TO_PARSE {
            return;
        }

        let path = entry.path();

        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_symlink() {
            continue;
        }

        if path.is_dir() {
            if !SKIP_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                collect_into(repo, &path, files);
            }
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };

        let relative = path
            .strip_prefix(repo)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        let text = read_text(&path);
        let language = if path.extension().and_then(|value| value.to_str()) == Some("h")
            && text.contains("@interface")
        {
            language_by_id("objectivec")
        } else {
            language_for_path(Path::new(&name))
        };
        let lines = line_count(&path);
        files.push(RepoFile {
            path,
            relative,
            language,
            bytes: metadata.len(),
            lines,
        });
    }
}

pub fn line_count(path: &Path) -> usize {
    let Ok(mut file) = fs::File::open(path) else {
        return 0;
    };

    let mut count = 0;
    let mut buffer = [0; 65_536];

    loop {
        let Ok(bytes) = std::io::Read::read(&mut file, &mut buffer) else {
            return count;
        };

        if bytes == 0 {
            return count;
        }

        count += buffer[..bytes]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
    }
}
