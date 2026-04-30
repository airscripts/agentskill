from test_support import create_repo, create_sample_repo

from agentskill.commands.symbols import _classify, _find_affixes, extract_symbols


def test_symbols_extracts_python_patterns(tmp_path):
    repo = create_sample_repo(tmp_path)
    result = extract_symbols(str(repo), "python")

    assert result["python"]["functions"]["total"] >= 3
    assert result["python"]["classes"]["patterns"]["PascalCase"]["count"] >= 1


def test_symbols_handles_empty_and_malformed_python_files_without_errors(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "pkg/empty.py": "",
            "pkg/bad.py": "def broken(:\n",
        },
    )

    result = extract_symbols(str(repo), "python")

    assert result["python"]["functions"]["total"] == 0
    assert result["python"]["classes"]["total"] == 0
    assert result["python"]["constants"]["total"] == 0

    assert result["python"]["private_members"] == {
        "single_underscore": 0,
        "double_underscore": 0,
        "examples": [],
    }

    assert result["python"]["files"]["total"] == 2


def test_symbols_classification_and_affixes():
    assert _classify("__init__") == "dunder"
    assert _classify("_hidden") == "private"
    assert _classify("VALUE_NAME") == "SCREAMING_SNAKE_CASE"
    assert _classify("snake_case") == "snake_case"
    assert _classify("PascalCase") == "PascalCase"
    assert _classify("camelCase") == "camelCase"
    assert _classify("misc") == "other"

    affixes = _find_affixes(
        ["buildGraph", "buildTree", "buildValue", "buildNode", "buildThing"]
    )

    assert any(
        entry["pattern"] == "bu_ prefix" or entry["pattern"] == "build_ prefix"
        for entry in affixes
    )


def test_symbols_extracts_typescript_and_go(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/app.ts": (
                "export function buildThing() {}\n"
                "const makeWidget = () => {}\n"
                "export class WidgetService {}\n"
                "export interface WidgetShape {}\n"
                "export type WidgetType = string\n"
                "export const VALUE_NAME = 1\n"
            ),
            "pkg/main.go": (
                "package main\n"
                "type Worker struct{}\n"
                "const (\n    MainValue = 1\n)\n"
                "var ExportedValue string\n"
                "func RunThing() {}\n"
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["typescript"]["classes"]["total"] >= 2

    assert (
        result["typescript"]["constants"]["patterns"]["SCREAMING_SNAKE_CASE"]["count"]
        >= 1
    )

    assert result["go"]["functions"]["total"] >= 1
    assert result["go"]["constants"]["total"] >= 1
    assert result["go"]["variables"]["total"] >= 1


def test_symbols_extracts_typescript_arrow_and_const_forms_precisely(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/app.ts": (
                "export const buildThing = () => {}\n"
                "const localThing = () => {}\n"
                "export const VALUE_NAME = 1\n"
                "const localValue = 2\n"
            ),
        },
    )

    result = extract_symbols(str(repo), "typescript")

    assert result["typescript"]["functions"]["total"] == 2
    assert result["typescript"]["constants"]["total"] == 1

    assert result["typescript"]["constants"]["patterns"] == {
        "SCREAMING_SNAKE_CASE": {"count": 1, "pct": 100.0}
    }


def test_symbols_extracts_go_grouped_constants_and_methods_precisely(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "pkg/main.go": (
                "package main\n"
                "type Worker struct{}\n"
                "type Reader interface{}\n"
                "type Alias string\n"
                "const (\n"
                "    FirstValue = 1\n"
                "    SecondValue = 2\n"
                ")\n"
                "var ExportedValue string\n"
                "func RunThing() {}\n"
                "func (w *Worker) StartThing() {}\n"
            ),
        },
    )

    result = extract_symbols(str(repo), "go")

    assert result["go"]["functions"]["total"] == 2
    assert result["go"]["methods"]["total"] == 1
    assert result["go"]["types"]["total"] == 1
    assert result["go"]["interfaces"]["total"] == 1
    assert result["go"]["type_aliases"]["total"] == 1
    assert result["go"]["constants"]["total"] == 2
    assert result["go"]["variables"]["total"] == 1


