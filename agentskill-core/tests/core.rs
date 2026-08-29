use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use agentskill_core::document::{Document, Section, merge, parse, serialize};
use agentskill_core::error::{AgentskillError, error_payload, validate_repo};
use agentskill_core::fs::{collect_files, line_count, read_text};
use agentskill_core::language::{
    LANGUAGES, LanguageKind, LanguageRole, is_test_path, language_by_id, language_for_content,
    language_for_file, language_for_path, language_kind, language_role,
};
use agentskill_core::output::{pretty_json, validate_out_path, write_value};
use agentskill_core::reference::{load_reference_documents, validate_references};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn preserves_supported_language_matrix() {
    assert_eq!(LANGUAGES.len(), 60);

    assert_eq!(
        language_by_id("python").map(|item| item.display_name),
        Some("Python")
    );

    assert_eq!(
        language_for_path(Path::new("src/main.rs")).map(|item| item.id),
        Some("rust")
    );

    assert_eq!(
        language_for_path(Path::new("src/index.tsx")).map(|item| item.id),
        Some("typescript")
    );

    assert_eq!(language_kind("hcl"), Some(LanguageKind::Infrastructure));
    assert_eq!(language_role("markdown"), Some(LanguageRole::Auxiliary));
    assert_eq!(LanguageKind::Stylesheet.as_str(), "stylesheet");
    assert_eq!(
        language_for_path(Path::new("Dockerfile")).map(|item| item.id),
        Some("dockerfile")
    );

    let directory = tempdir().unwrap();
    let objc = directory.path().join("App.m");
    fs::write(&objc, "int run(void) { return 0; }\n").unwrap();
    assert_eq!(
        language_for_file(&objc, Some(directory.path())).map(|item| item.id),
        Some("objectivec")
    );

    let matlab = directory.path().join("compute.m");
    fs::write(&matlab, "function result = compute()\nresult = 1;\nend\n").unwrap();
    assert_eq!(
        language_for_file(&matlab, Some(directory.path())).map(|item| item.id),
        Some("matlab")
    );

    let marker_free = directory.path().join("plain.m");
    fs::write(&marker_free, "value = 1;\n").unwrap();
    assert_eq!(
        language_for_file(&marker_free, Some(directory.path())).map(|item| item.id),
        Some("objectivec")
    );

    assert_eq!(
        language_for_content(
            Path::new("provided.m"),
            "function result = provided()\nresult = 1;\nend\n",
            None,
        )
        .map(|item| item.id),
        Some("matlab")
    );
}

#[test]
fn parses_and_merges_sectioned_documents() {
    let generated = Document {
        preamble: "# AGENTS.md\n\n".into(),
        sections: vec![Section {
            level: 2,
            heading: "Testing".into(),
            body: "Run cargo test.\n".into(),
        }],
    };

    let existing =
        "# AGENTS.md\n\nManual preamble.\n\n## Testing\n\nOld rule.\n\n## Custom\n\nKeep me.\n";

    let merged = merge(existing, &generated, &[], &[], false);
    let document = parse(&merged);

    assert!(
        document
            .sections
            .iter()
            .any(|section| section.heading == "Testing")
    );

    assert!(merged.contains("Run cargo test."));
    assert!(merged.contains("## Testing\n\nRun cargo test."));

    assert!(merged.contains("## Custom"));
    assert!(serialize(&document).ends_with('\n'));
}

#[test]
fn treats_agents_title_as_preamble_and_preserves_custom_markdown() {
    let document =
        parse("# AGENTS.md\n\n## Custom\n\nKeep this rule.\n\n### Detail\n\nMore context.\n");

    assert_eq!(document.preamble, "# AGENTS.md\n\n");
    assert_eq!(document.sections.len(), 2);
    assert_eq!(document.sections[0].heading, "Custom");
    assert_eq!(
        serialize(&document),
        "# AGENTS.md\n\n## Custom\n\nKeep this rule.\n\n### Detail\n\nMore context.\n\n"
    );
}

