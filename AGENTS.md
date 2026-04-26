# AGENTS.md

## 1. Overview

agentskill is a single-repo Python CLI that analyzes another repository and emits JSON signals used to synthesize an `AGENTS.md`. The codebase is organized around a thin root CLI in `cli.py`, concrete analyzers in `scripts/commands/`, shared output and orchestration helpers in `scripts/lib/`, low-level filesystem helpers in `scripts/common/`, reference docs and examples at the repo root, and a separate `tests/` tree that exercises both analyzer internals and CLI entrypoints.

## 2. Repository Structure

```text
agentskill/
  cli.py                    # root entry point; argument parsing and dispatch only
  pyproject.toml            # packaging, pytest, ruff, coverage config
  README.md                 # user-facing overview and command reference
  SYSTEM.md                 # synthesis spec for generated AGENTS.md files
  SKILL.md                  # operational workflow for the skill
  AGENTS.md                 # conventions for this repo
  LICENSE                   # MIT license text
  references/
    GOTCHAS.md              # extraction and synthesis failure modes
  examples/
    SINGLE_LANGUAGE.md      # reference output for a standard single-language repo
    MULTI_LANGUAGE.md       # reference output for a multi-language single repo
    MONOREPO.md             # reference output for a monorepo
  scripts/
    commands/               # analyzer implementations
      config.py             # formatter/linter/type-checker detection
      git.py                # commit and branch analysis
      graph.py              # import graph and boundary detection
      measure.py            # formatting measurements
      scan.py               # tree and file inventory
      symbols.py            # naming-pattern extraction
      tests.py              # test-structure analysis
    lib/
      output.py             # shared JSON output helpers
      runner.py             # aggregate analyzer orchestration
    common/
      constants.py          # shared repo-walk constants
      fs.py                 # small filesystem helpers
    scan.py                 # thin direct-execution wrapper
    measure.py              # thin direct-execution wrapper
    config.py               # thin direct-execution wrapper
    git.py                  # thin direct-execution wrapper
    graph.py                # thin direct-execution wrapper
    symbols.py              # thin direct-execution wrapper
    tests.py                # thin direct-execution wrapper
  tests/                    # pytest suite; separate tree, not colocated
    conftest.py             # sys.path test bootstrap
    test_support.py         # shared repo/setup helpers for tests
```

- New analyzer logic goes in `scripts/commands/`, not in `cli.py`.
- Shared CLI plumbing belongs in `scripts/lib/`; low-level reusable helpers belong in `scripts/common/`.
- Files under `scripts/*.py` stay as thin wrappers around `commands.<name>.main`.
- New tests go in `tests/` as `test_<subject>.py`; this repo does not colocate tests beside source files.
- New examples for unfamiliar repo shapes belong in `examples/`, not mixed into `references/`.
- Keep the repo root small: entrypoint, metadata, docs/spec files, and no business logic outside `cli.py`.

## 5. Commands and Workflows

```bash
# Install editable package
pip install -e .

# Install dev dependencies from project metadata
python -m pip install -r <(python - <<'PY'
import tomllib
with open("pyproject.toml", "rb") as f:
    deps = tomllib.load(f)["project"]["optional-dependencies"]["dev"]
print("\n".join(deps))
PY
)

# Run all analyzers
python cli.py analyze <repo> --pretty

# Run individual analyzers through the root CLI
python cli.py scan <repo> --pretty
python cli.py measure <repo> --lang python --pretty
python cli.py config <repo> --pretty
python cli.py git <repo> --pretty
python cli.py graph <repo> --pretty
python cli.py symbols <repo> --pretty
python cli.py tests <repo> --pretty

# Direct wrapper execution
python scripts/scan.py <repo> --pretty

# Local checks
ruff format .
ruff check .
pytest
```

- `python cli.py analyze <repo> --pretty` is the canonical aggregate workflow.
- `python cli.py <command> <repo> --pretty` is the main single-analyzer interface; `python scripts/<name>.py <repo> --pretty` remains supported as a thin direct wrapper.
- Use `pytest` as the canonical test command; that is what the repo advertises in `README.md` and what the analyzer detects from `pyproject.toml`.
- Use `ruff format .` and `ruff check .` for local formatting and lint passes; Ruff is the only configured code-quality tool in project metadata.

