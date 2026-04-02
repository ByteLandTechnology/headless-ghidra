# Example: Contract Violation For Script Authoring Review

## When To Use This Example

Use this example when a generated planning artifact weakens script
determinism, repository-relative registration, or checklist-governed review and
the reviewer needs a blocking finding format to send the work back for
correction.

## Purpose

Show the primary blocking failure path for this phase: a generated planning
artifact weakens the script contract.

## Observed Generated Output

Example problem statement:

```md
The plan mentions a helper script but leaves registration unspecified and says
review can happen informally after implementation.
```

## Audit Finding

- `artifact_name`: `plan.md`
- `severity`: `blocking`
- `violated_rule`: `Audit Checklist For Generated Artifacts -> tasks and plan
must preserve checklist-based review and replayable registration`
- `evidence`:
  - the generated `plan.md` says review can happen informally after
    implementation
  - the generated `plan.md` omits repository-relative registration details
- `required_correction`: refine or regenerate `plan.md` and any dependent
  `tasks.md` output so the reusable script remains deterministic, headless-only,
  and governed by a Markdown-reviewable checklist

## Expected Reviewer Response

- Do not weaken the phase contract.
- Regenerate or refine the planning artifacts with the missing review and
  registration requirements restored.

## Next Step Routing

- Send the review back through the same script authoring and review phase
  contract.
- If the missing detail actually belongs to evidence replay rather than script
  obligations, route the follow-up through
  [`../../headless-ghidra-evidence/SKILL.md`](../../headless-ghidra-evidence/SKILL.md)
  before regenerating the plan.

## Cross-Links

- Owning phase contract:
  [`../planning-brief.md`](../planning-brief.md)
