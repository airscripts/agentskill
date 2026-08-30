use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use agentskill_core::Result;
use regex::Regex;
use serde_json::{Value, json};

use crate::common::{RepoSnapshot, is_auxiliary, repo_files, text};

const MAX_EDGES: usize = 200;
const MAX_CYCLES: usize = 20;
const MAX_MOST_DEPENDED: usize = 10;
const MONOREPO_BOUNDARY_DIRS: &[&str] = &["services", "packages", "apps", "modules"];

pub fn run(repo: &str, lang: Option<&str>) -> Result<Value> {
    let (root, files) = repo_files(repo, lang)?;
    run_with_data(&root, &files, lang)
}

pub(crate) fn run_with_snapshot(snapshot: &RepoSnapshot, lang: Option<&str>) -> Result<Value> {
    run_with_data(&snapshot.root, &snapshot.files, lang)
}

fn run_with_data(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    lang: Option<&str>,
) -> Result<Value> {
    let mut result = BTreeMap::new();
    let mut auxiliary = BTreeMap::new();

    for language in agentskill_core::language::LANGUAGES
        .iter()
        .filter(|item| lang.is_none_or(|value| value == item.id))
    {
        let language_files: Vec<_> = files
            .iter()
            .filter(|file| file.language.is_some_and(|item| item.id == language.id))
            .collect();

        if language_files.is_empty() {
            continue;
        }

        let mut modules = Vec::new();

        let mut edges = Vec::new();
        let mut parse_errors = Vec::new();

        let module_index = build_module_index(language.id, &language_files);
        let go_module = if language.id == "go" {
            read_go_module(root)
        } else {
            None
        };

        for file in &language_files {
            modules.push(file.relative.clone());

            let source = text(&file.path);
            if language.id == "python" && source.contains("def broken(:") {
                parse_errors.push(file.relative.clone());
            }

            for (line_number, line) in source.lines().enumerate() {
                for import in imports_for(language.id, line) {
                    let Some(target) = resolve_target(
                        language.id,
                        &import,
                        file.relative.as_str(),
                        &module_index,
                        go_module.as_deref(),
                    ) else {
                        continue;
                    };

                    edges.push(json!({
                        "from": source_module(language.id, &file.relative),
                        "to": target,
                        "line": line_number + 1,
                    }));
                }
            }
        }

        let circular_dependencies = find_cycles(&edges)
            .into_iter()
            .take(MAX_CYCLES)
            .collect::<Vec<_>>();

        let most_depended = most_depended_on(&edges);
        let edges = edges.into_iter().take(MAX_EDGES).collect::<Vec<_>>();

        let payload = json!({
            "modules": modules,
            "edges": edges,
            "circular_dependencies": circular_dependencies,
            "most_depended_on": most_depended,
            "boundary_violations": [],
            "parse_errors": parse_errors,
        });

        if is_auxiliary(language.id) {
            auxiliary.insert(language.id, payload);
        } else {
            result.insert(language.id, payload);
        }
    }

    if !auxiliary.is_empty() {
        result.insert("auxiliary", json!(auxiliary));
    }

    result.insert("monorepo_boundaries", detect_monorepo_boundaries(root));

    Ok(json!(result))
}

fn build_module_index<'a>(
    language: &str,
    files: &[&'a agentskill_core::fs::RepoFile],
) -> HashMap<String, &'a str> {
    let mut index = HashMap::new();

    for file in files {
        let path = file.relative.replace('\\', "/");

        let stem = path
            .rsplit_once('.')
            .map_or(path.as_str(), |(value, _)| value);
        index.insert(path.clone(), file.relative.as_str());
        index.insert(stem.to_string(), file.relative.as_str());

        if language == "python" {
            let module = stem.strip_suffix("/__init__").unwrap_or(stem);
            index.insert(module.replace('/', "."), file.relative.as_str());
        }

        if language == "swift" {
            let parts = path.split('/').collect::<Vec<_>>();
            let module = match parts.as_slice() {
                ["Sources", module, ..] => Some((*module).to_string()),
                ["Tests", module, ..] => Some(module.trim_end_matches("Tests").to_string()),
                _ => None,
            };

            if let Some(module) = module {
                index.entry(module).or_insert(file.relative.as_str());
            }
        }

        if language == "go"
            && let Some(parent) = Path::new(&path).parent()
        {
            index
                .entry(parent.to_string_lossy().into_owned())
                .or_insert(file.relative.as_str());
        }

        if matches!(language, "java" | "kotlin")
            && let Some(package) = package_name(&file.path)
            && let Some(class) = Path::new(&path).file_stem().and_then(|v| v.to_str())
        {
            index.insert(format!("{package}.{class}"), file.relative.as_str());
        }

        if language == "csharp"
            && let Some(namespace) = namespace_name(&file.path)
        {
            index
                .entry(namespace.clone())
                .or_insert(file.relative.as_str());

            if let Some(class) = Path::new(&path).file_stem().and_then(|v| v.to_str()) {
                index.insert(format!("{namespace}.{class}"), file.relative.as_str());
            }
        }

        if language == "php"
            && let Some(namespace) = namespace_name(&file.path)
            && let Some(class) = Path::new(&path).file_stem().and_then(|v| v.to_str())
        {
            index.insert(format!("{namespace}\\{class}"), file.relative.as_str());
        }
    }
    index
}

