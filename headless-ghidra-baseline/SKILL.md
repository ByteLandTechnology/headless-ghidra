---
name: "headless-ghidra-baseline"
description: "P1 sub-skill: run Ghidra headless auto-analysis and export all baseline evidence as YAML files. Strictly forbidden from decompiling function bodies."
phase: "P1"
---

# Headless Ghidra Baseline — P1 Baseline Extraction

This skill runs Ghidra headless auto-analysis and exports all baseline evidence
as YAML files. It is **strictly forbidden** from decompiling function bodies,
performing semantic renames, or restoring prototypes.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | `gate-check.sh --gate P0` passes |
| **Exit gate** | `gate-check.sh --gate P1` passes |
| **Parallelism** | ⛔ Not parallelizable (Ghidra project lock) |

## Agent Role Definition

### Agent: `baseline-export`

| Property | Value |
|---|---|
| **Agent ID** | `baseline-export` |
| **Instances** | 1 |
| **Lifecycle** | Short-lived (though Ghidra execution may take time) |
| **Role** | Run Ghidra headless auto-analysis and export all baseline evidence |

**Inputs**:
- `intake/target-identity.yaml` (target-id, binary_path)
- `intake/ghidra-discovery.yaml` (analyzeHeadless path)

**Outputs**:
- `baseline/function-names.yaml`
- `baseline/imports-and-libraries.yaml`
- `baseline/strings-and-constants.yaml`
- `baseline/types-and-structs.yaml`
- `baseline/xrefs-and-callgraph.yaml`
- `baseline/decompiled-output.yaml` (empty placeholder: `functions: []`)
- (optional) `baseline/call-graph-detail.yaml`

**Available tools**:
- `scripts/run-headless-analysis.sh --action baseline`
- `scripts/run-headless-analysis.sh --action call-graph`
- `ghidra-scripts/ExportAnalysisArtifacts.java`
- `ghidra-scripts/ExportCallGraph.java`

**Strict prohibitions**:
- ⛔ **Must not decompile any function body** (`decompiled-output.yaml` must remain empty)
- ⛔ Must not modify function names, types, or prototypes
- ⛔ Must not execute Apply Renames/Signatures
- ⛔ Must not modify any files under `intake/`

**Termination conditions**:
- All 6 YAML files under `baseline/` generated
- `decompiled-output.yaml` has `functions` array length of 0
- All YAML files are parseable by `yq`

**System prompt**:

```
You are the P1 baseline export agent. Your responsibilities:
1. Read target-identity.yaml and ghidra-discovery.yaml
2. Run run-headless-analysis.sh --action baseline
3. Confirm all 6 baseline YAML files are generated
4. Confirm decompiled-output.yaml is an empty placeholder

You are strictly forbidden from decompiling function bodies, renaming, or
modifying any metadata at this stage. Those operations belong to later phases.
Violating this constraint is a blocking error.
```

## Baseline Artifact Formats

### `baseline/function-names.yaml`

```yaml
exported_at: "2026-04-09T10:10:00Z"
total_count: 42
functions:
  - address: "0x00102140"
    name: "FUN_00102140"
    type: "auto_generated"    # auto_generated | imported | user_defined
    entry_point: false
    size_bytes: 256
```

### Full Baseline File Manifest

| File | Contents |
|---|---|
| `function-names.yaml` | Function names/labels/addresses |
| `imports-and-libraries.yaml` | Import symbols and external libraries |
| `strings-and-constants.yaml` | Strings and constant values |
| `types-and-structs.yaml` | Types, structs, enums |
| `xrefs-and-callgraph.yaml` | Cross-references and call relationships |
| `decompiled-output.yaml` | Empty placeholder (`functions: []`) |

## Gate Check Matrix (P1)

| ID | Check | Type |
|---|---|---|
| P1_01 | `baseline/function-names.yaml` exists | blocking |
| P1_02 | `baseline/imports-and-libraries.yaml` exists | blocking |
| P1_03 | `baseline/strings-and-constants.yaml` exists | blocking |
| P1_04 | `baseline/types-and-structs.yaml` exists | blocking |
| P1_05 | `baseline/xrefs-and-callgraph.yaml` exists | blocking |
| P1_06 | `baseline/decompiled-output.yaml` exists with empty `functions` | blocking |
| P1_07 | All baseline files are YAML-parseable | blocking |
| P1_08 | function-names has at least 1 function | warning |

## Next Step Routing

- P1 gate passes → orchestrator automatically enters P2 (`headless-ghidra-evidence`).
