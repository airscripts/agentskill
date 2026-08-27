use std::path::Path;

use agentskill_core::{Result, error::validate_repo, fs::collect_files};
use serde_json::{Map, Value, json};

const PRETTIER_FILES: &[&str] = &[
    ".prettierrc",
    ".prettierrc.json",
    ".prettierrc.js",
    ".prettierrc.cjs",
    ".prettierrc.yml",
    ".prettierrc.yaml",
    ".prettierrc.toml",
];

const ESLINT_FILES: &[&str] = &[
    ".eslintrc.json",
    ".eslintrc.js",
    ".eslintrc.cjs",
    ".eslintrc.yml",
    ".eslintrc.yaml",
    ".eslintrc",
    "eslint.config.js",
    "eslint.config.mjs",
    "eslint.config.cjs",
];

const MAX_CONFIG_READ_BYTES: usize = 32_000;

pub fn run(repo: &str) -> Result<Value> {
    let root = validate_repo(repo)?;

    let files = collect_files(&root);
    let has_language = |language: &str| {
        files
            .iter()
            .any(|file| file.language.is_some_and(|item| item.id == language))
    };

    let editor_sections = parse_editorconfig(&read(&root, ".editorconfig"));
    let mut result = Map::new();

    let python = detect_python(&root);

    if !python.is_empty() {
        result.insert(
            "python".into(),
            json!(attach_editorconfig(python, &editor_sections, "python")),
        );
    }

    let package = parse_json(&read(&root, "package.json"));

    let has_typescript = has_language("typescript") || root.join("tsconfig.json").exists();
    let has_javascript = has_language("javascript");

    let javascript_config = detect_javascript(&root, &package, has_typescript);
    if !javascript_config.is_empty() {
        let language = if has_typescript || !has_javascript {
            "typescript"
        } else {
            "javascript"
        };
        result.insert(
            language.into(),
            json!(attach_editorconfig(
                javascript_config,
                &editor_sections,
                language
            )),
        );
    }

    if has_typescript && has_javascript {
        let config = detect_javascript(&root, &package, true);
        result.insert(
            "javascript".into(),
            json!(attach_editorconfig(config, &editor_sections, "javascript")),
        );
    }

    if has_language("go") || root.join("go.mod").exists() {
        result.insert(
            "go".into(),
            json!(attach_editorconfig(
                detect_go(&root),
                &editor_sections,
                "go"
            )),
        );
    }

    if has_language("rust") || root.join("Cargo.toml").exists() {
        result.insert(
            "rust".into(),
            json!(attach_editorconfig(
                detect_rust(&root),
                &editor_sections,
                "rust"
            )),
        );
    }

    add_java(&root, &files, &mut result, &editor_sections);
    add_kotlin(&root, &files, &mut result, &editor_sections);
    add_csharp(&root, &files, &mut result, &editor_sections);
    add_c_family(&root, &files, &mut result, "c", &editor_sections);
    add_c_family(&root, &files, &mut result, "cpp", &editor_sections);
    add_ruby(&root, &files, &mut result, &editor_sections);
    add_php(&root, &files, &mut result, &editor_sections);
    add_apple(&root, &files, &mut result, "swift", &editor_sections);
    add_apple(&root, &files, &mut result, "objectivec", &editor_sections);

    if !editor_sections.is_empty() {
        result.insert("editorconfig".into(), json!(editor_sections));
    }

    Ok(Value::Object(result))
}

