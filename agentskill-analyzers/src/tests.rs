use std::collections::BTreeMap;
use std::path::Path;

use agentskill_core::{Result, error::validate_repo, fs::RepoFile, language::is_test_path};
use regex::Regex;
use serde_json::{Value, json};

pub fn run(repo: &str) -> Result<Value> {
    let root = validate_repo(repo)?;

    let files = agentskill_core::fs::collect_files(&root);
    let mut result = BTreeMap::new();

    for language in agentskill_core::language::LANGUAGES {
        let language_files: Vec<_> = files
            .iter()
            .filter(|file| file.language.is_some_and(|item| item.id == language.id))
            .collect();

        if language_files.is_empty() {
            continue;
        }

        let sources = language_files
            .iter()
            .filter(|file| !is_test_path(&file.path, language))
            .copied()
            .collect::<Vec<_>>();

        let tests = language_files
            .iter()
            .filter(|file| is_test_path(&file.path, language))
            .copied()
            .collect::<Vec<_>>();

        let contents = language_files
            .iter()
            .map(|file| agentskill_core::fs::read_text(&file.path))
            .collect::<Vec<_>>();

        let all_content = contents.join("\n");
        let framework = detect_framework(language.id, &root, &all_content);

        let mappings = map_tests(&sources, &tests);
        let test_dir = test_directory(&tests);

        let file_pattern = tests
            .first()
            .map(|file| test_file_pattern(language.id, &file.relative));

        let fixture_info = fixture_data(language.id, &language_files);
        let run_command = run_command(language.id, &root, framework);

        let representative_test = tests.first().map(|file| file.relative.clone());
        result.insert(
            language.id,
            json!({
                "framework": framework,
                "run_command": run_command,
                "test_files": tests.len(),
                "source_files": sources.len(),
                "coverage_shape": mappings,
                "structure": {
                    "location": if test_dir.is_some() { "separate_dirs" } else { "colocated" },
                    "test_dir": test_dir,
                    "mirrors_source": mirrors_source_tree(&sources, &tests, test_dir.as_deref()),
                },
                "naming": {
                    "file_pattern": file_pattern,
                    "function_pattern": function_pattern(language.id),
                    "class_pattern": class_pattern(language.id),
                },
                "fixtures": fixture_info,
                "representative_test": representative_test,
            }),
        );
    }

    Ok(json!(result))
}

fn detect_framework(language: &str, root: &Path, content: &str) -> &'static str {
    match language {
        "python" if content.contains("unittest") => "unittest",
        "python" => "pytest",
        "typescript" | "javascript" => {
            let package = agentskill_core::fs::read_text(&root.join("package.json"));

            if package.contains("vitest") || package.contains("vitest run") {
                "vitest"
            } else if package.contains("jest") {
                "jest"
            } else if package.contains("mocha") {
                "mocha"
            } else {
                "jest"
            }
        }
        "go" => "go test",
        "rust" => "cargo test",
        "java" => "junit",
        "kotlin" => "kotlin-test",
        "csharp"
            if content.contains("Xunit")
                || content.contains("xunit")
                || content.contains("[Fact]") =>
        {
            "xunit"
        }
        "csharp" if content.contains("NUnit") || content.contains("[TestCase]") => "nunit",
        "csharp" if content.contains("MSTest") || content.contains("[TestMethod]") => "mstest",
        "csharp" => "unknown",
        "c" if content.contains("unity") => "unity",
        "c" if content.contains("cmocka") => "cmocka",
        "cpp" if content.contains("gtest") => "gtest",
        "cpp" if content.contains("catch2") || content.contains("TEST_CASE(") => "catch2",
        "ruby" if content.contains("RSpec") || content.contains("rspec") => "rspec",
        "ruby" if content.contains("minitest") => "minitest",
        "php" if content.contains("PHPUnit") => "phpunit",
        "swift" | "objectivec" if content.contains("XCTest") => "xctest",
        "bash" if content.contains("@test") || content.contains("bats") => "bats",
        _ => "unknown",
    }
}

fn run_command(language: &str, root: &Path, framework: &str) -> Option<String> {
    if let Ok(regex) = Regex::new(r"(?m)^(?:test|test-all|tests)\s*:.*\n\t+(.+)") {
        for name in ["Makefile", "makefile", "GNUmakefile"] {
            let makefile = agentskill_core::fs::read_text(&root.join(name));

            if let Some(command) = regex.captures(&makefile).and_then(|capture| capture.get(1)) {
                return Some(command.as_str().trim().to_string());
            }
        }
    }

    let package = agentskill_core::fs::read_text(&root.join("package.json"));

    if let Ok(data) = serde_json::from_str::<Value>(&package)
        && let Some(command) = data["scripts"]["test"].as_str()
    {
        return Some(command.to_string());
    }

    if framework == "pytest" {
        return Some("pytest".into());
    }

    match language {
        "go" => Some("go test ./...".into()),
        "rust" => Some("cargo test".into()),
        "java" | "kotlin" => Some("./gradlew test".into()),
        "csharp" => Some("dotnet test".into()),
        "ruby" if framework == "rspec" => Some("bundle exec rspec".into()),
        "php" => Some("vendor/bin/phpunit".into()),
        "swift" | "objectivec" => Some("swift test".into()),
        "bash" => Some("bats tests".into()),
        _ => None,
    }
}

