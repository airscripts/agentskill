# agentskill

Generate AGENTS.md from your actual coding style.

## What It Does

1. **Scans repos** -- source files, git history, tooling configs.
2. **Detects style** -- naming, error handling, comments, spacing, imports.
3. **Synthesizes** -- complete AGENTS.md with overview, per-language sections, git patterns, red lines.

## Install

```bash
# Generate AGENTS.md
python3 scripts/agentskill.py ~/projects/myapp

# Raw JSON analysis
python3 scripts/agentskill.py ~/projects/repo1 ~/projects/repo2 --json -o report.json

# Skip sections
python3 scripts/agentskill.py ~/projects/myapp --skip-git --skip-tooling

# Install as package
pip install .
agentskill ~/projects/myapp
```

## Detected Patterns

| Category | What It Finds |
|----------|---------------|
| **Naming** | Dominant case style per category (vars, functions, types, consts), average length |
| **Error Handling** | Rust: unwrap/expect/?/panic/Result counts. Python: try/except/raise/assert/with |
| **Comments** | Density, doc vs normal, style (/// vs // vs #) |
| **Spacing** | Average blank lines between code blocks |
| **Imports** | Rust: std/crate/external. Python: stdlib/third-party |
| **Git** | Commit prefixes, average length, branch naming, signing |
| **Tooling** | Cargo, npm, pytest, CI, lockfiles, Docker, editorconfig, license detection |

## Languages

Every language is supported. Deep analysis for Rust and Python. Generic heuristics (line counts, comment density, file metrics) for all others including Go, JavaScript, TypeScript, C, C++, Java, C#, Ruby, PHP, Swift, Kotlin, Scala, Zig, Nim, Haskell, OCaml, Elixir, Clojure, Lua, Perl, R, Julia, Dart, Groovy, F#, Crystal, D, and Bash.

## Structure

```
agentskill/
├── SKILL.md
├── README.md
├── setup.py
├── scripts/
│   └── agentskill.py       # CLI entry point
├── agentskill/              # Core package
│   ├── __init__.py
│   ├── cli.py
│   ├── constants.py
│   ├── extractors/
│   │   ├── git.py          # Commits, branches, config, remotes
│   │   └── filesystem.py   # Scanning, tooling, metadata
│   ├── analyzers/
│   │   ├── base.py         # LanguageAnalyzer interface
│   │   └── language/
│   │       ├── rust.py     # Deep analysis
│   │       ├── python.py   # Deep analysis
│   │       └── generic.py  # Fallback
│   └── synthesis/
│       └── __init__.py     # AGENTS.md generation
├── references/
│   ├── GOTCHAS.md
│   ├── OUTPUT-TEMPLATE.md
│   └── SYNTHESIS-PROMPT.md
├── examples/
│   └── SAMPLE.md
└── tests/
    └── test_agentskill.py  # 96 tests
```

## Testing

```bash
python3 tests/test_agentskill.py
```

96 tests covering constants, all analyzers, extractors, synthesis, and CLI.

## License

MIT