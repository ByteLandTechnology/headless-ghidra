---
name: "headless-ghidra-discovery"
description: "P3 sub-skill: analyze verified boundaries and baseline evidence to discover the next batch of frontier-eligible functions for decompilation."
phase: "P3"
---

# Headless Ghidra Discovery — P3 Batch Discovery

This skill analyzes verified boundaries and baseline evidence to determine the
next batch of frontier-eligible functions for decompilation.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | First round: `gate-check.sh --gate P2`; subsequent: previous P6 complete |
| **Exit gate** | `gate-check.sh --gate P3 --iteration <NNN>` passes |
| **Parallelism** | ⛔ Discovery itself is not parallelizable |

## Agent Role Definition

### Agent: `discovery`

| Property | Value |
|---|---|
| **Agent ID** | `discovery` |
| **Instances** | 1 (invoked once per iteration) |
| **Lifecycle** | Short-lived |
| **Role** | Analyze verified boundaries and baseline evidence to determine the next frontier-eligible batch |

**Inputs**:
- `pipeline-state.yaml` (`verified_boundaries`, `iterations` history)
- `evidence/anchor-summary.yaml`
- `baseline/xrefs-and-callgraph.yaml`
- `baseline/function-names.yaml`

**Outputs**:
- `iterations/<NNN>/batch-manifest.yaml`

**Frontier priority** (highest to lowest):

| Level | Type |
|---|---|
| 1 | Entry-adjacent dispatchers/helpers/wrappers/thunks |
| 2 | Other entry-adjacent frontier functions |
| 3 | Dispatcher/helper/wrapper children of verified boundaries |
| 4 | Other children of verified boundaries |
| 5 | Stable address-order tiebreak |

**`batch-manifest.yaml` format**:

```yaml
iteration: 1
created_at: "2026-04-09T10:35:00Z"
batch_size: 3
status: "pending"

functions:
  - id: "fn_001"
    name: "FUN_00102140"
    address: "0x00102140"
    priority: 1
    frontier_reason: "entry-adjacent dispatcher"
    relationship_type: "entry_adjacent"
    verified_parent: null
    triggering_evidence:
      - source: "strings-and-constants.yaml"
        detail: "'invalid packet'"
    question_to_answer: "does this function validate headers before dispatch?"
    status: "pending"
```

**Available tools**:
- `yq` — YAML read/write
- `ghidra-scripts/PlanTargetSelection.java`

**Strict prohibitions**:
- ⛔ Must not decompile any function
- ⛔ Must not modify Ghidra project metadata
- ⛔ Must not skip frontier priority rules (unless explicitly recording `deviation_reason`)
- ⛔ Must not modify `verified_boundaries` (read-only)

**Termination conditions**:
- `batch-manifest.yaml` written
- Contains at least 1 function
- Every function has `address`, `frontier_reason`, `question_to_answer`
- No duplicate addresses

**System prompt**:

```
You are the P3 batch discovery agent. Your responsibilities:
1. Read verified boundaries from pipeline-state.yaml
2. Compute the current frontier-eligible function set
3. Sort by 5-level priority
4. Output batch-manifest.yaml

You do not decompile, rename, or modify any existing artifacts.
```

## Gate Check Matrix (P3)

| ID | Check | Type |
|---|---|---|
| P3_01 | `iterations/<NNN>/batch-manifest.yaml` exists | blocking |
| P3_02 | `functions` list is non-empty | blocking |
| P3_03 | Every function has `address` + `frontier_reason` + `question_to_answer` | blocking |
| P3_04 | No duplicate addresses | blocking |
| P3_05 | Every function `status` = `pending` | blocking |

## Next Step Routing

- P3 gate passes → orchestrator shows dialog for user to confirm batch → enters P4+P5 (`headless-ghidra-batch-decompile`).
