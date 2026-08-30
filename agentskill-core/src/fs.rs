use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::language::{
    LanguageRole, LanguageSpec, is_test_path, language_for_content, language_role,
};

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
    pub role: FileRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileRole {
    Source,
    Test,
    Example,
    Documentation,
    Generated,
    Configuration,
    Auxiliary,
}

impl FileRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Test => "test",
            Self::Example => "example",
            Self::Documentation => "documentation",
            Self::Generated => "generated",
            Self::Configuration => "configuration",
            Self::Auxiliary => "auxiliary",
        }
    }
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
            if !SKIP_DIRS.contains(&name.as_str()) {
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
        let language = language_for_content(&path, &text, Some(repo));
        let lines = line_count(&path);
        let role = classify_role(&relative, &path, language);
        files.push(RepoFile {
            path,
            relative,
            language,
            bytes: metadata.len(),
            lines,
            role,
        });
    }
}

fn classify_role(relative: &str, path: &Path, language: Option<&'static LanguageSpec>) -> FileRole {
    let components = relative
        .split('/')
        .map(|item| item.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let name = path
        .file_name()
        .and_then(|item| item.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if components.iter().any(|item| {
        matches!(
            item.as_str(),
            "generated" | "gen" | "dist" | "build" | "out" | "target"
        )
    }) || name.contains("generated")
    {
        return FileRole::Generated;
    }

    if components.iter().any(|item| {
        matches!(
            item.as_str(),
            "examples" | "example" | "fixtures" | "fixture" | "testdata" | "samples"
        )
    }) {
        return FileRole::Example;
    }

    if components
        .iter()
        .any(|item| matches!(item.as_str(), "docs" | "documentation"))
        || matches!(name.as_str(), "readme.md" | "changelog.md" | "license")
    {
        return FileRole::Documentation;
    }

    if matches!(
        name.as_str(),
        ".editorconfig"
            | ".gitignore"
            | "cargo.toml"
            | "cargo.lock"
            | "package.json"
            | "pyproject.toml"
            | "makefile"
            | "cmakelists.txt"
            | "dockerfile"
            | "go.mod"
            | "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
    ) {
        return FileRole::Configuration;
    }

    if let Some(language) = language {
        if is_test_path(path, language) {
            return FileRole::Test;
        }

        if language_role(language.id) == Some(LanguageRole::Auxiliary) {
            return FileRole::Auxiliary;
        }
    }

    FileRole::Source
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
