//! Deterministic AGENTS.md rendering and update workflows.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use agentskill_analyzers::run_all;
use agentskill_core::document::{Document, Section, merge, normalize_section_name, serialize};
use agentskill_core::error::{Result, validate_repo};
use agentskill_core::fs::read_text;
use serde_json::Value;

pub const TITLE: &str = "# AGENTS.md\n\n";
pub const SECTION_ORDER: &[&str] = &[
    "overview",
    "repository structure",
    "service map",
    "cross-service boundaries",
    "commands and workflows",
    "code formatting",
    "naming conventions",
    "type annotations",
    "imports",
    "error handling",
    "comments and docstrings",
    "testing",
    "git",
    "dependencies and tooling",
    "red lines",
];

const SECTION_HEADINGS: &[(&str, &str)] = &[
    ("overview", "1. Overview"),
    ("repository structure", "2. Repository Structure"),
    ("service map", "3. Service Map"),
    ("cross-service boundaries", "4. Cross-Service Boundaries"),
    ("commands and workflows", "5. Commands and Workflows"),
    ("code formatting", "6. Code Formatting"),
    ("naming conventions", "7. Naming Conventions"),
    ("type annotations", "8. Type Annotations"),
    ("imports", "9. Imports"),
    ("error handling", "10. Error Handling"),
    ("comments and docstrings", "11. Comments and Docstrings"),
    ("testing", "12. Testing"),
    ("git", "13. Git"),
    ("dependencies and tooling", "14. Dependencies and Tooling"),
    ("red lines", "15. Red Lines"),
];

const SECTION_NUMBERS: &[(&str, usize)] = &[
    ("overview", 1),
    ("repository structure", 2),
    ("service map", 3),
    ("cross-service boundaries", 4),
    ("commands and workflows", 5),
    ("code formatting", 6),
    ("naming conventions", 7),
    ("type annotations", 8),
    ("imports", 9),
    ("error handling", 10),
    ("comments and docstrings", 11),
    ("testing", 12),
    ("git", 13),
    ("dependencies and tooling", 14),
    ("red lines", 15),
];

pub fn validate_profile(profile: &str) -> Result<&str> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "concise" => Ok("concise"),
        "comprehensive" => Ok("comprehensive"),
        _ => Err(agentskill_core::AgentskillError::InvalidArgument(format!(
            "unsupported output profile: {profile:?} (allowed: concise, comprehensive)"
        ))),
    }
}

pub fn validate_layout(layout: &str) -> Result<&str> {
    match layout.trim().to_ascii_lowercase().as_str() {
        "single" => Ok("single"),
        "split" => Ok("split"),
        "multifile" => Ok("multifile"),
        _ => Err(agentskill_core::AgentskillError::InvalidArgument(format!(
            "unsupported output layout: {layout:?} (allowed: single, split, multifile)"
        ))),
    }
}

pub fn render(
    repo: &Path,
    profile: &str,
    references: &[String],
    interactive: bool,
) -> Result<String> {
    render_with_answers(repo, profile, references, interactive, &BTreeMap::new())
}

pub fn render_with_answers(
    repo: &Path,
    profile: &str,
    references: &[String],
    interactive: bool,
    answers: &BTreeMap<String, String>,
) -> Result<String> {
    let profile = validate_profile(profile)?;

    let reference_documents = agentskill_core::reference::load_reference_documents(references)?;
    let analysis = run_all(repo.to_string_lossy().as_ref(), None);

    let mut sections = render_sections(repo, &analysis, profile)?;
    apply_interactive_answers(&mut sections, answers);

    if interactive && answers.is_empty() {
        sections.push(section(
            "Interactive Notes",
            "Review repository-specific conventions before committing this document.\n",
        ));
    }

    let mut document = Document {
        preamble: TITLE.into(),
        sections,
    };

    if !reference_documents.is_empty() {
        document
            .preamble
            .push_str(&reference_metadata(&reference_documents));
        document.preamble.push_str("\n\n");
    }

    if !references.is_empty() {
        document.preamble.push_str("> References: ");
        document.preamble.push_str(&references.join(", "));
        document.preamble.push_str("\n\n");
    }

    Ok(serialize(&document))
}

