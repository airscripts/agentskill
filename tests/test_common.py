from pathlib import Path

from common.constants import should_skip_dir
from common.fs import count_lines, read_text, validate_repo


def test_fs_helpers_handle_existing_and_missing_files(tmp_path):
    path = tmp_path / "sample.txt"
    path.write_text("a\nb\nc\n")

    assert count_lines(path) == 3
    assert read_text(path, max_bytes=3) == "a\nb"
    assert count_lines(tmp_path / "missing.txt") == 0
    assert read_text(tmp_path / "missing.txt") == ""


def test_validate_repo_accepts_directories_and_rejects_invalid_paths(tmp_path):
    repo = validate_repo(str(tmp_path))

    assert repo == tmp_path.resolve()
    assert isinstance(repo, Path)

    missing = tmp_path / "missing"

    try:
        validate_repo(str(missing))
        raise AssertionError("validate_repo should reject missing paths")
    except ValueError as exc:
        assert str(exc) == f"path does not exist: {missing}"

    file_path = tmp_path / "sample.txt"
    file_path.write_text("hello\n")

    try:
        validate_repo(str(file_path))
        raise AssertionError("validate_repo should reject file paths")
    except ValueError as exc:
        assert str(exc) == f"not a directory: {file_path}"


def test_should_skip_dir_covers_hidden_known_and_normal_names():
    assert should_skip_dir(".git") is True
    assert should_skip_dir("node_modules") is True
    assert should_skip_dir("src") is False
