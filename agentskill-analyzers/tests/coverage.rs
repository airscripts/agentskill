use std::fs;
use std::path::PathBuf;

use agentskill_analyzers::{run_all, run_many, run_one};
use agentskill_core::language::{LANGUAGES, LanguageRole, language_role};
use tempfile::tempdir;

fn examples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../agentskill-skill/examples")
}

#[test]
fn exercises_all_analyzers_across_supported_fixtures() {
    let root = examples_root();

    let fixtures = [
        "python",
        "javascript",
        "typescript",
        "go",
        "rust",
        "java",
        "kotlin",
        "csharp",
        "c",
        "cpp",
        "ruby",
        "php",
        "swift",
        "objectivec",
        "bash",
        "extended",
        "mixed",
    ];

    let analyzers = [
        "scan", "measure", "config", "git", "graph", "symbols", "tests",
    ];

    for fixture in fixtures {
        let path = root.join(fixture);

        let path = path.to_string_lossy();
        for analyzer in analyzers {
            let output = run_one(analyzer, &path, None);

            assert!(output.is_object(), "{analyzer} did not return an object");
            assert!(
                output.get("error").is_none(),
                "{analyzer} failed on {fixture}"
            );
        }
    }
}

#[test]
fn exercises_aggregate_filters_and_error_contracts() {
    let python = examples_root().join("python");

    let mixed = examples_root().join("mixed");
    let python = python.to_string_lossy().into_owned();

    let mixed = mixed.to_string_lossy().into_owned();

    let aggregate = run_all(&python, Some("python"));

    assert!(aggregate["scan"]["summary"]["by_language"]["python"].is_object());

    let many = run_many(&[python.clone(), mixed], None);

    assert_eq!(many.as_object().map(|value| value.len()), Some(2));

    let unknown = run_one("unknown", &python, None);

    assert_eq!(unknown["script"], "unknown");
    assert!(unknown["error"].is_string());

    let missing = run_one("scan", "/path/that/does/not/exist", None);

    assert_eq!(missing["script"], "scan");
    assert!(missing["error"].is_string());
}

#[test]
fn resolves_language_specific_internal_graph_edges() {
    let root = examples_root();

    let expected = [
        ("python", "src.app", "src.util", 1),
        ("javascript", "src/index.js", "src/util.js", 1),
        ("typescript", "src/index.ts", "src/user.ts", 1),
        ("go", "cmd/app", "internal/service", 3),
        ("rust", "src/lib.rs", "src/parser.rs", 1),
        (
            "java",
            "src/main/java/com/example/App.java",
            "src/main/java/com/example/service/UserService.java",
            3,
        ),
        (
            "kotlin",
            "src/main/kotlin/com/example/App.kt",
            "src/main/kotlin/com/example/service/UserService.kt",
            3,
        ),
        ("csharp", "src/App.cs", "src/Core/UserService.cs", 1),
        ("c", "src/main.c", "src/util.h", 1),
        ("cpp", "src/app.cpp", "include/example/service.hpp", 1),
        ("ruby", "lib/example/service.rb", "lib/example/helper.rb", 1),
        (
            "php",
            "src/Service/UserService.php",
            "src/Repository/UserRepository.php",
            4,
        ),
        (
            "objectivec",
            "Sources/UserService.m",
            "Sources/UserService.h",
            1,
        ),
        ("bash", "scripts/deploy.sh", "scripts/lib/common.sh", 3),
    ];

    for (language, from, to, line) in expected {
        let repo = root.join(language).to_string_lossy().into_owned();

        let output = run_one("graph", &repo, Some(language));
        assert!(
            output[language]["edges"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!({
                    "from": from,
                    "to": to,
                    "line": line,
                })),
            "{language}: {output}"
        );
    }
}

