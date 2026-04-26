"""Shared repository traversal helpers."""

from dataclasses import dataclass
from pathlib import Path

from common.constants import MAX_FILE_BYTES, MAX_FILES_TO_PARSE, should_skip_dir


@dataclass
class WalkStats:
    files_seen: int
    files_yielded: int
    hit_max_files: bool
    oversize_files: int


def walk_repo(
    repo: Path,
    *,
    max_files: int = MAX_FILES_TO_PARSE,
    max_file_bytes: int = MAX_FILE_BYTES,
) -> tuple[list[Path], WalkStats]:
    paths: list[Path] = []
    files_seen = 0
    oversize_files = 0

    def _walk(directory: Path) -> None:
        nonlocal files_seen, oversize_files

        entries = sorted(directory.iterdir(), key=lambda entry: entry.name)
        dirs = [entry for entry in entries if entry.is_dir()]
        files = [entry for entry in entries if entry.is_file()]

        for subdir in dirs:
            if should_skip_dir(subdir.name):
                continue

            if files_seen >= max_files:
                return

            _walk(subdir)

            if files_seen >= max_files:
                return

        for path in files:
            if files_seen >= max_files:
                return

            files_seen += 1

            try:
                size_bytes = path.stat().st_size
            except Exception:
                size_bytes = 0

            if size_bytes > max_file_bytes:
                oversize_files += 1

            paths.append(path)

    _walk(repo)

    stats = WalkStats(
        files_seen=files_seen,
        files_yielded=len(paths),
        hit_max_files=files_seen >= max_files,
        oversize_files=oversize_files,
    )

    return paths, stats