fn detect_python(root: &Path) -> Map<String, Value> {
    let pyproject = parse_toml(&read(root, "pyproject.toml"));

    let ruff = table(&pyproject, &["tool", "ruff"]);
    let black = table(&pyproject, &["tool", "black"]);

    let mypy = table(&pyproject, &["tool", "mypy"]);
    let mut result = Map::new();

    if !ruff.is_null() {
        let lint = table(&ruff, &["lint"]);
        result.insert(
            "linter".into(),
            tool(
                "ruff",
                "pyproject.toml",
                if lint.is_object() { lint } else { ruff.clone() },
            ),
        );

        if let Some(format) = ruff.get("format") {
            result.insert(
                "formatter".into(),
                tool("ruff", "pyproject.toml", format.clone()),
            );
        }
    }

    if root.join("ruff.toml").exists() {
        result.insert(
            "linter".into(),
            tool("ruff", "ruff.toml", parse_toml(&read(root, "ruff.toml"))),
        );
    }

    if !black.is_null() {
        result.insert("formatter".into(), tool("black", "pyproject.toml", black));
    }

    if !mypy.is_null() {
        result.insert("type_checker".into(), tool("mypy", "pyproject.toml", mypy));
    }

    if result.get("linter").is_none()
        && let Some((name, settings)) = first_ini_config(root, &[".flake8", "setup.cfg"], "flake8")
    {
        result.insert("linter".into(), tool("flake8", &name, settings));
    }

    if result.get("formatter").is_none() && root.join("black.toml").exists() {
        result.insert(
            "formatter".into(),
            tool("black", "black.toml", parse_toml(&read(root, "black.toml"))),
        );
    }

    if result.get("type_checker").is_none() {
        if let Some((name, settings)) = first_ini_config(root, &["mypy.ini", ".mypy.ini"], "mypy") {
            result.insert("type_checker".into(), tool("mypy", &name, settings));
        } else if root.join("pyrightconfig.json").exists() {
            result.insert(
                "type_checker".into(),
                tool(
                    "pyright",
                    "pyrightconfig.json",
                    parse_json(&read(root, "pyrightconfig.json")),
                ),
            );
        }
    }

    result
}

fn detect_javascript(root: &Path, package: &Value, typescript: bool) -> Map<String, Value> {
    let mut result = Map::new();

    if let Some(name) = first_existing(root, PRETTIER_FILES) {
        result.insert(
            "formatter".into(),
            tool("prettier", &name, parse_config(root, &name)),
        );
    } else if let Some(settings) = package.get("prettier") {
        result.insert(
            "formatter".into(),
            tool("prettier", "package.json", settings.clone()),
        );
    }

    if let Some(name) = first_existing(root, ESLINT_FILES) {
        result.insert(
            "linter".into(),
            tool("eslint", &name, parse_config(root, &name)),
        );
    } else if let Some(settings) = package.get("eslintConfig") {
        result.insert(
            "linter".into(),
            tool("eslint", "package.json", settings.clone()),
        );
    }

    if typescript && root.join("tsconfig.json").exists() {
        let config = parse_json(&read(root, "tsconfig.json"));
        result.insert(
            "type_checker".into(),
            tool(
                "tsc",
                "tsconfig.json",
                config.get("compilerOptions").cloned().unwrap_or(json!({})),
            ),
        );
    }

    if result.is_empty()
        && let Some(scripts) = package.get("scripts")
        && scripts.as_object().is_some_and(|value| !value.is_empty())
    {
        result.insert("scripts".into(), scripts.clone());
    }
    result
}

fn detect_go(root: &Path) -> Map<String, Value> {
    let mut result = Map::new();
    result.insert("formatter".into(), tool("gofmt", "null", json!({})));

    if let Some(name) = first_existing(
        root,
        &[
            ".golangci.yml",
            ".golangci.yaml",
            ".golangci.toml",
            ".golangci.json",
        ],
    ) {
        result.insert(
            "linter".into(),
            tool("golangci-lint", &name, parse_config(root, &name)),
        );
    }
    result
}

fn detect_rust(root: &Path) -> Map<String, Value> {
    let mut result = Map::new();

    if let Some(name) = first_existing(root, &["rustfmt.toml", ".rustfmt.toml"]) {
        result.insert(
            "formatter".into(),
            tool("rustfmt", &name, parse_toml(&read(root, &name))),
        );
    }

    if let Some(name) = first_existing(root, &["clippy.toml", ".clippy.toml"]) {
        result.insert(
            "linter".into(),
            tool("clippy", &name, parse_toml(&read(root, &name))),
        );
    }
    result
}

