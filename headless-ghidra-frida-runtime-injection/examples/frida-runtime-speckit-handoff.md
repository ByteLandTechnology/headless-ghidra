# Example: Frida Runtime Capture Handoff

## When To Use This Example

Use this example when a maintainer needs a bounded Frida runtime-capture plan
that selects a reusable script, records a capture manifest, and hands the
result into the Frida evidence-import phase.

## Purpose

Show the happy path for the runtime-injection phase: routing, reusable script
selection, CLI/headless capture, capture-manifest recording, and evidence
handoff.

## Source Contract

- [`../planning-brief.md`](../planning-brief.md)
- [`../frida-scripts/manifest.md`](../frida-scripts/manifest.md)
- [`../templates/frida-capture-manifest.md`](../templates/frida-capture-manifest.md)
- [`../../headless-ghidra-frida-evidence/SKILL.md`](../../headless-ghidra-frida-evidence/SKILL.md)

## Example Context

- Intake already normalized the target.
- The maintainer needs runtime call-tree tracing.
- The reusable script inventory already covers the requested scenario.

## Handoff Pattern

Example request shape:

```md
Use the Frida runtime-injection planning brief for this request.

Target context:

- normalized target already confirmed during intake
- runtime scope limited to one call-tree tracing question

Requested runtime evidence scenario:

- runtime call-tree tracing

Selected reusable common Frida scripts:

- `call-tree-trace`

Capture commands:

- `frida -f <target> -l call-tree-trace.js --runtime=v8 --no-pause`

Artifact root:

- `.work/ghidra-artifacts/<target-id>/`

Audit gates:

- confirm the selected script matches the scenario
- confirm produced artifacts are listed in the capture manifest
- confirm handoff goes to `headless-ghidra-frida-evidence`
```

## Expected Capture Manifest Notes

- `selected_script_ids` includes `call-tree-trace`
- `capture_commands` names the reproducible CLI/headless invocation
- `produced_artifacts` references the trace output under
  `.work/ghidra-artifacts/<target-id>/`
- `handoff_ready_for_evidence_import` is explicit
- `audit_gates` lists the checks that must stay visible before evidence import
  begins

## Expected Observations

- `spec.md` keeps runtime capture separate from evidence import.
- `plan.md` preserves the capture manifest and runtime artifact boundary.
- `tasks.md` includes reusable script selection, capture-manifest work, and the
  evidence-import handoff.
- Any later disagreement with static analysis is preserved in the evidence
  manifest rather than resolved inside the runtime phase.

## Next Step Routing

- Stay in the runtime-injection phase if `selected_script_ids`,
  `capture_commands`, produced artifacts, or audit-gate status are still
  incomplete.
- Move to
  [`../../headless-ghidra-frida-evidence/SKILL.md`](../../headless-ghidra-frida-evidence/SKILL.md)
  once the capture manifest and artifact references are complete.
- Move to
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
  instead if the reusable script inventory does not cover the request.

## Cross-Links

- Runtime review record:
  [`./runtime-routing-review-record.md`](./runtime-routing-review-record.md)
- Evidence-import handoff example:
  [`../../headless-ghidra-frida-evidence/examples/frida-trace-handoff.md`](../../headless-ghidra-frida-evidence/examples/frida-trace-handoff.md)
