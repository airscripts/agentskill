use std::fs;
use std::path::PathBuf;

use serde_yaml::Value;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn drift_action_and_workflow_keep_the_release_contract() {
    let root = repository_root();

    let action: Value = serde_yaml::from_str(
        &fs::read_to_string(root.join("agentskill-actions/drift/action.yml")).unwrap(),
    )
    .unwrap();

    assert_eq!(action["name"], "Agentskill Drift");
    assert_eq!(action["runs"]["using"], "composite");
    assert_eq!(action["inputs"]["version"]["required"], false);
    assert_eq!(action["inputs"]["source"]["default"], "false");
    assert_eq!(action["inputs"]["signature"]["default"], "auto");

    assert_eq!(
        action["runs"]["steps"][0]["if"],
        "runner.os != 'Windows' && inputs.source != 'true'"
    );

    assert!(action["outputs"]["report-path"]["value"].is_string());
    assert!(action["outputs"]["stale"]["value"].is_string());
    assert!(
        action["runs"]["steps"][0]["run"]
            .as_str()
            .unwrap()
            .contains("../shared/install-unix.sh")
    );
    assert!(
        action["runs"]["steps"][1]["run"]
            .as_str()
            .unwrap()
            .contains("../shared/install-windows.ps1")
    );
    assert!(
        action["runs"]["steps"][2]["run"]
            .as_str()
            .unwrap()
            .contains("### Agentskill Drift")
    );

    assert!(
        action["runs"]["steps"][3]["run"]
            .as_str()
            .unwrap()
            .contains("### Agentskill Drift")
    );

    assert!(
        fs::read_to_string(root.join("agentskill-actions/shared/install-unix.sh"))
            .unwrap()
            .contains("sha256sum --check")
    );
    assert!(
        fs::read_to_string(root.join("agentskill-actions/shared/install-windows.ps1"))
            .unwrap()
            .contains("Get-FileHash")
    );

    let workflow: Value = serde_yaml::from_str(
        &fs::read_to_string(root.join(".github/workflows/agentskill.yml")).unwrap(),
    )
    .unwrap();

    assert_eq!(workflow["name"], "Agentskill");
    assert_eq!(workflow["jobs"]["drift"]["name"], "Drift");
    assert_eq!(workflow["jobs"]["validate"]["name"], "Validate");
    assert!(workflow["on"]["workflow_call"].is_mapping());
    assert_eq!(
        workflow["on"]["workflow_call"]["inputs"]["ref"]["default"],
        ""
    );

    assert!(workflow["on"]["workflow_dispatch"].is_null());
    assert_eq!(
        workflow["jobs"]["drift"]["container"]["image"],
        "rust:1.89-bookworm"
    );

    assert!(
        workflow["jobs"]["drift"]["steps"][1]["run"]
            .as_str()
            .unwrap()
            .contains("git jq")
    );

    assert_eq!(
        workflow["jobs"]["drift"]["steps"][2]["name"],
        "Build Agentskill"
    );

    assert_eq!(workflow["jobs"]["drift"]["steps"][2]["shell"], "bash");
    assert_eq!(
        workflow["jobs"]["drift"]["steps"][3]["uses"],
        "./agentskill-actions/drift"
    );

    assert_eq!(
        workflow["jobs"]["drift"]["steps"][3]["with"]["source"],
        "true"
    );
    assert_eq!(
        workflow["jobs"]["validate"]["steps"][3]["uses"],
        "./agentskill-actions/validate"
    );

    assert_eq!(workflow["jobs"]["validate"]["steps"][2]["shell"], "bash");
    assert_eq!(
        workflow["jobs"]["validate"]["steps"][3]["with"]["source"],
        "true"
    );

    assert!(
        workflow["jobs"]["validate"]["steps"][1]["run"]
            .as_str()
            .unwrap()
            .contains("git jq")
    );

    assert_eq!(
        workflow["jobs"]["drift"]["steps"][4]["uses"],
        "actions/upload-artifact@v7"
    );

    assert_eq!(
        workflow["jobs"]["drift"]["steps"][4]["with"]["path"],
        "${{ steps.agentskill.outputs.report-path }}"
    );

    assert_eq!(
        workflow["jobs"]["drift"]["steps"][4]["with"]["if-no-files-found"],
        "error"
    );

    assert_eq!(
        workflow["jobs"]["validate"]["steps"][4]["with"]["path"],
        "${{ steps.agentskill.outputs.report-path }}"
    );

    let main: Value =
        serde_yaml::from_str(&fs::read_to_string(root.join(".github/workflows/main.yml")).unwrap())
            .unwrap();

    assert_eq!(
        main["jobs"]["agentskill"]["uses"],
        "./.github/workflows/agentskill.yml"
    );

    assert_eq!(main["jobs"]["agentskill"]["needs"], "test");

    let validate_action: Value = serde_yaml::from_str(
        &fs::read_to_string(root.join("agentskill-actions/validate/action.yml")).unwrap(),
    )
    .unwrap();

    assert_eq!(validate_action["name"], "Agentskill Validate");
    assert_eq!(validate_action["runs"]["using"], "composite");
    assert_eq!(validate_action["inputs"]["version"]["required"], false);
    assert_eq!(validate_action["inputs"]["source"]["default"], "false");
    assert_eq!(validate_action["inputs"]["signature"]["default"], "auto");

    assert_eq!(
        validate_action["runs"]["steps"][0]["if"],
        "runner.os != 'Windows' && inputs.source != 'true'"
    );

    assert!(validate_action["outputs"]["report-path"]["value"].is_string());
    assert!(
        validate_action["runs"]["steps"][2]["run"]
            .as_str()
            .unwrap()
            .contains("agentskill validate")
    );
    assert!(
        validate_action["runs"]["steps"][3]["run"]
            .as_str()
            .unwrap()
            .contains("### Agentskill Validate")
    );
}

#[test]
fn skill_contract_covers_managed_document_workflows() {
    let root = repository_root();

    let skill = fs::read_to_string(root.join("agentskill-skill/SKILL.md")).unwrap();
    let system = fs::read_to_string(root.join("agentskill-skill/SYSTEM.md")).unwrap();

    for workflow in ["init", "update", "audit", "explain"] {
        assert!(skill.contains(&format!("### `{workflow}`")));
    }

    assert!(skill.contains("Use `## Free Region` for maintainer customs"));
    assert!(skill.contains("agentskill scopes <repo> --pretty"));
    assert!(skill.contains("budget compact|standard|deep"));
    assert!(!skill.contains("`context`"));
    assert!(skill.contains("show a semantic diff"));
    assert!(system.contains("The LLM is the only author of semantic `AGENTS.md` content."));
    assert!(system.contains("preserve `## Free Region` verbatim"));
    assert!(system.contains("## Provenance And Decisions"));
    assert!(system.contains("## Scope"));
    assert!(system.contains("4,000 tokens"));
}
