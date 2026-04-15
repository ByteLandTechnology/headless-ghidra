---
name: "headless-ghidra-discovery"
description: "P3 sub-skill: analyze verified boundaries and reviewed evidence to record the next frontier-eligible automatic default target."
phase: "P3"
---

# Headless Ghidra Discovery — P3 Target Selection

This skill analyzes verified boundaries, baseline evidence, and the current
frontier review to determine the next automatic default target for
decompilation.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | First round: `gate-check.sh --gate P2`; subsequent: previous P6 complete |
| **Exit gate** | Current runtime: `gate-check.sh --gate P3`; legacy fallback: `gate-check.sh --gate P3 --iteration <NNN>` |
| **Parallelism** | ⛔ Discovery itself is not parallelizable |

## Agent Role Definition

### Agent: `discovery`

| Property | Value |
|---|---|
| **Agent ID** | `discovery` |
| **Instances** | 1 (invoked once per iteration) |
| **Lifecycle** | Short-lived |
| **Role** | Analyze verified boundaries and reviewed evidence to determine the next frontier-eligible automatic default target |

**Inputs**:
- `pipeline-state.yaml` (`verified_boundaries`, `iterations` history)
- `evidence-candidates.md`
- `xrefs-and-callgraph.md`
- `function-names.md`

**Outputs**:
- `target-selection.md` (current validated runtime surface)
- Legacy fallback only: `iterations/<NNN>/batch-manifest.yaml`

**Frontier priority** (highest to lowest):

| Level | Type |
|---|---|
| 1 | Entry-adjacent dispatchers/helpers/wrappers/thunks |
| 2 | Other entry-adjacent frontier functions |
| 3 | Dispatcher/helper/wrapper children of verified boundaries |
| 4 | Other children of verified boundaries |
| 5 | Stable address-order tiebreak |

**`target-selection.md` required fields**:

- `Selected Target`
- `Selection Mode`
- `Candidate Kind`
- `Frontier Reason`
- `Selection Reason`
- `Question To Answer`
- `Tie-Break Rationale`
- `Metrics Note`

Candidate review rows must preserve frontier eligibility, triggering evidence,
and status so downstream phases can see why the automatic default was chosen.

**Available tools**:
- `yq` — YAML read/write
- `ghidra-scripts/PlanTargetSelection.java`

**Strict prohibitions**:
- ⛔ Must not decompile any function
- ⛔ Must not modify Ghidra project metadata
- ⛔ Must not skip frontier priority rules (unless explicitly recording `deviation_reason`)
- ⛔ Must not modify `verified_boundaries` (read-only)

**Termination conditions**:
- `target-selection.md` written
- Automatic default selection table is present
- Candidate selection rows are present
- The selected row records frontier reason and question-to-answer context

**System prompt**:

```
You are the P3 batch discovery agent. Your responsibilities:
1. Read verified boundaries from pipeline-state.yaml
2. Compute the current frontier-eligible function set
3. Sort by 5-level priority
4. Output target-selection.md

You do not decompile, rename, or modify any existing artifacts.
```

## Gate Check Matrix (P3)

| ID | Check | Type |
|---|---|---|
| P3_01 | `target-selection.md` exists | blocking |
| P3_02 | `## Automatic Default Selection` is present | blocking |
| P3_03 | selection fields include selected target, frontier reason, and question | blocking |
| P3_04 | `## Candidate Selection Rows` is present | blocking |
| P3_05 | at least one row is marked ready or selected as the default | blocking |

## Next Step Routing

- P3 gate passes → orchestrator shows the reviewed selection to the user, then enters P4+P5 (`headless-ghidra-batch-decompile`).
