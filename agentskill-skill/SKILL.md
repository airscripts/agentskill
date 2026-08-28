---
name: agentskill
description: Let any agent produce code consistent with the existing codebase.
---

# Agentskill Skill

Use the agentskill binary to gather repository evidence, then author the final
`AGENTS.md` from that evidence. The skill is AI-led: analyzer output is raw
material, not the finished document.

## Workflow

1. Confirm the target repository path or paths.
2. Collect broad evidence with `agentskill analyze <repo> --pretty`.
3. Inspect representative entrypoints, core modules, tests, manifests, and
   configuration files directly.
4. Read `SYSTEM.md` fully before drafting the document.
5. Synthesize and validate the final `AGENTS.md` against observed conventions.

Use individual analyzers when a focused signal is needed:

```bash
agentskill scan <repo> --pretty
agentskill measure <repo> --pretty
agentskill config <repo> --pretty
agentskill git <repo> --pretty
agentskill graph <repo> --pretty
agentskill symbols <repo> --pretty
agentskill tests <repo> --pretty
```

References may be supplied to `analyze` with repeated `--reference` flags.
References are explicit inputs and must contain a readable `AGENTS.md`.

## Static CLI Workflows

Use these when the user explicitly wants deterministic runtime-generated
markdown rather than AI-authored synthesis:

```bash
agentskill generate <repo>
agentskill generate <repo> --profile comprehensive
agentskill generate <repo> --layout split --out AGENTS.md
agentskill generate <repo> --layout multifile --out AGENTS.md
agentskill update <repo>
agentskill update <repo> --section testing
```

`update` preserves untouched manual sections by default. `--force` rebuilds
from regenerated sections. `update` supports only the `single` layout.

## Evidence Rules

- Extract rules from source and configuration; do not guess.
- Treat analyzer counts as evidence, not prose for the final document.
- Scope every rule to the repository, service, or target language where it
  applies.
- Surface genuine uncertainty instead of silently inventing conventions.
- Keep supported target languages unchanged, including Python, while keeping
  the agentskill implementation Rust-only.

Read `SYSTEM.md` for the complete generated-document contract.
