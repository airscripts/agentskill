use std::collections::BTreeMap;
use std::path::Path;

use agentskill_core::Result;
use serde_json::json;

use crate::common::repo_files;

const ENTRY_POINT_NAMES: &[&str] = &[
    "main", "cli", "app", "index", "server", "cmd", "__main__", "manage", "wsgi", "asgi", "run",
];

pub fn run(repo: &str, lang: Option<&str>) -> Result<serde_json::Value> {
    let (_root, files) = repo_files(repo, lang)?;

    let mut tree = Vec::new();
    let mut summary: BTreeMap<String, serde_json::Value> = BTreeMap::new();

    for file in &files {
        let language = file.language.expect("filtered files have a language").id;
        tree.push(json!({"path": file.relative, "type": "file", "language": language, "size_bytes": file.bytes, "line_count": file.lines, "depth": Path::new(&file.relative).components().count()}));

        let entry = summary
            .entry(language.to_string())
            .or_insert_with(|| json!({"file_count": 0, "total_lines": 0}));
        entry["file_count"] = json!(entry["file_count"].as_u64().unwrap_or(0) + 1);
        entry["total_lines"] =
            json!(entry["total_lines"].as_u64().unwrap_or(0) + file.lines as u64);
    }

    let mut order = files
        .iter()
        .map(|file| {
            let stem = Path::new(&file.relative)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            (
                !ENTRY_POINT_NAMES.contains(&stem.as_str()),
                file.relative.clone(),
                file.lines,
            )
        })
        .collect::<Vec<_>>();
    order.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then(right.2.cmp(&left.2))
            .then(left.1.cmp(&right.1))
    });

    let depths = files
        .iter()
        .map(|file| Path::new(&file.relative).components().count())
        .collect::<Vec<_>>();

    let max_depth = depths.iter().copied().max().unwrap_or(0);
    let avg_depth = if depths.is_empty() {
        0.0
    } else {
        (depths.iter().sum::<usize>() as f64 / depths.len() as f64 * 10.0).round() / 10.0
    };

    Ok(
        json!({"tree": tree, "summary": {"total_files": files.len(), "by_language": summary, "max_depth": max_depth, "avg_depth": avg_depth}, "read_order": order.into_iter().map(|item| item.1).collect::<Vec<_>>() }),
    )
}
