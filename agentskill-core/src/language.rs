use std::io::Read;
use std::path::Path;

use serde::Serialize;

const MAX_DETECTION_BYTES: usize = 1_000_000;

/// Categorizes languages so analyzers can group source and metadata consistently.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageKind {
    Programming,
    Markup,
    Stylesheet,
    Query,
    Schema,
    Infrastructure,
    Build,
    Data,
    Prose,
}

impl LanguageKind {
    /// Provides the stable identifier used in analyzer JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Programming => "programming",
            Self::Markup => "markup",
            Self::Stylesheet => "stylesheet",
            Self::Query => "query",
            Self::Schema => "schema",
            Self::Infrastructure => "infrastructure",
            Self::Build => "build",
            Self::Data => "data",
            Self::Prose => "prose",
        }
    }
}

/// Separates source languages from formats that should not drive guidance.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageRole {
    Primary,
    Auxiliary,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LanguageSpec {
    pub id: &'static str,
    pub display_name: &'static str,
    pub extensions: &'static [&'static str],
    pub test_patterns: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
struct LanguageMetadata {
    id: &'static str,
    kind: LanguageKind,
    role: LanguageRole,
    filenames: &'static [&'static str],
    filename_prefixes: &'static [&'static str],
    filename_suffixes: &'static [&'static str],
    interpreters: &'static [&'static str],
}

macro_rules! language_registry {
    ($( {
        id: $id:literal,
        display: $display:literal,
        extensions: [$($extension:literal),* $(,)?],
        tests: [$($test:literal),* $(,)?],
        kind: $kind:ident,
        role: $role:ident,
        filenames: [$($filename:literal),* $(,)?],
        prefixes: [$($prefix:literal),* $(,)?],
        suffixes: [$($suffix:literal),* $(,)?],
        interpreters: [$($interpreter:literal),* $(,)?]
    }),+ $(,)?) => {
        pub const LANGUAGES: &[LanguageSpec] = &[
            $(LanguageSpec {
                id: $id,
                display_name: $display,
                extensions: &[$($extension),*],
                test_patterns: &[$($test),*],
            }),+
        ];

        const LANGUAGE_METADATA: &[LanguageMetadata] = &[
            $(LanguageMetadata {
                id: $id,
                kind: LanguageKind::$kind,
                role: LanguageRole::$role,
                filenames: &[$($filename),*],
                filename_prefixes: &[$($prefix),*],
                filename_suffixes: &[$($suffix),*],
                interpreters: &[$($interpreter),*],
            }),+
        ];
    };
}

