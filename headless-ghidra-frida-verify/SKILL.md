---
name: "headless-ghidra-frida-verify"
description: "Deprecated compatibility-only P6 alias: runtime observation is now part of P1/P4 hand-offs."
phase: "P6"
---

# Headless Ghidra Frida Verify — P6

Deprecated compatibility-only alias. Runtime observation is now part of the
P1 baseline/runtime setup and P4 function-substitution hand-off; do not route
new workflows here as a primary pipeline stage.

## Required ghidra-agent-cli Commands

- `ghidra-agent-cli frida fuzz-input-gen`
- `ghidra-agent-cli frida io-capture`
- `ghidra-agent-cli frida io-compare`
- `ghidra-agent-cli frida trace`
- `ghidra-agent-cli frida run`
- `ghidra-agent-cli frida invoke`
- `ghidra-agent-cli frida device-list`
- `ghidra-agent-cli frida device-attach`
- `ghidra-agent-cli ghidra rebuild-project`
- `ghidra-agent-cli gate check --phase P6`

## Inputs

- `artifacts/<target-id>/substitution/functions/<fn_id>/substitution.yaml`
- `artifacts/<target-id>/substitution/functions/<fn_id>/capture.yaml`
- Any function-local substitution outputs needed by the verification harness
- Verification binary or harness configuration
- Optional manual test-case YAML provided by the operator

## Outputs

- Compatibility verification YAML under
  `artifacts/<target-id>/substitution/functions/<fn_id>/`
- Optional function-local recording or input YAML such as runtime captures,
  fuzz inputs, and comparison reports

## Exit Expectations

- Verification status is explicit for every verified compatibility function.
- Verification status and verdict are explicit and reviewable.
- Divergences are recorded rather than silently repaired in-place.

## Constraints

- Do not modify the binary under test.
- Do not auto-edit reconstruction code during verification.
- Do not fabricate capture or comparison results.
- Do not bypass `ghidra-agent-cli` for supported Frida, rebuild, or gate
  operations.

## Next Step

- All selected functions verified → return to P3 for another round or finish.
- Diverged functions → schedule for another P4 iteration.
