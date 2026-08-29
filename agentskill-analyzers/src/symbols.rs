use std::collections::BTreeMap;

use agentskill_core::Result;
use regex::Regex;
use serde_json::{Map, Value, json};

use crate::common::{insert_language_result, repo_files, text};

pub fn run(repo: &str, lang: Option<&str>) -> Result<Value> {
    let (_root, files) = repo_files(repo, lang)?;

    let mut result = Map::new();
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

        let mut source = String::new();

        let mut file_names = Vec::new();
        for file in language_files {
            let mut file_name = std::path::Path::new(&file.relative)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_string();

            if matches!(language.id, "typescript" | "javascript") {
                file_name = file_name
                    .trim_end_matches(".test")
                    .trim_end_matches(".spec")
                    .to_string();
            }
            file_names.push(file_name);
            source.push_str(&text(&file.path));
            source.push('\n');
        }

        let source = if language.id == "python" {
            source
        } else {
            let source = strip_comments(&source);
            if matches!(language.id, "bash" | "ruby") {
                strip_hash_comments(&source)
            } else {
                source
            }
        };

        let functions = names(
            &source,
            r"(?m)\b(?:async\s+)?(?:pub\s+)?(?:fn|function|func|def|fun)\s+([A-Za-z_][A-Za-z0-9_]*)",
        );

        let arrow_functions = names(
            &source,
            r"(?m)\b(?:export\s+)?(?:const|let)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*\([^\n]*\)\s*=>",
        );

        let mut all_functions = functions;
        all_functions.extend(arrow_functions);
        all_functions.extend(language_functions(language.id, &source));

        let types = names(
            &source,
            r"(?m)\b(?:class|struct|interface|enum|trait|type|record)\s+([A-Za-z_][A-Za-z0-9_]*)",
        );

        let constants = constant_names(&source, language.id);
        let mut payload = Map::new();
        payload.insert("functions".into(), pattern_summary(&all_functions));
        payload.insert("classes".into(), pattern_summary(&types));
        payload.insert("types".into(), pattern_summary(&types));
        payload.insert("constants".into(), pattern_summary(&constants));
        payload.insert("files".into(), pattern_summary(&file_names));
        add_language_categories(language.id, &source, &mut payload);

        if language.id == "swift" && types.is_empty() {
            payload.remove("classes");
            payload.remove("types");
        }
        insert_language_result(&mut result, language.id, Value::Object(payload));
    }

    Ok(Value::Object(result))
}

fn names(source: &str, pattern: &str) -> Vec<String> {
    Regex::new(pattern)
        .expect("valid symbol regex")
        .captures_iter(source)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect()
}

fn language_functions(language: &str, source: &str) -> Vec<String> {
    let pattern = match language {
        "erlang" => r"(?m)^\s*([a-z][A-Za-z0-9_]*)\s*\([^)]*\)\s*->",
        "lua" => r"(?m)\bfunction\s+(?:[A-Za-z_][A-Za-z0-9_]*\.)?([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        "r" => r"(?m)^\s*([A-Za-z_][A-Za-z0-9_.]*)\s*<-\s*function",
        "julia" => r"(?m)^\s*([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)\s*=",
        "haskell" => r"(?m)^\s*([a-z][A-Za-z0-9_']*)\s+[^=\n]+\s*=",
        "clojure" => r"(?m)\(defn-?\s+([A-Za-z_*!?+.-]+)",
        "fsharp" | "ocaml" => r"(?m)^\s*let\s+(?:rec\s+)?([A-Za-z_][A-Za-z0-9_']*)",
        "nim" => r"(?m)^\s*(?:proc|func|iterator)\s+([A-Za-z_][A-Za-z0-9_]*)",
        "perl" => r"(?m)^\s*sub\s+([A-Za-z_][A-Za-z0-9_]*)",
        "fortran" => r"(?mi)^\s*(?:program|subroutine|function)\s+([A-Za-z_][A-Za-z0-9_]*)",
        "ada" => r"(?mi)^\s*(?:procedure|function)\s+([A-Za-z_][A-Za-z0-9_]*)",
        _ => return Vec::new(),
    };
    names(source, pattern)
}

