from common.constants import should_skip_dir
from common.fs import count_lines, read_text


def test_fs_helpers_handle_existing_and_missing_files(tmp_path):
    path = tmp_path / "sample.txt"
    path.write_text("a\nb\nc\n")

    assert count_lines(path) == 3
    assert read_text(path, max_bytes=3) == "a\nb"
    assert count_lines(tmp_path / "missing.txt") == 0
    assert read_text(tmp_path / "missing.txt") == ""


def test_should_skip_dir_covers_hidden_known_and_normal_names():
    assert should_skip_dir(".git") is True
    assert should_skip_dir("node_modules") is True
    assert should_skip_dir("src") is False
