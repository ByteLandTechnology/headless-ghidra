# Headless Ghidra Skill Family

This repository defines a YAML-first, headless-only decompilation workflow
around `ghidra-agent-cli`. A global orchestrator skill manages P0–P6, while the
CLI subproject provides the supported command surface and artifact semantics.

## Documentation Boundaries

- [`ghidra-agent-cli/SKILL.md`](./ghidra-agent-cli/SKILL.md): CLI usage,
  command groups, flags, output envelope, workspace layout, and YAML artifact
  meanings.
- [`headless-ghidra/SKILL.md`](./headless-ghidra/SKILL.md): the authoritative
  P0–P6 workflow, routing, and orchestration rules.
- `headless-ghidra-*/SKILL.md`: per-phase inputs, outputs, required CLI
  commands, and phase-local constraints.

## Architecture

```text
headless-ghidra                       ← global orchestrator
├── ghidra-agent-cli                  ← bundled CLI/release subproject
├── headless-ghidra-intake            ← P0 target intake
├── headless-ghidra-baseline          ← P1 baseline extraction
├── headless-ghidra-evidence          ← P2 evidence review
├── headless-ghidra-discovery         ← P3 target selection
├── headless-ghidra-batch-decompile   ← P4+P5 batch decompilation
└── headless-ghidra-frida-verify      ← P6 Frida I/O verification
```

## Pipeline Summary

```text
P0 Intake → P1 Baseline → P2 Evidence → [P3 Discovery → P4+P5 Decompile → P6 Verify]*
```

- P0–P2 are one-time initialization and evidence setup.
- P3–P6 form the iteration loop.
- `ghidra-agent-cli` is the required control-plane interface for supported
  operations.
- `ghidra-agent-cli gate check` is the authoritative gate validation for all
  pipeline phases (P0–P6). The legacy `gate-check.sh` has been removed.

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
├── third-party/
│   ├── identified.yaml
│   └── sources/
├── evidence-candidates.yaml
├── target-selection.yaml
├── decompilation/
│   ├── progress.yaml
│   ├── next-batch.yaml
│   └── functions/<fn_id>/
│       ├── decompilation-record.yaml
│       └── verification-result.yaml
├── gates/
└── scripts/
```

## Core Rules

- Headless-only workflows.
- Ghidra is the only approved decompilation backend.
- Supported workspace, metadata, Ghidra, Frida, progress, and gate operations
  must go through `ghidra-agent-cli`.
- Phase docs may define additional workflow logic, but they should reference the
  YAML artifacts above instead of inventing a parallel alternate runtime surface.

## Repository Notes

- `ghidra-agent-cli/` is tracked as a normal subdirectory of this repository.
- The preserved nested git metadata lives at `ghidra-agent-cli/.git-local-backup/`
  and is ignored by the outer repo.
- The authoritative release workflow/action live at
  `.github/workflows/release.yml` and `.github/actions/setup-build-env/action.yml`,
  operating on the `ghidra-agent-cli/` subdirectory.