def test_symbols_extracts_java_and_kotlin(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/main/java/com/acme/UserService.java": (
                "package com.acme;\n\n"
                "public class UserService {\n"
                "    public UserService() {}\n"
                "    public void start() {}\n"
                '    private String helper() { return ""; }\n'
                "}\n\n"
                "interface Store {}\n"
                "enum Status {}\n"
                "@interface Marker {}\n"
            ),
            "src/main/kotlin/com/acme/App.kt": (
                "package com.acme\n\n"
                "class UserService\n"
                "data class User(val id: String)\n"
                "sealed class Result\n"
                "interface Store\n"
                "object AppConfig\n"
                "enum class Status { OK }\n\n"
                "fun start() {}\n"
                "private fun helper() {}\n"
                'const val VERSION = "1"\n'
                'val name = "x"\n'
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["java"]["classes"]["patterns"]["PascalCase"]["count"] >= 1
    assert result["java"]["methods"]["total"] >= 2
    assert result["java"]["interfaces"]["total"] >= 1
    assert result["java"]["enums"]["total"] >= 1
    assert result["java"]["annotations"]["total"] >= 1
    assert result["java"]["constructors"]["total"] >= 1

    assert result["kotlin"]["classes"]["total"] >= 3
    assert result["kotlin"]["interfaces"]["total"] >= 1
    assert result["kotlin"]["objects"]["total"] >= 1
    assert result["kotlin"]["enums"]["total"] >= 1
    assert result["kotlin"]["functions"]["total"] >= 2
    assert result["kotlin"]["constants"]["total"] >= 1
    assert result["kotlin"]["properties"]["total"] >= 1


def test_symbols_extracts_csharp_c_and_cpp(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/UserService.cs": (
                "namespace Acme.Service;\n\n"
                "public class UserService {\n"
                "    public void Start() {}\n"
                '    private string Normalize() { return ""; }\n'
                "}\n\n"
                "public interface IUserStore {}\n"
                "internal struct UserId {}\n"
                "public enum Status {}\n"
                "public record User(string Id);\n"
            ),
            "src/main.c": (
                "#define MAX_SIZE 100\n\n"
                "typedef struct User User;\n"
                "struct User {};\n\n"
                "enum Status { OK };\n\n"
                "int add(int a, int b) {\n    return a + b;\n}\n"
            ),
            "src/app.cpp": (
                "namespace acme {}\n\n"
                "template <typename T>\n"
                "class Box {};\n"
                "class UserService {};\n"
                "struct User {};\n"
                "enum class Status {};\n\n"
                "int add(int a, int b) {\n    return a + b;\n}\n"
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["csharp"]["classes"]["total"] >= 1
    assert result["csharp"]["methods"]["total"] >= 2
    assert result["csharp"]["interfaces"]["total"] >= 1
    assert result["csharp"]["structs"]["total"] >= 1
    assert result["csharp"]["enums"]["total"] >= 1
    assert result["csharp"]["records"]["total"] >= 1

    assert result["c"]["functions"]["total"] >= 1
    assert result["c"]["structs"]["total"] >= 1
    assert result["c"]["enums"]["total"] >= 1
    assert result["c"]["typedefs"]["total"] >= 1
    assert result["c"]["macros"]["total"] >= 1

    assert result["cpp"]["functions"]["total"] >= 1
    assert result["cpp"]["namespaces"]["total"] >= 1
    assert result["cpp"]["classes"]["total"] >= 2
    assert result["cpp"]["structs"]["total"] >= 1
    assert result["cpp"]["enums"]["total"] >= 1
    assert result["cpp"]["templates"]["total"] >= 1


def test_symbols_extracts_ruby_php_and_bash(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "lib/user_service.rb": (
                "module MyApp\nend\n\n"
                "class UserService\n"
                "  def call\n  end\n\n"
                "  def self.build\n  end\n"
                "end\n"
            ),
            "src/Service/UserService.php": (
                "<?php\nnamespace App\\Service;\n\n"
                "class UserService {\n"
                "    public function start() {}\n"
                "    private function helper() {}\n"
                "}\n\n"
                "interface Store {}\n"
                "trait HasLogger {}\n"
                "enum Status {}\n"
                "function utility() {}\n"
            ),
            "scripts/deploy": (
                "#!/usr/bin/env bash\n"
                "deploy() {\n  echo deploy\n}\n\n"
                "function build() {\n  echo build\n}\n"
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["ruby"]["modules"]["total"] >= 1
    assert result["ruby"]["classes"]["total"] >= 1
    assert result["ruby"]["methods"]["total"] >= 1
    assert result["ruby"]["class_methods"]["total"] >= 1

    assert result["php"]["classes"]["total"] >= 1
    assert result["php"]["methods"]["total"] >= 2
    assert result["php"]["interfaces"]["total"] >= 1
    assert result["php"]["traits"]["total"] >= 1
    assert result["php"]["enums"]["total"] >= 1
    assert result["php"]["functions"]["total"] >= 1

    assert result["bash"]["functions"]["total"] >= 2


def test_symbols_ignores_commented_bash_and_php_declarations(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "scripts/deploy": (
                "#!/usr/bin/env bash\n# fake() {\n# }\nreal() {\n  echo deploy\n}\n"
            ),
            "src/Service/UserService.php": (
                "<?php\n"
                "// function fake() {}\n"
                "/* class Fake {}\n"
                "function nope() {}\n"
                "*/\n"
                "class UserService {\n"
                "    public function start() {}\n"
                "}\n"
                "function utility() {}\n"
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["bash"]["functions"]["total"] == 1
    assert result["php"]["classes"]["total"] == 1
    assert result["php"]["methods"]["total"] == 1
    assert result["php"]["functions"]["total"] == 1


def test_symbols_extracts_swift_and_objectivec(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "Sources/MyApp/User.swift": (
                "public struct User {}\n"
                "final class UserService {}\n"
                "public enum Status {}\n"
                "protocol Store {}\n\n"
                "public func makeUser() {}\n"
                "private func helper() {}\n\n"
                "extension UserService {}\n"
            ),
            "Sources/UserService.h": "@interface UserService : NSObject\n@end\n",
            "Sources/UserService.m": (
                "@implementation UserService\n"
                "- (void)start {}\n"
                "+ (instancetype)shared {}\n"
                "@end\n\n"
                "@protocol UserStore\n@end\n"
            ),
        },
    )

    result = extract_symbols(str(repo))

    assert result["swift"]["structs"]["total"] >= 1
    assert result["swift"]["classes"]["total"] >= 1
    assert result["swift"]["enums"]["total"] >= 1
    assert result["swift"]["protocols"]["total"] >= 1
    assert result["swift"]["functions"]["total"] >= 2
    assert result["swift"]["extensions"]["total"] >= 1

    assert result["objectivec"]["interfaces"]["total"] >= 1
    assert result["objectivec"]["implementations"]["total"] >= 1
    assert result["objectivec"]["methods"]["total"] >= 1
    assert result["objectivec"]["class_methods"]["total"] >= 1
    assert result["objectivec"]["protocols"]["total"] >= 1


def test_symbols_extracts_swift_extensions_without_extra_types(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "Sources/MyApp/Extensions.swift": (
                "extension UserService {}\nextension AppStore {}\n"
            ),
        },
    )

    result = extract_symbols(str(repo), "swift")

    assert result["swift"]["extensions"]["total"] == 2
    assert result["swift"]["functions"]["total"] == 0
    assert "classes" not in result["swift"]


def test_symbols_distinguishes_objectivec_instance_and_class_methods(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "Sources/UserService.h": "@interface UserService : NSObject\n@end\n",
            "Sources/UserService.m": (
                "@implementation UserService\n"
                "- (void)start {}\n"
                "+ (instancetype)shared {}\n"
                "@end\n"
            ),
        },
    )

    result = extract_symbols(str(repo), "objectivec")

    assert result["objectivec"]["methods"]["total"] == 1
    assert result["objectivec"]["class_methods"]["total"] == 1


def test_symbols_reports_invalid_repo_paths(tmp_path):
    missing = tmp_path / "missing"

    assert extract_symbols(str(missing)) == {
        "error": f"path does not exist: {missing}",
        "script": "symbols",
    }
