# agentskill

Generate AGENTS.md from your actual coding style.

## What It Does

1. **Scans repos** -- git history, source files, configs.
2. **Detects style** -- naming, errors, comments, spacing.
3. **Synthesizes** -- clean AGENTS.md via LLM.

## Install

```bash
# Generate AGENTS.md
python3 scripts/agentskill.py ~/projects/myapp

# Raw JSON analysis
python3 scripts/agentskill.py ~/projects/repo1 ~/projects/repo2 --json -o report.json

# Install as package
pip install .
agentskill ~/projects/myapp
```

## Usage

With OpenClaw:
```
> Analyze my coding style from ~/projects/myapp
```

Standalone:
```bash
python3 scripts/agentskill.py ~/projects/myapp -o report.json
```

## Detected Patterns

| Category | Patterns |
|----------|----------|
| **Naming** | case style, avg length, descriptiveness |
| **Spacing** | blank lines between blocks |
| **Comments** | style, density, what gets explained |
| **Errors** | unwrap vs ? vs Result |
| **Git** | commit prefixes, branches |
| **Tooling** | linters, CI configs |

## Languages

Every language is supported.

## Structure

```
agentskill/
├── SKILL.md
├── README.md
├── scripts/
│   └── agentskill.py       # CLI entry point
├── agentskill/              # Core package
│   ├── __init__.py
│   ├── cli.py
│   ├── constants.py
│   ├── extractors/
│   │   ├── git.py
│   │   └── filesystem.py
│   ├── analyzers/
│   │   ├── base.py
│   │   └── language/
│   │       ├── rust.py
│   │       ├── python.py
│   │       └── generic.py
│   └── synthesis/
│       └── __init__.py
├── references/
│   ├── GOTCHAS.md
│   ├── OUTPUT-TEMPLATE.md
│   └── SYNTHESIS-PROMPT.md
├── examples/
│   └── SAMPLE.md
└── tests/
    └── test_agentskill.py
```

## License

MIT
