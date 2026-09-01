use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use agentskill_core::Result;
use agentskill_core::document::{normalize_section_name, parse};
use agentskill_core::error::AgentskillError;
use agentskill_core::fs::collect_files;
use serde_json::{Value, json};

const PROJECT_MARKERS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Package.swift",
    "pubspec.yaml",
    "mix.exs",
    "rebar.config",
    "build.sbt",
    "CMakeLists.txt",
    "Makefile",
    "makefile",
    "GNUmakefile",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScopeEntry {
    path: String,
    parent: String,
    markers: Vec<String>,
    operational: bool,
    reference: bool,
    managed: bool,
}

pub fn run(repo: &str, selected: Option<&[String]>) -> Result<Value> {
    let root = agentskill_core::error::validate_repo(repo)?;
    let all_entries = discover(&root)?;
    let mut entries = all_entries.clone();
    let mut resolution_entries = all_entries.clone();

    let entries = if let Some(selected) = selected {
        let selected = selected
            .iter()
            .map(|path| normalize_scope(&root, path))
            .collect::<Result<Vec<_>>>()?;
        let known = entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<BTreeSet<_>>();
        let candidates = entries
            .iter()
            .map(|entry| entry.path.clone())
            .chain(selected.iter().cloned())
            .collect::<BTreeSet<_>>();

        for path in selected.iter().filter(|path| !known.contains(*path)) {
            let entry = ScopeEntry {
                parent: nearest_parent(path, &candidates),
                path: path.clone(),
                markers: Vec::new(),
                operational: root.join(path).join("AGENTS.md").is_file(),
                reference: root.join(path).join("AGENTS.reference.md").is_file(),
                managed: path.is_empty() || has_scope_metadata(&root.join(path).join("AGENTS.md")),
            };
            entries.push(entry.clone());
            resolution_entries.push(entry);
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));

        entries
            .into_iter()
            .filter(|entry| selected.iter().any(|path| path == &entry.path))
            .collect()
    } else {
        entries
    };

    let manifest_entries = entries
        .into_iter()
        .map(|entry| scope_value(entry, &resolution_entries))
        .collect::<Vec<_>>();

    Ok(json!({
        "schema_version": 1,
        "repository": root,
        "scopes": manifest_entries,
    }))
}

