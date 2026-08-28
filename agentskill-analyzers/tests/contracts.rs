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
