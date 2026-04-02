# Example: Frida Runtime Contract Violation

## When To Use This Example

Use this example when a planning request weakens runtime-capture constraints,
skips the reusable script-selection contract, or tries to bypass script review
for a coverage gap or behavior change.

## Purpose

Show the primary blocking paths for the runtime phase: unsupported live
exploration, missing reusable library coverage, and changed script behavior
with no script-review escalation.

## Invalid Request Shapes

```md
Run Frida interactively until the trace looks useful, then summarize it later.
```

```md
No existing script covers this dispatch case, but just copy one of the current
scripts and tweak the output format inline during planning.
```

## Why This Fails

- The runtime phase is limited to reproducible CLI/headless capture, not open
  interactive exploration.
- Missing reusable script coverage must route to script review rather than be
  handled as ad hoc maintainer work.
- Behavior or output changes to a reusable script must route to script review
  so manifest expectations remain deterministic and reviewable.

## Expected Blocking Response

- Reject the request as not ready for the runtime-injection happy path.
- Require a reusable script match from
  [`../frida-scripts/manifest.md`](../frida-scripts/manifest.md) or escalate to
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md).
- Require a reviewable capture manifest before evidence import proceeds.

## Next Step Routing

- Return to the same runtime phase after script review resolves the coverage
  gap or behavior change.
- Do not hand off to evidence import until the capture-manifest and audit gates
  are complete.

## Cross-Links

- Runtime happy-path example:
  [`./frida-runtime-speckit-handoff.md`](./frida-runtime-speckit-handoff.md)
- Script-review escalation:
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
- Runtime review record:
  [`./runtime-routing-review-record.md`](./runtime-routing-review-record.md)