language_registry![
    { id: "python", display: "Python", extensions: [".py"], tests: ["test_", "_test.py"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["python", "python3"] },
    { id: "typescript", display: "TypeScript", extensions: [".ts", ".tsx"], tests: [".test.", ".spec."], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["ts-node", "tsx"] },
    { id: "javascript", display: "JavaScript", extensions: [".js", ".jsx", ".mjs", ".cjs"], tests: [".test.", ".spec."], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["node", "nodejs", "deno", "bun"] },
    { id: "go", display: "Go", extensions: [".go"], tests: ["_test.go"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["go"] },
    { id: "rust", display: "Rust", extensions: [".rs"], tests: ["_test.rs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["rust-script"] },
    { id: "java", display: "Java", extensions: [".java"], tests: ["Test.java", "Tests.java"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["java"] },
    { id: "kotlin", display: "Kotlin", extensions: [".kt", ".kts"], tests: ["Test.kt", "Tests.kt"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["kotlin", "kotlinc"] },
    { id: "csharp", display: "C#", extensions: [".cs"], tests: ["Test.cs", "Tests.cs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["dotnet-script"] },
    { id: "c", display: "C", extensions: [".c", ".h"], tests: ["_test.c", "_tests.c"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "cpp", display: "C++", extensions: [".cpp", ".cc", ".cxx", ".hpp", ".hh", ".hxx"], tests: ["_test.", "_tests."], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "ruby", display: "Ruby", extensions: [".rb"], tests: ["_spec.rb", "test_"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["ruby"] },
    { id: "php", display: "PHP", extensions: [".php"], tests: ["Test.php"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["php"] },
    { id: "swift", display: "Swift", extensions: [".swift"], tests: ["Tests.swift"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["swift"] },
    { id: "objectivec", display: "Objective-C", extensions: [".m", ".mm"], tests: ["Tests.m", "Tests.mm"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "bash", display: "Bash", extensions: [".sh", ".bash"], tests: ["test_", "_test.sh", ".bats"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["bash", "sh"] },
    { id: "dart", display: "Dart", extensions: [".dart"], tests: ["_test.dart"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["dart"] },
    { id: "scala", display: "Scala", extensions: [".scala", ".sc"], tests: ["Test.scala", "Spec.scala"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["scala"] },
    { id: "elixir", display: "Elixir", extensions: [".ex", ".exs"], tests: ["_test.exs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["elixir", "iex"] },
    { id: "erlang", display: "Erlang", extensions: [".erl", ".hrl"], tests: ["_test.erl", "_SUITE.erl"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["escript", "erl"] },
    { id: "lua", display: "Lua", extensions: [".lua"], tests: ["_spec.lua", "_test.lua"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["lua", "luajit"] },
    { id: "r", display: "R", extensions: [".r"], tests: ["test-", "-test.r"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["R", "Rscript"] },
    { id: "julia", display: "Julia", extensions: [".jl"], tests: ["runtests.jl", "_test.jl"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["julia"] },
    { id: "haskell", display: "Haskell", extensions: [".hs", ".lhs"], tests: ["Spec.hs", "Test.hs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["runhaskell", "ghci"] },
    { id: "clojure", display: "Clojure", extensions: [".clj", ".cljs", ".cljc"], tests: ["_test.clj", "_test.cljs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["clojure"] },
    { id: "fsharp", display: "F#", extensions: [".fs", ".fsi", ".fsx"], tests: ["Tests.fs", "Test.fs"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["fsi"] },
    { id: "groovy", display: "Groovy", extensions: [".groovy", ".gvy", ".gradle"], tests: ["Test.groovy", "Spec.groovy"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["groovy"] },
    { id: "powershell", display: "PowerShell", extensions: [".ps1", ".psm1", ".psd1"], tests: [".Tests.", ".Test."], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["pwsh", "powershell"] },
    { id: "visualbasic", display: "Visual Basic .NET", extensions: [".vb"], tests: ["Test.vb", "Tests.vb"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "zig", display: "Zig", extensions: [".zig"], tests: ["_test.zig"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["zig"] },
    { id: "d", display: "D", extensions: [".d", ".di"], tests: ["_test.d", "_tests.d"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["rdmd"] },
    { id: "nim", display: "Nim", extensions: [".nim", ".nims"], tests: ["_test.nim", "test_"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["nim"] },
    { id: "crystal", display: "Crystal", extensions: [".cr"], tests: ["_spec.cr"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["crystal"] },
    { id: "ocaml", display: "OCaml", extensions: [".ml", ".mli"], tests: ["_test.ml", "_test.mli"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["ocaml"] },
    { id: "perl", display: "Perl", extensions: [".pl", ".pm", ".t"], tests: [".t", "_test.pl"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["perl"] },
    { id: "matlab", display: "MATLAB", extensions: [".m"], tests: ["Test.m"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["matlab"] },
    { id: "fortran", display: "Fortran", extensions: [".f", ".for", ".f77", ".f90", ".f95", ".f03", ".f08"], tests: ["_test.f90", "_test.f95"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "ada", display: "Ada", extensions: [".ada", ".adb", ".ads"], tests: ["_test.adb", "_tests.adb"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "gdscript", display: "GDScript", extensions: [".gd"], tests: ["test_", "_test.gd"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "solidity", display: "Solidity", extensions: [".sol"], tests: [".t.sol", "Test.sol"], kind: Programming, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "html", display: "HTML", extensions: [".html", ".htm"], tests: [".test.", "_test."], kind: Markup, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "vue", display: "Vue", extensions: [".vue"], tests: [".spec.", ".test."], kind: Markup, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "svelte", display: "Svelte", extensions: [".svelte"], tests: [".spec.", ".test."], kind: Markup, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "astro", display: "Astro", extensions: [".astro"], tests: [".spec.", ".test."], kind: Markup, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "css", display: "CSS", extensions: [".css"], tests: [".test.", "_test."], kind: Stylesheet, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "sass", display: "Sass/SCSS", extensions: [".scss", ".sass"], tests: [".test.", "_test."], kind: Stylesheet, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "less", display: "Less", extensions: [".less"], tests: [".test.", "_test."], kind: Stylesheet, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "sql", display: "SQL", extensions: [".sql"], tests: ["_test.sql", ".test.sql"], kind: Query, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "graphql", display: "GraphQL", extensions: [".graphql", ".gql"], tests: [".test.", ".spec."], kind: Query, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "protobuf", display: "Protocol Buffers", extensions: [".proto"], tests: ["_test.proto"], kind: Schema, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "hcl", display: "HCL/Terraform", extensions: [".hcl", ".tf", ".tfvars"], tests: ["_test.tf", ".tftest.hcl"], kind: Infrastructure, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "nix", display: "Nix", extensions: [".nix"], tests: ["_test.nix"], kind: Infrastructure, role: Primary, filenames: [], prefixes: [], suffixes: [], interpreters: ["nix"] },
    { id: "dockerfile", display: "Dockerfile", extensions: [], tests: [".test", "_test"], kind: Build, role: Primary, filenames: ["Dockerfile"], prefixes: ["Dockerfile."], suffixes: [".dockerfile"], interpreters: [] },
    { id: "make", display: "Make", extensions: [".mk"], tests: ["test", "check"], kind: Build, role: Primary, filenames: ["Makefile", "makefile", "GNUmakefile"], prefixes: [], suffixes: [], interpreters: ["make"] },
    { id: "cmake", display: "CMake", extensions: [".cmake"], tests: ["test", "CTest"], kind: Build, role: Primary, filenames: ["CMakeLists.txt"], prefixes: [], suffixes: [], interpreters: [] },
    { id: "starlark", display: "Starlark", extensions: [".bzl", ".star"], tests: ["_test.bzl", "_test.star"], kind: Build, role: Primary, filenames: ["BUILD", "BUILD.bazel", "WORKSPACE", "WORKSPACE.bazel"], prefixes: [], suffixes: [], interpreters: [] },
    { id: "yaml", display: "YAML", extensions: [".yaml", ".yml"], tests: ["test", "spec"], kind: Data, role: Auxiliary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "json", display: "JSON", extensions: [".json", ".jsonc"], tests: [".test.", ".spec."], kind: Data, role: Auxiliary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "toml", display: "TOML", extensions: [".toml"], tests: [".test.", ".spec."], kind: Data, role: Auxiliary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "xml", display: "XML", extensions: [".xml"], tests: ["Test.xml", "Tests.xml"], kind: Markup, role: Auxiliary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
    { id: "markdown", display: "Markdown", extensions: [".md", ".markdown", ".mdx"], tests: [".test.", ".spec."], kind: Prose, role: Auxiliary, filenames: [], prefixes: [], suffixes: [], interpreters: [] },
];

/// Looks up the registry entry used to describe a language.
pub fn language_by_id(id: &str) -> Option<&'static LanguageSpec> {
    LANGUAGES.iter().find(|language| language.id == id)
}

/// Exposes the category used to group a registered language in analyzer output.
pub fn language_kind(id: &str) -> Option<LanguageKind> {
    metadata_by_id(id).map(|metadata| metadata.kind)
}

/// Exposes whether a registered language contributes source or supporting data.
pub fn language_role(id: &str) -> Option<LanguageRole> {
    metadata_by_id(id).map(|metadata| metadata.role)
}

/// Detects a language from a path and its contents when the path is ambiguous.
pub fn language_for_path(path: &Path) -> Option<&'static LanguageSpec> {
    let content = read_detection_content(path);
    language_for_content(path, &content, None)
}

/// Resolves ambiguous extensions with repository-level markers.
pub fn language_for_file(path: &Path, repo_root: Option<&Path>) -> Option<&'static LanguageSpec> {
    let content = read_detection_content(path);
    language_for_content(path, &content, repo_root)
}

/// Resolves a language without rereading content already available to the caller.
pub fn language_for_content(
    path: &Path,
    content: &str,
    repo_root: Option<&Path>,
) -> Option<&'static LanguageSpec> {
    language_for_path_with_content(path, content, repo_root)
}

fn language_for_path_with_content(
    path: &Path,
    content: &str,
    repo_root: Option<&Path>,
) -> Option<&'static LanguageSpec> {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if let Some(metadata) = LANGUAGE_METADATA.iter().find(|metadata| {
        metadata.filenames.contains(&file_name)
            || metadata
                .filename_prefixes
                .iter()
                .any(|prefix| file_name.starts_with(prefix))
            || metadata
                .filename_suffixes
                .iter()
                .any(|suffix| file_name.ends_with(suffix))
    }) {
        return language_by_id(metadata.id);
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{}", value.to_ascii_lowercase()));

    if let Some(extension) = extension.as_deref() {
        let candidates = LANGUAGE_METADATA.iter().filter(|metadata| {
            language_by_id(metadata.id)
                .is_some_and(|language| language.extensions.contains(&extension))
        });

        for metadata in candidates {
            if metadata.id == "matlab" {
                if looks_like_matlab(content, repo_root) {
                    return language_by_id(metadata.id);
                }
                continue;
            }

            if metadata.id == "objectivec" && extension == ".m" {
                if looks_like_objectivec(content, repo_root) || content.is_empty() {
                    return language_by_id(metadata.id);
                }
                continue;
            }

            if metadata.id == "c" && extension == ".h" && content.contains("@interface") {
                return language_by_id("objectivec");
            }

            return language_by_id(metadata.id);
        }

        if extension == ".m" {
            return language_by_id("objectivec");
        }
    }

    let first_line = content.lines().next().unwrap_or_default().trim();
    if first_line.starts_with("#!") {
        let interpreter = first_line
            .trim_start_matches("#!")
            .split_whitespace()
            .last()
            .unwrap_or_default()
            .rsplit('/')
            .next()
            .unwrap_or_default();
        if let Some(metadata) = LANGUAGE_METADATA
            .iter()
            .find(|metadata| metadata.interpreters.contains(&interpreter))
        {
            return language_by_id(metadata.id);
        }
    }

    None
}

fn metadata_by_id(id: &str) -> Option<&'static LanguageMetadata> {
    LANGUAGE_METADATA.iter().find(|metadata| metadata.id == id)
}

fn read_detection_content(path: &Path) -> String {
    let Ok(file) = std::fs::File::open(path) else {
        return String::new();
    };

    let mut bytes = Vec::new();
    if file
        .take(MAX_DETECTION_BYTES as u64)
        .read_to_end(&mut bytes)
        .is_err()
    {
        return String::new();
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

fn looks_like_objectivec(content: &str, repo_root: Option<&Path>) -> bool {
    let project_marker = repo_root.is_some_and(|root| {
        ["Podfile", "Podfile.lock"]
            .iter()
            .any(|marker| root.join(marker).exists())
    });
    project_marker
        || content.contains("@interface")
        || content.contains("@implementation")
        || content.contains("#import")
        || content.contains("NS_")
}

fn looks_like_matlab(content: &str, repo_root: Option<&Path>) -> bool {
    let project_marker = repo_root.is_some_and(|root| {
        root.read_dir().ok().is_some_and(|entries| {
            entries.flatten().any(|entry| {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                name.ends_with(".prj") || name.ends_with(".mlproj")
            })
        })
    });
    project_marker
        || content.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("function ") || line.starts_with("classdef ") || line.starts_with('%')
        })
}

pub fn is_test_path(path: &Path, language: &LanguageSpec) -> bool {
    let text = path.to_string_lossy();

    let file = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let named_match = language
        .test_patterns
        .iter()
        .any(|pattern| file.contains(pattern) || text.contains(pattern));

    if named_match {
        return true;
    }

    path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("test" | "tests" | "__tests__" | "spec" | "specs")
        )
    })
}