fn map_tests(sources: &[&RepoFile], tests: &[&RepoFile]) -> Value {
    let mut mapped = Vec::new();

    let mut unmatched_tests = Vec::new();
    let mut matched_sources = Vec::new();

    for test in tests {
        let test_stem = normalized_stem(&test.relative);

        let source = sources.iter().find(|source| {
            normalized_stem(&source.relative) == test_stem
                || test_stem.ends_with(&normalized_stem(&source.relative))
                || normalized_stem(&source.relative).ends_with(&test_stem)
        });

        if let Some(source) = source {
            mapped.push(json!({"source": source.relative, "test": test.relative}));

            matched_sources.push(source.relative.clone());
        } else {
            unmatched_tests.push(test.relative.clone());
        }
    }

    let untested = sources
        .iter()
        .filter(|source| !matched_sources.contains(&source.relative))
        .map(|source| source.relative.clone())
        .collect::<Vec<_>>();
    json!({
        "mapped": mapped,
        "untested_source_files": untested,
        "test_files_without_source_match": unmatched_tests,
    })
}

fn normalized_stem(path: &str) -> String {
    let file = Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    let mut value = file.to_string();
    for suffix in [
        ".test", ".spec", "_test", "_tests", "_spec", "Test", "Tests",
    ] {
        value = value.trim_end_matches(suffix).to_string();
    }
    value.trim_start_matches("test_").to_ascii_lowercase()
}

fn test_directory(tests: &[&RepoFile]) -> Option<String> {
    let path = tests.first()?.relative.replace('\\', "/");

    let root = path.split('/').next()?;
    if matches!(root, "test" | "tests" | "spec") {
        Some(format!("{root}/"))
    } else {
        None
    }
}

fn mirrors_source_tree(sources: &[&RepoFile], tests: &[&RepoFile], test_dir: Option<&str>) -> bool {
    let Some(test_dir) = test_dir else {
        return false;
    };
    tests.iter().any(|test| {
        let relative = test
            .relative
            .strip_prefix(test_dir)
            .unwrap_or(&test.relative);

        let stem = Path::new(relative)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        sources
            .iter()
            .any(|source| normalized_stem(&source.relative) == normalized_stem(stem))
    })
}

fn test_file_pattern(language: &str, path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    match language {
        "python" => "test_<module>.py".into(),
        "typescript" | "javascript" if path.contains(".spec.") => {
            format!("<module>.spec.{extension}")
        }
        "typescript" | "javascript" => format!("<module>.test.{extension}"),
        "go" => "<module>_test.go".into(),
        "rust" => "<module>_test.rs".into(),
        "ruby" if path.contains("spec/") => "<module>_spec.rb".into(),
        "ruby" => "test_<module>.rb".into(),
        _ => format!("<module>Test.{extension}"),
    }
}

fn function_pattern(language: &str) -> Option<&'static str> {
    match language {
        "python" => Some("test_<description>"),
        "go" => Some("Test<Description>"),
        "rust" => Some("<description>"),
        _ => None,
    }
}

fn class_pattern(language: &str) -> Option<&'static str> {
    match language {
        "python" => None,
        "java" | "kotlin" => Some("<ClassName>Test"),
        _ => None,
    }
}

fn fixture_data(language: &str, files: &[&RepoFile]) -> Value {
    if language != "python" {
        return json!({"uses_conftest": false, "conftest_locations": [], "fixture_names": []});
    }

    let locations = files
        .iter()
        .filter(|file| file.relative.ends_with("conftest.py"))
        .map(|file| file.relative.clone())
        .collect::<Vec<_>>();

    let names = files
        .iter()
        .filter(|file| file.relative.ends_with("conftest.py"))
        .flat_map(|file| {
            let content = agentskill_core::fs::read_text(&file.path);

            let lines = content.lines().map(str::trim).collect::<Vec<_>>();
            lines
                .windows(2)
                .filter(|pair| pair[0].starts_with("@pytest.fixture"))
                .filter_map(|pair| pair[1].strip_prefix("def "))
                .filter_map(|name| name.split('(').next())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    json!({"uses_conftest": !locations.is_empty(), "conftest_locations": locations, "fixture_names": names})
}
