from pathlib import Path

from test_support import EXAMPLES_DIR

from agentskill.commands.config import detect
from agentskill.commands.graph import build_graph
from agentskill.commands.measure import measure
from agentskill.commands.scan import scan
from agentskill.commands.symbols import extract_symbols
from agentskill.commands.tests import analyze_tests
from agentskill.common.languages import all_language_specs

RELEASE_LANGUAGE_MATRIX = (
    "python",
    "javascript",
    "typescript",
    "go",
    "rust",
    "java",
    "kotlin",
    "csharp",
    "c",
    "cpp",
    "ruby",
    "php",
    "swift",
    "objectivec",
    "bash",
)


def _example_path(language: str) -> Path:
    return EXAMPLES_DIR / language


def _assert_no_error_payload(result: dict) -> None:
    assert isinstance(result, dict)
    assert "error" not in result


def _symbol_key(language: str) -> str:
    return "typescript" if language == "javascript" else language


def _test_key(language: str) -> str:
    return "typescript" if language == "javascript" else language


def _assert_config_signal(language: str, config_result: dict) -> None:
    if language == "python":
        assert config_result["python"]["linter"]["name"] == "ruff"
    elif language == "javascript":
        assert config_result["javascript"]["formatter"]["name"] == "prettier"
    elif language == "typescript":
        assert config_result["typescript"]["type_checker"]["name"] == "tsc"
    elif language == "go":
        assert config_result["go"]["formatter"]["name"] == "gofmt"
    elif language == "rust":
        assert config_result["rust"]["formatter"]["name"] == "rustfmt"
        assert config_result["rust"]["linter"]["name"] == "clippy"
    elif language == "java":
        assert config_result["java"]["build_tool"] == "maven"
    elif language == "kotlin":
        assert config_result["kotlin"]["build_tool"] == "gradle"
    elif language == "csharp":
        assert config_result["csharp"]["build_tool"] == "msbuild"
    elif language == "c":
        assert config_result["c"]["build_tool"] == "make"
    elif language == "cpp":
        assert config_result["cpp"]["build_tool"] == "cmake"
    elif language == "ruby":
        assert config_result["ruby"]["build_tool"] == "bundler"
    elif language == "php":
        assert config_result["php"]["build_tool"] == "composer"
    elif language == "swift":
        assert config_result["swift"]["build_tool"] == "swiftpm"
    elif language == "objectivec":
        assert config_result["objectivec"]["build_tool"] == "cocoapods"
    elif language == "bash":
        assert "[*.sh]" in config_result["editorconfig"]
    else:
        raise AssertionError(f"unexpected language: {language}")


def _assert_representative_signal(
    language: str,
    graph_result: dict,
    symbols_result: dict,
    tests_result: dict,
) -> None:
    if language == "python":
        assert {"from": "src.app", "to": "src.util", "line": 1} in graph_result[
            "python"
        ]["edges"]
    elif language == "javascript":
        assert {
            "from": "src/index.js",
            "to": "src/util.js",
            "line": 1,
        } in graph_result["javascript"]["edges"]
    elif language == "typescript":
        assert {
            "from": "src/index.ts",
            "to": "src/user.ts",
            "line": 1,
        } in graph_result["typescript"]["edges"]
    elif language == "go":
        assert {"from": "cmd/app", "to": "internal/service", "line": 3} in graph_result[
            "go"
        ]["edges"]
    elif language == "rust":
        assert {"from": "src/lib.rs", "to": "src/parser.rs", "line": 1} in graph_result[
            "rust"
        ]["edges"]
    elif language == "java":
        assert {
            "from": "src/main/java/com/example/App.java",
            "to": "src/main/java/com/example/service/UserService.java",
            "line": 3,
        } in graph_result["java"]["edges"]
    elif language == "kotlin":
        assert {
            "from": "src/main/kotlin/com/example/App.kt",
            "to": "src/main/kotlin/com/example/service/UserService.kt",
            "line": 3,
        } in graph_result["kotlin"]["edges"]
    elif language == "csharp":
        assert {
            "from": "src/App.cs",
            "to": "src/Core/UserService.cs",
            "line": 1,
        } in graph_result["csharp"]["edges"]
    elif language == "c":
        assert {"from": "src/main.c", "to": "src/util.h", "line": 1} in graph_result[
            "c"
        ]["edges"]
    elif language == "cpp":
        assert {
            "from": "src/app.cpp",
            "to": "include/example/service.hpp",
            "line": 1,
        } in graph_result["cpp"]["edges"]
    elif language == "ruby":
        assert {
            "from": "lib/example/service.rb",
            "to": "lib/example/helper.rb",
            "line": 1,
        } in graph_result["ruby"]["edges"]
    elif language == "php":
        assert {
            "from": "src/Service/UserService.php",
            "to": "src/Repository/UserRepository.php",
            "line": 4,
        } in graph_result["php"]["edges"]
    elif language == "swift":
        assert symbols_result["swift"]["structs"]["total"] >= 1
    elif language == "objectivec":
        assert {
            "from": "Sources/UserService.m",
            "to": "Sources/UserService.h",
            "line": 1,
        } in graph_result["objectivec"]["edges"]
    elif language == "bash":
        assert {
            "from": "scripts/deploy.sh",
            "to": "scripts/lib/common.sh",
            "line": 3,
        } in graph_result["bash"]["edges"]
    else:
        raise AssertionError(f"unexpected language: {language}")

    assert tests_result[_test_key(language)]["test_files"] >= 1


def test_release_language_matrix_matches_registry_and_examples():
    registered = {spec.id for spec in all_language_specs()}
    example_dirs = {path.name for path in EXAMPLES_DIR.iterdir() if path.is_dir()}

    assert set(RELEASE_LANGUAGE_MATRIX) == registered
    assert set(RELEASE_LANGUAGE_MATRIX) <= example_dirs


def test_release_language_support_matrix_is_verified_end_to_end():
    verdicts: dict[str, str] = {}

    for language in RELEASE_LANGUAGE_MATRIX:
        repo = _example_path(language)

        scan_result = scan(str(repo))
        measure_result = measure(str(repo))
        config_result = detect(str(repo))
        graph_result = build_graph(str(repo))
        symbols_result = extract_symbols(str(repo))
        tests_result = analyze_tests(str(repo))

        _assert_no_error_payload(scan_result)
        _assert_no_error_payload(measure_result)
        _assert_no_error_payload(config_result)
        _assert_no_error_payload(graph_result)
        _assert_no_error_payload(symbols_result)
        _assert_no_error_payload(tests_result)

        assert language in scan_result["summary"]["by_language"]
        assert language in measure_result
        assert language in graph_result
        assert _symbol_key(language) in symbols_result
        assert _test_key(language) in tests_result

        _assert_config_signal(language, config_result)
        _assert_representative_signal(
            language,
            graph_result,
            symbols_result,
            tests_result,
        )

        verdicts[language] = "verified"

    assert verdicts == dict.fromkeys(RELEASE_LANGUAGE_MATRIX, "verified")
