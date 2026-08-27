use std::collections::BTreeMap;
use std::fs;

use agentskill_generation::{generate, render, render_with_answers, update};
use tempfile::tempdir;

#[test]
fn renders_deterministic_sections() {
    let example =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../agentskill-skill/examples/rust");

    let markdown = render(&example, "concise", &[], false).unwrap();
    assert!(markdown.starts_with("# AGENTS.md"));

    assert!(markdown.contains("## 1. Overview"));
    assert!(markdown.contains("## 12. Testing"));

    let comprehensive = render(&example, "comprehensive", &[], true).unwrap();

    assert!(comprehensive.contains("Interactive Notes"));
    assert!(comprehensive.contains("Verify this rule"));
}

#[test]
fn embeds_reference_metadata_in_generated_documents() {
    let directory = tempdir().unwrap();

    let reference = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        reference.path().join("AGENTS.md"),
        "# Reference\n\nUse cargo test.\n",
    )
    .unwrap();

    let markdown = render(
        directory.path(),
        "concise",
        &[reference.path().to_string_lossy().into_owned()],
        false,
    )
    .unwrap();

    assert!(markdown.starts_with("# AGENTS.md\n\n<!-- agentskill-metadata\n"));
    assert!(markdown.contains("\"source_path\": \"AGENTS.md\""));

    assert!(markdown.contains(&reference.path().to_string_lossy().to_string()));
}

#[test]
fn generates_and_updates_requested_output() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();

    let repo = directory.path().to_string_lossy().to_string();
    let path = directory.path().join("AGENTS.md");

    let output = path.to_string_lossy().to_string();
    generate(&repo, Some(&output), &[], false, "concise", "single").unwrap();

    assert!(path.exists());
    update(
        &repo,
        None,
        &["testing".into()],
        &[],
        false,
        "concise",
        "single",
    )
    .unwrap();

    assert!(fs::read_to_string(path).unwrap().contains("## 12. Testing"));
}

#[test]
fn supports_split_multifile_and_validation_paths() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();

    let repo = directory.path().to_string_lossy().to_string();

    generate(&repo, None, &[], false, "concise", "split").unwrap();

    assert!(directory.path().join("AGENTS.md").exists());
    assert!(directory.path().join("AGENTS.reference.md").exists());

    let output = directory.path().join("docs/AGENTS.md");

    let output = output.to_string_lossy().to_string();
    generate(
        &repo,
        Some(&output),
        &[],
        false,
        " COMPREHENSIVE ",
        " MULTIFILE ",
    )
    .unwrap();

    assert!(
        directory
            .path()
            .join("docs/.agentskill/01_OVERVIEW.md")
            .exists()
    );

    assert!(agentskill_generation::validate_profile("bad").is_err());

    assert!(agentskill_generation::validate_layout("bad").is_err());
    assert!(generate(&repo, None, &[], false, "bad", "single").is_err());

    assert!(update(&repo, None, &[], &[], false, "concise", "split").is_err());
    assert!(
        agentskill_generation::render(
            directory.path(),
            "concise",
            &["/missing/reference".into()],
            false,
        )
        .is_err()
    );
}

#[test]
fn preserves_references_in_multifile_index() {
    let directory = tempdir().unwrap();
    let reference = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(reference.path().join("AGENTS.md"), "# Reference\n").unwrap();

    let repo = directory.path().to_string_lossy().to_string();
    let output = directory.path().join("docs/AGENTS.md");
    let output = output.to_string_lossy().to_string();

    generate(
        &repo,
        Some(&output),
        &[reference.path().to_string_lossy().into_owned()],
        false,
        "concise",
        "multifile",
    )
    .unwrap();

    let index = fs::read_to_string(directory.path().join("docs/AGENTS.md")).unwrap();
    assert!(index.contains("<!-- agentskill-metadata"));
    assert!(index.contains(&reference.path().to_string_lossy().to_string()));
}

#[test]
fn update_preserves_and_filters_manual_sections() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Testing\n\nmanual testing\n\n## Custom\n\nkeep this\n",
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().to_string();

    update(
        &repo,
        None,
        &[],
        &["testing".into()],
        false,
        "concise",
        "single",
    )
    .unwrap();

    let merged = fs::read_to_string(directory.path().join("AGENTS.md")).unwrap();
    assert!(merged.contains("manual testing"));

    assert!(merged.contains("keep this"));

    update(&repo, None, &[], &[], true, "concise", "single").unwrap();

    let forced = fs::read_to_string(directory.path().join("AGENTS.md")).unwrap();
    assert!(!forced.contains("keep this"));
}

#[test]
fn applies_interactive_answers_and_feedback_sidecar() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Git\n\nTeam merge policy.\n",
    )
    .unwrap();
    fs::write(
        directory.path().join(".agentskill-feedback.json"),
        r#"{
            "sections": {
                "Testing": {
                    "prepend_notes": ["Keep integration tests fast."],
                    "pinned_facts": ["The test command is cargo test."]
                }
            },
            "preserve_sections": ["Git", "git"]
        }"#,
    )
    .unwrap();

    let repo = directory.path().to_string_lossy().to_string();
    let mut answers = BTreeMap::new();
    answers.insert("test_command".into(), "cargo test --all".into());

    let markdown = render_with_answers(directory.path(), "concise", &[], true, &answers).unwrap();
    assert!(markdown.contains("Use `cargo test --all` as the canonical test command."));

    update(&repo, None, &[], &[], false, "concise", "single").unwrap();

    let updated = fs::read_to_string(directory.path().join("AGENTS.md")).unwrap();
    assert!(updated.contains("Keep integration tests fast."));

    assert!(updated.contains("## Git"));

    fs::write(directory.path().join(".agentskill-feedback.json"), "[]").unwrap();

    assert!(update(&repo, None, &[], &[], false, "concise", "single").is_err());
    assert!(
        update(
            &repo,
            None,
            &["unknown".into()],
            &[],
            false,
            "concise",
            "single",
        )
        .is_err()
    );
}

#[test]
fn renders_detected_monorepo_services() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("services/api")).unwrap();
    fs::create_dir_all(directory.path().join("services/web")).unwrap();
    fs::write(
        directory.path().join("services/api/main.rs"),
        "fn main() {}\n",
    )
    .unwrap();
    fs::write(
        directory.path().join("services/web/main.rs"),
        "fn main() {}\n",
    )
    .unwrap();

    let markdown = render(directory.path(), "concise", &[], false).unwrap();

    assert!(markdown.contains("## 3. Service Map"));
    assert!(markdown.contains("- `api`: service root at `api`"));
    assert!(markdown.contains("- `web`: service root at `web`"));
}