fn constant_names(source: &str, language: &str) -> Vec<String> {
    let mut values = if matches!(language, "typescript" | "javascript") {
        names(
            source,
            r"(?m)^\s*(?:export\s+)?const\s+([A-Z_][A-Z0-9_]*)\s*[=:]",
        )
    } else if language == "go" {
        go_constants(source)
    } else {
        names(
            source,
            r"(?m)^\s*(?:pub\s+)?(?:const|let|var)\s+([A-Za-z_][A-Za-z0-9_]*)",
        )
    };

    if language == "python" {
        values.extend(names(source, r"(?m)^\s*([A-Z][A-Z0-9_]{2,})\s*="));
    }

    if matches!(language, "c" | "cpp") {
        values.extend(names(source, r"(?m)^\s*#define\s+([A-Za-z_][A-Za-z0-9_]*)"));
    }
    values
}

fn add_language_categories(language: &str, source: &str, payload: &mut Map<String, Value>) {
    let methods = names(
        source,
        r"(?m)\b(?:public|private|protected|internal|static|final|virtual|override|\s)+[A-Za-z_][A-Za-z0-9_<>,.?\[\]]*\s+([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)",
    );

    let structs = names(source, r"(?m)\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)");
    let interfaces = names(source, r"(?m)\binterface\s+([A-Za-z_][A-Za-z0-9_]*)");

    let enums = names(
        source,
        r"(?m)\benum(?:\s+class)?\s+([A-Za-z_][A-Za-z0-9_]*)",
    );

    let records = names(source, r"(?m)\brecord\s+([A-Za-z_][A-Za-z0-9_]*)");
    let types = names(
        source,
        r"(?m)\b(?:class|struct|interface|enum|trait|type|record)\s+([A-Za-z_][A-Za-z0-9_]*)",
    );

    match language {
        "python" => {
            let private_names = names(source, r"(?m)\b(?:def|class)\s+(_+[A-Za-z_][A-Za-z0-9_]*)");

            let single = private_names
                .iter()
                .filter(|name| name.starts_with('_') && !name.starts_with("__"))
                .count();

            let double = private_names
                .iter()
                .filter(|name| name.starts_with("__"))
                .count();
            payload.insert(
                "private_members".into(),
                json!({"single_underscore": single, "double_underscore": double, "examples": private_names}),
            );
        }
        "typescript" | "javascript" => {
            payload.insert("interfaces".into(), pattern_summary(&interfaces));
            payload.insert("types".into(), pattern_summary(&types_for(source)));
        }
        "go" => {
            payload.insert(
                "methods".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\bfunc\s*\([^)]*\)\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "interfaces".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s+interface",
                )),
            );
            payload.insert(
                "structs".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\btype\s+([A-Za-z_][A-Za-z0-9_]*)\s+struct",
                )),
            );
            payload.insert(
                "variables".into(),
                pattern_summary(&names(source, r"(?m)^\s*var\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "type_aliases".into(),
                pattern_summary(&go_type_aliases(source)),
            );
        }
        "rust" => {
            payload.insert("structs".into(), pattern_summary(&structs));
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert(
                "traits".into(),
                pattern_summary(&names(source, r"(?m)\btrait\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "impls".into(),
                pattern_summary(&names(source, r"(?m)^\s*impl(?:<[^>]+>)?\s+([^\s{]+)")),
            );
            payload.insert(
                "statics".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*(?:pub\s+)?static(?:\s+mut)?\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
        }
        "java" => {
            payload.insert("methods".into(), pattern_summary(&methods));
            payload.insert("interfaces".into(), pattern_summary(&interfaces));
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert(
                "constructors".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\b([A-Z][A-Za-z0-9_]*)\s*\([^)]*\)\s*\{",
                )),
            );
            payload.insert(
                "annotations".into(),
                pattern_summary(&names(source, r"(?m)@interface\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
        }
        "kotlin" => {
            payload.insert("interfaces".into(), pattern_summary(&interfaces));
            payload.insert(
                "objects".into(),
                pattern_summary(&names(source, r"(?m)\bobject\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert(
                "properties".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*(?:public\s+)?(?:val|var)\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
        }
        "csharp" => {
            payload.insert("methods".into(), pattern_summary(&methods));
            payload.insert("interfaces".into(), pattern_summary(&interfaces));
            payload.insert("structs".into(), pattern_summary(&structs));
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert("records".into(), pattern_summary(&records));
        }
        "c" => {
            payload.insert("structs".into(), pattern_summary(&structs));
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert(
                "typedefs".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\btypedef\s+[^;]+?\s+([A-Za-z_][A-Za-z0-9_]*)\s*;",
                )),
            );
            payload.insert(
                "macros".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*#define\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
        }
        "cpp" => {
            payload.insert(
                "namespaces".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\bnamespace\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert("structs".into(), pattern_summary(&structs));
            payload.insert("enums".into(), pattern_summary(&enums));
            payload.insert(
                "templates".into(),
                pattern_summary(&names(source, r"(?m)\b(template)\s*<")),
            );
        }
        "ruby" => {
            payload.insert(
                "modules".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*module\s+([A-Za-z_][A-Za-z0-9_:]*)",
                )),
            );

            let ruby_methods = names(source, r"(?m)^\s*def\s+([A-Za-z_][A-Za-z0-9_!?\.]*)")
                .into_iter()
                .filter(|name| !name.starts_with("self."))
                .collect::<Vec<_>>();
            payload.insert("methods".into(), pattern_summary(&ruby_methods));
            payload.insert(
                "class_methods".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*def\s+self\.([A-Za-z_][A-Za-z0-9_!?]*)",
                )),
            );
        }
        "php" => {
            payload.insert(
                "methods".into(),
                pattern_summary(&names(source, r"(?m)\bfunction\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert("interfaces".into(), pattern_summary(&interfaces));
            payload.insert(
                "traits".into(),
                pattern_summary(&names(source, r"(?m)\btrait\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert("enums".into(), pattern_summary(&enums));
        }
        "swift" => {
            payload.insert("structs".into(), pattern_summary(&structs));
            payload.insert("classes".into(), pattern_summary(&types));
            payload.insert(
                "enums".into(),
                pattern_summary(&names(source, r"(?m)\benum\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "protocols".into(),
                pattern_summary(&names(source, r"(?m)\bprotocol\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "extensions".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\bextension\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
        }
        "objectivec" => {
            payload.insert(
                "interfaces".into(),
                pattern_summary(&names(source, r"(?m)@interface\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "methods".into(),
                pattern_summary(&names(source, r"(?m)-\s*\([^)]*\)([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "class_methods".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\+\s*\([^)]*\)([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "implementations".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)@implementation\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "protocols".into(),
                pattern_summary(&names(source, r"(?m)@protocol\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
        }
        "bash" => {
            payload.insert(
                "functions".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*(?:function\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)",
                )),
            );
        }
        "html" | "vue" | "svelte" | "astro" => {
            payload.insert(
                "elements".into(),
                pattern_summary(&names(source, r"(?m)<([A-Za-z][A-Za-z0-9:-]*)\b")),
            );
            payload.insert(
                "ids".into(),
                pattern_summary(&names(
                    source,
                    r#"(?m)\bid=["']([A-Za-z_][A-Za-z0-9_-]*)["']"#,
                )),
            );
            payload.insert(
                "components".into(),
                pattern_summary(&names(source, r"(?m)<([A-Z][A-Za-z0-9_]*)\b")),
            );
        }
        "css" | "sass" | "less" => {
            payload.insert(
                "selectors".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)^\s*\.([A-Za-z_][A-Za-z0-9_-]*)\s*[,{]",
                )),
            );
            payload.insert(
                "variables".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)(?:--|@|\$)([A-Za-z_][A-Za-z0-9_-]*)\s*:",
                )),
            );
            payload.insert(
                "mixins".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)@(?:mixin|include)\s+([A-Za-z_][A-Za-z0-9_-]*)",
                )),
            );
        }
        "sql" => {
            payload.insert(
                "tables".into(),
                pattern_summary(&names(source, r"(?mi)\bcreate\s+(?:temporary\s+)?table\s+(?:if\s+not\s+exists\s+)?([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "views".into(),
                pattern_summary(&names(
                    source,
                    r"(?mi)\bcreate\s+view\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "routines".into(),
                pattern_summary(&names(source, r"(?mi)\bcreate\s+(?:or\s+replace\s+)?(?:function|procedure)\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
        }
        "graphql" => {
            payload.insert(
                "operations".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\b(?:query|mutation|subscription)\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "schema_types".into(),
                pattern_summary(&names(
                    source,
                    r"(?m)\b(?:type|input|interface|enum|scalar|union)\s+([A-Za-z_][A-Za-z0-9_]*)",
                )),
            );
            payload.insert(
                "fragments".into(),
                pattern_summary(&names(source, r"(?m)\bfragment\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
        }
        "protobuf" => {
            payload.insert(
                "messages".into(),
                pattern_summary(&names(source, r"(?m)\bmessage\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "services".into(),
                pattern_summary(&names(source, r"(?m)\bservice\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
            payload.insert(
                "enums".into(),
                pattern_summary(&names(source, r"(?m)\benum\s+([A-Za-z_][A-Za-z0-9_]*)")),
            );
        }
        "hcl" | "nix" | "dockerfile" | "make" | "cmake" | "starlark" => {
            payload.insert(
                "resources_or_targets".into(),
                pattern_summary(&names(source, r#"(?m)^\s*(?:resource|data|target|task|rule|stage|service|module|load)\s+["']?([A-Za-z_][A-Za-z0-9_./:-]*)"#)),
            );
            payload.insert(
                "variables".into(),
                pattern_summary(&names(
                    source,
                    r#"(?m)^\s*(?:variable|locals?|set|export)\s*["']?([A-Za-z_][A-Za-z0-9_-]*)"#,
                )),
            );
        }
        "markdown" => {
            payload.insert(
                "headings".into(),
                pattern_summary(&names(source, r"(?m)^#{1,6}\s+(.+?)\s*$")),
            );
            payload.insert(
                "links".into(),
                pattern_summary(&names(source, r"(?m)!?\[[^]]*\]\(([^)]+)\)")),
            );
        }
        "yaml" | "json" | "toml" | "xml" => {
            payload.insert(
                "keys_or_elements".into(),
                pattern_summary(&names(
                    source,
                    r#"(?m)^\s*["']?([A-Za-z_][A-Za-z0-9_.:-]*)["']?\s*[:=]"#,
                )),
            );
        }
        _ => {}
    }
}

fn types_for(source: &str) -> Vec<String> {
    names(source, r"(?m)\btype\s+([A-Za-z_][A-Za-z0-9_]*)")
}

fn go_type_aliases(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("type ")?;

            let mut parts = rest.split_whitespace();
            let name = parts.next()?;

            let kind = parts.next()?;
            (!matches!(kind, "struct" | "interface")
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            .then(|| name.to_string())
        })
        .collect()
}

fn go_constants(source: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut grouped = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("const (") {
            grouped = true;
            continue;
        }

        if grouped && trimmed == ")" {
            grouped = false;
            continue;
        }

        let candidate = if grouped {
            trimmed
        } else if let Some(value) = trimmed.strip_prefix("const ") {
            value.trim()
        } else {
            continue;
        };

        if let Some(name) = candidate.split(['=', ' ', '\t']).next()
            && !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            values.push(name.to_string());
        }
    }
    values
}

fn strip_comments(source: &str) -> String {
    let block = Regex::new(r"(?s)/\*.*?\*/").expect("valid block comment regex");

    let line = Regex::new(r"(?m)^\s*//.*$").expect("valid line comment regex");
    line.replace_all(&block.replace_all(source, ""), "")
        .into_owned()
}

fn strip_hash_comments(source: &str) -> String {
    Regex::new(r"(?m)^\s*#.*$")
        .expect("valid hash comment regex")
        .replace_all(source, "")
        .into_owned()
}

fn pattern_summary(values: &[String]) -> Value {
    let mut patterns = BTreeMap::new();

    for value in values {
        let key = classify(value);
        *patterns.entry(key).or_insert(0usize) += 1;
    }

    let total = values.len();

    let patterns_json: Map<String, Value> = patterns
        .into_iter()
        .map(|(key, count)| {
            (
                key.to_string(),
                json!({"count": count, "pct": if total == 0 { 0.0 } else { (count as f64 / total as f64 * 1000.0).round() / 10.0 }}),
            )
        })
        .collect();
    json!({"total": total, "patterns": patterns_json, "codebase_specific": []})
}

fn classify(value: &str) -> &'static str {
    if value.len() > 1
        && value.chars().all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
    {
        "SCREAMING_SNAKE_CASE"
    } else if value.chars().next().is_some_and(char::is_uppercase) {
        "PascalCase"
    } else if value.contains('_') {
        "snake_case"
    } else if value.chars().any(char::is_uppercase) {
        "camelCase"
    } else {
        "other"
    }
}
