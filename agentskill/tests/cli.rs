use std::process::Command;

#[test]
fn both_binaries_report_version() {
    for binary in [env!("CARGO_BIN_EXE_agentskill"), env!("CARGO_BIN_EXE_agsk")] {
        let output = Command::new(binary).arg("--version").output().unwrap();

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("2.1.0"));
    }
}

#[test]
fn cli_emits_json_for_analyzer() {
    let example = format!(
        "{}/../agentskill-skill/examples/rust",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = Command::new(env!("CARGO_BIN_EXE_agentskill"))
        .args(["scan", &example, "--pretty"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(value["summary"]["total_files"], 3);
}
