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
python3 scripts/agentskill.py ~/projects/myapp

# Raw JSON analysis
python3 scripts/agentskill.py ~/projects/repo1 ~/projects/repo2 --json -o report.json

# Skip git or tooling
python3 scripts/agentskill.py ~/projects/myapp --skip-git --skip-tooling

# Install as package
pip install .
agentskill ~/projects/myapp
```

## Principles

- **Extract, don't guess.** Metrics from actual code, not assumptions.
- **Multi-language.** Document every language found. Rust and Python have deep analyzers; all others use generic heuristics.
- **Triangulate.** Patterns across repos = personal; single repo = project-specific.
- **Actionable.** "Prefer early extraction" > "Keep functions short."
- **Minimal.** Distinctive rules only, no universal noise.

## Architecture

```
scripts/agentskill.py       CLI entry point (delegates to package)
agentskill/
  constants.py              All constants and configuration
  cli.py                   Argument parsing, orchestration
  extractors/
    git.py                  Commit messages, branches, config, remotes
    filesystem.py           File scanning, tooling, project metadata
  analyzers/
    base.py                Abstract LanguageAnalyzer + AnalysisResult
    language/
      rust.py              Deep: naming, errors, comments, imports, spacing
      python.py            Deep: naming, errors, comments, imports, spacing
      generic.py           Fallback: line counts, comment density
  synthesis/
    __init__.py            AgentSynthesizer + SynthesisConfig
```

## References

- **GOTCHAS.md** — Extraction/synthesis errors to avoid.
- **synthesis-prompt.md** — LLM prompt template.
- **output-template.md** — Structure guide.
- **examples/** — Successful AGENTS.md samples.