## 6. Code Formatting

### Python

Configured tooling: Ruff is configured in `pyproject.toml` for linting, and the repo documents `ruff format .` as the formatting command. No formatter-specific overrides are declared, so follow the observed formatting directly.

**Indentation:** 4 spaces.

```python
def _single_script_cmd(command_name: str, args: argparse.Namespace) -> int:
    metadata = COMMANDS[command_name]
    extra_kwargs = {}

    if metadata["supports_lang"]:
        extra_kwargs["lang_filter"] = getattr(args, "lang", None)
```

**Line length:** keep ordinary code in the mid-70s or below; measured p95 is 76. Long regex literals and long docstring summary lines still appear.

```python
CONVENTIONAL_PREFIX_RE = re.compile(r"^([a-z][a-z0-9_-]*)(\([^)]+\))?(!)?\s*:\s*(.+)$")
```

**Blank lines — top-level:** 2 blank lines between top-level functions and constants-to-functions transitions.

```python
from lib.output import run_and_output, write_output
from lib.runner import COMMANDS, run_many


def cmd_analyze(args: argparse.Namespace) -> int:
    result = run_many(args.repos, getattr(args, "lang", None))
```

**Blank lines — methods:** not applicable in the dominant code path; classes are effectively absent in source.

**Blank lines — class open:** not applicable in source for the same reason.

**Blank lines — after imports:** usually 1 blank line before module constants; 2 blank lines before the first function when a file goes straight from imports into functions.

```python
from lib.output import run_and_output

GIT_TIMEOUT = 30
GIT_HASH_LENGTH = 40
```

```python
from test_support import create_sample_repo

import cli


def test_cli_scan_outputs_json(tmp_path, capsys):
```

**Blank lines — end of file:** every file ends with exactly 1 trailing newline.

**Trailing whitespace:** stripped.

**Brace / bracket placement:** opening delimiters stay on the same line; multiline calls and literals use hanging indentation with the closing delimiter on its own line.

```python
return run_and_output(
    metadata["fn"],
    repo=args.repo,
    pretty=args.pretty,
    out=getattr(args, "out", None),
    script_name=command_name,
    extra_kwargs=extra_kwargs,
)
```

**Quote style:** double quotes everywhere in normal Python code and docstrings.

```python
if not repo.exists():
    return {"error": f"path does not exist: {repo_path}", "script": "git"}
```

**Spacing — operators:** spaces around assignment and binary operators.

```python
avg_parents = sum(parent_counts) / len(parent_counts)
bucket = prefix if prefix else "unprefixed"
```

**Spacing — inside brackets:** no inner padding.

```python
if COMMANDS[command_name]["supports_lang"]:
```

**Spacing — after commas:** always a single space.

```python
return None, None, False
```

**Spacing — colons:** no space before `:`, one space after `:` in dict literals, none in type annotations.

```python
prefixes[k] = {
    "count": v["count"],
    "pct": round(v["count"] / total * 100, 1),
    "example": v["example"],
}
```

```python
def run_many(repos: list[str], lang_filter: str | None = None) -> dict:
```

**Spacing — decorators:** decorators are flush with the function they decorate, with no blank line between decorator and `def`.

```python
@pytest.fixture
def repo_fixture(tmp_path):
    return create_sample_repo(tmp_path)
```

**Import block formatting:** one import per line; groups are separated by a blank line. In source files, stdlib imports come first and local package imports follow. Tests often place local test-support imports before a direct `import cli`.

```python
import json

from test_support import create_sample_repo

import cli
```

**Trailing commas:** used in multiline calls, dicts, lists, and imports.

```python
exit_code = cli.main(
    ["analyze", str(repo_one), str(repo_two), "--out", str(out_file)]
)
```

**Line continuation:** implicit via open brackets; no backslash continuations.

**Semicolons:** absent.

## 7. Naming Conventions

### Python

