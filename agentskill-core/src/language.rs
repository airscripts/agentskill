use std::io::BufRead;
use std::path::Path;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LanguageSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub extensions: &'static [&'static str],
    pub test_patterns: &'static [&'static str],
}

pub const LANGUAGES: &[LanguageSpec] = &[
    LanguageSpec {
        id: "python",
        display_name: "Python",
        extensions: &[".py"],
        test_patterns: &["test_", "_test.py"],
    },
    LanguageSpec {
        id: "typescript",
        display_name: "TypeScript",
        extensions: &[".ts", ".tsx"],
        test_patterns: &[".test.", ".spec."],
    },
    LanguageSpec {
        id: "javascript",
        display_name: "JavaScript",
        extensions: &[".js", ".jsx", ".mjs", ".cjs"],
        test_patterns: &[".test.", ".spec."],
    },
    LanguageSpec {
        id: "go",
        display_name: "Go",
        extensions: &[".go"],
        test_patterns: &["_test.go"],
    },
    LanguageSpec {
        id: "rust",
        display_name: "Rust",
        extensions: &[".rs"],
        test_patterns: &["_test.rs"],
    },
    LanguageSpec {
        id: "java",
        display_name: "Java",
        extensions: &[".java"],
        test_patterns: &["Test.java", "Tests.java"],
    },
    LanguageSpec {
        id: "kotlin",
        display_name: "Kotlin",
        extensions: &[".kt", ".kts"],
        test_patterns: &["Test.kt", "Tests.kt"],
    },
    LanguageSpec {
        id: "csharp",
        display_name: "C#",
        extensions: &[".cs"],
        test_patterns: &["Test.cs", "Tests.cs"],
    },
    LanguageSpec {
        id: "c",
        display_name: "C",
        extensions: &[".c", ".h"],
        test_patterns: &["_test.c", "_tests.c"],
    },
    LanguageSpec {
        id: "cpp",
        display_name: "C++",
        extensions: &[".cpp", ".cc", ".cxx", ".hpp", ".hh", ".hxx"],
        test_patterns: &["_test.", "_tests."],
    },
    LanguageSpec {
        id: "ruby",
        display_name: "Ruby",
        extensions: &[".rb"],
        test_patterns: &["_spec.rb", "test_"],
    },
    LanguageSpec {
        id: "php",
        display_name: "PHP",
        extensions: &[".php"],
        test_patterns: &["Test.php"],
    },
    LanguageSpec {
        id: "swift",
        display_name: "Swift",
        extensions: &[".swift"],
        test_patterns: &["Tests.swift"],
    },
    LanguageSpec {
        id: "objectivec",
        display_name: "Objective-C",
        extensions: &[".m", ".mm"],
        test_patterns: &["Tests.m", "Tests.mm"],
    },
    LanguageSpec {
        id: "bash",
        display_name: "Bash",
        extensions: &[".sh", ".bash"],
        test_patterns: &["test_", "_test.sh", ".bats"],
    },
];

pub fn language_by_id(id: &str) -> Option<&'static LanguageSpec> {
    LANGUAGES.iter().find(|language| language.id == id)
}

pub fn language_for_path(path: &Path) -> Option<&'static LanguageSpec> {
    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        let extension = format!(".{}", extension.to_ascii_lowercase());

        if let Some(language) = LANGUAGES
            .iter()
            .find(|language| language.extensions.iter().any(|item| *item == extension))
        {
            return Some(language);
        }
    }

    let file = std::fs::File::open(path).ok()?;
    let mut first_line = String::new();
    std::io::BufReader::new(file)
        .read_line(&mut first_line)
        .ok()
        .filter(|bytes| *bytes > 0)?;

    let first_line = first_line.trim();
    (first_line.starts_with("#!")
        && (first_line.contains("/bash")
            || first_line.contains("/sh")
            || first_line.ends_with("bash")
            || first_line.ends_with("sh")))
    .then(|| language_by_id("bash"))
    .flatten()
}

pub fn is_test_path(path: &Path, language: &LanguageSpec) -> bool {
    let text = path.to_string_lossy();

    let file = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let named_match = language
        .test_patterns
        .iter()
        .any(|pattern| file.contains(pattern) || text.contains(pattern));

    if named_match {
        return true;
    }

    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("test" | "tests" | "__tests__" | "spec" | "specs")
        )
    })
}
