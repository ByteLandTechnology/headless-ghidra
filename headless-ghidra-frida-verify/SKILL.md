---
name: "headless-ghidra-frida-verify"
description: "P6 sub-skill: function-level parallel Frida I/O verification. Records reviewed runtime behavior, executes reconstructed function, and compares I/O. Results serve as the pipeline gate."
phase: "P6"
---

# Headless Ghidra Frida Verify — P6 Frida I/O Verification

This skill hooks the selected verification function via Frida to record I/O, then executes
the reconstructed function with the same inputs and compares outputs case by
case. Verification results are the sole basis for the gate decision.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | Per function: `gate-check.sh --gate P5 --iteration <NNN> --function <fn_id>` |
| **Exit gate** | Per function: `gate-check.sh --gate P6 --iteration <NNN> --function <fn_id>` |
| **Parallelism** | ✅ Function-level full parallelism (Frida can attach to multiple targets simultaneously) |

## Test Input Three-Source Strategy

| Source | Method | Artifact | Priority |
|---|---|---|---|
| Runtime recording | Frida hook on the verification binary, record real I/O | `test-inputs/runtime-recorded.yaml` | Highest |
| Fuzz generation | Auto-generate boundary inputs from function signature | `test-inputs/fuzz-generated.yaml` | Medium |
| Manual provision | Analyst writes targeted test cases | `test-inputs/manual-cases.yaml` | On demand |

## Agent Role Definition

### Agent: `verify-<fn_id>`

| Property | Value |
|---|---|
| **Agent ID pattern** | `verify-fn_001`, `verify-fn_002`, ... |
| **Instances** | N (one per function in the batch, parallel) |
| **Lifecycle** | Medium (includes Frida attach + test execution) |
| **Role** | Execute full Frida I/O verification for a single function |

**Inputs**:
- `iterations/<NNN>/functions/<fn_id>/decompiled-output/`
- `iterations/<NNN>/functions/<fn_id>/decompilation-record.yaml`
- `.work/reconstruction/<target-id>/src/<function_name>.c`
- `.work/reconstruction/<target-id>/CMakeLists.txt`
- Verification binary path
- Optional reviewed mock verification binary path when runtime capture should
  happen against a dedicated verification build instead of the imported target
- (optional) `test-inputs/manual-cases.yaml`

**Outputs (written to `iterations/<NNN>/functions/<fn_id>/`)**:
- `test-inputs/runtime-recorded.yaml`
- `test-inputs/fuzz-generated.yaml`
- `frida-io-recording.yaml`
- `verification-result.yaml`

**Workflow**:

```
Phase A — Test Input Preparation
  1. Read decompilation-record.yaml to get function signature
  2. Run fuzz-input-gen.js to generate boundary inputs from signature
  3. Check for manual-cases.yaml

Phase B — Runtime Behavior Recording
  4. Frida attach to the selected verification binary
  5. Run io-capture.js to hook target function
  6. Drive execution with Phase A input set
  7. Write runtime-recorded.yaml and frida-io-recording.yaml

Special case — reviewed mock verification binaries
  - When runtime capture should use a dedicated mock binary instead of the
    imported analysis target, record that path in `frida-io-recording.yaml`.
  - The verification binary must be behaviorally equivalent for the exercised
    cases and actually launchable in the current environment.

Phase C — Reconstructed Function Execution
  9. Build reconstruction project (cmake --build build/)
  10. Execute reconstructed function with each test case
  11. Record outputs

Phase D — Comparison and Verdict
  12. Compare case by case: return values + side effects
  13. Generate verification-result.yaml
  14. Set gate_verdict: pass | fail | conditional
```

**`verification-result.yaml` format**:

```yaml
function_id: "fn_001"
function_name: "packet_validate_and_dispatch"
function_address: "0x00102140"
iteration: 1
verified_at: "2026-04-09T11:35:00Z"

status: "verified"  # verified | diverged | failed | blocked

test_summary:
  runtime_recorded: { total: 3, passed: 3, failed: 0 }
  fuzz_generated: { total: 10, passed: 10, failed: 0 }
  manual_cases: { total: 2, passed: 2, failed: 0 }
  overall: { total: 15, passed: 15, failed: 0 }

divergences: []

gate_verdict: "pass"  # pass | fail | conditional
```

**Available tools**:
- Frida CLI (`frida`, `frida-trace`)
- `frida-scripts/io-capture.js`
- `frida-scripts/io-compare.js`
- `frida-scripts/fuzz-input-gen.js`
- `gcc`/`clang` — compile reconstructed code
- `cmake` + `make`/`ninja` — build reconstruction project

**Strict prohibitions**:
- ⛔ **Must not modify the verification binary under test**
- ⛔ **Must not modify `decompiled-output/`** (decompilation output is read-only input)
- ⛔ Must not modify other functions' verification results
- ⛔ Must not automatically modify reconstruction code on verification failure (fixes belong to the next batch-decompile iteration)
- ⛔ Must not fabricate test results

**Termination conditions**:
- `verification-result.yaml` written
- `status` is one of `verified`, `diverged`, `failed`, or `blocked`
- `gate_verdict` is set
- All test case comparison records are complete

**System prompt**:

```
You are the Frida I/O verification agent ({fn_id}: {function_name}@{address}).
Your responsibilities:
1. Generate fuzz test inputs based on function prototype
2. Frida hook the selected verification function, record I/O for each input
3. Build reconstruction code, execute with same inputs
4. Compare return values and side effects case by case
5. Write verification-result.yaml as the gate verdict basis

Key constraints:
- You only verify — you never modify reconstruction code
- You never fabricate test results
- If divergences are found, record them in detail (case_id, field, expected,
  actual, analysis)
- Verification failure is not your bug — record the divergence and let the
  next batch-decompile iteration fix it
```

## Gate Check Matrix (P6, per function)

| ID | Check | Type |
|---|---|---|
| P6_01 | `test-inputs/` contains at least 1 source file | blocking |
| P6_02 | `frida-io-recording.yaml` exists | blocking |
| P6_03 | `verification-result.yaml` exists | blocking |
| P6_04 | `status` == `verified` | blocking |
| P6_05 | `test_summary.overall.failed` == 0 | blocking |
| P6_06 | `gate_verdict` == `pass` | blocking |

## Next Step Routing

- All functions pass P6 gate → orchestrator marks iteration complete, returns to P3 or finishes.
- Any function fails P6 gate → orchestrator adds it to the next retry round.