**Functions and methods:** public entrypoints use plain snake_case names like `analyze`, `measure`, `build_graph`, `extract_symbols`; internal helpers use `_snake_case`.

```python
def analyze(repo_path: str) -> dict:
def build_graph(repo_path: str, lang_filter: str | None = None) -> dict:
def _detect_merge_strategy(cwd: str) -> tuple[str, str]:
```

**CLI command helpers:** root CLI helper names read as verbs or command phrases.

```python
def cmd_analyze(args: argparse.Namespace) -> int:
def _single_script_cmd(command_name: str, args: argparse.Namespace) -> int:
```

**Constants:** module constants use `SCREAMING_SNAKE_CASE`.

```python
GIT_TIMEOUT = 30
PRETTIER_CONFIG_FILES = [
MAKEFILE_NAMES = ["Makefile", "makefile", "GNUmakefile"]
```

**Private members:** internal helpers overwhelmingly use a single leading underscore; there is no meaningful double-underscore pattern.

```python
def _parse_toml_value(s: str):
def _measure_line_lengths(all_lengths: list[int]) -> dict:
def _command_kwargs(command_name: str, lang_filter: str | None) -> dict:
```

**File names:** source and helper files use lowercase snake_case; test files use `test_<subject>.py`; package markers use `__init__.py`.

```text
scripts/lib/output.py
scripts/common/constants.py
tests/test_measure.py
tests/test_support.py
```

**Directory names:** lowercase simple nouns: `scripts`, `commands`, `common`, `lib`, `tests`, `references`, `examples`.

**Test function names:** `test_<behavior>` with long descriptive tails is the dominant pattern.

```python
def test_cli_writes_out_file_and_multi_repo_results(tmp_path):
def test_graph_detects_relative_imports_cycles_and_parse_errors(tmp_path):
```

**Fixture names:** when fixtures appear, they use snake_case nouns.

```python
def sample_fixture():
def repo_fixture(tmp_path):
```

## 8. Type Annotations

### Python

- Public functions are annotated on parameters and return types.
- Internal helpers are also usually annotated; this repo does not reserve annotations only for public APIs.
- Use built-in generics like `list[str]`, `dict[str, int]`, and union syntax like `str | None` instead of `List`, `Dict`, or `Optional`.
- Container-rich return types are accepted directly in signatures instead of being hidden behind aliases.
- No dedicated type-checker is configured; `pyproject.toml` declares Ruff lint settings only.

```python
def main(argv: list[str] | None = None) -> int:
def _run(cmd: list[str], cwd: str) -> tuple[int, str]:
def run_many(repos: list[str], lang_filter: str | None = None) -> dict:
```

```python
def _analyze_branches(cwd: str) -> tuple[dict[str, int], int, list[str]]:
```

## 9. Imports

### Python

- Import order is not strict stdlib/third-party/local in the test suite; document and mimic the local file pattern instead of forcing generic ordering.
- In source files, stdlib imports come first, then local package imports separated by one blank line.
- In tests, plain `import cli` may intentionally sit after `from test_support import ...`.
- No wildcard imports.
- No `__future__` imports appear.

Canonical source import block:

```python
import re
import subprocess
import sys
from pathlib import Path

from lib.output import run_and_output
```

Canonical test import block:

```python
import json

from test_support import create_sample_repo

import cli
```

## 10. Error Handling

### Python

- Functions that validate filesystem or git state usually return structured error dicts instead of raising outward.
- Small wrappers catch broad exceptions and convert them into machine-readable payloads.
- File helpers return empty-string or zero fallbacks on read failure.
- Tests assert exact error payloads rather than exception classes.

```python
if not repo.exists():
    return {"error": f"path does not exist: {repo_path}", "script": "git"}

if not (repo / ".git").exists():
    return {"error": "not a git repository", "script": "git"}
```

```python
try:
    result = command_fn(repo, **kwargs)
except Exception as exc:
    result = {"error": str(exc), "script": script_name}
```

```python
def read_text(path: Path, max_bytes: int | None = None) -> str:
    try:
        content = path.read_text(errors="ignore")
    except Exception:
        return ""
```

