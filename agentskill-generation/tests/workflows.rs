use std::fs;
use std::path::PathBuf;

use agentskill_core::config::SignatureMode;
use agentskill_core::document::parse;
use agentskill_validation::{drift, reconcile_signature, validate, validate_with_mode};
use tempfile::tempdir;

#[test]
fn validates_operational_and_reference_documents() {
    let directory = tempdir().unwrap();

    fs::create_dir_all(directory.path().join("src")).unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Mission\n\nRead `src/main.rs`.\n\n## Free Region\n\nCustom instructions.\n\nRead `AGENTS.reference.md`.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").unwrap();

    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# AGENTS Reference\n\n## Provenance And Decisions\n\n- Evidence Schema Version: `3`\n- Repository Revision: `unknown`\n- Configuration: default signature enabled.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\nDetailed context.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();

    assert_eq!(result["valid"], true);
}

#[test]
fn reports_missing_references_and_paths() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\nRead `missing/src.rs`.\n\nRead `AGENTS.reference.md`.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], false);
    assert!(result["errors"].as_array().unwrap().len() >= 2);
}

#[test]
fn rejects_duplicate_normalized_headings() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Testing\n\nRun tests.\n\n## 12. Testing\n\nDuplicate.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], false);
    assert!(
        result["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| error.as_str().unwrap().contains("duplicate heading"))
    );
}

#[test]
fn drift_is_read_only_and_reports_repository_revision() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["stale"], false);
    assert!(result.get("repository_revision").is_some());
}

#[test]
fn allows_signature_opt_out_and_workflow_override() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("agentskill.toml"),
        "signature = false\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], true);

    let overridden = validate_with_mode(
        directory.path().to_string_lossy().as_ref(),
        SignatureMode::On,
    )
    .unwrap();
    assert_eq!(overridden["valid"], false);
    assert_eq!(overridden["configuration"]["mode"], "on");

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let contradiction = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert!(
        contradiction["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["kind"] == "configuration_contradiction")
    );

    let drift = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert!(
        drift["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["kind"] == "configuration_contradiction")
    );
}

#[test]
fn rejects_duplicate_free_regions_and_reports_signature_findings() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nOne.\n\n## Free Region\n\nTwo.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], false);
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["kind"] == "duplicate_free_region" })
    );
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["kind"] == "missing_signature" })
    );
}

#[test]
fn detects_stale_reference_provenance_without_blocking_drift() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# Reference\n\n## Provenance And Decisions\n\n- Evidence Schema Version: `3`\n- Repository Revision: `old-revision`\n- Configuration: default signature enabled.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["stale"], true);
    assert!(
        result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["kind"] == "stale_revision" })
    );
}

#[test]
fn rejects_malformed_repository_configuration() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n",
    )
    .unwrap();

    fs::write(directory.path().join("agentskill.toml"), "unknown = true\n").unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], false);
    assert!(
        result["errors"].as_array().unwrap()[0]
            .as_str()
            .unwrap()
            .contains("agentskill.toml")
    );
    assert!(drift(directory.path().to_string_lossy().as_ref()).is_err());
}

#[test]
fn requires_visible_provenance_fields() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# Reference\n\n## Provenance And Decisions\n\n- Repository Revision: `current`\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], false);
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["kind"] == "missing_provenance_schema")
    );
}

#[test]
fn reports_unsupported_and_uncertain_fact_references() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# Reference\n\n## Provenance And Decisions\n\n- Evidence Schema Version: `3`\n- Repository Revision: `current`\n- Configuration: invalid configuration is uncertain.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\nThe `configuration.signature` fact is uncertain.\nThe `test.command.999` fact is unsupported.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(directory.path().join("agentskill.toml"), "unknown = true\n").unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    let findings = result["findings"].as_array().unwrap();
    assert!(findings.iter().any(|finding| {
        finding["kind"] == "low_confidence_fact" && finding["fact"] == "configuration.signature"
    }));
    assert!(findings.iter().any(|finding| {
        finding["kind"] == "unsupported_fact" && finding["fact"] == "test.command.999"
    }));
}

#[test]
fn reports_commands_without_repository_support() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\nRun `make missing` before committing.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["valid"], true);
    assert!(
        result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["kind"] == "unverified_command")
    );

    let drift = drift(directory.path().to_string_lossy().as_ref()).unwrap();

    assert!(
        drift["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["kind"] == "unverified_command")
    );
}

#[test]
fn exercises_guidance_fixtures() {
    let fixtures =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../agentskill-tests/fixtures/guidance");

    assert!(!fixtures.join("new-repository/AGENTS.md").exists());
    assert_eq!(
        validate(&fixtures.join("new-repository").to_string_lossy()).unwrap()["valid"],
        false
    );

    for name in [
        "existing-reference",
        "signature-enabled",
        "signature-disabled",
        "declined-questions",
    ] {
        let result = validate(&fixtures.join(name).to_string_lossy()).unwrap();
        assert_eq!(result["valid"], true, "fixture {name} should validate");
    }

    for document in [
        "existing-reference/AGENTS.md",
        "existing-reference/AGENTS.reference.md",
    ] {
        let content = fs::read_to_string(fixtures.join(document)).unwrap();
        assert_eq!(reconcile_signature(&content, true), content);
    }

    let before = fs::read_to_string(fixtures.join("managed-update-before/AGENTS.md")).unwrap();

    let after = fs::read_to_string(fixtures.join("managed-update-after/AGENTS.md")).unwrap();

    let before_document = parse(&before);
    let before_free = before_document
        .sections
        .iter()
        .find(|section| section.heading == "Free Region")
        .unwrap();

    let after_document = parse(&after);
    let after_free = after_document
        .sections
        .iter()
        .find(|section| section.heading == "Free Region")
        .unwrap();

    assert_eq!(before_free.body, after_free.body);
    assert_ne!(before, after);

    let manual = validate(&fixtures.join("manual-root").to_string_lossy()).unwrap();

    assert_eq!(manual["valid"], false);

    for name in ["duplicate-signature", "malformed-signature"] {
        let result = validate(&fixtures.join(name).to_string_lossy()).unwrap();
        assert_eq!(result["valid"], false, "fixture {name} should fail");
    }

    let custom = fs::read_to_string(fixtures.join("custom-footer/AGENTS.md")).unwrap();

    let custom_document = parse(&custom);
    let free_region = custom_document
        .sections
        .iter()
        .find(|section| section.heading == "Free Region")
        .unwrap();

    assert_eq!(
        free_region.body,
        "\nMaintainer footer:\n\n> Keep this note with the repository.\n"
    );

    let enabled = reconcile_signature(&custom, true);

    assert!(enabled.contains("> Keep this note with the repository."));
    assert!(enabled.contains("> Generated and maintained by [Agentskill]"));
    assert_eq!(reconcile_signature(&enabled, true), enabled);
    assert!(!reconcile_signature(&custom, false).contains("Generated and maintained by"));
}