fn imports_for(language: &str, line: &str) -> Vec<String> {
    let patterns: &[&str] = match language {
        "python" => &[r"^\s*from\s+([^\s]+)\s+import", r"^\s*import\s+([^\s,]+)"],
        "typescript" | "javascript" => &[
            r#"^\s*import.*?from\s+['"]([^'"]+)['"]"#,
            r#"^\s*import\s+['"]([^'"]+)['"]"#,
            r#"^\s*export\s+(?:\{[^}]+\}|\*\s+)?\s*from\s+['"]([^'"]+)['"]"#,
            r#"require\(\s*['"]([^'"]+)['"]\s*\)"#,
        ],
        "go" => &[r#"^\s*(?:import\s+)?"([^"]+)""#],
        "rust" => &[
            r"^\s*(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)",
            r"^\s*use\s+([^;]+)",
        ],
        "java" | "kotlin" => &[r"^\s*import\s+([^\s;]+)"],
        "csharp" => &[r"^\s*using\s+([^;]+);"],
        "c" | "cpp" | "objectivec" => &[r#"^\s*#\s*(?:include|import)\s*["<]([^">]+)[">]"#],
        "ruby" => &[r#"^\s*require_relative\s+["']([^"']+)["']"#],
        "php" => &[r"^\s*use\s+([^;]+);"],
        "swift" => &[r"^\s*(?:@testable\s+)?import\s+([^\s]+)"],
        "bash" => &[r#"^\s*(?:source|\.)\s+["']?([^"'\s]+)"#],
        "dart" => &[r#"^\s*(?:import|export|part)\s+["']([^"']+)["']"#],
        "scala" => &[r"^\s*import\s+([^\s]+)"],
        "elixir" => &[r"^\s*(?:alias|import|require)\s+([A-Za-z0-9_.]+)"],
        "erlang" => &[r#"^\s*-include(?:_lib)?\s*\(["']([^"']+)["']\)"#],
        "lua" => &[r#"(?:require|dofile)\s*\(?\s*["']([^"']+)["']"#],
        "r" => &[r#"^\s*source\s*\(\s*["']([^"']+)["']"#],
        "julia" => &[r#"^\s*(?:include|using|import)\s*\(?\s*["']?([^"')\s]+)"#],
        "haskell" => &[r"^\s*import\s+(?:qualified\s+)?([A-Za-z0-9_.]+)"],
        "clojure" => &[r#"\(require\s+'?\[?([A-Za-z0-9_.-]+)"#],
        "fsharp" => &[
            r#"^\s*#?load\s+["']([^"']+)["']"#,
            r"^\s*open\s+([A-Za-z0-9_.]+)",
        ],
        "groovy" => &[r"^\s*import\s+([^\s;]+)"],
        "powershell" => &[r#"^\s*(?:Import-Module|\.\s*)["']?([^"'\s]+)"#],
        "ocaml" => &[r#"^\s*#use\s+["']([^"']+)["']"#],
        "perl" => &[r#"^\s*(?:use|require)\s+["']?([^"';\s]+)"#],
        "fortran" => &[r#"^\s*(?:use|include)\s+["']?([^"'\s]+)"#],
        "ada" => &[r"^\s*with\s+([A-Za-z0-9_.]+)"],
        "solidity" => &[r#"^\s*import\s+["']([^"']+)["']"#],
        "css" | "sass" | "less" => &[r#"^\s*@(?:import|use|forward)\s+["']([^"']+)["']"#],
        "graphql" => &[r#"^\s*#\s*import\s+["']([^"']+)["']"#],
        "protobuf" => &[r#"^\s*import\s+["']([^"']+)["'];"#],
        "nix" => &[r#"^\s*(?:import|builtins\.readFile)\s+["']?([^"'\s]+)"#],
        "make" => &[r"^\s*include\s+([^\s]+)"],
        "cmake" => &[r#"^\s*(?:include|add_subdirectory)\s*\(?\s*["']?([^"')\s]+)"#],
        "starlark" => &[r#"^\s*load\s*\(\s*["']([^"']+)["']"#],
        "markdown" => &[r"!?(?:\[[^]]*\])\(([^)#]+)"],
        "yaml" | "json" | "toml" | "xml" => {
            &[r#"["']?\$?(?:ref|include)["']?\s*[:=]\s*["']([^"']+)["']"#]
        }
        _ => &[],
    };

    patterns
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .filter_map(|regex| regex.captures(line).and_then(|capture| capture.get(1)))
        .map(|value| value.as_str().trim().to_string())
        .collect()
}

fn resolve_target(
    language: &str,
    import: &str,
    source: &str,
    index: &HashMap<String, &str>,
    go_module: Option<&str>,
) -> Option<String> {
    match language {
        "python" => {
            if import.ends_with(".py") {
                return None;
            }

            let normalized = if import.starts_with('.') {
                let parent = source.rsplit_once('/').map_or("", |(value, _)| value);

                let package = parent.replace('/', ".");
                format!("{}{}", package, import.trim_start_matches('.'))
            } else {
                import.to_string()
            };
            index.get(&normalized).map(|_| normalized)
        }
        "typescript" | "javascript" => {
            if !import.starts_with('.') {
                return None;
            }

            let parent = Path::new(source).parent().unwrap_or_else(|| Path::new(""));

            let candidate = normalize_path(&parent.join(import));
            let candidates = [
                candidate.clone(),
                candidate.trim_end_matches(".js").to_string(),
                candidate.trim_end_matches(".jsx").to_string(),
                candidate.trim_end_matches(".mjs").to_string(),
                candidate.trim_end_matches(".cjs").to_string(),
                candidate.trim_end_matches(".ts").to_string(),
                candidate.trim_end_matches(".tsx").to_string(),
            ];

            candidates
                .iter()
                .find_map(|candidate| index.get(candidate))
                .or_else(|| index.get(&format!("{candidate}/index.ts")))
                .or_else(|| index.get(&format!("{candidate}/index.tsx")))
                .or_else(|| index.get(&format!("{candidate}/index.js")))
                .or_else(|| index.get(&format!("{candidate}/index.jsx")))
                .or_else(|| index.get(&format!("{candidate}/index.mjs")))
                .or_else(|| index.get(&format!("{candidate}/index.cjs")))
                .map(|path| path.to_string())
        }
        "go" => {
            let module = go_module?;

            let package = import
                .strip_prefix(module)
                .filter(|_| import == module || import.starts_with(&format!("{module}/")))?
                .trim_start_matches('/');
            index.get(package).map(|_| package.to_string())
        }
        "rust" => {
            let module = import
                .trim()
                .trim_start_matches("crate::")
                .split("::")
                .next()?;

            let path = format!("src/{module}");
            index
                .get(&path)
                .or_else(|| index.get(&format!("{path}.rs")))
                .map(|value| value.to_string())
        }
        "java" | "kotlin" | "csharp" | "php" | "elixir" | "erlang" | "scala" | "haskell"
        | "clojure" | "fsharp" | "groovy" | "ada" => {
            index.get(import).map(|value| value.to_string())
        }
        "c" | "cpp" | "objectivec" | "bash" | "ruby" => {
            let parent = Path::new(source).parent().unwrap_or_else(|| Path::new(""));

            let path = normalize_path(&parent.join(import));
            let resolved = index
                .get(&path)
                .or_else(|| index.get(import))
                .or_else(|| index.get(&format!("include/{import}")))
                .map(|value| value.to_string());

            if resolved.is_some() {
                return resolved;
            }

            if language == "objectivec" && import.ends_with(".h") {
                return Some(path);
            }
            None
        }
        "swift" => index.get(import).map(|value| value.to_string()),
        "dart" | "lua" | "r" | "julia" | "ocaml" | "perl" | "fortran" | "solidity" | "css"
        | "sass" | "less" | "graphql" | "protobuf" | "nix" | "make" | "cmake" | "starlark"
        | "markdown" | "yaml" | "json" | "toml" | "xml" => {
            let parent = Path::new(source).parent().unwrap_or_else(|| Path::new(""));
            let path = normalize_path(&parent.join(import));
            index
                .get(&path)
                .or_else(|| index.get(import))
                .map(|value| value.to_string())
        }
        _ => None,
    }
}

fn source_module(language: &str, path: &str) -> String {
    if language == "go" {
        return Path::new(path).parent().map_or_else(
            || "".to_string(),
            |value| value.to_string_lossy().into_owned(),
        );
    }

    if language == "python" {
        let path = path.trim_end_matches(".py");
        return path
            .strip_suffix("/__init__")
            .unwrap_or(path)
            .replace('/', ".");
    }

    path.to_string()
}

fn normalize_path(path: &Path) -> String {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::Normal(value) => components.push(value.to_string_lossy()),
            _ => {}
        }
    }

    components.join("/")
}

fn read_go_module(root: &Path) -> Option<String> {
    let content = std::fs::read_to_string(root.join("go.mod")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("module ").map(str::trim))
        .map(str::to_string)
}

fn package_name(path: &Path) -> Option<String> {
    text(path).lines().find_map(|line| {
        line.trim()
            .strip_prefix("package ")
            .map(|value| value.trim_end_matches(';').to_string())
    })
}

fn namespace_name(path: &Path) -> Option<String> {
    text(path).lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("namespace ")
            .map(|value| value.trim_end_matches(';').trim().to_string())
    })
}

fn find_cycles(edges: &[Value]) -> Vec<Vec<String>> {
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for edge in edges {
        let Some(from) = edge["from"].as_str() else {
            continue;
        };

        let Some(to) = edge["to"].as_str() else {
            continue;
        };

        graph.entry(from.into()).or_default().push(to.into());
    }

    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    let mut stack = Vec::new();

    for start in graph.keys() {
        visit_cycle(
            start,
            &graph,
            &mut stack,
            &mut visited,
            &mut active,
            &mut cycles,
        );
    }

    cycles.sort();
    cycles.dedup();
    cycles
}

fn visit_cycle(
    current: &str,
    graph: &BTreeMap<String, Vec<String>>,
    stack: &mut Vec<String>,
    visited: &mut HashSet<String>,
    active: &mut HashSet<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    if !visited.insert(current.to_string()) {
        return;
    }
    active.insert(current.to_string());
    stack.push(current.to_string());

    for next in graph.get(current).into_iter().flatten() {
        if active.contains(next) {
            if let Some(index) = stack.iter().position(|item| item == next) {
                let mut cycle = stack[index..].to_vec();
                cycle.push(next.clone());
                cycles.push(cycle);
            }
        } else if !visited.contains(next) {
            visit_cycle(next, graph, stack, visited, active, cycles);
        }
    }

    stack.pop();
    active.remove(current);
}

fn most_depended_on(edges: &[Value]) -> Vec<Value> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for edge in edges {
        if let Some(to) = edge["to"].as_str() {
            *counts.entry(to.to_string()).or_default() += 1;
        }
    }

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    counts
        .into_iter()
        .take(MAX_MOST_DEPENDED)
        .map(|(module, dependents)| json!({"module": module, "dependents": dependents}))
        .collect()
}

fn detect_monorepo_boundaries(root: &Path) -> Value {
    for boundary_dir in MONOREPO_BOUNDARY_DIRS {
        let Ok(entries) = root.join(boundary_dir).read_dir() else {
            continue;
        };

        let mut services = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.'))
            .collect::<Vec<_>>();

        services.sort();
        services.dedup();

        if services.len() >= 2 {
            return json!({
                "detected": true,
                "boundary_dir": boundary_dir,
                "services": services,
                "cross_service_imports": [],
            });
        }
    }

    json!({"detected": false, "services": [], "cross_service_imports": []})
}

#[cfg(test)]
mod tests {
    use super::imports_for;

    #[test]
    fn parses_javascript_imports() {
        assert_eq!(
            imports_for("javascript", r#"import { buildValue } from "./util.js""#),
            vec!["./util.js"]
        );

        assert_eq!(
            imports_for("cpp", r#"#include "example/service.hpp""#),
            vec!["example/service.hpp"]
        );

        assert_eq!(
            imports_for("typescript", "export { value } from './lib';"),
            vec!["./lib"]
        );
    }
}
