---
name: "headless-ghidra-batch-decompile"
description: "P4+P5 sub-skill: apply metadata, decompile selected functions through Ghidra, and record per-function YAML outputs."
phase: "P4+P5"
---

# Headless Ghidra Batch Decompile — P4+P5

P4+P5 consumes the current selected batch, applies metadata changes, runs the
approved Ghidra decompilation path, and records per-function artifacts.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli ghidra apply-renames`
- `ghidra-agent-cli ghidra verify-renames`
- `ghidra-agent-cli ghidra apply-signatures`
- `ghidra-agent-cli ghidra verify-signatures`
- `ghidra-agent-cli ghidra decompile`
- `ghidra-agent-cli ghidra rebuild-project`
- `ghidra-agent-cli progress mark-decompiled`
- `ghidra-agent-cli progress show`
- `ghidra-agent-cli progress list`
- `ghidra-agent-cli gate check --phase P5`

Queueing via `ghidra-queue.sh` and Java/headless helpers remains a backend
detail. The public workflow surface is the CLI plus the YAML outputs below.

## Inputs

- `artifacts/<target-id>/target-selection.yaml`
- `artifacts/<target-id>/evidence-candidates.yaml`
- `artifacts/<target-id>/baseline/*.yaml`
- `artifacts/<target-id>/third-party/identified.yaml` and any vendored sources
- `artifacts/<target-id>/decompilation/next-batch.yaml`

## Outputs

- `artifacts/<target-id>/decompilation/progress.yaml`
- `artifacts/<target-id>/decompilation/functions/<fn_id>/decompilation-record.yaml`
- Additional per-function YAML such as semantic, rename, signature, or lint
  reports when the workflow records them

## Exit Expectations

- The selected functions are reflected in `progress.yaml`.
- Each completed function has a `decompilation-record.yaml` with provenance.
- P5 gate material is available under `decompilation/functions/<fn_id>/`.

## Constraints

- Use Ghidra as the only decompilation backend.
- Acquire the Ghidra queue/lock before mutating or reading shared Ghidra state
  when the backend requires it.
- Do not modify artifacts for functions outside the active batch.
- Do not bypass `ghidra-agent-cli` for supported apply/verify/decompile/progress
  actions.

## Next Step

- P5 gate passes for the batch → `headless-ghidra-frida-verify`
