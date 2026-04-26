from pathlib import Path

from common.constants import MAX_FILE_BYTES, should_skip_dir
from common.fs import count_lines, read_text, validate_repo
from common.walk import walk_repo


def test_fs_helpers_handle_existing_and_missing_files(tmp_path):
    path = tmp_path / "sample.txt"
    path.write_text("a\nb\nc\n")

    assert count_lines(path) == 3
    assert read_text(path, max_bytes=3) == "a\nb"
    assert count_lines(tmp_path / "missing.txt") == 0
    assert read_text(tmp_path / "missing.txt") == ""


def test_read_text_uses_real_byte_limits_and_tolerates_binary_data(tmp_path):
    path = tmp_path / "sample.txt"
    path.write_bytes(b"abc\xffdef\n")

    assert read_text(path) == "abcdef\n"
    assert read_text(path, max_bytes=4) == "abc"

    large_path = tmp_path / "large.txt"
    large_path.write_text("x" * (MAX_FILE_BYTES + 10))

    assert len(read_text(large_path)) == MAX_FILE_BYTES


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


def test_walk_repo_skips_hidden_dirs_orders_paths_and_tracks_limits(tmp_path):
    (tmp_path / ".git").mkdir()
    (tmp_path / ".git" / "config").write_text("[core]\n")
    (tmp_path / "node_modules").mkdir()
    (tmp_path / "node_modules" / "lib.js").write_text("alert(1)\n")
    (tmp_path / "pkg").mkdir()
    (tmp_path / "pkg" / "b.py").write_text("print('b')\n")
    (tmp_path / "pkg" / "a.py").write_text("print('a')\n")
    (tmp_path / "root.py").write_text("print('root')\n")

    paths, stats = walk_repo(tmp_path)

    assert [str(path.relative_to(tmp_path)) for path in paths] == [
        "pkg/a.py",
        "pkg/b.py",
        "root.py",
    ]

    assert stats.files_seen == 3
    assert stats.files_yielded == 3
    assert stats.hit_max_files is False
    assert stats.oversize_files == 0


def test_walk_repo_honors_max_files_and_marks_oversize_files(tmp_path):
    (tmp_path / "a.py").write_text("a\n")
    (tmp_path / "b.py").write_text("b\n")
    (tmp_path / "big.py").write_text("x" * 20)

    paths, stats = walk_repo(tmp_path, max_files=2, max_file_bytes=10)

    assert [path.name for path in paths] == ["a.py", "b.py"]
    assert stats.files_seen == 2
    assert stats.files_yielded == 2
    assert stats.hit_max_files is True

    paths, stats = walk_repo(tmp_path, max_files=10, max_file_bytes=10)

    assert [path.name for path in paths] == ["a.py", "b.py", "big.py"]
    assert stats.oversize_files == 1
