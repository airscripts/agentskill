"""Small filesystem helpers shared by analyzer commands."""

from pathlib import Path


def count_lines(path: Path) -> int:
    try:
        with open(path, "rb") as file_obj:
            return file_obj.read().count(b"\n")
    except Exception:
        return 0


def read_text(path: Path, max_bytes: int | None = None) -> str:
    try:
        content = path.read_text(errors="ignore")
    except Exception:
        return ""

    if max_bytes is None:
        return content

    return content[:max_bytes]
