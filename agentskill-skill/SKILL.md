---
name: agentskill
description: Let any agent produce code consistent with the existing codebase.
---

# Agentskill Skill

Use the agentskill binary to gather repository evidence, then author and
maintain `AGENTS.md` through the LLM. Analyzer output is raw material, not the
finished document.

## Workflow

1. Confirm the target repository path.
2. Collect normalized evidence with `agentskill evidence <repo> --pretty`.
3. Inspect representative entrypoints, core modules, tests, manifests, CI, and
   configuration files directly.
4. Read `SYSTEM.md` fully before drafting or updating documents.
5. Ask only high-impact questions that evidence cannot resolve.
6. Write compact `AGENTS.md` guidance and optional reference context.
7. Run `agentskill validate <repo>` and show the semantic diff.

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

References are explicit inputs to the LLM workflow and must contain a readable
`AGENTS.md`. Compare reference conventions against target-repository evidence;
never copy them as unquestioned truth.

## LLM Workflows

Use these workflows for semantic repository guidance:

```bash
agentskill evidence <repo> --pretty
agentskill validate <repo>
agentskill drift <repo>
```

The LLM skill owns `init`, `enrich`, `scope`, `context`, `update`, and `audit`.
The Rust CLI never writes semantic Markdown.

## Evidence Rules

- Extract rules from source and configuration; do not guess.
- Treat analyzer counts as evidence, not prose for the final document.
- Scope every rule to the repository, service, or target language where it
  applies.
- Surface genuine uncertainty instead of silently inventing conventions.
- Keep supported target languages unchanged, including Python, while keeping
  the agentskill implementation Rust-only.

Read `SYSTEM.md` for the complete generated-document contract.
