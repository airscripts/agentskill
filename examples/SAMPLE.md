# AGENTS.md — Coding Style

## Overview

Multi-language codebase spanning python. Analyzed 1 repository.

Key principles distilled from actual patterns:
- Self-documenting code over verbose comments
- Descriptive names over terse abbreviations

## Cross-Language Patterns

Patterns holding across all detected languages:

### Naming
- **vars:** snake_case
- **functions:** snake_case
- **types:** PascalCase
- **consts:** SCREAMING_SNAKE_CASE


## Python
### Naming
- **Vars:** snake_case
- **Functions:** snake_case
- **Types:** PascalCase
- **Consts:** SCREAMING_SNAKE_CASE

### Error Handling
- `try_block`: 21 occurrences
- `catch_block`: 23 occurrences
- `throw`: 3 occurrences

### Comments
- **Density:** 9.9%

## Repository Structure

### File Naming
- **Dominant style:** PascalCase

### Directory Depth
- **Max:** 3 levels
- **Average:** 3.0 levels

### Test Organization
- **Location:** separate_dirs

### Module Patterns
- **Init files:** __init__.py detected

## Commands and Workflows

### Install
```bash
pip install -e .
```
```bash
python setup.py install
```

### Dev
```bash
python scripts/agentskill.py
```


## Git

### Commits
- **Prefixes:** `refactor`, `feat`, `docs`, `fix`, `test`
- **Avg length:** 55 chars

## Tooling

Detected configurations:
- git
- license
- readme
- setuptools
- test-framework

## Dependencies

No dependency information detected.

## Red Lines

Explicit avoidances based on actual patterns:

- No mixing naming conventions within categories

## Code Examples

Actual patterns from the codebase:

### Example 1

```
# From test_agentskill.py:

class TestRunner:
    def __init__(self):
```

### Example 2

```
# From cli.py:

def analyze_repository(repo_path: str) -> dict:
    """Analyze a single repository using the language-agnostic engine."""
```

### Example 3

```
# From engine.py:
@dataclass
class AnalysisResult:
    """Result of analyzing a codebase."""
```

### Example 4

```
# From filesystem.py:

def is_hidden_path(root: Path) -> bool:
    """Check if path contains hidden directories."""
```

### Example 5

```
# From commands.py:

def extract_commands(repo_path: str) -> Dict:
    """Extract build, test, and dev commands from project configs."""
```

### Example 6

```
# From structure.py:

def extract_repo_structure(repo_path: str) -> Dict:
    """Extract repository directory structure and conventions."""
```

### Example 7

```
# From git.py:

def run_git_log(repo_path: str) -> str:
    """Run git log and return output."""
```

### Example 8

```
# From __init__.py:
@dataclass
class SynthesisConfig:
    """Configuration for AGENTS.md generation."""
```


---

**Source:** agentskill
**Files analyzed:** 13
**Confidence:** High on naming patterns; Medium on tooling (config-dependent)
