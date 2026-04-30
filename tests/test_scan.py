import json
import subprocess
import sys
from pathlib import Path

from test_support import ROOT, create_repo, create_sample_repo

from agentskill.commands import scan as scan_command
from agentskill.commands.scan import scan
from agentskill.common.walk import walk_repo as shared_walk_repo


def test_scan_collects_language_summary(tmp_path):
    repo = create_sample_repo(tmp_path)
    result = scan(str(repo))

    assert result["summary"]["by_language"]["python"]["file_count"] >= 4
    assert "pkg/main.py" in result["read_order"]


def test_scan_wrapper_still_executes_directly(tmp_path):
    repo = create_sample_repo(tmp_path)

    completed = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "scan.py"), str(repo), "--pretty"],
        capture_output=True,
        text=True,
        check=True,
    )

    output = json.loads(completed.stdout)

    assert output["summary"]["total_files"] >= 4


def test_scan_reports_invalid_repo_paths(tmp_path):
    missing = tmp_path / "missing"
    file_path = tmp_path / "file.txt"
    file_path.write_text("hello\n")

    assert scan(str(missing)) == {
        "error": f"path does not exist: {missing}",
        "script": "scan",
    }

    assert scan(str(file_path)) == {
        "error": f"not a directory: {file_path}",
        "script": "scan",
    }


def test_scan_excludes_skipped_directories_and_keeps_language_filters(tmp_path):
    repo = create_sample_repo(tmp_path)
    (repo / ".git").mkdir()
    (repo / ".git" / "ignored.py").write_text("print('ignored')\n")
    (repo / "node_modules").mkdir()
    (repo / "node_modules" / "ignored.js").write_text("console.log('ignored')\n")

    result = scan(str(repo), "python")

    assert all(not path.startswith(".git/") for path in result["read_order"])
    assert all(not path.startswith("node_modules/") for path in result["read_order"])
    assert set(result["summary"]["by_language"]) == {"python"}


def test_scan_handles_walk_repo_truncation_without_error(monkeypatch, tmp_path):
    repo = create_sample_repo(tmp_path)

    monkeypatch.setattr(
        scan_command,
        "walk_repo",
        lambda path: shared_walk_repo(path, max_files=1),
    )

    result = scan(str(repo))

    assert result["summary"]["total_files"] <= 1
    assert len(result["tree"]) <= 1


def test_scan_detects_shebang_bash_scripts_without_extension(tmp_path):
    repo = tmp_path / "sample_repo"
    repo.mkdir()
    script = repo / "deploy"
    script.write_text("#!/usr/bin/env bash\necho deploy\n")

    result = scan(str(repo))

    assert result["summary"]["by_language"]["bash"]["file_count"] == 1
    assert "deploy" in result["read_order"]


def test_scan_empty_repo_returns_empty_structures(tmp_path):
    repo = create_repo(tmp_path)

    result = scan(str(repo))

    assert result == {
        "tree": [],
        "summary": {
            "total_files": 0,
            "by_language": {},
            "max_depth": 0,
            "avg_depth": 0.0,
        },
        "read_order": [],
    }


def test_scan_zero_match_filter_returns_no_results(tmp_path):
    repo = create_repo(tmp_path, {"pkg/main.py": "def run():\n    return 1\n"})

    result = scan(str(repo), "typescript")

    assert result["tree"] == []
    assert result["summary"]["total_files"] == 0
    assert result["summary"]["by_language"] == {}
    assert result["read_order"] == []


def _supports_symlinks(tmp_path: Path) -> bool:
    target = tmp_path / "target.txt"
    link = tmp_path / "link.txt"
    target.write_text("x")

    try:
        link.symlink_to(target)
    except (NotImplementedError, OSError):
        return False

    return link.is_symlink()


def test_scan_skips_symlinked_files(tmp_path):
    if not _supports_symlinks(tmp_path):
        return

    repo = create_repo(
        tmp_path,
        {
            "pkg/main.py": "def run():\n    return 1\n",
        },
    )

    (repo / "pkg" / "main_link.py").symlink_to(repo / "pkg" / "main.py")
    result = scan(str(repo))

    paths = [entry["path"] for entry in result["tree"]]
    assert paths == ["pkg/main.py"]


def test_scan_skips_symlinked_directories(tmp_path):
    if not _supports_symlinks(tmp_path):
        return

    repo = create_repo(
        tmp_path,
        {
            "pkg/main.py": "def run():\n    return 1\n",
            "shared/util.py": "def util():\n    return 2\n",
        },
    )

    (repo / "pkg" / "linked_shared").symlink_to(
        repo / "shared", target_is_directory=True
    )

    result = scan(str(repo))

    paths = [entry["path"] for entry in result["tree"]]
    assert "pkg/linked_shared/util.py" not in paths
    assert paths == ["pkg/main.py", "shared/util.py"]


def test_scan_skips_binary_like_files_in_valid_repo(tmp_path):
    repo = create_repo(
        tmp_path,
        {
            "pkg/main.py": "def run():\n    return 1\n",
            "assets/logo.png": "fakepng\n",
            "archive/data.zip": "fakezip\n",
        },
    )

    result = scan(str(repo))

    assert [entry["path"] for entry in result["tree"]] == ["pkg/main.py"]
