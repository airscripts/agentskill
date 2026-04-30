"""Small filesystem helpers shared by analyzer commands."""

from pathlib import Path

from agentskill.common.constants import MAX_FILE_BYTES


def validate_repo(path: str) -> Path:
    repo = Path(path).resolve()

    if not repo.exists():
        raise ValueError(f"path does not exist: {path}")

    if not repo.is_dir():
        raise ValueError(f"not a directory: {path}")

    return repo


def count_lines(path: Path) -> int:
    try:
        with open(path, "rb") as file_obj:
            return sum(
                chunk.count(b"\n") for chunk in iter(lambda: file_obj.read(65_536), b"")
            )
    except Exception:
        return 0


def read_text(path: Path, max_bytes: int | None = MAX_FILE_BYTES) -> str:
    try:
        with open(path, "rb") as file_obj:
            raw = file_obj.read() if max_bytes is None else file_obj.read(max_bytes)
    except Exception:
        return ""

    return raw.decode(errors="ignore")