#[test]
fn validates_paths_reads_files_and_skips_links() {
    let directory = tempdir().unwrap();

    let source = directory.path().join("main.rs");
    fs::write(&source, "fn main() {}\n").unwrap();
    fs::create_dir(directory.path().join(".hidden")).unwrap();
    fs::write(
        directory.path().join(".hidden/ignored.rs"),
        "fn ignored() {}\n",
    )
    .unwrap();

    assert_eq!(
        validate_repo(directory.path().to_str().unwrap()).unwrap(),
        directory.path().canonicalize().unwrap()
    );

    assert_eq!(read_text(&source), "fn main() {}\n");
    assert_eq!(line_count(&source), 1);

    assert_eq!(read_text(&directory.path().join("missing")), "");
    assert_eq!(line_count(&directory.path().join("missing")), 0);

    assert_eq!(collect_files(directory.path()).len(), 1);

    let file_path = source.to_string_lossy().into_owned();

    let error = validate_repo(&file_path).unwrap_err();
    assert!(matches!(error, AgentskillError::InvalidPath(_)));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&source, directory.path().join("linked.rs")).unwrap();

        assert_eq!(collect_files(directory.path()).len(), 1);
    }
}

#[test]
fn preserves_byte_limited_reads_and_newline_based_line_counts() {
    let directory = tempdir().unwrap();
    let source = directory.path().join("script");
    fs::write(&source, "#!/usr/bin/env bash\necho ok").unwrap();

    assert_eq!(language_for_path(&source).map(|item| item.id), Some("bash"));
    assert_eq!(line_count(&source), 1);

    let rust = directory.path().join("main.rs");
    fs::write(&rust, "fn main() {}").unwrap();
    assert_eq!(line_count(&rust), 0);
}

#[test]
fn detects_test_directories_for_all_registered_languages() {
    let python = language_by_id("python").unwrap();
    let typescript = language_by_id("typescript").unwrap();

    assert!(is_test_path(Path::new("tests/unit/app.py"), python));
    assert!(is_test_path(Path::new("src/__tests__/app.ts"), typescript));
    assert!(is_test_path(
        Path::new("src/test/java/App.java"),
        language_by_id("java").unwrap()
    ));
}

#[test]
fn validates_references_and_serializes_output() {
    let reference = tempdir().unwrap();
    fs::write(reference.path().join("AGENTS.md"), "# AGENTS.md\n").unwrap();

    let reference_path = reference.path().to_string_lossy().into_owned();

    validate_references(std::slice::from_ref(&reference_path)).unwrap();
    validate_references(&["https://example.com/agentskill.git".into()]).unwrap();

    let duplicate = validate_references(&[reference_path.clone(), reference_path]);

    assert!(matches!(
        duplicate,
        Err(AgentskillError::InvalidArgument(_))
    ));

    let missing = validate_references(&["/missing/reference".into()]);
    assert!(matches!(missing, Err(AgentskillError::InvalidPath(_))));

    let empty = tempdir().unwrap();
    fs::write(empty.path().join("AGENTS.md"), "\n").unwrap();

    let empty_path = empty.path().to_string_lossy().into_owned();
    assert!(validate_references(&[empty_path]).is_err());

    let value = json!({"ok": true});

    assert!(pretty_json(&value, true).contains("\n"));
    let output = format!("agentskill-output-{}.json", std::process::id());
    write_value(&value, false, Some(&output)).unwrap();

    assert_eq!(fs::read_to_string(&output).unwrap(), "{\"ok\":true}\n");
    fs::remove_file(output).unwrap();
    assert_eq!(error_payload("bad", "scan")["script"], "scan");
}

