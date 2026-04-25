from commands.graph import build_graph
from test_support import create_repo, create_sample_repo


def test_graph_detects_python_import_edge(tmp_path):
    repo = create_sample_repo(tmp_path)
    result = build_graph(str(repo), "python")
    edges = result["python"]["edges"]

    assert {"from": "pkg.main", "to": "pkg.util", "line": 1} in edges


def test_graph_detects_relative_imports_cycles_and_parse_errors(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "pkg/__init__.py": "\n",
            "pkg/a.py": "from .b import run_b\n\n\ndef run_a():\n    return run_b()\n",
            "pkg/b.py": "from .a import run_a\n\n\ndef run_b():\n    return run_a()\n",
            "pkg/bad.py": "def broken(:\n",
        },
    )

    result = build_graph(str(repo), "python")

    assert {"from": "pkg.a", "to": "pkg.b", "line": 1} in result["python"]["edges"]
    assert result["python"]["circular_dependencies"]
    assert "pkg/bad.py" in result["python"]["parse_errors"]


def test_graph_detects_ts_go_and_monorepo_boundaries(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "package.json": "{}\n",
            "src/util.ts": "export function util() { return 1 }\n",
            "src/app.ts": "import './util'\nconst util = require('./util')\n",
            "go.mod": "module example.com/demo\n",
            "pkg/helper/helper.go": "package helper\n",
            "pkg/main.go": 'package main\nimport (\n    "example.com/demo/pkg/helper"\n)\n',
            "services/api/main.py": "\n",
            "services/web/main.py": "\n",
        },
    )

    result = build_graph(str(repo))

    assert {"from": "src/app", "to": "src/util", "line": 1} in result["typescript"][
        "edges"
    ]

    assert {"from": "pkg", "to": "pkg/helper", "line": 0} in result["go"]["edges"]
    assert result["monorepo_boundaries"]["detected"] is True
