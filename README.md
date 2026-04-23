# Headless Ghidra Skill Family

This repository defines a YAML-first, headless-only decompilation workflow
around `ghidra-agent-cli`. A global orchestrator skill manages P0–P4, while the
CLI subproject provides the supported command surface and artifact semantics.

## Documentation Boundaries

- [`ghidra-agent-cli/SKILL.md`](./ghidra-agent-cli/SKILL.md): CLI usage,
  command groups, flags, output envelope, workspace layout, and YAML artifact
  meanings.
- [`headless-ghidra/SKILL.md`](./headless-ghidra/SKILL.md): the authoritative
  P0–P4 workflow, routing, and orchestration rules.
- `headless-ghidra-*/SKILL.md`: per-phase inputs, outputs, required CLI
  commands, and phase-local constraints.

## Architecture

```text
headless-ghidra                       ← global orchestrator
├── ghidra-agent-cli                  ← bundled CLI/release subproject
├── headless-ghidra-intake            ← P0 intake and scope
├── headless-ghidra-baseline          ← P1 baseline and runtime setup
├── headless-ghidra-evidence          ← P2 third-party identification
├── headless-ghidra-discovery         ← P3 metadata enrichment
├── headless-ghidra-batch-decompile   ← P4 function substitution
├── headless-ghidra-scope             ← deprecated P0.5 compatibility alias
└── headless-ghidra-frida-verify      ← deprecated P6 compatibility alias
```

## Pipeline Summary

```text
P0 Intake → P1 Baseline+Runtime → P2 Third-Party → [P3 Metadata Enrichment → P4 Function Substitution]*
```

- P0–P2 are one-time initialization, runtime setup, and third-party setup.
- P3–P4 form the iterative metadata and substitution loop.
- `ghidra-agent-cli` is the required control-plane interface for supported
  operations.
- `ghidra-agent-cli gate check` is the authoritative gate validation for all
  pipeline phases (P0–P4). The legacy `gate-check.sh` has been removed.
- Old P0.5, P5, and P6 docs or CLI aliases are compatibility-only and must not
  be presented as primary stages.

## Shared Workspace Model

```text
targets/<target-id>/
└── ghidra-projects/

artifacts/<target-id>/
├── pipeline-state.yaml
├── scope.yaml
├── intake/
├── baseline/
│   ├── functions.yaml
│   ├── callgraph.yaml
│   ├── types.yaml
│   ├── vtables.yaml
│   ├── constants.yaml
│   ├── strings.yaml
│   └── imports.yaml
├── runtime/
│   ├── project/
│   ├── fixtures/
│   ├── run-manifest.yaml
│   ├── run-records/
│   └── hotpaths/call-chain.yaml
├── third-party/
│   ├── identified.yaml
│   ├── pristine/<library>@<version>/
│   └── compat/<library>@<version>/
├── metadata/
│   ├── renames.yaml
│   ├── signatures.yaml
│   ├── types.yaml
│   ├── constants.yaml
│   ├── strings.yaml
│   └── apply-records/
├── substitution/
│   ├── template/
│   ├── next-batch.yaml
│   └── functions/<fn_id>/
├── gates/
└── scripts/
```

## Core Rules

- Headless-only workflows.
- Ghidra is the only approved decompilation backend.
- All workflow artifacts live under `artifacts/<target-id>/`.
- YAML artifacts are created, updated, and validated through
  `ghidra-agent-cli`.
- The CLI must not automatically create git commits.
- Gate transitions require relevant artifacts to be tracked or staged in git.
- Supported workspace, metadata, Ghidra, Frida, progress, and gate operations
  must go through `ghidra-agent-cli`.
- All Ghidra project operations must go through `ghidra-agent-cli`. If the CLI
  lacks a required capability, pause and ask the user before creating or running
  a new Ghidra script.
- Phase docs may define additional workflow logic, but they should reference the
  YAML artifacts above instead of inventing a parallel alternate runtime surface.

## Repository Notes

- `ghidra-agent-cli/` is tracked as a normal subdirectory of this repository.
- The preserved nested git metadata lives at `ghidra-agent-cli/.git-local-backup/`
  and is ignored by the outer repo.
- The authoritative release workflow/action live at
  `.github/workflows/release.yml` and `.github/actions/setup-build-env/action.yml`,
  operating on the `ghidra-agent-cli/` subdirectory.
