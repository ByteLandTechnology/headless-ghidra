# Example: Script Authoring Review Audit

## When To Use This Example

Use this example when a generated `spec.md`, `plan.md`, or `tasks.md`
introduces reusable headless Ghidra scripts and you need to confirm that
deterministic behavior, registration, and checklist-based review survived the
handoff.

## Purpose

Show how the script authoring and review contract is used both for the planning
handoff and for post-generation review.

## Source Contract

- [`../planning-brief.md`](../planning-brief.md)

## Example Context

- The plan introduces a reusable headless export script.
- The repository expects deterministic parameters and a review checklist.
- Review findings must remain readable in Markdown.

## Handoff Pattern

Example request shape:

```md
Use the script authoring and review planning brief for this request.

Prepare `spec.md`, `plan.md`, and `tasks.md` for a reusable headless Ghidra
script.

Script scope:

- export deterministic analysis artifacts

Registration requirements:

- repository-relative script location
- explicit review checklist

Non-negotiable constraints:

- headless-only workflow
- evidence-backed claims
- reproducible script execution and review
- reviewable Markdown outputs
- no downstream speckit extension or constitution change required

Local rule overlay:

- repository may require a tighter naming pattern for reusable scripts
- repository may not drop deterministic registration or checklist review
```

## Audit Walkthrough

Review the generated artifacts with these checks:

- `spec.md` states the script objective and deterministic expectations.
- `plan.md` preserves repository-relative registration and replay steps.
- `tasks.md` includes checklist-governed authoring and review work.
- If any of these checks fail, refine or regenerate the planning artifacts.
  Never weaken the contract to match the bad output.

## Review Notes

- If a generated artifact drops the review checklist, treat that as a contract
  failure.
- The response is to refine or regenerate the planning artifacts, not to relax
  the contract.

## Next Step Routing

- Return to evidence if the missing detail is really an upstream replay or
  artifact-capture problem.
- Stay in script authoring and review when the missing detail is about script
  determinism, repository-relative placement, registration, or checklist-based
  review.

## Cross-Links

- Umbrella routing:
  [`../../headless-ghidra/SKILL.md`](../../headless-ghidra/SKILL.md)
- Contract violation example:
  [`./contract-violation-example.md`](./contract-violation-example.md)