pub fn normalize_scope(root: &Path, scope: &str) -> Result<String> {
    let path = Path::new(scope);
    if path.is_absolute() {
        return Err(AgentskillError::InvalidArgument(format!(
            "scope must be a repository-relative path: {scope}"
        )));
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => normalized.push(value),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(AgentskillError::InvalidArgument(format!(
                        "scope cannot escape the repository: {scope}"
                    )));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AgentskillError::InvalidArgument(format!(
                    "scope must be a repository-relative path: {scope}"
                )));
            }
        }
    }

    let path = if normalized.as_os_str().is_empty() {
        root.to_path_buf()
    } else {
        root.join(&normalized)
    };
    if !path.is_dir() {
        return Err(AgentskillError::InvalidPath(format!(
            "scope directory does not exist: {scope}"
        )));
    }

    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn discover(root: &Path) -> Result<Vec<ScopeEntry>> {
    let files = collect_files(root);
    // A nested document declaring `path: .` owns a separate repository-local
    // hierarchy; do not reinterpret its child fixtures as outer scopes.
    let nested_repositories = files
        .iter()
        .filter_map(|file| {
            let path = Path::new(&file.relative);
            if path.file_name().and_then(|value| value.to_str()) == Some("AGENTS.md") {
                path.parent()
                    .map(|parent| parent.to_string_lossy().replace('\\', "/"))
            } else {
                None
            }
        })
        .filter(|path| {
            !path.is_empty() && has_root_scope_metadata(&root.join(path).join("AGENTS.md"))
        })
        .collect::<BTreeSet<_>>();

    let mut paths = BTreeSet::from([String::new()]);
    let mut markers = std::collections::BTreeMap::<String, BTreeSet<String>>::new();
    let mut operational = BTreeSet::new();
    let mut references = BTreeSet::new();

    if root.join("AGENTS.md").is_file() {
        operational.insert(String::new());
    }
    if root.join("AGENTS.reference.md").is_file() {
        references.insert(String::new());
    }

    for file in files {
        if nested_repositories.iter().any(|repository| {
            file.relative == *repository || file.relative.starts_with(&format!("{repository}/"))
        }) {
            continue;
        }
        let path = Path::new(&file.relative);
        let Some(parent) = path.parent() else {
            continue;
        };
        let parent = parent.to_string_lossy().replace('\\', "/");
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if name == "AGENTS.md" {
            if !parent.is_empty() {
                paths.insert(parent.clone());
                operational.insert(parent.clone());
            }
        } else if name == "AGENTS.reference.md" && !parent.is_empty() {
            paths.insert(parent.clone());
            references.insert(parent.clone());
        }

        if PROJECT_MARKERS.contains(&name) && !parent.is_empty() {
            paths.insert(parent.clone());
            markers.entry(parent).or_default().insert(name.to_string());
        }
    }

    let candidates = paths.clone();
    let mut entries = paths
        .into_iter()
        .map(|path| {
            let parent = nearest_parent(&path, &candidates);
            let mut entry_markers = markers
                .remove(&path)
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            entry_markers.sort();
            ScopeEntry {
                operational: operational.contains(&path),
                reference: references.contains(&path),
                managed: path.is_empty() || has_scope_metadata(&root.join(&path).join("AGENTS.md")),
                parent,
                markers: entry_markers,
                path,
            }
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn has_root_scope_metadata(path: &Path) -> bool {
    std::fs::read_to_string(path).ok().is_some_and(|content| {
        parse(&content).sections.iter().any(|section| {
            section.level == 2
                && normalize_section_name(&section.heading) == "scope"
                && section.body.lines().any(|line| {
                    line.trim().eq_ignore_ascii_case("- path: .")
                        || line.trim().eq_ignore_ascii_case("path: .")
                })
        })
    })
}

fn nearest_parent(path: &str, candidates: &BTreeSet<String>) -> String {
    let mut current = PathBuf::from(path);
    while let Some(parent_path) = current.parent() {
        let parent = parent_path.to_string_lossy().replace('\\', "/");
        if candidates.contains(&parent) {
            return if parent.is_empty() {
                ".".into()
            } else {
                parent
            };
        }
        current = parent_path.to_path_buf();
    }
    ".".into()
}

fn scope_value(entry: ScopeEntry, all_entries: &[ScopeEntry]) -> Value {
    let path = if entry.path.is_empty() {
        "."
    } else {
        &entry.path
    };
    let document_prefix = if entry.path.is_empty() {
        String::new()
    } else {
        format!("{}/", entry.path)
    };
    let status = if entry.operational && entry.managed {
        "managed"
    } else if entry.operational || entry.reference {
        "legacy"
    } else {
        "candidate"
    };

    let ancestors = ancestor_paths(&entry.path, all_entries);
    let fallback = nearest_managed_ancestor(&entry.path, all_entries);

    json!({
        "path": path,
        "parent": entry.parent,
        "markers": entry.markers,
        "status": status,
        "resolution": {
            "ancestors": ancestors,
            "fallback": fallback,
            "precedence": "nearest-scope-wins",
        },
        "files": {
            "operational": entry.operational.then(|| format!("{document_prefix}AGENTS.md")),
            "reference": entry.reference.then(|| format!("{document_prefix}AGENTS.reference.md")),
        },
    })
}

fn ancestor_paths(path: &str, entries: &[ScopeEntry]) -> Vec<String> {
    if path.is_empty() {
        return Vec::new();
    }

    let by_path = entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut ancestors = Vec::new();
    let Some(entry) = by_path.get(path) else {
        return ancestors;
    };
    let mut current = internal_scope_path(&entry.parent);
    let mut visited = BTreeSet::new();
    while visited.insert(current.clone()) {
        let Some(entry) = by_path.get(current.as_str()) else {
            break;
        };
        ancestors.push(display_scope_path(&entry.path));
        if entry.path.is_empty() {
            break;
        }
        current = internal_scope_path(&entry.parent);
    }
    ancestors
}

fn nearest_managed_ancestor(path: &str, entries: &[ScopeEntry]) -> String {
    let by_path = entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let Some(entry) = by_path.get(path) else {
        return ".".into();
    };
    let mut current = internal_scope_path(&entry.parent);
    let mut visited = BTreeSet::new();
    while visited.insert(current.clone()) {
        let Some(entry) = by_path.get(current.as_str()) else {
            break;
        };
        if entry.operational && entry.managed {
            return display_scope_path(&entry.path);
        }
        if entry.path.is_empty() {
            break;
        }
        current = internal_scope_path(&entry.parent);
    }
    ".".into()
}

fn display_scope_path(path: &str) -> String {
    if path.is_empty() {
        ".".into()
    } else {
        path.into()
    }
}

fn internal_scope_path(path: &str) -> String {
    if path == "." {
        String::new()
    } else {
        path.into()
    }
}

fn has_scope_metadata(path: &Path) -> bool {
    std::fs::read_to_string(path).ok().is_some_and(|content| {
        parse(&content).sections.iter().any(|section| {
            section.level == 2 && normalize_section_name(&section.heading) == "scope"
        })
    })
}
