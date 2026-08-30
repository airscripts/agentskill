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
        &fs::read_to_string(root.join("agentskill-action/action.yml")).unwrap(),
    )
    .unwrap();

    assert_eq!(action["name"], "Agentskill Drift");
    assert_eq!(action["runs"]["using"], "composite");
    assert_eq!(action["inputs"]["version"]["required"], true);
    assert_eq!(action["inputs"]["signature"]["default"], "auto");
    assert!(action["outputs"]["report-path"]["value"].is_string());
    assert!(action["outputs"]["stale"]["value"].is_string());
    assert!(
        action["runs"]["steps"][2]["run"]
            .as_str()
            .unwrap()
            .contains("### Agentskill Report")
    );
    assert!(
        action["runs"]["steps"][3]["run"]
            .as_str()
            .unwrap()
            .contains("### Agentskill Report")
    );
    assert!(
        action["runs"]["steps"][0]["run"]
            .as_str()
            .unwrap()
            .contains("sha256sum --check")
    );
    assert!(
        action["runs"]["steps"][1]["run"]
            .as_str()
            .unwrap()
            .contains("Get-FileHash")
    );

    let workflow: Value = serde_yaml::from_str(
        &fs::read_to_string(root.join(".github/workflows/drift.yml")).unwrap(),
    )
    .unwrap();

    assert!(workflow["on"]["workflow_dispatch"].is_null());
    assert_eq!(workflow["on"]["push"]["branches"][0], "main");
    assert_eq!(workflow["on"]["pull_request"]["branches"][0], "main");
    assert_eq!(
        workflow["jobs"]["drift"]["steps"][1]["uses"],
        "./agentskill-action"
    );
    assert_eq!(
        workflow["jobs"]["drift"]["steps"][2]["uses"],
        "actions/upload-artifact@v7"
    );
    assert_eq!(
        workflow["jobs"]["drift"]["steps"][2]["with"]["if-no-files-found"],
        "error"
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
    assert!(skill.contains("show a semantic diff"));
    assert!(system.contains("The LLM is the only author of semantic `AGENTS.md` content."));
    assert!(system.contains("preserve `## Free Region` verbatim"));
    assert!(system.contains("## Provenance And Decisions"));
}
