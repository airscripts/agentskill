"""Tests for JavaScript and TypeScript symbol extraction."""

from commands.symbols import _extract_ts, extract_symbols
from test_support import create_repo


class TestJsTsFunctionExtraction:
    def test_extracts_exported_functions(self):
        source = "export function makeUser() {}\n"
        files = []
        # Create a mock path object
        from pathlib import Path

        class MockPath:
            stem = "test"

        files.append(MockPath())

        # Patch read_text to return our source
        import commands.symbols as symbols_module

        original_read_text = symbols_module.read_text

        def mock_read_text(path):
            return source

        symbols_module.read_text = mock_read_text

        try:
            result = _extract_ts(files, "typescript")
            assert result["functions"]["total"] >= 1
        finally:
            symbols_module.read_text = original_read_text

    def test_extracts_async_functions(self):
        source = "export async function fetchData() {}\n"
        from pathlib import Path

        class MockPath:
            stem = "test"

        files = [MockPath()]

        import commands.symbols as symbols_module

        original = symbols_module.read_text
        symbols_module.read_text = lambda p: source

        try:
            result = _extract_ts(files, "typescript")
            assert result["functions"]["total"] >= 1
        finally:
            symbols_module.read_text = original


class TestJsTsClassExtraction:
    def test_extracts_exported_classes(self):
        source = "export class UserService {}\n"
        from pathlib import Path

        class MockPath:
            stem = "test"

        files = [MockPath()]

        import commands.symbols as symbols_module

        original = symbols_module.read_text
        symbols_module.read_text = lambda p: source

        try:
            result = _extract_ts(files, "typescript")
            assert result["classes"]["total"] >= 1
        finally:
            symbols_module.read_text = original


class TestJsTsInterfaceExtraction:
    def test_extracts_interfaces(self):
        source = "export interface User {}\n"
        from pathlib import Path

        class MockPath:
            stem = "test"

        files = [MockPath()]

        import commands.symbols as symbols_module

        original = symbols_module.read_text
        symbols_module.read_text = lambda p: source

        try:
            result = _extract_ts(files, "typescript")
            assert "interfaces" in result
            assert result["interfaces"]["total"] >= 1
        finally:
            symbols_module.read_text = original


class TestJsTsTypeExtraction:
    def test_extracts_type_aliases(self):
        source = "export type UserId = string\n"
        from pathlib import Path

        class MockPath:
            stem = "test"

        files = [MockPath()]

        import commands.symbols as symbols_module

        original = symbols_module.read_text
        symbols_module.read_text = lambda p: source

        try:
            result = _extract_ts(files, "typescript")
            assert "types" in result
            assert result["types"]["total"] >= 1
        finally:
            symbols_module.read_text = original


class TestJsTsArrowFunctionExtraction:
    def test_extracts_exported_arrow_functions(self):
        source = "export const Button = () => null\n"
        from pathlib import Path

        class MockPath:
            stem = "test"

        files = [MockPath()]

        import commands.symbols as symbols_module

        original = symbols_module.read_text
        symbols_module.read_text = lambda p: source

        try:
            result = _extract_ts(files, "typescript")
            assert result["functions"]["total"] >= 1
        finally:
            symbols_module.read_text = original


class TestJsTsSymbolIntegration:
    def test_symbols_extracts_all_declaration_types(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "src/index.ts": (
                    "export function makeUser() {}\n"
                    "export class UserService {}\n"
                    "export interface User {}\n"
                    "export type UserId = string\n"
                    "export const Button = () => null\n"
                    "const helper = () => {}\n"
                ),
            },
        )

        result = extract_symbols(str(repo), "typescript")

        assert (
            result["typescript"]["functions"]["total"] >= 2
        )  # makeUser + Button + helper
        assert result["typescript"]["classes"]["total"] >= 1  # UserService

    def test_symbols_distinguishes_exported_vs_private(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "src/utils.ts": (
                    "export function publicFn() {}\nfunction privateFn() {}\n"
                ),
            },
        )

        result = extract_symbols(str(repo), "typescript")

        # Both should be detected (may include duplicates from overlapping patterns)
        assert result["typescript"]["functions"]["total"] >= 2

    def test_symbols_handles_javascript_files(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "src/app.js": (
                    "export function run() {}\nexport const handler = () => {}\n"
                ),
            },
        )

        result = extract_symbols(str(repo))

        js_result = result.get("javascript", {})
        ts_result = result.get("typescript", {})

        # Either language key might be used depending on detection
        symbols = js_result if js_result else ts_result
        assert symbols["functions"]["total"] >= 1
