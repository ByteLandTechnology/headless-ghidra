# Example: Frida Trace Handoff

## When To Use This Example

Use this example when the target and scope are already normalized and planning
must incorporate externally captured Frida evidence without making live
instrumentation part of the supported workflow.

## Purpose

Show how imported Frida evidence is handed into `speckit` as a reviewable
planning contract after a runtime-capture manifest already exists, and how the
same contract guides post-generation audit.

## Source Contract

- [`../planning-brief.md`](../planning-brief.md)

## Example Context

- Intake is already stable for the target.
- The maintainer has a Frida-derived observation bundle linked to a
  runtime-capture manifest and artifacts under
  `.work/ghidra-artifacts/<target-id>/`.
- Reviewers need Markdown-visible proof of provenance, replay notes, and known
  gaps before planning can proceed.

## Handoff Pattern

Example request shape:

```md
Use the Frida evidence planning brief for this request after runtime capture.

Prepare `spec.md`, `plan.md`, and `tasks.md` for a headless-only workflow that
imports Frida-derived evidence as a reviewable external input.

Target context:

- normalized target already confirmed during intake

Imported Frida evidence bundle:

- externally captured trace summary
- hook profile summary

Linked runtime-capture manifest:

- `.work/ghidra-artifacts/<target-id>/frida-capture-manifest.md`
- selected reusable script ids and capture commands already recorded

Provenance surface:

- evidence mapped to the target and capture context
- verification notes recorded in Markdown

Observed claims:

- direct runtime observations from the trace

Inferred claims:

- analyst interpretation kept separate from direct observations

Static evidence context:

- any conflicting decompiler or listing interpretation stays visible

Conflict record surface:

- explicit `target_subject`, `static_evidence`, `dynamic_evidence`,
  `reviewer_decision`, and `decision_rationale`

Non-negotiable constraints:

- import-only workflow
- headless-only review and planning
- evidence-backed claims
- reproducible provenance and replay expectations
- reviewable Markdown outputs
- no downstream speckit extension or constitution change required

Local rule overlay:

- repository may require a named Frida evidence manifest section
- repository may not replace provenance notes with maintainer memory
```

## Expected Observations

- `spec.md` names the imported Frida evidence bundle, linked capture manifest,
  and why they matter.
- `plan.md` keeps provenance, replay expectations, and conflict adjudication
  explicit.
- `tasks.md` includes reviewable work for validating gaps, ambiguity,
  follow-up actions, and preserved conflict records.

## Audit Walkthrough

- Confirm generated artifacts preserve import-only wording.
- Confirm provenance and replay notes remain concrete enough to verify the
  evidence bundle and its linked runtime-capture manifest.
- Confirm observed claims stay distinct from inferred claims.
- Confirm any dynamic-vs-static disagreement preserves both evidence sides.
- Confirm outputs stay visible in Markdown.
- If a generated artifact weakens the Frida evidence contract, refine or
  regenerate the planning artifacts instead of weakening the contract.

## Local Rule Interpretation

- Valid additive overlay: the repository requires a Markdown manifest table for
  imported traces and provenance.
- Invalid weakening attempt: a repository allows Frida observations to be
  summarized informally with no artifact or provenance surface.

## Next Step Routing

- Return to intake if target identity or scope is still unstable.
- Return to the runtime-injection phase if the linked capture manifest,
  `selected_script_ids`, `capture_commands`, or runtime audit-gate status are
  incomplete.
- Stay in Frida evidence while provenance, replay, or evidence-gap handling is
  still being defined.
- Move to script authoring and review when the plan introduces reusable
  normalization or manifest-generation scripts.

## Cross-Links

- Umbrella routing:
  [`../../headless-ghidra/SKILL.md`](../../headless-ghidra/SKILL.md)
- Runtime-capture source:
  [`../../headless-ghidra-frida-runtime-injection/SKILL.md`](../../headless-ghidra-frida-runtime-injection/SKILL.md)
- Runtime happy-path example:
  [`../../headless-ghidra-frida-runtime-injection/examples/frida-runtime-speckit-handoff.md`](../../headless-ghidra-frida-runtime-injection/examples/frida-runtime-speckit-handoff.md)
- Script authoring/review next step:
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
- Evidence review record:
  [`./evidence-adjudication-review-record.md`](./evidence-adjudication-review-record.md)
