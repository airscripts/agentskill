use std::fs;
use std::path::PathBuf;
use std::process::Command;

use agentskill_core::config::SignatureMode;
use agentskill_core::document::parse;
use agentskill_validation::{
    drift, drift_with_mode_and_scopes, reconcile_signature, validate, validate_with_mode,
    validate_with_mode_and_scopes,
};
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
        "# AGENTS Reference\n\n## Provenance And Decisions\n\n- Agentskill Version: `2.0.0`\n- Evidence Schema Version: `4`\n- Repository Revision: `unknown`\n- Configuration: default signature enabled.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\nDetailed context.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = validate(directory.path().to_string_lossy().as_ref()).unwrap();

    assert_eq!(result["valid"], true);

    let without_reference_link = fs::read_to_string(directory.path().join("AGENTS.md"))
        .unwrap()
        .replace("\nRead `AGENTS.reference.md`.", "");
    fs::write(directory.path().join("AGENTS.md"), without_reference_link).unwrap();

    let unreferenced = validate(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(unreferenced["valid"], false);
    assert!(
        unreferenced["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["kind"] == "unreferenced_reference_document"
                    && finding["document"] == "AGENTS.md"
            })
    );
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
    assert_eq!(result["agentskill_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(result["revision_changed"], false);
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
fn reports_changed_reference_revision_without_marking_drift_stale() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# Reference\n\n## Provenance And Decisions\n\n- Agentskill Version: `2.0.0`\n- Evidence Schema Version: `4`\n- Repository Revision: `old-revision`\n- Configuration: default signature enabled.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    Command::new("git")
        .args(["init", "--quiet"])
        .current_dir(directory.path())
        .status()
        .unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(directory.path())
        .status()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=Agentskill Tests",
            "-c",
            "user.email=tests@agentskill.invalid",
            "-c",
            "commit.gpgSign=false",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ])
        .current_dir(directory.path())
        .status()
        .unwrap();

    let result = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["stale"], false);
    assert_eq!(result["revision_changed"], true);
    assert!(
        result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["kind"] == "changed_revision" && issue["severity"] == "info" })
    );
}

#[test]
fn reports_stale_reference_version() {
    let directory = tempdir().unwrap();

    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nCustom instructions.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# Reference\n\n## Provenance And Decisions\n\n- Agentskill Version: `1.9.0`\n- Evidence Schema Version: `4`\n- Repository Revision: `old-revision`\n- Configuration: default signature enabled.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
    )
    .unwrap();

    let result = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["stale"], true);
    assert!(
        result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["kind"] == "stale_version")
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
        "# Reference\n\n## Provenance And Decisions\n\n- Agentskill Version: `2.0.0`\n- Evidence Schema Version: `4`\n- Repository Revision: `current`\n- Configuration: invalid configuration is uncertain.\n- Maintainer-Confirmed Decisions: None recorded.\n- Unresolved Uncertainty: None recorded.\n\nThe `configuration.signature` fact is uncertain.\nThe `test.command.999` fact is unsupported.\n\n---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.\n",
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
        "scoped",
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

#[test]
fn validates_nested_scopes_and_reports_missing_candidates_advisory() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("packages/api/src")).unwrap();
    fs::create_dir_all(directory.path().join("packages/web")).unwrap();
    fs::write(
        directory.path().join("packages/api/package.json"),
        "{\"name\":\"api\"}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# API\n\n## Scope\n\n- Path: packages/api\n- Parent: .\n- Inheritance: additive; nearest scope wins.\n\n## Free Region\n\nAPI custom instructions.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/web/package.json"),
        "{\"name\":\"web\"}\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();
    let report = validate_with_mode(&repo, SignatureMode::Off).unwrap();

    assert_eq!(report["valid"], true);
    assert!(
        report["validated_scopes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|scope| scope == "packages/api")
    );
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["kind"] == "missing_scoped_document"
                    && finding["severity"] == "info"
                    && finding["document"] == "packages/web"
            })
    );
}

#[test]
fn validates_nested_references_with_inherited_signature_configuration() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("agentskill.toml"),
        "signature = false\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# Root Guidance\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("packages/api/src")).unwrap();
    fs::write(
        directory.path().join("packages/api/src/lib.rs"),
        "pub fn api() {}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# API Guidance\n\n## Scope\n\n- Path: packages/api\n- Parent: .\n- Inheritance: additive; nearest scope wins.\n\n## Mission\n\nRead `src/lib.rs`.\n\nRead `AGENTS.reference.md` for local API context.\n\n## Free Region\n\nAPI custom instructions.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.reference.md"),
        "# API Reference\n\n## Provenance And Decisions\n\n- Agentskill Version: `2.0.0`.\n- Evidence Schema Version: `4`.\n- Repository Revision: `fixture-revision`.\n- Configuration: inherited signature disabled.\n- Maintainer-Confirmed Decisions: Scope uses local API instructions.\n- Unresolved Uncertainty: None recorded.\n\nLocal API reference context.\n",
    )
    .unwrap();

    let root = fs::read_to_string(directory.path().join("AGENTS.md")).unwrap();
    let scoped = fs::read_to_string(directory.path().join("packages/api/AGENTS.md")).unwrap();
    let repo = directory.path().to_string_lossy().into_owned();

    let report = validate_with_mode_and_scopes(&repo, SignatureMode::Auto, None).unwrap();
    assert_eq!(report["valid"], true);
    assert_eq!(report["configuration"]["signature"], false);
    assert!(
        report["validated_scopes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "packages/api")
    );

    let root_free = parse(&root)
        .sections
        .into_iter()
        .find(|section| section.heading == "Free Region")
        .unwrap()
        .body;
    let scoped_free = parse(&scoped)
        .sections
        .into_iter()
        .find(|section| section.heading == "Free Region")
        .unwrap()
        .body;
    assert_ne!(root_free, scoped_free);

    let without_reference_link =
        scoped.replace("\nRead `AGENTS.reference.md` for local API context.", "");
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        without_reference_link,
    )
    .unwrap();

    let unreferenced = validate_with_mode_and_scopes(&repo, SignatureMode::Auto, None).unwrap();
    assert_eq!(unreferenced["valid"], false);
    assert!(
        unreferenced["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["kind"] == "unreferenced_reference_document"
                    && finding["document"] == "AGENTS.md"
            })
    );

    fs::write(directory.path().join("packages/api/AGENTS.md"), &scoped).unwrap();

    let overridden = validate_with_mode_and_scopes(&repo, SignatureMode::On, None).unwrap();
    assert_eq!(overridden["valid"], false);
    assert!(
        overridden["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["kind"] == "missing_signature")
    );

    let disabled = validate_with_mode_and_scopes(&repo, SignatureMode::Off, None).unwrap();
    assert_eq!(disabled["valid"], true);
}