#[test]
fn exposes_language_specific_symbol_categories() {
    let root = examples_root();

    let expectations = [
        ("python", "constants"),
        ("go", "structs"),
        ("kotlin", "functions"),
        ("swift", "structs"),
        ("csharp", "methods"),
        ("cpp", "namespaces"),
        ("ruby", "modules"),
        ("php", "methods"),
        ("objectivec", "methods"),
        ("bash", "functions"),
    ];

    for (language, category) in expectations {
        let repo = root.join(language).to_string_lossy().into_owned();

        let output = run_one("symbols", &repo, Some(language));
        assert!(
            output[language][category]["total"].as_u64().unwrap_or(0) > 0,
            "{language} {category}: {output}"
        );
    }

    let python = run_one(
        "symbols",
        &root.join("python").to_string_lossy(),
        Some("python"),
    );

    assert_eq!(
        python["python"]["constants"]["patterns"]["SCREAMING_SNAKE_CASE"]["count"],
        1
    );
}

#[test]
fn maps_fixture_tests_and_detects_frameworks() {
    let root = examples_root();

    let expected = [
        ("python", "pytest"),
        ("javascript", "jest"),
        ("typescript", "vitest"),
        ("go", "go test"),
        ("rust", "cargo test"),
        ("java", "junit"),
        ("kotlin", "kotlin-test"),
        ("csharp", "xunit"),
        ("cpp", "gtest"),
        ("ruby", "rspec"),
        ("php", "phpunit"),
        ("swift", "xctest"),
        ("objectivec", "xctest"),
        ("bash", "unknown"),
    ];

    for (language, framework) in expected {
        let repo = root.join(language).to_string_lossy().into_owned();

        let output = run_one("tests", &repo, None);
        assert_eq!(output[language]["framework"], framework);

        assert!(
            !output[language]["coverage_shape"]["mapped"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn preserves_configuration_settings_and_project_markers() {
    let directory = tempdir().unwrap();

    let root = directory.path();
    fs::write(
        root.join("pyproject.toml"),
        "[tool.ruff]\nselect = [\"E\"]\n[tool.black]\nline-length = 88\n[tool.mypy]\npython_version = \"3.11\"\n",
    )
    .unwrap();
    fs::write(
        root.join(".editorconfig"),
        "[*]\nindent_style = tab\n[*.py]\nindent_size = 4\n",
    )
    .unwrap();
    fs::write(root.join(".prettierrc.yaml"), "semi: false\ntabWidth: 2\n").unwrap();
    fs::write(root.join(".eslintrc.yaml"), "rules:\n  semi: false\n").unwrap();
    fs::write(
        root.join("tsconfig.json"),
        "{\"compilerOptions\":{\"strict\":true}}\n",
    )
    .unwrap();
    fs::write(root.join("main.py"), "VALUE = 1\n").unwrap();
    fs::write(root.join("main.ts"), "export const value = 1;\n").unwrap();
    fs::write(root.join("pom.xml"), "<project/>\n").unwrap();
    fs::create_dir_all(root.join("src/main/java")).unwrap();
    fs::write(root.join("src/main/java/App.java"), "class App {}\n").unwrap();
    fs::write(root.join("Example.sln"), "\n").unwrap();
    fs::write(root.join("Example.csproj"), "<Project />\n").unwrap();
    fs::write(root.join("main.cs"), "class App {}\n").unwrap();

    let repo = root.to_string_lossy();

    let result = run_one("config", &repo, None);
    assert_eq!(
        result["python"]["linter"]["settings"]["select"],
        serde_json::json!(["E"]),
        "{result}"
    );

    assert_eq!(result["python"]["editorconfig"]["indent_size"], "4");
    assert_eq!(result["typescript"]["formatter"]["settings"]["semi"], false);

    assert_eq!(
        result["typescript"]["type_checker"]["settings"]["strict"],
        true
    );

    assert!(
        result["java"]["project_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "src/main/java")
    );

    assert!(
        result["csharp"]["project_markers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item == "Example.csproj")
    );
}

#[test]
fn extended_fixture_covers_new_language_kinds_and_auxiliary_output() {
    let root = examples_root().join("extended");
    let repo = root.to_string_lossy();
    let output = run_one("scan", &repo, None);

    assert_eq!(
        output["summary"]["by_language"].as_object().unwrap().len(),
        40
    );

    assert_eq!(
        output["auxiliary"]["summary"]["by_language"]
            .as_object()
            .unwrap()
            .len(),
        5
    );

    assert_eq!(
        output["summary"]["by_kind"]["programming"]["file_count"],
        38
    );

    assert_eq!(output["summary"]["by_kind"]["markup"]["file_count"], 4);
    assert_eq!(output["summary"]["by_kind"]["stylesheet"]["file_count"], 3);
    assert_eq!(output["summary"]["by_kind"]["query"]["file_count"], 2);
    assert_eq!(output["summary"]["by_kind"]["schema"]["file_count"], 1);
    assert_eq!(
        output["summary"]["by_kind"]["infrastructure"]["file_count"],
        2
    );

    assert_eq!(output["summary"]["by_kind"]["build"]["file_count"], 4);

    let auxiliary = run_one("symbols", &repo, None);
    assert!(
        auxiliary["auxiliary"]["markdown"]["headings"]["total"]
            .as_u64()
            .unwrap()
            > 0
    );

    let config = run_one("config", &repo, None);
    assert!(config["auxiliary"]["yaml"].is_object());
    assert!(config["auxiliary"]["toml"].is_object());
}

#[test]
fn extended_fixture_has_contract_output_for_every_new_language() {
    const PRIMARY: &[&str] = &[
        "dart",
        "scala",
        "elixir",
        "erlang",
        "lua",
        "r",
        "julia",
        "haskell",
        "clojure",
        "fsharp",
        "groovy",
        "powershell",
        "visualbasic",
        "zig",
        "d",
        "nim",
        "crystal",
        "ocaml",
        "perl",
        "matlab",
        "fortran",
        "ada",
        "gdscript",
        "solidity",
        "html",
        "vue",
        "svelte",
        "astro",
        "css",
        "sass",
        "less",
        "sql",
        "graphql",
        "protobuf",
        "hcl",
        "nix",
        "dockerfile",
        "make",
        "cmake",
        "starlark",
    ];
    const AUXILIARY: &[&str] = &["yaml", "json", "toml", "xml", "markdown"];

    let registered_primary = LANGUAGES
        .iter()
        .filter(|language| language_role(language.id) == Some(LanguageRole::Primary))
        .map(|language| language.id)
        .collect::<Vec<_>>();
    let registered_auxiliary = LANGUAGES
        .iter()
        .filter(|language| language_role(language.id) == Some(LanguageRole::Auxiliary))
        .map(|language| language.id)
        .collect::<Vec<_>>();
    assert!(
        PRIMARY
            .iter()
            .all(|language| registered_primary.contains(language))
    );

    assert_eq!(registered_auxiliary, AUXILIARY);

    let repo = examples_root().join("extended");
    let repo = repo.to_string_lossy();
    let outputs = [
        ("scan", run_one("scan", &repo, None)),
        ("measure", run_one("measure", &repo, None)),
        ("config", run_one("config", &repo, None)),
        ("graph", run_one("graph", &repo, None)),
        ("symbols", run_one("symbols", &repo, None)),
        ("tests", run_one("tests", &repo, None)),
    ];

    for language in PRIMARY.iter().chain(AUXILIARY.iter()) {
        for (analyzer, output) in &outputs {
            let section = if *analyzer == "scan" {
                if AUXILIARY.contains(language) {
                    &output["auxiliary"]["summary"]["by_language"][*language]
                } else {
                    &output["summary"]["by_language"][*language]
                }
            } else if AUXILIARY.contains(language) {
                &output["auxiliary"][*language]
            } else {
                &output[*language]
            };
            assert!(
                section.is_object(),
                "{analyzer} has no contract payload for {language}: {output}"
            );
        }
    }

    let primary_scan = outputs[0].1["summary"]["by_language"].as_object().unwrap();
    assert_eq!(primary_scan.len(), PRIMARY.len());
    for language in PRIMARY {
        assert!(
            primary_scan.contains_key(*language),
            "missing scan language {language}"
        );
    }

    let auxiliary_scan = outputs[0].1["auxiliary"]["summary"]["by_language"]
        .as_object()
        .unwrap();
    assert_eq!(auxiliary_scan.len(), AUXILIARY.len());
    for language in AUXILIARY {
        assert!(
            auxiliary_scan.contains_key(*language),
            "missing auxiliary scan language {language}"
        );
    }
}

#[test]
fn does_not_invent_new_language_test_commands_without_evidence() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("main.dart"),
        "String run() => 'ok';\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let output = run_one("tests", &repo, None);

    assert_eq!(output["dart"]["framework"], "unknown");
    assert_eq!(output["dart"]["run_command"], serde_json::Value::Null);
}

#[test]
fn does_not_assign_tool_specific_commands_without_tool_markers() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("migration_test.sql"), "select 1;\n").unwrap();
    fs::write(
        directory.path().join("schema_test.proto"),
        "syntax = \"proto3\";\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let output = run_one("tests", &repo, None);

    assert_eq!(output["sql"]["run_command"], serde_json::Value::Null);
    assert_eq!(output["protobuf"]["run_command"], serde_json::Value::Null);
}

#[test]
fn detects_configuration_only_javascript_projects() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join(".prettierrc.yaml"), "semi: false\n").unwrap();

    let repo = directory.path().to_string_lossy();
    let result = run_one("config", &repo, None);

    assert_eq!(result["typescript"]["formatter"]["name"], "prettier");
    assert_eq!(result["typescript"]["formatter"]["settings"]["semi"], false);
}

#[test]
fn detects_test_commands_from_makefile_variants() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        directory.path().join("GNUmakefile"),
        "test:\n\tcargo test --all\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();

    let result = run_one("tests", &repo, None);

    assert_eq!(result["rust"]["run_command"], "cargo test --all");
}

#[test]
fn preserves_graph_reexports_and_nested_index_resolution() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("src/lib")).unwrap();
    fs::write(
        directory.path().join("src/index.ts"),
        "export { value } from './lib';\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("src/lib/index.ts"),
        "export const value = 1;\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let result = run_one("graph", &repo, Some("typescript"));

    assert!(
        result["typescript"]["edges"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!({
                "from": "src/index.ts",
                "to": "src/lib/index.ts",
                "line": 1,
            }))
    );
}

