import json
import subprocess
import sys

from commands import scan as scan_command
from commands.scan import scan
from common.walk import walk_repo as shared_walk_repo
from test_support import ROOT, create_sample_repo


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