fn add_java(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
) {
    let mut markers = existing_markers(
        root,
        &[
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
    );
    markers.extend(source_roots(root, &["src/main/java", "src/test/java"]));

    let build_tool = if markers.iter().any(|item| item == "pom.xml") {
        "maven"
    } else {
        "gradle"
    };
    add_language_project(files, result, sections, "java", markers, build_tool);
}

fn add_kotlin(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
) {
    let mut markers = existing_markers(
        root,
        &[
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
        ],
    );
    markers.extend(source_roots(root, &["src/main/kotlin", "src/test/kotlin"]));
    add_language_project(files, result, sections, "kotlin", markers, "gradle");
}

fn add_csharp(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
) {
    let mut markers = existing_markers(root, &["Directory.Build.props", "Directory.Build.targets"]);
    markers.extend(root_files_matching(root, &[".sln", ".csproj"]));
    add_language_project(files, result, sections, "csharp", markers, "msbuild");
}

fn add_c_family(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    language: &str,
    sections: &Map<String, Value>,
) {
    let mut markers = existing_markers(
        root,
        &["CMakeLists.txt", "Makefile", "makefile", "GNUmakefile"],
    );
    markers.extend(root_files_matching(root, &[".cmake", ".vcxproj"]));

    let build_tool = if markers
        .iter()
        .any(|item| item.ends_with("CMakeLists.txt") || item.ends_with(".cmake"))
    {
        "cmake"
    } else {
        "make"
    };
    add_language_project(files, result, sections, language, markers, build_tool);
}

fn add_ruby(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
) {
    let mut markers = existing_markers(root, &["Gemfile", "Gemfile.lock"]);
    markers.extend(root_files_matching(root, &[".gemspec"]));
    add_language_project(files, result, sections, "ruby", markers, "bundler");
}

fn add_php(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
) {
    let markers = existing_markers(root, &["composer.json", "composer.lock"]);

    if !language_present(files, "php") && markers.is_empty() {
        return;
    }

    let mut config = project_value(&markers, "composer");

    let composer = parse_json(&read(root, "composer.json"));
    if let Some(value) = composer.pointer("/autoload/psr-4") {
        config.insert("autoload_psr4".into(), value.clone());
    }

    if let Some(value) = composer.pointer("/autoload-dev/psr-4") {
        config.insert("autoload_dev_psr4".into(), value.clone());
    }

    if composer.pointer("/require-dev/phpunit/phpunit").is_some() {
        config.insert("test_framework".into(), json!("phpunit"));
    }
    result.insert(
        "php".into(),
        json!(attach_editorconfig(config, sections, "php")),
    );
}

fn add_apple(
    root: &Path,
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    language: &str,
    sections: &Map<String, Value>,
) {
    let static_markers: &[&str] = if language == "swift" {
        &["Package.swift", "Package.resolved"]
    } else {
        &["Podfile", "Podfile.lock"]
    };

    let mut markers = existing_markers(root, static_markers);
    markers.extend(root_files_matching(root, &[".xcodeproj", ".xcworkspace"]));

    let build_tool = if language == "swift" {
        if markers.iter().any(|item| item == "Package.swift") {
            "swiftpm"
        } else {
            "xcode"
        }
    } else if markers
        .iter()
        .any(|item| item == "Podfile" || item == "Podfile.lock")
    {
        "cocoapods"
    } else {
        "xcode"
    };
    add_language_project(files, result, sections, language, markers, build_tool);
}

fn add_language_project(
    files: &[agentskill_core::fs::RepoFile],
    result: &mut Map<String, Value>,
    sections: &Map<String, Value>,
    language: &str,
    markers: Vec<String>,
    build_tool: &str,
) {
    if !language_present(files, language) && markers.is_empty() {
        return;
    }

    let mut config = if markers.is_empty() {
        Map::new()
    } else {
        project_value(&markers, build_tool)
    };

    if language == "java" {
        config.insert(
            "build_tool".into(),
            json!(if markers.iter().any(|item| item == "pom.xml") {
                "maven"
            } else {
                build_tool
            }),
        );
        config.insert("project_markers".into(), json!(markers));
    }
    result.insert(
        language.into(),
        json!(attach_editorconfig(config, sections, language)),
    );
}

fn project_value(markers: &[String], build_tool: &str) -> Map<String, Value> {
    let mut value = Map::new();
    value.insert("build_tool".into(), json!(build_tool));
    value.insert("project_markers".into(), json!(markers));
    value
}

fn attach_editorconfig(
    mut value: Map<String, Value>,
    sections: &Map<String, Value>,
    language: &str,
) -> Map<String, Value> {
    let settings = editorconfig_for_language(sections, language);

    if !settings.is_empty() {
        value.insert("editorconfig".into(), json!(settings));
    }
    value
}

fn language_present(files: &[agentskill_core::fs::RepoFile], language: &str) -> bool {
    files
        .iter()
        .any(|file| file.language.is_some_and(|item| item.id == language))
}

fn existing_markers(root: &Path, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|name| root.join(name).exists())
        .map(|name| (*name).into())
        .collect()
}

fn source_roots(root: &Path, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|name| root.join(name).is_dir())
        .map(|name| (*name).into())
        .collect()
}

fn root_files_matching(root: &Path, suffixes: &[&str]) -> Vec<String> {
    let mut matches = Vec::new();
    collect_matching_files(root, suffixes, &mut matches);

    matches.sort();
    matches.dedup();
    matches
}

