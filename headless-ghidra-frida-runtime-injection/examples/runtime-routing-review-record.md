# Example: Runtime Routing Review Record

## When To Use This Example

Use this review record after a runtime-capture plan is drafted and a reviewer
needs to confirm that the request still fits the supported Frida runtime phase
before handing off into evidence import.

## Purpose

Preserve a local review-record surface for runtime-routing decisions without
depending on unpublished external spec files.

## Review Scope

- requested scenario maps to the shipped reusable script library
- selected script ids and capture commands are explicit
- artifact outputs stay under `.work/ghidra-artifacts/`
- evidence-import handoff is blocked until the capture manifest is complete

## Example Review Record

- `review_id`: `frida-runtime-routing-sample`
- `phase`: `headless-ghidra-frida-runtime-injection`
- `review_surface`:
  [`./frida-runtime-speckit-handoff.md`](./frida-runtime-speckit-handoff.md)
- `linked_manifest_template`:
  [`../templates/frida-capture-manifest.md`](../templates/frida-capture-manifest.md)
- `reviewer_decision`: `route_to_evidence_import_when_manifest_complete`
- `decision_rationale`:
  the capture path stays headless-only, script selection is explicit, and the
  next supported step is Frida evidence import rather than ad hoc runtime work
- `blocking_issues`: `none`

## If The Review Fails

- Keep the request in the runtime-injection phase until script coverage,
  capture commands, and audit gates are complete.
- Route reusable-library gaps to
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md).
- Do not hand off to Frida evidence import with an incomplete manifest.

## Cross-Links

- Owning phase contract:
  [`../planning-brief.md`](../planning-brief.md)
- Positive handoff example:
  [`./frida-runtime-speckit-handoff.md`](./frida-runtime-speckit-handoff.md)
- Blocking example:
  [`./frida-runtime-contract-violation.md`](./frida-runtime-contract-violation.md)
