"""Filesystem extraction utilities."""

import os
from pathlib import Path
from typing import Dict, List

from ..constants import EXTENSIONS, SKIP_DIRS, HIDDEN_PREFIX, GIT_DIR


def is_hidden_path(root: Path) -> bool:
    """Check if path contains hidden directories."""
    return any(part.startswith(HIDDEN_PREFIX) for part in root.parts)


def should_skip_dir(root: Path) -> bool:
    """Check if directory should be skipped."""
    return bool(SKIP_DIRS.intersection(root.parts))


def is_git_repo(repo_path: str) -> bool:
    """Check if path is a git repository."""
    return os.path.isdir(os.path.join(repo_path, GIT_DIR))


def scan_source_files(repo_path: str) -> Dict[str, List[Path]]:
    """Scan for source files by language."""
    files_by_lang = {lang: [] for lang in EXTENSIONS}

    for root_str, _, files in os.walk(repo_path):
        root = Path(root_str)
        if is_hidden_path(root) or should_skip_dir(root):
            continue

        for file in files:
            filepath = root / file
            for lang, exts in EXTENSIONS.items():
                if any(file.endswith(ext) for ext in exts):
                    files_by_lang[lang].append(filepath)

    return {k: v for k, v in files_by_lang.items() if v}


def detect_tooling(repo_path: str) -> Dict:
    """Detect tooling configs (linters, formatters, CI)."""
    from ..constants import TOOL_FILES

    repo = Path(repo_path)
    detected = {}

    for file_pattern, tool in TOOL_FILES.items():
        if (repo / file_pattern).exists():
            detected[tool] = True

    if (repo / ".github" / "workflows").exists():
        detected["GitHub Actions CI"] = True

    # Detect lockfiles
    lockfiles = {
        "Cargo.lock": "cargo",
        "package-lock.json": "npm",
        "yarn.lock": "yarn",
        "pnpm-lock.yaml": "pnpm",
        "poetry.lock": "poetry",
        "Pipfile.lock": "pipenv",
        "go.sum": "go",
        "Gemfile.lock": "bundler",
        "composer.lock": "composer",
        "mix.lock": "mix",
        "flake.lock": "nix",
    }
    for lockfile, tool in lockfiles.items():
        if (repo / lockfile).exists():
            detected[f"{tool} (locked)"] = True

    # Detect test configs
    test_configs = [
        "pytest.ini", "setup.cfg", "tox.ini", ".pytest_cache",
        "jest.config.js", "vitest.config.ts", "karma.conf.js",
        "Cargo.toml",  # has [dev-dependencies]
        "go.mod",      # has _test.go convention
    ]
    for tc in test_configs:
        if (repo / tc).exists():
            detected["test-framework"] = True
            break

    return detected


def get_project_metadata(repo_path: str) -> Dict:
    """Extract project metadata from common files."""
    repo = Path(repo_path)
    meta = {}

    # Read README for project name hint
    readme_files = ["README.md", "README.rst", "README.txt", "README"]
    for rf in readme_files:
        if (repo / rf).exists():
            content = (repo / rf).read_text(errors='ignore')[:500]
            lines = content.split('\n')
            if lines:
                meta["project_name"] = lines[0].strip().lstrip('#').strip()
            break

    # Detect license
    license_files = ["LICENSE", "LICENSE.txt", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE"]
    for lf in license_files:
        if (repo / lf).exists():
            meta["has_license"] = True
            # Try to detect license type
            content = (repo / lf).read_text(errors='ignore')[:500].lower()
            if "mit" in content:
                meta["license_type"] = "MIT"
            elif "apache" in content:
                meta["license_type"] = "Apache-2.0"
            elif "gpl" in content:
                meta["license_type"] = "GPL"
            elif "bsd" in content:
                meta["license_type"] = "BSD"
            break

    return meta
