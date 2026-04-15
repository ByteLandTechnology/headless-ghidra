---
name: "headless-ghidra-baseline"
description: "P1 sub-skill: run Ghidra headless auto-analysis and export the validated baseline Markdown runtime surfaces. Strictly forbidden from decompiling function bodies."
phase: "P1"
---

# Headless Ghidra Baseline — P1 Baseline Extraction

This skill runs Ghidra headless auto-analysis and exports the validated
baseline Markdown runtime surfaces. It is **strictly forbidden** from
decompiling function bodies,
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
- `function-names.md`
- `imports-and-libraries.md`
- `strings-and-constants.md`
- `types-and-structs.md`
- `xrefs-and-callgraph.md`
- `decompiled-output.md` (blocked placeholder; function bodies remain unavailable at P1)
- `renaming-log.md`
- `signature-log.md`
- (optional) `call-graph-detail.md`

**Available tools**:
- `scripts/run-headless-analysis.sh --action baseline`
- `scripts/run-headless-analysis.sh --action call-graph`
- `ghidra-scripts/ExportAnalysisArtifacts.java`
- `ghidra-scripts/ExportCallGraph.java`

**Strict prohibitions**:
- ⛔ **Must not decompile any function body** (`decompiled-output.md` must retain the blocked placeholder text)
- ⛔ Must not modify function names, types, or prototypes
- ⛔ Must not execute Apply Renames/Signatures
- ⛔ Must not modify any files under `intake/`
- ⛔ **Python / Jython scripts are strictly forbidden**. If you need custom Ghidra scripts, write them in Java. The file name MUST strictly match the public class name (e.g. `CustomAnalysis.java` -> `public class CustomAnalysis extends GhidraScript`).

**Termination conditions**:
- The required Markdown baseline files are generated under the artifact root
- `decompiled-output.md` still states that decompiled bodies are intentionally blocked in this stage
- All exported Markdown surfaces are reviewable and non-empty

**System prompt**:

```
You are the P1 baseline export agent. Your responsibilities:
1. Read target-identity.yaml and ghidra-discovery.yaml
2. Run run-headless-analysis.sh --action baseline
3. Confirm the baseline Markdown files are generated
4. Confirm decompiled-output.md remains a blocked placeholder

You are strictly forbidden from decompiling function bodies, renaming, or
modifying any metadata at this stage. Those operations belong to later phases.
Violating this constraint is a blocking error.

CRITICAL INSTRUCTION FOR SCRIPT AUTHORING:
If you need to write a custom Ghidra script:
1. DO NOT write Python/Jython (`.py`). The extension is missing and it will crash.
2. DO write Java (`.java`) extending `ghidra.app.script.GhidraScript`.
3. The file name and public class name MUST perfectly match to prevent `ClassNotFoundException`.
```

## Baseline Artifact Manifest

Current validated runtime exports write these Markdown files under the artifact
root. Older `baseline/*.md` layouts may still appear in legacy replays, but new
automation should use the root-level files below.

| File | Contents |
|---|---|
| `function-names.md` | Function names/labels/addresses |
| `imports-and-libraries.md` | Import symbols and external libraries |
| `strings-and-constants.md` | Strings and constant values |
| `types-and-structs.md` | Types, structs, enums |
| `xrefs-and-callgraph.md` | Cross-references and call relationships |
| `decompiled-output.md` | Blocked placeholder noting that bodies stay unavailable in P1 |
| `renaming-log.md` | Rename plan surface; should remain review-only at this stage |
| `signature-log.md` | Signature plan surface; should remain review-only at this stage |

## Gate Check Matrix (P1)

| ID | Check | Type |
|---|---|---|
| P1_01 | `function-names.md` exists | blocking |
| P1_02 | `imports-and-libraries.md` exists | blocking |
| P1_03 | `strings-and-constants.md` exists | blocking |
| P1_04 | `types-and-structs.md` exists | blocking |
| P1_05 | `xrefs-and-callgraph.md` exists | blocking |
| P1_06 | `decompiled-output.md` retains the blocked placeholder text | blocking |
| P1_07 | Required baseline Markdown files are non-empty | blocking |
| P1_08 | function-names has at least 1 function row | warning |

## Next Step Routing

- P1 gate passes → orchestrator automatically enters P2 (`headless-ghidra-evidence`).
