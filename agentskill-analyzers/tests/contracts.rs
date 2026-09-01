use std::fs;
use std::path::PathBuf;

use agentskill_core::output::ANALYZER_NAMES;
use serde_json::Value;

#[test]
fn aggregate_output_contains_all_public_analyzers() {
    let example = format!(
        "{}/../agentskill-skill/examples/python",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = agentskill_analyzers::run_all(&example, None);
    let object = output
        .as_object()
        .expect("aggregate output must be an object");

    for name in ANALYZER_NAMES {
        assert!(object.contains_key(*name), "missing analyzer {name}");

        assert!(
            object[*name].is_object(),
            "analyzer {name} must be an object"
        );
    }
}

#[test]
fn analyzer_errors_keep_public_shape() {
    let output = agentskill_analyzers::run_one("scan", "/missing/repository", None);

    assert_eq!(output["script"], Value::String("scan".into()));
    assert!(output["error"].is_string());
}

#[test]
fn language_filter_limits_scan() {
    let example = format!(
        "{}/../agentskill-skill/examples/mixed",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = agentskill_analyzers::run_one("scan", &example, Some("go"));
    let languages = output["summary"]["by_language"].as_object().unwrap();

    assert_eq!(languages.keys().collect::<Vec<_>>(), vec!["go"]);
}

#[test]
fn aggregate_language_filter_preserves_repository_wide_analyzers() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(directory.path().join("src")).unwrap();

    std::fs::create_dir_all(directory.path().join("tests")).unwrap();

    std::fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").unwrap();

    std::fs::write(
        directory.path().join("src/tool.py"),
        "def tool():\n    return True\n",
    )
    .unwrap();
    std::fs::write(
        directory.path().join("tests/test_tool.py"),
        "def test_tool():\n    assert True\n",
    )
    .unwrap();

    std::fs::write(
        directory.path().join("pyproject.toml"),
        "[tool.ruff]\nline-length = 88\n",
    )
    .unwrap();

    let output =
        agentskill_analyzers::run_all(directory.path().to_string_lossy().as_ref(), Some("rust"));

    assert!(output["scan"]["summary"]["by_language"]["rust"].is_object());
    assert!(output["config"]["python"]["linter"].is_object());
    assert!(output["tests"]["python"].is_object());
}

#[test]
fn evidence_bundle_contains_scoped_facts_and_provenance() {
    let example = format!(
        "{}/../agentskill-skill/examples/python",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = agentskill_analyzers::run_evidence(&example, None).unwrap();
    assert_eq!(output["schema_version"], 4);
    assert_eq!(output["agentskill_version"], env!("CARGO_PKG_VERSION"));
    assert!(output["repository"]["root"].is_string());
    assert!(output["repository"]["dirty"].is_boolean());
    assert!(output["budget"]["input_tokens"].is_number());
    assert!(output["scopes"].is_array());
    assert!(output["scope_evidence"].is_array());
    let facts = output["facts"].as_array().unwrap();
    assert!(!facts.is_empty());
    assert!(output["repository"]["configuration"]["valid"].is_boolean());
    assert!(output["repository"]["configuration"]["signature"].is_boolean());
    assert!(
        facts
            .iter()
            .any(|fact| fact["id"] == "configuration.signature")
    );

    for fact in facts {
        assert!(fact["id"].is_string());
        assert!(fact["scope"].is_string());
        assert!(fact["confidence"].is_string());
        assert!(fact["evidence"].is_array());
    }

    let ids = output["facts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fact| fact["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort_unstable();
    assert_eq!(ids, sorted_ids);
}

#[test]
fn scope_manifest_and_budgeted_evidence_are_deterministic() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("AGENTS.md"), "# AGENTS\n").unwrap();
    fs::create_dir_all(directory.path().join("packages/api/src")).unwrap();
    fs::create_dir_all(directory.path().join("packages/web")).unwrap();

    fs::write(
        directory.path().join("packages/api/package.json"),
        "{\"name\":\"api\"}\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# API\n\n## Scope\n\n- Path: packages/api\n- Parent: .\n- Inheritance: additive.\n\n## Free Region\n\nLocal.\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("packages/api/src/lib.rs"),
        "pub fn api() {}\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("packages/api/src/index.js"),
        "import { util } from './util.js'; export const api = util;\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("packages/api/src/util.js"),
        "export const util = true;\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("packages/web/index.js"),
        "export const web = true;\n",
    )
    .unwrap();

    fs::create_dir_all(directory.path().join("src")).unwrap();
    for index in 0..40 {
        fs::write(
            directory.path().join(format!("src/file{index}.rs")),
            "pub fn generated_fixture() {}\n",
        )
        .unwrap();
    }

    let repo = directory.path().to_string_lossy().into_owned();
    let manifest = agentskill_analyzers::run_scopes(&repo, None).unwrap();
    let paths = manifest["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|scope| scope["path"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(paths, vec![".", "packages/api"]);

    let selected = ["packages/api".to_string()];

    for (mode, input_tokens, output_tokens, follow_up_rounds) in [
        ("compact", 4_000, 512, 1),
        ("standard", 8_000, 1_000, 2),
        ("deep", 16_000, 2_000, 4),
    ] {
        let profile =
            agentskill_analyzers::run_evidence_scoped(&repo, None, Some(&selected), Some(mode))
                .unwrap();

        assert_eq!(profile["budget"]["mode"], mode);
        assert_eq!(profile["budget"]["input_tokens"], input_tokens);
        assert_eq!(profile["budget"]["output_tokens"], output_tokens);
        assert_eq!(profile["budget"]["follow_up_rounds"], follow_up_rounds);
        let tree_len = profile["analyzers"]["scan"]["tree"]
            .as_array()
            .unwrap()
            .len();
        if mode == "compact" {
            assert!(tree_len <= 32);
        } else {
            assert!(tree_len > 32);
        }
    }

    let evidence =
        agentskill_analyzers::run_evidence_scoped(&repo, None, Some(&selected), Some("compact"))
            .unwrap();

    assert_eq!(evidence["budget"]["mode"], "compact");
    assert_eq!(evidence["scopes"][0]["path"], "packages/api");
    assert_eq!(evidence["scopes"][0]["resolution"]["fallback"], ".");
    assert_eq!(evidence["scopes"][0]["resolution"]["ancestors"][0], ".");
    assert_eq!(evidence["scope_evidence"][0]["fallback"], ".");
    assert_eq!(evidence["scope_evidence"][0]["ancestors"][0], ".");

    assert!(
        evidence["scope_evidence"][0]["graph_files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "packages/api/src/util.js")
    );

    assert_eq!(
        evidence["scopes"][0]["resolution"]["precedence"],
        "nearest-scope-wins"
    );

    assert!(
        evidence["scope_evidence"][0]["local_files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "packages/api/src/lib.rs")
    );

    assert!(
        evidence["scope_evidence"][0]["excluded_siblings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "packages/web/index.js")
    );

    assert!(
        evidence["analyzers"]["scan"]["tree"]
            .as_array()
            .unwrap()
            .len()
            <= 32
    );
    assert!(
        evidence["analyzers"]["scan"]["read_order"]
            .as_array()
            .unwrap()
            .len()
            <= 16
    );
    for result in evidence["analyzers"]["graph"].as_object().unwrap().values() {
        if let Some(edges) = result["edges"].as_array() {
            assert!(edges.len() <= 32);
        }
    }

    let standard =
        agentskill_analyzers::run_evidence_scoped(&repo, None, Some(&selected), Some("standard"))
            .unwrap();

    assert!(
        standard["analyzers"]["scan"]["tree"]
            .as_array()
            .unwrap()
            .len()
            > 32
    );

    let deep =
        agentskill_analyzers::run_evidence_scoped(&repo, None, Some(&selected), Some("deep"))
            .unwrap();
    assert!(deep["analyzers"]["scan"]["tree"].as_array().unwrap().len() > 32);
    assert!(
        deep["analyzers"]["scan"]["tree"].as_array().unwrap().len()
            >= standard["analyzers"]["scan"]["tree"]
                .as_array()
                .unwrap()
                .len()
    );
    assert!(
        agentskill_analyzers::run_evidence_scoped(&repo, None, Some(&selected), Some("unknown"))
            .is_err()
    );

    let explicit = ["src".to_string()];
    let explicit_manifest = agentskill_analyzers::run_scopes(&repo, Some(&explicit)).unwrap();
    assert_eq!(explicit_manifest["scopes"][0]["path"], "src");
    assert_eq!(
        explicit_manifest["scopes"][0]["resolution"]["ancestors"][0],
        "."
    );
}

#[test]
fn evidence_reports_malformed_agentskill_configuration_safely() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        directory.path().join("agentskill.toml"),
        "signature = false\nunknown = true\n",
    )
    .unwrap();

    let output =
        agentskill_analyzers::run_evidence(directory.path().to_str().unwrap(), None).unwrap();

    assert_eq!(output["repository"]["configuration"]["valid"], false);
    assert_eq!(output["repository"]["configuration"]["signature"], true);
    let fact = output["facts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fact| fact["id"] == "configuration.signature")
        .unwrap();

    assert_eq!(fact["confidence"], "uncertain");
    assert_eq!(fact["value"]["enabled"], true);
}

#[test]
fn evidence_does_not_fabricate_config_paths_for_builtin_tools() {
    let example = format!(
        "{}/../agentskill-skill/examples/mixed",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = agentskill_analyzers::run_evidence(&example, None).unwrap();
    let fact = output["facts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|fact| fact["id"] == "tool.go.formatter")
        .expect("mixed fixture should expose gofmt evidence");

    assert_eq!(fact["confidence"], "inferred");
    assert!(fact["evidence"].as_array().unwrap().is_empty());
}

#[test]
fn compatibility_scan_contract_fixtures_are_exercised() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = [
        ("scan_python.json", "python", "scan"),
        ("analyze_mixed.json", "mixed", "scan"),
    ];

    for (fixture, example, analyzer) in fixtures {
        let schema_path = root.join("../agentskill-tests/contracts").join(fixture);
        let schema: Value = serde_json::from_str(
            &fs::read_to_string(&schema_path).expect("contract fixture must be readable"),
        )
        .expect("contract fixture must be valid JSON");
        let schema = schema.get(analyzer).unwrap_or(&schema);
        let example = root.join("../agentskill-skill/examples").join(example);
        let output = agentskill_analyzers::run_one(analyzer, example.to_str().unwrap(), None);

        assert_contract_schema(&output, schema, fixture);
    }
}

fn assert_contract_schema(actual: &Value, schema: &Value, path: &str) {
    match schema {
        Value::Object(expected) => {
            let actual = actual
                .as_object()
                .unwrap_or_else(|| panic!("{path} must be an object"));
            for (key, expected_value) in expected {
                let actual_value = actual
                    .get(key)
                    .unwrap_or_else(|| panic!("missing contract key {path}.{key}"));
                assert_contract_schema(actual_value, expected_value, &format!("{path}.{key}"));
            }
        }
        Value::Array(expected_items) => {
            let actual = actual
                .as_array()
                .unwrap_or_else(|| panic!("{path} must be an array"));
            if let Some(expected_item) = expected_items.first() {
                for (index, actual_item) in actual.iter().enumerate() {
                    assert_contract_schema(actual_item, expected_item, &format!("{path}[{index}]"));
                }
            }
        }
        Value::String(kind) => match kind.as_str() {
            "str" => assert!(actual.is_string(), "{path} must be a string: {actual}"),
            "number" => assert!(actual.is_number(), "{path} must be a number: {actual}"),
            "bool" => assert!(actual.is_boolean(), "{path} must be a boolean: {actual}"),
            "null" => assert!(actual.is_null(), "{path} must be null: {actual}"),
            other => panic!("unknown contract type {other} at {path}"),
        },
        other => panic!("unsupported contract schema value at {path}: {other}"),
    }
}
