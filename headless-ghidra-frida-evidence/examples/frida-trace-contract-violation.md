# Example: Contract Violation For Frida Trace Handoff

## When To Use This Example

Use this example when a generated planning artifact weakens import-only,
provenance, or Markdown-visible review expectations for Frida-backed evidence
and the reviewer needs a blocking finding format.

## Purpose

Show the primary blocking failure path for this phase: a generated planning
artifact turns imported evidence into an undocumented live-instrumentation
workflow or silently collapses a dynamic-vs-static conflict.

## Observed Generated Output

Example problem statement:

```md
The plan says the maintainer can run Frida interactively during analysis and
summarize the interesting results later if needed.

The same plan says the runtime observation should replace the decompiler result
by default, so no explicit conflict record is necessary.
```

## Audit Finding

- `artifact_name`: `plan.md`
- `severity`: `blocking`
- `violated_rule`: `Audit Checklist For Generated Artifacts -> plan must
preserve import-only provenance, reviewable replay expectations, and
explicit conflict adjudication`
- `evidence`:
  - the generated `plan.md` introduces live Frida execution as an allowed step
  - the generated `plan.md` replaces explicit provenance with an informal
    summary of later observations
  - the generated `plan.md` removes the static evidence side and defaults to
    dynamic evidence with no reviewer decision
- `required_correction`: refine or regenerate `plan.md` and any dependent
  `tasks.md` output so the workflow returns to imported, reviewable, and
  replayable Frida evidence with both conflict surfaces preserved

## Expected Reviewer Response

- Do not weaken the phase contract.
- Regenerate or refine the planning artifacts with the missing provenance,
  import-only requirements, and conflict record restored.
- If the missing detail is really new manifest-generation or normalization
  behavior, or if the evidence library needs new reusable helper coverage,
  route it through script review before regenerating the plan.

## Next Step Routing

- Send the review back through the same Frida evidence phase contract.
- If the real missing detail is a reusable normalization or manifest-generation
  script, route the follow-up through
  [`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
  before regenerating the plan.

## Cross-Links

- Owning phase contract:
  [`../planning-brief.md`](../planning-brief.md)
- Runtime blocking example:
  [`../../headless-ghidra-frida-runtime-injection/examples/frida-runtime-contract-violation.md`](../../headless-ghidra-frida-runtime-injection/examples/frida-runtime-contract-violation.md)
- Evidence review record:
  [`./evidence-adjudication-review-record.md`](./evidence-adjudication-review-record.md)
