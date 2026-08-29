use std::fs;

use agentskill_validation::{drift, validate};
use tempfile::tempdir;

#[test]
fn validates_operational_and_reference_documents() {
    let directory = tempdir().unwrap();
    fs::create_dir_all(directory.path().join("src")).unwrap();
    fs::write(
        directory.path().join("AGENTS.md"),
        "# AGENTS.md\n\n## Mission\n\nRead `src/main.rs`.\n\nRead `AGENTS.reference.md`.\n",
    )
    .unwrap();
    fs::write(directory.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        directory.path().join("AGENTS.reference.md"),
        "# AGENTS Reference\n\nDetailed context.\n",
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
    fs::write(directory.path().join("AGENTS.md"), "# AGENTS.md\n").unwrap();

    let result = drift(directory.path().to_string_lossy().as_ref()).unwrap();
    assert_eq!(result["stale"], false);
    assert!(result.get("repository_revision").is_some());
}
