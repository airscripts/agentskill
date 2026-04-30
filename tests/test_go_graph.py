"""Tests for Go graph extraction."""

from test_support import create_repo

from agentskill.commands.graph import (
    _detect_go_module,
    _detect_go_packages,
    _extract_go_imports,
    _strip_go_comments,
    build_graph,
)


class TestGoCommentStripping:
    def test_strip_line_comments(self):
        source = 'import "fmt" // standard library'
        result = _strip_go_comments(source)
        assert "//" not in result
        assert "import" in result

    def test_strip_block_comments(self):
        source = '/* comment */ import "fmt"'
        result = _strip_go_comments(source)
        assert "/*" not in result
        assert "*/" not in result


class TestGoImportExtraction:
    def test_extract_single_import(self):
        source = 'import "fmt"\n'
        imports = _extract_go_imports(source)
        specs = [spec for spec, _ in imports]
        assert "fmt" in specs

    def test_extract_import_block(self):
        source = 'import (\n\t"fmt"\n\t"github.com/org/project/internal/foo"\n)\n'
        imports = _extract_go_imports(source)
        specs = [spec for spec, _ in imports]
        assert "fmt" in specs
        assert "github.com/org/project/internal/foo" in specs

    def test_ignores_imports_in_comments(self):
        source = '// import "fmt"\n/* import "os" */\n'
        imports = _extract_go_imports(source)
        assert len(imports) == 0


class TestGoModuleDetection:
    def test_detect_module_path(self, tmp_path):
        (tmp_path / "go.mod").write_text("module github.com/acme/service\n")
        result = _detect_go_module(tmp_path)
        assert result == "github.com/acme/service"

    def test_returns_empty_without_go_mod(self, tmp_path):
        result = _detect_go_module(tmp_path)
        assert result == ""


class TestGoPackageDetection:
    def test_detect_package_names(self, tmp_path):
        (tmp_path / "pkg").mkdir()
        (tmp_path / "pkg" / "main.go").write_text("package main\n")
        (tmp_path / "pkg" / "helper.go").write_text("package main\n")

        files = [tmp_path / "pkg" / "main.go", tmp_path / "pkg" / "helper.go"]
        result = _detect_go_packages(files, tmp_path)
        assert result.get("pkg") == "main"


class TestGoGraphIntegration:
    def test_graph_resolves_internal_imports(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "go.mod": "module github.com/acme/service\n",
                "cmd/server/main.go": (
                    'package main\n\nimport (\n\t"fmt"\n\t"github.com/acme/service/internal/api"\n)\n'
                ),
                "internal/api/router.go": "package api\n",
            },
        )

        result = build_graph(str(repo), "go")

        assert any(
            e["from"] == "cmd/server" and e["to"] == "internal/api"
            for e in result["go"]["edges"]
        )

    def test_graph_ignores_stdlib_imports(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "go.mod": "module github.com/acme/service\n",
                "main.go": 'package main\n\nimport "fmt"\n',
            },
        )

        result = build_graph(str(repo), "go")
        assert len(result["go"]["edges"]) == 0

    def test_graph_detects_package_boundaries(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "go.mod": "module example.com/demo\n",
                "pkg/helper/helper.go": "package helper\n",
                "pkg/main.go": 'package main\nimport (\n\t"example.com/demo/pkg/helper"\n)\n',
            },
        )

        result = build_graph(str(repo))
        assert "go" in result
        assert any(e["to"] == "pkg/helper" for e in result["go"]["edges"])
