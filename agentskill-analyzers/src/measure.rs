use std::collections::BTreeMap;

use agentskill_core::Result;
use serde_json::{Map, Value, json};

use crate::common::{insert_language_result, percentile, repo_files, text};

pub fn run(repo: &str, lang: Option<&str>) -> Result<Value> {
    let (_root, files) = repo_files(repo, lang)?;

    let mut result = Map::new();
    let mut by_language: BTreeMap<&str, Vec<_>> = BTreeMap::new();

    for file in &files {
        by_language
            .entry(file.language.expect("language").id)
            .or_default()
            .push(file);
    }

    for (language, language_files) in by_language {
        insert_language_result(
            &mut result,
            language,
            measure_language(language, &language_files),
        );
    }

    Ok(Value::Object(result))
}

fn measure_language(language: &str, files: &[&agentskill_core::fs::RepoFile]) -> Value {
    let mut lengths = Vec::new();

    let mut spaces = Vec::new();
    let mut tabs = 0;

    let mut present = 0;
    let mut absent = 0;

    let mut trailing = 0;
    let mut tab_files = Vec::new();

    let mut mixed_files = Vec::new();

    for file in files {
        let raw = text(&file.path);

        let lines = raw.lines().collect::<Vec<_>>();
        if raw.ends_with('\n') {
            present += 1;
        } else {
            absent += 1;
        }

        if lines
            .iter()
            .any(|line| line.ends_with(' ') || line.ends_with('\t'))
        {
            trailing += 1;
        }

        let file_has_tabs = lines.iter().any(|line| line.starts_with('\t'));

        let file_has_spaces = lines.iter().any(|line| {
            let trimmed = line.trim_start();
            !trimmed.is_empty() && line.len() != trimmed.len()
        });

        if file_has_tabs {
            tabs += lines.iter().filter(|line| line.starts_with('\t')).count();
            tab_files.push(file.relative.clone());
        }

        if file_has_tabs && file_has_spaces {
            mixed_files.push(file.relative.clone());
        }

        for line in &lines {
            if line.trim().is_empty() {
                continue;
            }
            lengths.push(line.len());

            let count = line.len() - line.trim_start_matches(' ').len();
            if count > 0 {
                spaces.push(count);
            }
        }
    }

    let (unit, size) = if tabs > 0 && spaces.is_empty() {
        ("tabs", 1)
    } else if spaces.is_empty() {
        ("unknown", 0)
    } else {
        ("spaces", common_indent(&spaces))
    };

    let line_length = line_length(&mut lengths);
    let blank_lines = if language == "python" {
        python_blank_lines(files)
    } else {
        generic_blank_lines(files)
    };

    json!({
        "indentation": {
            "unit": unit,
            "size": size,
            "tab_files": tab_files,
            "mixed_files": mixed_files,
        },
        "line_length": line_length,
        "blank_lines": blank_lines,
        "trailing_newline": {"present": present, "absent": absent},
        "trailing_whitespace": {"files_with_trailing_ws": trailing},
    })
}

fn common_indent(values: &[usize]) -> usize {
    let mut counts = BTreeMap::new();

    for value in values {
        *counts.entry(*value).or_insert(0usize) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(value, count)| (*count, std::cmp::Reverse(*value)))
        .map_or(4, |(value, _)| value)
}

fn line_length(lengths: &mut [usize]) -> Value {
    if lengths.len() < 5 {
        return json!({});
    }
    json!({
        "p50": percentile(lengths, 50),
        "p75": percentile(lengths, 75),
        "p95": percentile(lengths, 95),
        "p99": percentile(lengths, 99),
        "max": lengths.iter().copied().max().unwrap_or(0),
    })
}

fn python_blank_lines(files: &[&agentskill_core::fs::RepoFile]) -> Value {
    let mut after_imports = Vec::new();

    let mut between_methods = Vec::new();
    let mut after_class = Vec::new();

    let mut between_top_level = Vec::new();
    for file in files {
        let lines = text(&file.path)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();

        let imports_end = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.trim_start().starts_with("import ") || line.trim_start().starts_with("from ")
            })
            .map(|(index, _)| index)
            .max();

        if let Some(index) = imports_end {
            after_imports.push(blank_run(&lines, index + 1));
        }

        let defs = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.trim_start().starts_with("def "))
            .map(|(index, line)| (index, line.starts_with(' ')))
            .collect::<Vec<_>>();

        for window in defs.windows(2) {
            let count = blank_run(&lines, window[0].0 + 1);

            if window[0].1 && window[1].1 {
                between_methods.push(count);
            } else {
                between_top_level.push(count);
            }
        }

        for (index, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("class ") {
                after_class.push(blank_run(&lines, index + 1));
            }
        }
    }
    json!({
        "after_imports": distribution(after_imports),
        "between_methods": distribution(between_methods),
        "after_class_declaration": distribution(after_class),
        "between_top_level_defs": distribution(between_top_level),
    })
}

fn generic_blank_lines(files: &[&agentskill_core::fs::RepoFile]) -> Value {
    let mut values = Vec::new();

    for file in files {
        let lines = text(&file.path)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();

        let definitions = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| is_top_level_definition(line))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        for window in definitions.windows(2) {
            values.push(blank_run(&lines, window[0] + 1));
        }
    }
    json!({"between_top_level_defs": distribution(values)})
}

fn is_top_level_definition(line: &str) -> bool {
    let trimmed = line.trim_start();
    !line.starts_with(' ')
        && !line.starts_with('\t')
        && (trimmed.starts_with("fn ")
            || trimmed.starts_with("func ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("export function ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("struct "))
}

fn blank_run(lines: &[String], start: usize) -> usize {
    lines
        .iter()
        .skip(start)
        .take_while(|line| line.trim().is_empty())
        .count()
}

fn distribution(values: Vec<usize>) -> Value {
    if values.is_empty() {
        return json!({});
    }

    let mut counts = BTreeMap::new();

    for value in values {
        *counts.entry(value).or_insert(0usize) += 1;
    }

    let mode = counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map_or(0, |(value, _)| *value);
    json!({"mode": mode, "distribution": counts})
}
