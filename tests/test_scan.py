import json
import subprocess
import sys

from commands.scan import scan
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
