use std::fs;
use std::path::{Path, PathBuf};

use agentskill_core::document::parse;
use agentskill_core::error::{AgentskillError, error_payload, validate_repo};
use agentskill_core::fs::{FileRole, collect_files, line_count, read_text};
use agentskill_core::language::{
    LANGUAGES, LanguageKind, LanguageRole, is_test_path, language_by_id, language_for_content,
    language_for_file, language_for_path, language_kind, language_role,
};
use agentskill_core::output::{pretty_json, validate_out_path, write_value};
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
fn treats_agents_title_as_preamble_and_preserves_custom_markdown() {
    let document =
        parse("# AGENTS.md\n\n## Custom\n\nKeep this rule.\n\n### Detail\n\nMore context.\n");

    assert_eq!(document.preamble, "# AGENTS.md\n\n");
    assert_eq!(document.sections.len(), 2);
    assert_eq!(document.sections[0].heading, "Custom");
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

    assert_eq!(collect_files(directory.path()).len(), 2);

    let file_path = source.to_string_lossy().into_owned();

    let error = validate_repo(&file_path).unwrap_err();
    assert!(matches!(error, AgentskillError::InvalidPath(_)));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&source, directory.path().join("linked.rs")).unwrap();

        assert_eq!(collect_files(directory.path()).len(), 2);
    }
}

#[test]
fn classifies_repository_file_roles_for_evidence() {
    let directory = tempdir().unwrap();
    for (path, content) in [
        ("src/main.rs", "fn main() {}\n"),
        ("tests/main_test.rs", "#[test]\nfn works() {}\n"),
        ("examples/sample.rs", "fn sample() {}\n"),
        ("docs/guide.md", "# Guide\n"),
        ("Cargo.toml", "[package]\nname = \"sample\"\n"),
        ("data.json", "{}\n"),
        ("generated/output.rs", "fn generated() {}\n"),
    ] {
        let path = directory.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    let files = collect_files(directory.path());
    let role = |path: &str| {
        files
            .iter()
            .find(|file| file.relative == path)
            .map(|file| file.role)
            .unwrap()
    };

    assert_eq!(role("src/main.rs"), FileRole::Source);
    assert_eq!(role("tests/main_test.rs"), FileRole::Test);
    assert_eq!(role("examples/sample.rs"), FileRole::Example);
    assert_eq!(role("docs/guide.md"), FileRole::Documentation);
    assert_eq!(role("Cargo.toml"), FileRole::Configuration);
    assert_eq!(role("data.json"), FileRole::Auxiliary);
    assert_eq!(role("generated/output.rs"), FileRole::Generated);
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
fn serializes_output_and_formats_error_payloads() {
    let value = json!({"ok": true});

    assert!(pretty_json(&value, true).contains("\n"));
    let output = format!("agentskill-output-{}.json", std::process::id());
    write_value(&value, false, Some(&output)).unwrap();

    assert_eq!(fs::read_to_string(&output).unwrap(), "{\"ok\":true}\n");
    fs::remove_file(output).unwrap();
    assert_eq!(error_payload("bad", "scan")["script"], "scan");
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
