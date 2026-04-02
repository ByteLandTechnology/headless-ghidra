# Example: Evidence Adjudication Review Record

## When To Use This Example

Use this review record after a Frida evidence-import handoff is drafted and a
reviewer needs a compact, Markdown-visible place to confirm provenance,
conflict handling, and planning readiness.

## Purpose

Preserve the review-record surface that decides whether imported Frida evidence
is ready for planning or must be sent back for correction.

## Review Scope

- linked runtime-capture manifest is present
- provenance and target linkage are explicit
- observed claims stay separate from inferred claims
- dynamic-vs-static conflicts keep both evidence sides visible

## Example Review Record

- `review_id`: `frida-evidence-adjudication-sample`
- `phase`: `headless-ghidra-frida-evidence`
- `review_surface`: [`./frida-trace-handoff.md`](./frida-trace-handoff.md)
- `linked_manifest_template`:
  [`../templates/frida-evidence-manifest.md`](../templates/frida-evidence-manifest.md)
- `reviewer_decision`: `ready_for_planning`
- `decision_rationale`:
  imported evidence stays tied to a runtime-capture manifest, preserves
  provenance, and records any unresolved conflict in Markdown
- `blocking_issues`: `none`

## If The Review Fails

- Send the work back through the same Frida evidence phase contract.
- Preserve both dynamic and static evidence until a reviewer adjudicates them.
- Route any reusable manifest-generation or normalization gap to
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md).

## Cross-Links

- Owning phase contract:
  [`../planning-brief.md`](../planning-brief.md)
- Positive handoff example:
  [`./frida-trace-handoff.md`](./frida-trace-handoff.md)
- Blocking example:
  [`./frida-trace-contract-violation.md`](./frida-trace-contract-violation.md)
