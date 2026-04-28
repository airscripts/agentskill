"""Tests for Rust symbol extraction."""

from commands.symbols import extract_symbols
from test_support import create_repo


class TestRustSymbolExtraction:
    def test_extracts_public_functions(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "pub fn parse() {}\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert result["rust"]["functions"]["total"] >= 1

    def test_extracts_private_functions(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "fn helper() {}\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert result["rust"]["functions"]["total"] >= 1

    def test_extracts_structs(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "pub struct Parser {}\nstruct Internal {}\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert result["rust"]["structs"]["total"] >= 2

    def test_extracts_enums(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "pub enum Status { Active, Inactive }\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert result["rust"]["enums"]["total"] >= 1

    def test_extracts_traits(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "pub trait Store {}\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert "traits" in result["rust"]
        assert result["rust"]["traits"]["total"] >= 1

    def test_extracts_impls(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "impl Parser {}\nimpl Store for Parser {}\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert "impls" in result["rust"]
        assert result["rust"]["impls"]["total"] >= 1

    def test_extracts_constants(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": 'pub const VERSION: &str = "1";\n'},
        )
        result = extract_symbols(str(repo), "rust")
        assert result["rust"]["constants"]["total"] >= 1

    def test_extracts_statics(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"src/lib.rs": "static COUNTER: u64 = 0;\n"},
        )
        result = extract_symbols(str(repo), "rust")
        assert "statics" in result["rust"]
        assert result["rust"]["statics"]["total"] >= 1
