# Example: Frida Contract Violation

## When To Use This Example

Use this example when a planning request mentions Frida but the supplied
material does not satisfy the import-only, provenance, or replay expectations
required by the supported handoff contract.

## Purpose

Show how the Frida evidence phase blocks unsupported requests instead of
allowing missing provenance, missing runtime-capture linkage, or unverifiable
observations to leak into planning artifacts.

## Failure Context

- The request asks the skill family to run Frida live during analysis
- The evidence bundle has no linked runtime-capture manifest
- The evidence bundle does not identify a stable target or provenance source
- The request assumes missing reusable normalization or manifest-generation
  behavior can be improvised during review with no script-review escalation
- Observations are mixed with analyst interpretation and no claim-status labels
  are present

## Invalid Request Shape

```md
Use the Frida child skill and figure it out from here.

Frida notes:

- I hooked a few things live and saw suspicious calls
- I do not remember which binary build this came from
- We can add the replay details later if needed
```

## Why This Fails

- The contract is import-only and does not support live Frida execution inside
  the workflow.
- The evidence bundle does not link back to a reviewable runtime-capture
  manifest.
- The evidence bundle lacks enough provenance to tie the observations to a
  normalized target.
- Replay and review expectations are missing, so another reviewer cannot check
  what was actually observed.
- Verified observations are not separated from inference or speculation.
- Any missing reusable normalization coverage or behavior change would still
  require script review once a valid evidence bundle exists.

## Expected Blocking Response

- Reject the request as not ready for Frida evidence handoff.
- Require a reviewable evidence bundle with a linked runtime-capture manifest,
  provenance, target linkage, and replay notes before planning continues.
- Route reusable normalization, manifest-generation, missing-coverage, or
  helper behavior changes to script review only after a valid evidence bundle
  exists.

## Cross-Links

- Frida phase contract:
  [`../planning-brief.md`](../planning-brief.md)
- Positive handoff example:
  [`./frida-trace-handoff.md`](./frida-trace-handoff.md)
- Conflict-preservation example:
  [`./frida-trace-contract-violation.md`](./frida-trace-contract-violation.md)
- Script-review escalation:
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
