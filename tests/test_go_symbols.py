"""Tests for Go symbol extraction."""

from test_support import create_repo

from agentskill.commands.symbols import extract_symbols


class TestGoSymbolExtraction:
    def test_extracts_exported_functions(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"pkg/main.go": "package main\n\nfunc NewServer() {}\n"},
        )
        result = extract_symbols(str(repo), "go")
        assert result["go"]["functions"]["total"] >= 1

    def test_extracts_unexported_functions(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"pkg/main.go": "package main\n\nfunc helper() {}\n"},
        )
        result = extract_symbols(str(repo), "go")
        assert result["go"]["functions"]["total"] >= 1

    def test_extracts_methods(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"pkg/main.go": "package main\n\nfunc (s *Server) Start() {}\n"},
        )
        result = extract_symbols(str(repo), "go")
        assert result["go"]["methods"]["total"] >= 1

    def test_extracts_structs_and_interfaces(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/main.go": (
                    "package main\n\ntype Server struct{}\ntype Store interface{}\n"
                ),
            },
        )
        result = extract_symbols(str(repo), "go")
        assert result["go"]["types"]["total"] >= 1
        assert "interfaces" in result["go"]

    def test_extracts_constants(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/main.go": ("package main\n\nconst (\n    DefaultPort = 8080\n)\n"),
            },
        )
        result = extract_symbols(str(repo), "go")
        assert result["go"]["constants"]["total"] >= 1

    def test_extracts_variables(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/main.go": 'package main\n\nvar ErrNotFound = errors.New("not found")\n'
            },
        )
        result = extract_symbols(str(repo), "go")
        assert "variables" in result["go"]
        assert result["go"]["variables"]["total"] >= 1

    def test_extracts_type_aliases(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {"pkg/main.go": "package main\n\ntype Status string\n"},
        )
        result = extract_symbols(str(repo), "go")
        assert "type_aliases" in result["go"]