## 11. Comments and Docstrings

### Python

- Modules almost always begin with a triple-double-quoted docstring.
- Function docstrings are short, declarative, and describe return shape or intent; many small helpers omit them.
- Inline comments are rare and only appear when a small detail would otherwise be unclear.
- Comments are not used for narration of obvious code.

```python
"""Aggregate analyzer execution for the top-level CLI."""
```

```python
def _detect_merge_strategy(cwd: str) -> tuple[str, str]:
    """Return (strategy, evidence)."""
```

```python
j = last_import  # 1-indexed -> 0-indexed
```

## 12. Testing

### Python

- Framework: `pytest`.
- Tests live in `tests/`, not beside source files.
- Test files are named `test_<subject>.py`.
- `tests/conftest.py` is used for shared import-path bootstrap; helper setup code is centralized in `tests/test_support.py`.
- Tests use plain `assert` statements and `tmp_path`, `capsys`, and `monkeypatch` fixtures heavily.
- The suite tests both pure functions and command-line entrypoints.

```python
def test_cli_scan_outputs_json(tmp_path, capsys):
    repo = create_sample_repo(tmp_path)
    exit_code = cli.main(["scan", str(repo), "--pretty"])

    assert exit_code == 0

    output = json.loads(capsys.readouterr().out)
    assert output["summary"]["total_files"] >= 4
```

```python
def test_detect_merge_strategy_paths(monkeypatch):
    monkeypatch.setattr(git_command, "_run", lambda cmd, cwd: (1, ""))
    assert _detect_merge_strategy("repo") == ("unknown", "insufficient data")
```

```python
def write(repo: Path, rel_path: str, content: str) -> Path:
    path = repo / rel_path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    return path
```

## 13. Git

- Commit subjects follow conventional-commit-style prefixes without scopes in normal repo history: `refactor:`, `feat:`, `docs:`, `fix:`, `chore:`, `test:`.
- Branch names use a slash-separated prefix pattern when they are not trunk branches.
- History is linear enough that the analyzer detects a rebase workflow rather than merge commits.
- Commit bodies exist, but not on every commit.

Examples from current history:

```text
refactor: code clean up
feat: add AGENTS.md file
docs: mark all roadmap items complete
fix: add missing frontmatter to SKILL.md
```

Branch example:

```text
chore/strip-comments
```

## 14. Dependencies and Tooling

- Packaging uses `setuptools.backends.legacy:build` with `setuptools>=68` in `build-system.requires`.
- The published console script is `agentskill = "cli:main"`.
- Runtime requirement is Python `>=3.10`.
- Dev dependencies are `pytest`, `pytest-cov`, and `ruff`.
- Ruff targets `py39`, excludes cache and virtualenv directories, and lint rules are configured in `pyproject.toml` with `select = ["E4", "E7", "E9", "F", "I"]` and `ignore = ["E402"]`.
- Coverage omits `tests/*`.
- License is MIT.

```toml
[build-system]
requires = ["setuptools>=68"]
build-backend = "setuptools.backends.legacy:build"

[project]
name = "agentskill"
version = "0.1.0"
requires-python = ">=3.10"

[project.scripts]
agentskill = "cli:main"
```

```toml
[tool.ruff]
target-version = "py39"

[tool.ruff.lint]
select = ["E4", "E7", "E9", "F", "I"]
ignore = ["E402"]
```

## 15. Red Lines

- Do not put analyzer implementation logic into `cli.py`; keep it as dispatch and orchestration only.
- Do not add colocated tests under `scripts/`; tests belong under `tests/`.
- Do not introduce `Optional[...]`, `List[...]`, or `Dict[...]` annotation style; the repo uses built-in generics and `| None`.
- Do not switch quote style to single quotes in ordinary Python code.
- Do not start using wildcard imports or `__future__` imports without a repo-wide reason.
- Do not rely on exceptions escaping CLI/output wrappers when the existing pattern returns `{"error": ..., "script": ...}` payloads.
- Do not fold example reference files into `references/` or analyzer code; keep sample outputs in `examples/`.