#[test]
fn loads_local_references_and_reports_reference_validation_failures() {
    let reference = tempdir().unwrap();
    fs::write(reference.path().join("AGENTS.md"), "# Reference\n\nRule.\n").unwrap();
    let reference_path = reference.path().to_string_lossy().into_owned();

    let documents = load_reference_documents(std::slice::from_ref(&reference_path)).unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].source.kind, "local");
    assert_eq!(documents[0].source.value, reference_path);
    assert_eq!(documents[0].content, "# Reference\n\nRule.\n");
    assert_eq!(documents[0].source_path, "AGENTS.md");
    assert!(documents[0].commit_sha.is_none());

    let file = reference.path().join("not-a-directory");
    fs::write(&file, "content").unwrap();
    let file_path = file.to_string_lossy().into_owned();
    assert!(matches!(
        validate_references(&[file_path]),
        Err(AgentskillError::InvalidPath(_))
    ));

    let missing_document = tempdir().unwrap();
    let missing_path = missing_document.path().to_string_lossy().into_owned();
    assert!(matches!(
        validate_references(&[missing_path]),
        Err(AgentskillError::InvalidPath(_))
    ));
}

#[test]
fn rejects_unreachable_remote_references_without_external_network() {
    let directory = tempdir().unwrap();
    let remote = format!("file://{}", directory.path().join("missing.git").display());
    let result = load_reference_documents(&[remote]);

    assert!(matches!(result, Err(AgentskillError::InvalidPath(_))));
}

#[cfg(unix)]
#[test]
fn loads_file_url_references_and_records_commit_sha() {
    let repository = tempdir().unwrap();
    let repository_path = repository.path().to_str().unwrap();

    for args in [
        vec!["init", "--quiet", repository_path],
        vec![
            "-C",
            repository_path,
            "config",
            "user.email",
            "test@example.com",
        ],
        vec![
            "-C",
            repository_path,
            "config",
            "user.name",
            "agentskill tests",
        ],
    ] {
        assert!(Command::new("git").args(args).status().unwrap().success());
    }

    fs::write(
        repository.path().join("AGENTS.md"),
        "# Remote reference\n\nRule.\n",
    )
    .unwrap();
    assert!(
        Command::new("git")
            .args(["-C", repository_path, "add", "AGENTS.md"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-C",
                repository_path,
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--quiet",
                "-m",
                "reference"
            ])
            .status()
            .unwrap()
            .success()
    );

    let reference = format!("file://{repository_path}");
    let documents = load_reference_documents(std::slice::from_ref(&reference)).unwrap();

    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].source.kind, "remote");
    assert_eq!(documents[0].source.value, reference);
    assert_eq!(documents[0].content, "# Remote reference\n\nRule.\n");
    assert_eq!(documents[0].source_path, "AGENTS.md");
    assert!(
        documents[0]
            .commit_sha
            .as_deref()
            .is_some_and(|sha| sha.len() == 40)
    );
}

#[test]
fn formats_and_converts_all_error_variants() {
    let io = AgentskillError::from(std::io::Error::other("io failure"));
    assert_eq!(io.to_string(), "io failure");

    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let json = AgentskillError::from(json_error);
    assert!(!json.to_string().is_empty());

    assert_eq!(
        AgentskillError::InvalidPath("bad path".into()).to_string(),
        "bad path"
    );
    assert_eq!(
        AgentskillError::InvalidArgument("bad argument".into()).to_string(),
        "bad argument"
    );
    assert_eq!(AgentskillError::Other("other".into()).to_string(), "other");
}

#[test]
fn validates_and_writes_safe_output_paths() {
    let absolute = std::env::current_dir().unwrap().join("report.json");

    assert_eq!(
        validate_out_path(absolute.to_str().unwrap())
            .unwrap_err()
            .to_string(),
        format!(
            "invalid output path: absolute paths are not allowed: {}",
            absolute.display()
        )
    );

    assert!(validate_out_path("../report.json").is_err());

    let output = PathBuf::from(format!("agentskill-output-{}", std::process::id()));
    let relative = output.join("nested/report.json");
    write_value(
        &serde_json::json!({"ok": true}),
        false,
        Some(relative.to_str().unwrap()),
    )
    .unwrap();
    assert!(relative.exists());
    std::fs::remove_dir_all(output).unwrap();
}