#[test]
fn resolves_swift_module_imports() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("Sources/App")).unwrap();
    fs::create_dir_all(directory.path().join("Sources/Core")).unwrap();
    fs::write(
        directory.path().join("Sources/App/App.swift"),
        "import Core\npublic struct App {}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("Sources/Core/Service.swift"),
        "public struct Service {}\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let result = run_one("graph", &repo, Some("swift"));

    assert!(
        result["swift"]["edges"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!({
                "from": "Sources/App/App.swift",
                "to": "Sources/Core/Service.swift",
                "line": 1,
            }))
    );
}

#[test]
fn detects_package_and_app_monorepo_boundaries() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("packages/one")).unwrap();
    fs::create_dir_all(directory.path().join("packages/two")).unwrap();
    fs::write(
        directory.path().join("packages/one/main.rs"),
        "fn main() {}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/two/main.rs"),
        "fn main() {}\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let result = run_one("graph", &repo, None);

    assert_eq!(result["monorepo_boundaries"]["boundary_dir"], "packages");
    assert_eq!(
        result["monorepo_boundaries"]["services"],
        serde_json::json!(["one", "two"])
    );
}

#[test]
fn preserves_language_specific_symbol_categories_and_precision() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("src")).unwrap();
    fs::write(
        directory.path().join("src/app.ts"),
        "export const VALUE_NAME = 1;\nexport function run() {}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("src/main.go"),
        "package main\nconst (\n    FirstValue = 1\n    SecondValue = 2\n)\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("src/lib.rs"),
        "pub struct Parser;\npub enum Status { Ready }\npub trait Store {}\nstatic COUNTER: u64 = 0;\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("src/main.c"),
        "#define MAX_SIZE 10\nint main(void) { return 0; }\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy();
    let typescript = run_one("symbols", &repo, Some("typescript"));
    let go = run_one("symbols", &repo, Some("go"));
    let rust = run_one("symbols", &repo, Some("rust"));
    let c = run_one("symbols", &repo, Some("c"));

    assert_eq!(typescript["typescript"]["constants"]["total"], 1);
    assert_eq!(go["go"]["constants"]["total"], 2);
    assert_eq!(rust["rust"]["traits"]["total"], 1);
    assert_eq!(rust["rust"]["statics"]["total"], 1);
    assert_eq!(c["c"]["macros"]["total"], 1);
}
