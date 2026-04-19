---
name: "headless-ghidra-frida-verify"
description: "P6 sub-skill: record runtime behavior with Frida, compare reconstructed outputs, and write verification YAML results."
phase: "P6"
---

# Headless Ghidra Frida Verify — P6

P6 verifies selected decompiled functions by capturing runtime behavior,
executing the reconstructed implementation, and writing the comparison result as
YAML.

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

- `artifacts/<target-id>/decompilation/functions/<fn_id>/decompilation-record.yaml`
- Any function-local decompilation outputs needed by the verification harness
- Verification binary or harness configuration
- Optional manual test-case YAML provided by the operator

## Outputs

- `artifacts/<target-id>/decompilation/functions/<fn_id>/verification-result.yaml`
- Optional function-local recording or input YAML such as runtime captures,
  fuzz inputs, and comparison reports

## Exit Expectations

- `verification-result.yaml` exists for every verified function.
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
- Diverged functions → schedule for another P4+P5 iteration.
