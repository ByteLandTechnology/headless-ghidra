---
name: "headless-ghidra-evidence"
description: "P2 sub-skill: review baseline YAML metadata, identify third-party code, and synthesize frontier evidence into YAML outputs."
phase: "P2"
---

# Headless Ghidra Evidence — P2

P2 reviews the baseline YAML exports and records the current frontier evidence
needed by discovery. It may also identify third-party libraries and attach
optional Frida observations when static evidence is insufficient.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli functions list`
- `ghidra-agent-cli functions show`
- `ghidra-agent-cli imports list`
- `ghidra-agent-cli strings list`
- `ghidra-agent-cli types list`
- `ghidra-agent-cli callgraph list`
- `ghidra-agent-cli callgraph callers`
- `ghidra-agent-cli callgraph callees`
- `ghidra-agent-cli third-party add`
- `ghidra-agent-cli third-party set-version`
- `ghidra-agent-cli third-party list`
- `ghidra-agent-cli third-party classify-function`
- `ghidra-agent-cli third-party vendor-pristine`
- `ghidra-agent-cli execution-log append`
- `ghidra-agent-cli gate check --phase P2`

Optional runtime supplementation may also use:

- `ghidra-agent-cli frida device-list`
- `ghidra-agent-cli frida device-attach`

## Inputs

- `artifacts/<target-id>/baseline/functions.yaml`
- `artifacts/<target-id>/baseline/callgraph.yaml`
- `artifacts/<target-id>/baseline/types.yaml`
- `artifacts/<target-id>/baseline/constants.yaml`
- `artifacts/<target-id>/baseline/strings.yaml`
- `artifacts/<target-id>/baseline/imports.yaml`
- Existing `artifacts/<target-id>/third-party/identified.yaml` if present

## Outputs

- `artifacts/<target-id>/evidence-candidates.yaml`
- `artifacts/<target-id>/third-party/identified.yaml`
- Optional phase-owned runtime supplement YAML under `artifacts/<target-id>/evidence/`

## Exit Expectations

- Evidence review exists as `evidence-candidates.yaml`.
- Third-party matches and classifications are recorded in YAML, not Markdown.
- The next phase has enough evidence to choose the next frontier candidate or
  batch.

## Constraints

- Do not mutate baseline YAML exports directly.
- Do not claim unsupported evidence without recording the supporting source.
- Do not bypass `ghidra-agent-cli` for supported baseline reads, third-party
  writes, execution logging, or supported Frida setup.

## Next Step

- P2 gate passes → `headless-ghidra-discovery`
