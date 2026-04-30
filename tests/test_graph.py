from test_support import create_repo, create_sample_repo

from agentskill.commands.graph import build_graph


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

    assert {"from": "src/app.ts", "to": "src/util.ts", "line": 1} in result[
        "typescript"
    ]["edges"]

    assert {"from": "pkg", "to": "pkg/helper", "line": 2} in result["go"]["edges"]
    assert result["monorepo_boundaries"]["detected"] is True


def test_graph_resolves_typescript_relative_require_and_ignores_external_imports(
    tmp_path,
):
    repo = create_repo(
        tmp_path,
        {
            "src/util.ts": "export function util() { return 1 }\n",
            "src/app.ts": (
                'import React from "react"\n'
                "import './util'\n"
                "const util = require('./util')\n"
            ),
        },
    )

    result = build_graph(str(repo), "typescript")

    assert result["typescript"]["edges"] == [
        {"from": "src/app.ts", "to": "src/util.ts", "line": 2},
        {"from": "src/app.ts", "to": "src/util.ts", "line": 3},
    ]


def test_graph_resolves_typescript_index_and_nested_relative_imports(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/features/app.ts": ("import '../lib'\nimport '../shared/util'\n"),
            "src/lib/index.ts": "export const value = 1\n",
            "src/shared/util.ts": "export const util = 1\n",
        },
    )

    result = build_graph(str(repo), "typescript")

    assert result["typescript"]["edges"] == [
        {"from": "src/features/app.ts", "to": "src/lib/index.ts", "line": 1},
        {"from": "src/features/app.ts", "to": "src/shared/util.ts", "line": 2},
    ]


def test_graph_resolves_go_internal_imports_and_ignores_external_packages(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "go.mod": "module example.com/demo\n",
            "pkg/helper/helper.go": "package helper\n",
            "internal/deep/deep.go": "package deep\n",
            "pkg/main.go": (
                "package main\n"
                "import (\n"
                '    "fmt"\n'
                '    "example.com/demo/pkg/helper"\n'
                '    "example.com/demo/internal/deep"\n'
                ")\n"
            ),
        },
    )

    result = build_graph(str(repo), "go")

    assert result["go"]["edges"] == [
        {"from": "pkg", "to": "pkg/helper", "line": 2},
        {"from": "pkg", "to": "internal/deep", "line": 2},
    ]


def test_graph_resolves_go_single_import_to_nested_package(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "go.mod": "module example.com/demo\n",
            "pkg/service/http/client.go": "package http\n",
            "pkg/app/app.go": (
                'package app\n\nimport "example.com/demo/pkg/service/http"\n'
            ),
        },
    )

    result = build_graph(str(repo), "go")

    assert result["go"]["edges"] == [
        {"from": "pkg/app", "to": "pkg/service/http", "line": 3}
    ]


def test_graph_detects_java_and_kotlin_internal_imports(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/main/java/com/acme/App.java": (
                "package com.acme;\n\n"
                "import com.acme.service.UserService;\n"
                "import java.util.List;\n\n"
                "public class App {}\n"
            ),
            "src/main/java/com/acme/service/UserService.java": (
                "package com.acme.service;\n\npublic class UserService {}\n"
            ),
            "src/main/kotlin/com/acme/Main.kt": (
                "package com.acme\n\n"
                "import com.acme.service.UserService\n"
                "import kotlinx.coroutines.runBlocking\n\n"
                "fun main() {}\n"
            ),
            "src/main/kotlin/com/acme/service/UserService.kt": (
                "package com.acme.service\n\nclass UserService\n"
            ),
        },
    )

    result = build_graph(str(repo))

    assert {
        "from": "src/main/java/com/acme/App.java",
        "to": "src/main/java/com/acme/service/UserService.java",
        "line": 3,
    } in result["java"]["edges"]

    assert {
        "from": "src/main/kotlin/com/acme/Main.kt",
        "to": "src/main/kotlin/com/acme/service/UserService.kt",
        "line": 3,
    } in result["kotlin"]["edges"]


