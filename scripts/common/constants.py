"""Shared constants for repository walking."""

MAX_FILES_TO_PARSE = 10_000
MAX_FILE_BYTES = 1_000_000

SKIP_DIRS: set[str] = {
    "node_modules",
    "__pycache__",
    "dist",
    "build",
    "out",
    "target",
    "vendor",
    "third_party",
    ".eggs",
    "site-packages",
    "venv",
    ".venv",
    ".tox",
    ".nox",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "htmlcov",
    ".next",
    ".nuxt",
    "coverage",
}


def should_skip_dir(name: str) -> bool:
    return name in SKIP_DIRS or name.startswith(".")
