---
name: "headless-ghidra-batch-decompile"
description: "P4+P5 sub-skill: function-level parallel source comparison, semantic reconstruction, and Ghidra-only decompilation. Analysis phases run in parallel; Ghidra operations are serialized via a queue."
phase: "P4+P5"
---

# Headless Ghidra Batch Decompile — P4+P5 Batch Decompilation

This skill executes the full decompilation pipeline for each function in a batch:
source comparison → semantic reconstruction → decompilation. Analysis phases
run in parallel at the function level; Ghidra read/write operations are
serialized through a queue.

Selected Decompilation has exactly one supported backend in this repository:
`scripts/run-headless-analysis.sh --action decompile-selected`, which invokes
the registered Java Ghidra scripts and Ghidra's decompiler API. Direct shell
disassembly or alternate decompilation tooling is out of policy.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | `gate-check.sh --gate P3 --iteration <NNN>` passes |
| **Exit gate** | Per function: `gate-check.sh --gate P5 --iteration <NNN> --function <fn_id>` |
| **Parallelism** | ✅ Function-level parallel (analysis parallel, Ghidra operations queued) |

## Agent Role Definition

### Agent: `decompile-<fn_id>`

| Property | Value |
|---|---|
| **Agent ID pattern** | `decompile-fn_001`, `decompile-fn_002`, ... |
| **Instances** | N (one per function in the batch, parallel) |
| **Lifecycle** | Medium (includes analysis + Ghidra queue wait) |
| **Role** | Execute full P4 source comparison + P5 semantic reconstruction + decompilation for a single function |

**Inputs**:
- This function's entry from `iterations/<NNN>/batch-manifest.yaml`
- All `baseline/` files
- `evidence/` anchor info (including `derived_from_library` tags)
- Ghidra lock path (`ghidra_queue.lock_file`)
- (if library match) `third_party/<library_name>/` source code

**Outputs (written to `iterations/<NNN>/functions/<fn_id>/`)**:
- `source-comparison.yaml`
- `semantic-record.yaml`
- `renaming-log.yaml`
- `signature-log.yaml`
- `apply-report.yaml`
- `verify-report.yaml`
- `lint-report.yaml`
- `decompilation-record.yaml` (`decompilation_backend` and
  `decompilation_action` are required)
- `decompiled-output/` directory

**Also written to reconstruction project**:
- `.work/reconstruction/<target-id>/src/<function_name>.c`
- `.work/reconstruction/<target-id>/include/<function_name>.h`
- Updated `reconstruction-manifest.yaml`
- Updated `stubs/stub_manifest.yaml`

**Workflow**:

```
Phase A — Analysis (no Ghidra lock needed, parallelizable)
  1. Review baseline evidence, perform source comparison → source-comparison.yaml
     If function has derived_from_library tag: prioritize comparing against library source
  2. Analyze role/name/prototype evidence → semantic-record.yaml
  3. Write renaming-log.yaml + signature-log.yaml

Phase B — Ghidra Operations (requires lock, queued execution)
  4. ghidra-queue.sh acquire
  5. Apply Renames → apply-report.yaml
  6. Verify Renames
  7. Apply Signatures
  8. Verify Signatures → verify-report.yaml
  9. Lint → lint-report.yaml
  10. Run `run-headless-analysis.sh --action decompile-selected` for the
      selected function → decompiled-output/
  11. ghidra-queue.sh release

Phase C — Post-processing (no lock needed)
  12. Write decompilation-record.yaml with `decompilation_backend:
      ghidra_headless` and `decompilation_action: decompile-selected`
  13. Write cleaned source to reconstruction project
  14. Update reconstruction-manifest.yaml
```

**Ghidra operation queue**:

| Operation | Requires queue | Notes |
|---|---|---|
| Source comparison analysis | ❌ | Does not access Ghidra |
| Semantic planning | ❌ | Does not access Ghidra |
| Apply Renames | ✅ | Writes to Ghidra project |
| Verify Renames | ✅ | Reads Ghidra project |
| Apply Signatures | ✅ | Writes to Ghidra project |
| Verify Signatures | ✅ | Reads Ghidra project |
| Lint | ✅ | Reads Ghidra project |
| Decompile export | ✅ | Reads Ghidra project |

