use std::fs;
use std::process::Command;

use tempfile::tempdir;

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

#[test]
fn both_binaries_validate_with_each_signature_mode() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nMaintainer instructions.\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();

    for binary in [env!("CARGO_BIN_EXE_agentskill"), env!("CARGO_BIN_EXE_agsk")] {
        for mode in ["auto", "on", "off"] {
            let output = Command::new(binary)
                .args(["validate", &repo, "--signature", mode])
                .output()
                .unwrap();
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

            assert_eq!(value["configuration"]["mode"], mode);
            assert_eq!(output.status.success(), mode == "off");
        }
    }
}