#[test]
fn legacy_scoped_documents_are_advisory_until_explicitly_adopted() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# Root Guidance\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("packages/api")).unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# Legacy API Guidance\n\n## Free Region\n\nLegacy custom instructions.\n",
    )
    .unwrap();

    let path = directory.path().join("packages/api/AGENTS.md");
    let before = fs::read_to_string(&path).unwrap();
    let repo = directory.path().to_string_lossy().into_owned();
    let scopes = agentskill_analyzers::run_scopes(&repo, None).unwrap();
    let legacy = scopes["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|scope| scope["path"] == "packages/api")
        .unwrap();

    assert_eq!(legacy["status"], "legacy");

    let report = validate_with_mode_and_scopes(&repo, SignatureMode::Off, None).unwrap();
    assert_eq!(report["valid"], true);
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["kind"] == "legacy_scoped_document"
                    && finding["severity"] == "warning"
                    && finding["document"] == "packages/api"
            })
    );
    assert!(
        !report["validated_scopes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "packages/api")
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn scoped_validation_rejects_incorrect_parent_metadata() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("services/payments")).unwrap();
    fs::write(
        directory.path().join("services/payments/AGENTS.md"),
        "# Payments\n\n## Scope\n\n- Path: services/payments\n- Parent: services\n- Inheritance: additive.\n\n## Free Region\n\nLocal instructions.\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();
    let report = validate_with_mode(&repo, SignatureMode::Off).unwrap();

    assert_eq!(report["valid"], false);
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| { finding["kind"] == "invalid_scope_metadata" })
    );
}

#[test]
fn scoped_validation_reports_inherited_duplicates_and_conflicts() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Rules\n\n- Always run cargo test.\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("packages/api")).unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# API\n\n## Scope\n\n- Path: packages/api\n- Parent: .\n- Inheritance: additive; nearest scope wins.\n\n## Rules\n\n- Always run cargo test.\n\n## Free Region\n\nAPI custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("services/payments")).unwrap();
    fs::write(
        directory.path().join("services/payments/AGENTS.md"),
        "# Payments\n\n## Scope\n\n- Path: services/payments\n- Parent: .\n- Inheritance: additive; nearest scope wins.\n\n## Rules\n\n- Never run cargo test.\n\n## Free Region\n\nPayments custom instructions.\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();
    let report = validate_with_mode(&repo, SignatureMode::Off).unwrap();
    let findings = report["findings"].as_array().unwrap();

    assert!(
        findings
            .iter()
            .any(|finding| finding["kind"] == "duplicated_inherited_rule")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding["kind"] == "conflicting_inherited_rule")
    );
    assert_eq!(report["valid"], true);
}

#[test]
fn filtered_scope_validation_checks_the_nearest_managed_ancestor() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Rules\n\n- Always run cargo test.\n\n## Free Region\n\nRoot custom instructions.\n",
    )
    .unwrap();
    fs::create_dir_all(directory.path().join("packages/api")).unwrap();
    fs::write(
        directory.path().join("packages/AGENTS.md"),
        "# Packages\n\n## Scope\n\n- Path: packages\n- Parent: .\n- Inheritance: additive.\n\n## Rules\n\n- Always run cargo test.\n\n## Free Region\n\nPackage instructions.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("packages/api/AGENTS.md"),
        "# API\n\n## Scope\n\n- Path: packages/api\n- Parent: packages\n- Inheritance: additive.\n\n## Rules\n\n- Never run cargo test.\n\n## Free Region\n\nAPI instructions.\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().into_owned();
    let selected = ["packages/api".to_string()];
    let report = validate_with_mode_and_scopes(&repo, SignatureMode::Off, Some(&selected)).unwrap();

    assert_eq!(report["scopes"].as_array().unwrap().len(), 1);
    assert!(
        report["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| {
                finding["kind"] == "conflicting_inherited_rule" && finding["ancestor"] == "packages"
            })
    );

    let drift = drift_with_mode_and_scopes(&repo, SignatureMode::Off, Some(&selected)).unwrap();
    assert!(drift["issues"].as_array().unwrap().iter().any(|issue| {
        issue["kind"] == "conflicting_inherited_rule" && issue["ancestor"] == "packages"
    }));
}
