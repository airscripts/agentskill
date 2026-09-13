use std::fs;
use std::process::Command;

use tempfile::tempdir;

#[test]
fn both_binaries_report_version() {
    for binary in [env!("CARGO_BIN_EXE_agentskill"), env!("CARGO_BIN_EXE_agsk")] {
        let flag_output = Command::new(binary).arg("--version").output().unwrap();
        let command_output = Command::new(binary).arg("version").output().unwrap();

        assert!(flag_output.status.success());
        assert!(command_output.status.success());
        assert_eq!(command_output.stdout, flag_output.stdout);
        assert!(String::from_utf8_lossy(&flag_output.stdout).contains("2.1.0"));
    }
}

#[test]
fn both_binaries_show_the_banner_in_help() {
    for binary in [env!("CARGO_BIN_EXE_agentskill"), env!("CARGO_BIN_EXE_agsk")] {
        let output = Command::new(binary).arg("--help").output().unwrap();

        assert!(output.status.success());
        let help = String::from_utf8_lossy(&output.stdout);
        assert!(help.contains("█████╗"));
        assert!(help.contains("Commands:"));
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

#[test]
fn exposes_scopes_budget_and_scope_filters() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nInstructions.\n",
    )
    .unwrap();

    fs::create_dir_all(directory.path().join("packages/api")).unwrap();
    fs::write(
        directory.path().join("packages/api/package.json"),
        "{\"name\":\"api\"}\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();

    let scopes = Command::new(env!("CARGO_BIN_EXE_agentskill"))
        .args(["scopes", &repo, "--pretty"])
        .output()
        .unwrap();

    assert!(scopes.status.success());
    let scopes: serde_json::Value = serde_json::from_slice(&scopes.stdout).unwrap();
    assert_eq!(scopes["scopes"][1]["path"], "packages/api");

    let evidence = Command::new(env!("CARGO_BIN_EXE_agsk"))
        .args([
            "evidence",
            &repo,
            "--scope",
            "packages/api",
            "--budget",
            "compact",
        ])
        .output()
        .unwrap();

    assert!(evidence.status.success());
    let evidence: serde_json::Value = serde_json::from_slice(&evidence.stdout).unwrap();
    assert_eq!(evidence["budget"]["mode"], "compact");
    assert_eq!(evidence["scopes"][0]["path"], "packages/api");
}
