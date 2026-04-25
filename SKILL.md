---
name: agentskill
description: Analyze code repositories to extract coding style conventions and synthesize an AGENTS.md file. Use when generating, updating, or improving AGENTS.md from actual code. Triggers on 'generate AGENTS.md', 'extract my coding style', 'analyze my repos for style', 'create style guide from my code'.
---

# agentskill

Analyze repos, synthesize AGENTS.md.

## Workflow

1. **Collect** — Ask for repo paths (local or remote).
2. **Extract** — Run `scripts/agentskill.py <repos...>` for JSON or AGENTS.md output.
3. **Check GOTCHAS.md** — Read `references/GOTCHAS.md` before synthesis.
4. **Review examples/** — Browse for templates if needed.
5. **Synthesize** — Use `references/synthesis-prompt.md` with extraction data.
6. **Iterate** — Present draft, adjust per feedback.
7. **Save** — Write final AGENTS.md.

## CLI Usage

```bash
# Generate AGENTS.md (default)
agentskill ~/projects/myapp

# Raw JSON analysis
agentskill ~/projects/repo1 ~/projects/repo2 --json -o report.json

# Skip git or tooling
agentskill ~/projects/myapp --skip-git --skip-tooling

# Install as package
pip install .
agentskill ~/projects/myapp
```

## Principles

- **Extract, don't guess.** Metrics from actual code, not assumptions.
- **Multi-language.** Document every language found. All languages analyzed via the same language-agnostic engine in `engine.py`.
- **Triangulate.** Patterns across repos = personal; single repo = project-specific.
- **Actionable.** "Prefer early extraction" > "Keep functions short."
- **Minimal.** Distinctive rules only, no universal noise.

## Architecture

```
agentskill/
  constants.py              All constants and configuration
  cli.py                    Argument parsing, orchestration
  engine.py                 Language-agnostic code analysis engine
  extractors/
    git.py                  Commit messages, branches, config, remotes
    filesystem.py           File scanning, tooling, project metadata
    structure.py            Directory structure and conventions
    commands.py             Build/test/dev command extraction
  synthesis/
    __init__.py             AgentSynthesizer + SynthesisConfig
```

The analysis engine in `engine.py` is fully language-agnostic. It detects naming conventions, comments, spacing, imports, and errors via text patterns — no language-specific analyzers needed. All languages are analyzed with the same pipeline.

## References

- **GOTCHAS.md** — Extraction/synthesis errors to avoid.
- **synthesis-prompt.md** — LLM prompt template.
- **output-template.md** — Structure guide.
- **examples/** — Successful AGENTS.md samples.