def test_graph_detects_csharp_and_c_family_internal_edges(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "src/App.cs": (
                "using System;\n"
                "using Acme.Service.Core;\n\n"
                "namespace Acme.Service;\n\n"
                "public class App {}\n"
            ),
            "src/Core/UserService.cs": (
                "namespace Acme.Service.Core;\n\npublic class UserService {}\n"
            ),
            "src/main.c": (
                '#include "util.h"\n'
                '#include "../include/project/config.h"\n'
                "#include <stdio.h>\n"
            ),
            "src/util.h": "int add(int a, int b);\n",
            "include/project/config.h": '#define APP_NAME "demo"\n',
            "src/app.cpp": '#include "project/service.hpp"\n#include <vector>\n',
            "include/project/service.hpp": "class UserService {};\n",
        },
    )

    result = build_graph(str(repo))

    assert {
        "from": "src/App.cs",
        "to": "src/Core/UserService.cs",
        "line": 2,
    } in result["csharp"]["edges"]

    assert {"from": "src/main.c", "to": "src/util.h", "line": 1} in result["c"]["edges"]

    assert {
        "from": "src/main.c",
        "to": "include/project/config.h",
        "line": 2,
    } in result["c"]["edges"]

    assert {
        "from": "src/app.cpp",
        "to": "include/project/service.hpp",
        "line": 1,
    } in result["cpp"]["edges"]


def test_graph_detects_ruby_php_and_bash_internal_edges(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "lib/my_app/service.rb": "class UserService\nend\n",
            "lib/my_app/helper.rb": "module MyApp\nend\n",
            "app/main.rb": (
                'require "json"\n'
                'require "my_app/service"\n'
                'require_relative "../lib/my_app/helper"\n'
            ),
            "src/Service/UserService.php": (
                "<?php\nnamespace App\\Service;\n\n"
                "use App\\Repository\\UserRepository;\n"
                "use DateTime;\n\nclass UserService {}\n"
            ),
            "src/Repository/UserRepository.php": (
                "<?php\nnamespace App\\Repository;\n\nclass UserRepository {}\n"
            ),
            "scripts/deploy.sh": (
                "#!/usr/bin/env bash\n"
                "source ./lib/common.sh\n"
                ". ./lib/common.sh\n"
                'source "$DYNAMIC_FILE"\n'
            ),
            "scripts/lib/common.sh": "echo common\n",
        },
    )

    result = build_graph(str(repo))

    assert {"from": "app/main.rb", "to": "lib/my_app/service.rb", "line": 2} in result[
        "ruby"
    ]["edges"]

    assert {"from": "app/main.rb", "to": "lib/my_app/helper.rb", "line": 3} in result[
        "ruby"
    ]["edges"]

    assert {
        "from": "src/Service/UserService.php",
        "to": "src/Repository/UserRepository.php",
        "line": 4,
    } in result["php"]["edges"]

    assert {
        "from": "scripts/deploy.sh",
        "to": "scripts/lib/common.sh",
        "line": 2,
    } in result["bash"]["edges"]


def test_graph_detects_swift_and_objectivec_internal_edges(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "Sources/MyApp/App.swift": (
                "import Foundation\n"
                "import MyAppCore\n"
                "@testable import MyApp\n\n"
                "public struct App {}\n"
            ),
            "Sources/MyAppCore/Service.swift": "public struct Service {}\n",
            "Sources/UserService.m": (
                '#import "UserService.h"\n'
                '#import "Config.h"\n'
                "#import <Foundation/Foundation.h>\n"
            ),
            "Sources/UserService.h": "@interface UserService : NSObject\n@end\n",
            "Sources/Config.h": '#define APP_NAME @"demo"\n',
        },
    )

    result = build_graph(str(repo))

    assert {
        "from": "Sources/MyApp/App.swift",
        "to": "Sources/MyAppCore/Service.swift",
        "line": 2,
    } in result["swift"]["edges"]

    assert {
        "from": "Sources/UserService.m",
        "to": "Sources/UserService.h",
        "line": 1,
    } in result["objectivec"]["edges"]

    assert {
        "from": "Sources/UserService.m",
        "to": "Sources/Config.h",
        "line": 2,
    } in result["objectivec"]["edges"]


def test_graph_reports_invalid_repo_paths(tmp_path):
    missing = tmp_path / "missing"

    assert build_graph(str(missing)) == {
        "error": f"path does not exist: {missing}",
        "script": "graph",
    }