**Available tools**:
- `yq` — YAML read/write
- `scripts/ghidra-queue.sh acquire/release` — Ghidra lock management
- `scripts/run-headless-analysis.sh --action apply-renames|verify-renames|apply-signatures|verify-signatures|lint-review-artifacts|decompile-selected`
- Corresponding Java scripts

**Strict prohibitions**:
- ⛔ Must not execute Ghidra operations without acquiring the lock first
- ⛔ Must not modify other functions' artifact directories
- ⛔ Must not modify `baseline/` or `evidence/` files
- ⛔ Must not force changes when role/name/prototype evidence is all weak
- ⛔ Must not execute Frida (verification is P6's responsibility)
- ⛔ Must not use `objdump`, `otool`, `llvm-objdump`, `nm`, `readelf`, `gdb`, `lldb`, `radare2`, or equivalent direct binary disassembly tooling to produce or justify `decompiled-output/`
- ⛔ Must not treat compare, trace, or symbol-dump commands as an alternate decompilation backend
- ⛔ **Python / Jython scripts are strictly forbidden**. If you need custom Ghidra scripts, write them in Java. The file name MUST strictly match the public class name (e.g. `CustomAnalysis.java` -> `public class CustomAnalysis extends GhidraScript`).

**Termination conditions**:
- All Phase A-C output files generated
- `decompilation-record.yaml` has all required fields non-empty, including
  Ghidra provenance
- Reconstruction project `.c` + `.h` files written
- Ghidra lock released

**System prompt**:

```
You are the decompilation agent ({fn_id}: {function_name}@{address}). Your
responsibilities:
1. [Phase A parallel] Analyze source derivation, evaluate evidence, write
   rename/signature plans
   - If this function is tagged derived_from_library, prioritize comparing
     against the matched library source
2. [Phase B queued] Acquire Ghidra lock, execute Apply/Verify/Lint/Decompile,
   release lock
3. [Phase C parallel] Write decompilation record, output code to reconstruction
   project

Key constraints:
- Do not run any Ghidra commands before acquiring the lock
- Only operate on your own function's artifact directory
- Role/name/prototype evidence must have at least 2 items available before
  allowing changes
- You do not perform Frida verification (that is P6's responsibility)
- Selected Decompilation must come from `run-headless-analysis.sh --action
  decompile-selected`; do not use `objdump`, `otool`, `llvm-objdump`, `nm`,
  `readelf`, `gdb`, `lldb`, or `radare2` as substitute backends
- `decompilation-record.yaml` must include `decompilation_backend:
  ghidra_headless` and `decompilation_action: decompile-selected`
- CRITICAL SCRIPTING RULE: If you must write a new script, DO NOT write Python (`.py`). You MUST write Java (`.java`) and ensure the file name matches the public class name to avoid `ClassNotFoundException`.
```

## Gate Check Matrix (P5, per function)

| ID | Check | Type |
|---|---|---|
| P5_01 | `decompiled-output/` contains `.c` file | blocking |
| P5_02 | `decompilation-record.yaml` exists | blocking |
| P5_03 | Record contains all required fields, including provenance fields | blocking |
| P5_04 | Provenance declares `ghidra_headless` via `decompile-selected` | blocking |
| P5_05 | `semantic-record.yaml` exists | blocking |
| P5_06 | Role/name/prototype evidence has at least 2 items non-empty | blocking |
| P5_07 | `source-comparison.yaml` exists | blocking |
| P5_08 | `reference_status` is set | blocking |
| P5_09 | verify-report has no `failed` entries | blocking |
| P5_10 | Reconstruction project `.c` + `.h` written | blocking |
| P5_11 | `reconstruction-manifest.yaml` updated | blocking |

## Next Step Routing

- All functions pass P5 gate → orchestrator enters P6 (`headless-ghidra-frida-verify`).