fn collect_matching_files(directory: &Path, suffixes: &[&str], matches: &mut Vec<String>) {
    let Ok(entries) = directory.read_dir() else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if path.is_dir() {
            if !name.starts_with('.') {
                collect_matching_files(&path, suffixes, matches);
            }
        } else if path.is_file() && suffixes.iter().any(|suffix| name.ends_with(suffix)) {
            matches.push(name);
        }
    }
}

fn first_existing(root: &Path, names: &[&str]) -> Option<String> {
    names
        .iter()
        .find(|name| root.join(name).exists())
        .map(|name| (*name).into())
}

fn first_ini_config(root: &Path, names: &[&str], section: &str) -> Option<(String, Value)> {
    names.iter().find_map(|name| {
        root.join(name).exists().then(|| {
            (
                (*name).into(),
                json!(parse_ini_section(&read(root, name), section)),
            )
        })
    })
}

fn tool(name: &str, config_file: &str, settings: Value) -> Value {
    json!({"name": name, "config_file": if config_file == "null" { Value::Null } else { json!(config_file) }, "settings": settings})
}

fn read(root: &Path, name: &str) -> String {
    std::fs::read(root.join(name))
        .map(|bytes| {
            String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_CONFIG_READ_BYTES)]).into_owned()
        })
        .unwrap_or_default()
}

fn parse_json(content: &str) -> Value {
    serde_json::from_str(content).unwrap_or_else(|_| json!({}))
}

fn parse_toml(content: &str) -> Value {
    toml::from_str::<Value>(content).unwrap_or_else(|_| json!({}))
}

fn parse_config(root: &Path, name: &str) -> Value {
    let content = read(root, name);

    if name.ends_with(".toml") {
        return parse_toml(&content);
    }

    if name.ends_with(".yml") || name.ends_with(".yaml") {
        return serde_yaml::from_str(&content).unwrap_or_else(|_| json!({}));
    }
    parse_json(&content)
}

fn table(value: &Value, path: &[&str]) -> Value {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
        .cloned()
        .unwrap_or(Value::Null)
}

fn parse_ini_section(content: &str, section: &str) -> Map<String, Value> {
    let wanted = section.trim_matches(['[', ']']);

    let mut active = false;
    let mut result = Map::new();

    for line in content.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            active = line.trim_matches(['[', ']']) == wanted;
        } else if active
            && !line.is_empty()
            && !line.starts_with(['#', ';'])
            && let Some((key, value)) = line.split_once('=')
        {
            result.insert(key.trim().into(), json!(value.trim()));
        }
    }
    result
}

fn parse_editorconfig(content: &str) -> Map<String, Value> {
    let mut result = Map::new();

    let mut section = String::new();
    let mut values = Map::new();

    for line in content.lines().map(str::trim) {
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if !section.is_empty() {
                result.insert(section.clone(), json!(values));
            }
            section = line.into();
            values = Map::new();
        } else if let Some((key, value)) = line.split_once('=') {
            values.insert(
                key.trim().to_ascii_lowercase(),
                json!(value.trim().to_ascii_lowercase()),
            );
        }
    }

    if !section.is_empty() {
        result.insert(section, json!(values));
    }
    result
}

fn editorconfig_for_language(sections: &Map<String, Value>, language: &str) -> Map<String, Value> {
    let patterns: &[&str] = match language {
        "python" => &["*.py"],
        "typescript" => &["*.ts", "*.tsx"],
        "javascript" => &["*.js", "*.jsx", "*.mjs"],
        "go" => &["*.go"],
        "rust" => &["*.rs"],
        "java" => &["*.java"],
        "kotlin" => &["*.kt", "*.kts"],
        "csharp" => &["*.cs"],
        "c" => &["*.c", "*.h"],
        "cpp" => &["*.cpp", "*.cc", "*.cxx", "*.hpp", "*.hh", "*.hxx"],
        "ruby" => &["*.rb"],
        "php" => &["*.php"],
        "bash" => &["*.sh", "*.bash"],
        "swift" => &["*.swift"],
        "objectivec" => &["*.m", "*.mm", "*.h"],
        _ => &[],
    };

    let mut result = sections
        .get("[*]")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for pattern in patterns {
        let section = format!("[{pattern}]");

        if let Some(values) = sections.get(&section).and_then(Value::as_object) {
            result.extend(values.clone());
        }
    }
    result
}