pub fn collect_interactive_answers(
    repo: &str,
    references: &[String],
) -> Result<BTreeMap<String, String>> {
    let root = validate_repo(repo)?;

    let documents = agentskill_core::reference::load_reference_documents(references)?;
    let analysis = run_all(root.to_string_lossy().as_ref(), None);

    let reference_text = documents
        .iter()
        .map(|document| document.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let mut gaps = Vec::new();
    let has_test_command = analysis["tests"]
        .as_object()
        .into_iter()
        .flat_map(|items| items.values())
        .any(|item| {
            item["run_command"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
        });

    if !has_test_command {
        gaps.push((
            "test_command",
            "I couldn't determine the canonical test command. Enter it, or press Enter to skip: ",
            extract_reference(&reference_text, "Run command"),
        ));
    }

    if analysis["git"]["error"].is_string() {
        gaps.push((
            "commit_prefixes",
            "Git history is unavailable. Enter preferred commit prefixes, or press Enter to skip: ",
            extract_reference(&reference_text, "Commit prefixes observed"),
        ));
        gaps.push((
            "merge_strategy",
            "Git history is unavailable. Enter the preferred merge strategy, or press Enter to skip: ",
            extract_reference(&reference_text, "Merge strategy"),
        ));
    }

    let stdin = io::stdin();

    let mut input = stdin.lock();
    let mut answers = BTreeMap::new();

    for (key, prompt, inferred) in gaps {
        if let Some(value) = inferred {
            answers.insert(key.to_string(), value);
            continue;
        }
        eprint!("{prompt}");
        io::stderr().flush()?;

        let mut answer = String::new();
        input.read_line(&mut answer)?;

        let answer = answer.trim();
        if !answer.is_empty() {
            answers.insert(key.to_string(), answer.to_string());
        }
    }

    Ok(answers)
}

fn reference_metadata(documents: &[agentskill_core::reference::ReferenceDocument]) -> String {
    let references = documents
        .iter()
        .map(|document| {
            let mut value = serde_json::Map::new();
            value.insert("kind".into(), Value::String(document.source.kind.clone()));
            value.insert("value".into(), Value::String(document.source.value.clone()));
            value.insert(
                "source_path".into(),
                Value::String(document.source_path.clone()),
            );

            if let Some(sha) = &document.commit_sha {
                value.insert("commit_sha".into(), Value::String(sha.clone()));
            }
            Value::Object(value)
        })
        .collect::<Vec<_>>();

    let metadata = serde_json::json!({
        "agentskill_version": env!("CARGO_PKG_VERSION"),
        "references": references,
    });

    format!(
        "<!-- agentskill-metadata\n{}\n-->",
        serde_json::to_string_pretty(&metadata).unwrap_or_default()
    )
}

fn extract_reference(text: &str, label: &str) -> Option<String> {
    let needle = format!("{label}:");
    text.lines().find_map(|line| {
        let value = line.split_once(&needle)?.1.trim();

        let value = value.strip_prefix('`')?.split('`').next()?;
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn apply_interactive_answers(sections: &mut [Section], answers: &BTreeMap<String, String>) {
    let mut notes = BTreeMap::<&str, Vec<String>>::new();

    if let Some(command) = answers.get("test_command") {
        notes
            .entry("Testing")
            .or_default()
            .push(format!("Use `{command}` as the canonical test command."));
        notes
            .entry("Commands and Workflows")
            .or_default()
            .push(format!("Use `{command}` as the canonical test command."));
    }

    if let Some(prefixes) = answers.get("commit_prefixes") {
        notes
            .entry("Git")
            .or_default()
            .push(format!("Preferred commit prefixes: `{prefixes}`."));
    }

    if let Some(strategy) = answers.get("merge_strategy") {
        notes
            .entry("Git")
            .or_default()
            .push(format!("Preferred merge strategy: `{strategy}`."));
    }

    for section in sections {
        let key = normalize_section_name(&section.heading);

        if let Some(entries) = notes
            .iter()
            .find(|(heading, _)| normalize_section_name(heading) == key)
            .map(|(_, entries)| entries)
        {
            let prefix = format!(
                "Interactive Answers:\n{}\n\n",
                entries
                    .iter()
                    .map(|entry| format!("- {entry}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            section.body = format!("{prefix}{}", section.body);
        }
    }
}

pub fn generate(
    repo: &str,
    out: Option<&str>,
    references: &[String],
    interactive: bool,
    profile: &str,
    layout: &str,
) -> Result<()> {
    generate_with_answers(
        repo,
        out,
        references,
        interactive,
        profile,
        layout,
        &BTreeMap::new(),
    )
}

pub fn generate_with_answers(
    repo: &str,
    out: Option<&str>,
    references: &[String],
    interactive: bool,
    profile: &str,
    layout: &str,
    answers: &BTreeMap<String, String>,
) -> Result<()> {
    let root = validate_repo(repo)?;

    let profile = validate_profile(profile)?;
    let layout = validate_layout(layout)?;

    let markdown = render_with_answers(&root, profile, references, interactive, answers)?;
    match layout {
        "single" => write_or_print(out.map(PathBuf::from), markdown)?,
        "split" => {
            let primary = out
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("AGENTS.md"));

            let companion = companion_path(&primary);
            let linked = format!(
                "{}\nSee [the comprehensive reference](./{}).\n",
                markdown.trim_end(),
                companion.file_name().unwrap_or_default().to_string_lossy()
            );
            write_file(&primary, linked)?;

            let comprehensive =
                render_with_answers(&root, "comprehensive", references, interactive, answers)?;

            let comprehensive = comprehensive
                .strip_prefix(TITLE)
                .map_or(comprehensive.clone(), |body| {
                    format!("# AGENTS Reference\n\n{body}")
                });
            write_file(&companion, comprehensive)?
        }
        "multifile" => {
            let primary = out
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("AGENTS.md"));
            let reference_documents =
                agentskill_core::reference::load_reference_documents(references)?;
            let analysis = run_all(root.to_string_lossy().as_ref(), None);

            let dir = primary.parent().unwrap_or(&root).join(".agentskill");
            fs::create_dir_all(&dir)?;

            let mut sections = render_sections(&root, &analysis, profile)?;
            apply_interactive_answers(&mut sections, answers);

            let mut index = String::from("# AGENTS.md\n\n");
            for section in &sections {
                let key = normalize_section_name(&section.heading);

                let number = SECTION_NUMBERS
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map_or(0, |(_, number)| *number);

                if number == 0 {
                    continue;
                }

                let filename_heading = section
                    .heading
                    .split_once('.')
                    .map_or(section.heading.as_str(), |(_, value)| value.trim());

                let filename = format!(
                    "{:02}_{}.md",
                    number,
                    filename_heading.replace(' ', "_").to_ascii_uppercase()
                );
                index.push_str(&format!(
                    "- [{}](.agentskill/{filename})\n",
                    section.heading
                ));
                write_file(
                    &dir.join(&filename),
                    format!("# {}\n\n{}", section.heading, section.body),
                )?;
            }

            if !reference_documents.is_empty() {
                let metadata = reference_metadata(&reference_documents);
                index = format!(
                    "# AGENTS.md\n\n{metadata}\n\n{}",
                    index.trim_start_matches("# AGENTS.md\n\n")
                );
            }
            write_file(&primary, index)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

pub fn update(
    repo: &str,
    out: Option<&str>,
    only: &[String],
    exclude: &[String],
    force: bool,
    profile: &str,
    layout: &str,
) -> Result<()> {
    let layout = validate_layout(layout)?;

    if layout != "single" {
        return Err(agentskill_core::AgentskillError::InvalidArgument(format!(
            "update with layout '{layout}' is not implemented yet"
        )));
    }

    let root = validate_repo(repo)?;

    let profile = validate_profile(profile)?;
    let feedback = load_feedback(&root)?;

    let mut effective_exclude = exclude.to_vec();
    if !force {
        effective_exclude.extend(feedback.preserve_sections.iter().cloned());
    }

    let mut generated_sections = render_sections(
        &root,
        &run_all(root.to_string_lossy().as_ref(), None),
        profile,
    )?;
    apply_feedback(&mut generated_sections, &feedback);
    validate_requested_sections(only, &effective_exclude, &generated_sections)?;

    let generated = Document {
        preamble: TITLE.into(),
        sections: generated_sections,
    };

    let existing_path = root.join("AGENTS.md");
    let existing = if existing_path.exists() {
        read_text(&existing_path)
    } else {
        String::new()
    };

    let result = merge(&existing, &generated, only, &effective_exclude, force);
    write_file(&out.map(PathBuf::from).unwrap_or(existing_path), result)
}

#[derive(Default)]
struct Feedback {
    sections: BTreeMap<String, FeedbackSection>,
    preserve_sections: Vec<String>,
}

#[derive(Default)]
struct FeedbackSection {
    prepend_notes: Vec<String>,
    pinned_facts: Vec<String>,
}

fn load_feedback(root: &Path) -> Result<Feedback> {
    let path = root.join(".agentskill-feedback.json");

    if !path.exists() {
        return Ok(Feedback::default());
    }

    let value: Value = serde_json::from_str(&read_text(&path)).map_err(|error| {
        agentskill_core::AgentskillError::InvalidArgument(format!("invalid feedback JSON: {error}"))
    })?;

    let object = value.as_object().ok_or_else(|| {
        agentskill_core::AgentskillError::InvalidArgument("feedback must be an object".into())
    })?;

    let mut feedback = Feedback::default();
    if let Some(preserve) = object.get("preserve_sections") {
        feedback.preserve_sections = string_list(preserve, "feedback.preserve_sections")?
            .into_iter()
            .map(|name| normalize_section_name(&name))
            .collect();
        feedback.preserve_sections.sort();
        feedback.preserve_sections.dedup();
    }

    if let Some(sections) = object.get("sections") {
        let sections = sections.as_object().ok_or_else(|| {
            agentskill_core::AgentskillError::InvalidArgument(
                "feedback.sections must be an object".into(),
            )
        })?;

        for (name, value) in sections {
            let value = value.as_object().ok_or_else(|| {
                agentskill_core::AgentskillError::InvalidArgument(format!(
                    "feedback.sections.{name} must be an object"
                ))
            })?;

            let key = normalize_section_name(name);
            if feedback.sections.contains_key(&key) {
                return Err(agentskill_core::AgentskillError::InvalidArgument(format!(
                    "duplicate feedback section after normalization: {name}"
                )));
            }
            feedback.sections.insert(
                key,
                FeedbackSection {
                    prepend_notes: value
                        .get("prepend_notes")
                        .map(|value| {
                            string_list(value, &format!("feedback.sections.{name}.prepend_notes"))
                        })
                        .transpose()?
                        .unwrap_or_default(),
                    pinned_facts: value
                        .get("pinned_facts")
                        .map(|value| {
                            string_list(value, &format!("feedback.sections.{name}.pinned_facts"))
                        })
                        .transpose()?
                        .unwrap_or_default(),
                },
            );
        }
    }

    Ok(feedback)
}

fn string_list(value: &Value, label: &str) -> Result<Vec<String>> {
    value
        .as_array()
        .ok_or_else(|| {
            agentskill_core::AgentskillError::InvalidArgument(format!(
                "{label} must be a list of strings"
            ))
        })?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                agentskill_core::AgentskillError::InvalidArgument(format!(
                    "{label} must be a list of strings"
                ))
            })
        })
        .collect()
}

fn apply_feedback(sections: &mut [Section], feedback: &Feedback) {
    for section in sections {
        let key = normalize_section_name(&section.heading);

        let Some(feedback_section) = feedback.sections.get(&key) else {
            continue;
        };

        let mut notes = Vec::new();
        if !feedback_section.prepend_notes.is_empty() {
            notes.push(format!(
                "Maintainer Notes From `.agentskill-feedback.json`:\n{}",
                feedback_section
                    .prepend_notes
                    .iter()
                    .map(|note| format!("- {note}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !feedback_section.pinned_facts.is_empty() {
            notes.push(format!(
                "Pinned Facts From `.agentskill-feedback.json`:\n{}",
                feedback_section
                    .pinned_facts
                    .iter()
                    .map(|fact| format!("- {fact}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        if !notes.is_empty() {
            section.body = format!("{}\n\n{}", notes.join("\n\n"), section.body);
        }
    }
}

fn validate_requested_sections(
    only: &[String],
    exclude: &[String],
    generated: &[Section],
) -> Result<()> {
    let supported = generated
        .iter()
        .map(|section| normalize_section_name(&section.heading))
        .collect::<std::collections::HashSet<_>>();

    for name in only.iter().chain(exclude) {
        let key = normalize_section_name(name);

        if !supported.contains(&key) {
            return Err(agentskill_core::AgentskillError::InvalidArgument(format!(
                "unsupported or unavailable section: {name}"
            )));
        }
    }

    Ok(())
}

fn render_sections(repo: &Path, analysis: &Value, profile: &str) -> Result<Vec<Section>> {
    let summary = &analysis["scan"]["summary"];

    let languages = summary["by_language"]
        .as_object()
        .map(|items| items.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let language_summary = if languages.is_empty() {
        "none".to_string()
    } else {
        languages.join(", ")
    };

    let mut sections = vec![
        section(
            "1. Overview",
            format!(
                "agentskill analyzes repositories and synthesizes precise `AGENTS.md` guidance.\n\n- Repository: `{}`\n- Languages detected: {}\n",
                repo.display(),
                language_summary
            ),
        ),
        section(
            "2. Repository Structure",
            format!(
                "The repository contains {} analyzed source files.\n\n```text\n{}\n```\n\nRead order is derived from file size and entrypoint priority.\n",
                summary["total_files"].as_u64().unwrap_or(0),
                top_level_layout(summary, analysis)
            ),
        ),
        section("5. Commands and Workflows", commands_body(analysis)),
        section("6. Code Formatting", formatting_body(analysis)),
        section("7. Naming Conventions", naming_body(analysis)),
        section(
            "8. Type Annotations",
            "Use the type system and annotation style established by the repository's source and configuration.\n",
        ),
        section(
            "9. Imports",
            "Keep imports grouped and ordered consistently with the existing source files.\n",
        ),
        section(
            "10. Error Handling",
            "Handle expected failures at the command boundary and preserve machine-readable error output.\n",
        ),
        section(
            "11. Comments and Docstrings",
            "Write comments and documentation only where they clarify behavior that is not obvious from the code.\n",
        ),
        section("12. Testing", testing_body(analysis, languages.len())),
        section("13. Git", git_body(analysis)),
        section(
            "14. Dependencies and Tooling",
            "Use the repository's declared dependency manager and lockfile. Keep tooling configuration version-controlled.\n",
        ),
        section(
            "15. Red Lines",
            "Do not change public contracts, generated-document semantics, or repository-specific conventions without updating their tests and documentation.\n",
        ),
    ];

    let boundaries = analysis["graph"]["monorepo_boundaries"].clone();
    if boundaries["detected"].as_bool().unwrap_or(false) {
        let services = boundaries["services"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let service_lines = services
            .iter()
            .filter_map(Value::as_str)
            .map(|service| format!("- `{service}`: service root at `{service}`"))
            .collect::<Vec<_>>();
        sections.insert(
            2,
            section("3. Service Map", format!("{}\n", service_lines.join("\n"))),
        );

        let imports = boundaries["cross_service_imports"]
            .as_array()
            .is_some_and(|items| !items.is_empty());
        sections.insert(
            3,
            section(
                "4. Cross-Service Boundaries",
                if imports {
                    "- Cross-service imports were detected; review shared contracts before changing service boundaries.\n"
                } else {
                    "- No cross-service imports were detected; preserve service boundaries unless a shared contract layer is introduced.\n"
                },
            ),
        );
    }

    for item in &mut sections {
        if let Some((_, heading)) = SECTION_HEADINGS
            .iter()
            .find(|(key, _)| normalize_section_name(&item.heading) == *key)
        {
            item.heading = (*heading).into();
        }
    }

    if profile == "comprehensive" {
        for section in &mut sections {
            section.body.push_str(
                "\nVerify this rule against representative source files before making a change.\n",
            );
        }
    }

    Ok(sections)
}

fn top_level_layout(summary: &Value, analysis: &Value) -> String {
    let mut groups = BTreeMap::<String, usize>::new();

    if let Some(tree) = analysis["scan"]["tree"].as_array() {
        for entry in tree {
            if let Some(path) = entry["path"].as_str() {
                let root = path.split('/').next().unwrap_or(path);
                *groups.entry(root.to_string()).or_default() += 1;
            }
        }
    }

    if groups.is_empty() {
        return format!("# {} files", summary["total_files"].as_u64().unwrap_or(0));
    }
    groups
        .into_iter()
        .map(|(name, count)| format!("{name}  # {count} files"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn commands_body(analysis: &Value) -> String {
    let mut commands = Vec::new();

    if let Some(items) = analysis["tests"].as_object() {
        for value in items.values() {
            if let Some(command) = value["run_command"].as_str()
                && !command.is_empty()
                && !commands.iter().any(|item| item == command)
            {
                commands.push(command.to_string());
            }
        }
    }

    if commands.is_empty() {
        commands.push("No canonical test command was detected.".into());
    }

    format!(
        "```bash\n{}\n```\n\n- Keep local verification aligned with the repository's configured test and tooling commands.\n",
        commands.join("\n")
    )
}

fn formatting_body(analysis: &Value) -> String {
    let mut body = String::new();

    if let Some(languages) = analysis["config"].as_object() {
        for (language, config) in languages {
            if language == "editorconfig" {
                continue;
            }

            let tools = ["formatter", "linter", "type_checker"]
                .iter()
                .filter_map(|kind| config[*kind]["name"].as_str().map(|name| (*kind, name)))
                .collect::<Vec<_>>();

            if tools.is_empty() {
                continue;
            }
            body.push_str(&format!("### {}\n\n", title_case(language)));

            for (kind, name) in tools {
                body.push_str(&format!("- Use `{name}` as the configured {kind}.\n"));
            }
            body.push('\n');
        }
    }

    if body.is_empty() {
        body.push_str("No formatter or linter configuration was detected.\n");
    }
    body
}

fn naming_body(analysis: &Value) -> String {
    let mut lines = vec![
        "Match the dominant identifier and file naming patterns already present in each language."
            .into(),
    ];

    if let Some(languages) = analysis["symbols"].as_object() {
        for (language, symbols) in languages {
            let mut patterns = Vec::new();

            for kind in ["functions", "classes", "constants"] {
                if let Some(items) = symbols[kind]["patterns"].as_object() {
                    patterns.extend(items.keys().cloned());
                }
            }
            patterns.sort();
            patterns.dedup();

            if !patterns.is_empty() {
                lines.push(format!(
                    "- `{}` uses observed patterns: `{}`.",
                    title_case(language),
                    patterns.join("`, `")
                ));
            }
        }
    }

    format!("{}\n", lines.join("\n"))
}

fn testing_body(analysis: &Value, language_count: usize) -> String {
    let mut lines = vec![format!(
        "Run the detected test commands and preserve coverage for the {language_count} detected language families."
    )];

    if let Some(items) = analysis["tests"].as_object() {
        for (language, value) in items {
            let framework = value["framework"].as_str().unwrap_or("unknown");

            let command = value["run_command"].as_str().unwrap_or("unknown");
            lines.push(format!(
                "- `{}`: `{framework}` via `{command}`; {} source files and {} test files.",
                title_case(language),
                value["source_files"].as_u64().unwrap_or(0),
                value["test_files"].as_u64().unwrap_or(0)
            ));
        }
    }

    format!("{}\n", lines.join("\n"))
}

fn git_body(analysis: &Value) -> String {
    let git = &analysis["git"];

    if let Some(prefixes) = git["prefixes"].as_object() {
        let names = prefixes.keys().cloned().collect::<Vec<_>>().join(", ");

        return format!(
            "Observed commit prefixes include `{names}`. Preserve the repository's branch and merge conventions.\n"
        );
    }
    "Git history was unavailable; confirm commit, branch, and merge conventions before contributing.\n".into()
}

fn title_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn section(heading: &str, body: impl Into<String>) -> Section {
    Section {
        level: 2,
        heading: heading.into(),
        body: body.into(),
    }
}

fn companion_path(primary: &Path) -> PathBuf {
    primary.with_file_name("AGENTS.reference.md")
}
fn write_or_print(path: Option<PathBuf>, text: String) -> Result<()> {
    match path {
        Some(path) => write_file(&path, text),
        None => {
            print!("{text}");

            Ok(())
        }
    }
}
fn write_file(path: &Path, text: String) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;

    Ok(())
}
