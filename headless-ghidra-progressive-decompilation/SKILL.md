---
name: "headless-ghidra-progressive-decompilation"
description: "Phase skill for Stage 6 selected decompilation, incremental compare, and audit of generated planning artifacts."
phase: "selected_decompilation_incremental_compare"
---

# Headless Ghidra Progressive Decompilation

Use this phase skill when the earlier evidence, target-selection, and compare
boundary surfaces are already reviewable and you need the standalone Stage 6
contract for `Selected Decompilation And Incremental Compare`.

The canonical contract is [`./planning-brief.md`](./planning-brief.md). That
brief is the portable handoff surface for `speckit`, then becomes the audit
checklist for generated `spec.md`, `plan.md`, and `tasks.md`.

## When To Use This Phase Skill

Use this skill when the request has already reached Stage 6 and the current
outside-in step is concrete enough to classify as `ready`, `qualified`, or
`blocked`.

## Canonical Stage 6 Name

- **Phase skill name**: `Headless Ghidra Progressive Decompilation`
- **Canonical workflow stage name**:
  `Selected Decompilation And Incremental Compare`

The skill name is the independently invocable entrypoint. The longer Stage 6
name stays unchanged across the repository's existing reverse-engineering
artifacts.

## Full Stage 6 Scope

This phase covers:

- one selected outside-in decompilation step at the current frontier
- the incremental compare required for the replaced boundary
- explicit carry-forward of any caveat that keeps the step `qualified`
- audit of generated planning artifacts against the same Stage 6 contract

## Non-Negotiable Stage 6 Constraints

- Stage 6 covers selected decompilation plus incremental compare as one
  contract.
- Decompilation remains selected-only and late-stage.
- Outside-in ordering remains the default progression rule.
- A current target, `selection_reason`, and `question_to_answer` must already
  be visible before direct invocation is `ready`.
- The current step must keep `replacement_boundary`, `fallback_strategy`, and a
  reviewable compare record explicit.
- Direct invocation does not waive headless-only, evidence-backed,
  reproducible, or Markdown-reviewable workflow expectations.
- No downstream `speckit` extension or constitution change is required.

## Required Prerequisite Artifacts

- Stage guidance and canonical Stage 6 rules:
  [`../headless-ghidra/examples/analysis-selection-playbook.md`](../headless-ghidra/examples/analysis-selection-playbook.md)
- Current Stage 6 selection and compare-input surface:
  [`../headless-ghidra/examples/artifacts/sample-target/input-inventory.md`](../headless-ghidra/examples/artifacts/sample-target/input-inventory.md)
- Replayable Stage 6 command surface:
  [`../headless-ghidra/examples/artifacts/sample-target/command-manifest.md`](../headless-ghidra/examples/artifacts/sample-target/command-manifest.md)
- Reviewable incremental compare record:
  [`../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md`](../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md)

When a source-derived claim is part of the current step, the upstream reference
posture must also already be reviewable. Route back to source-comparison
artifacts before proceeding when that posture is still deferred or stale.

## Direct Invocation Readiness Model

| State       | Use it when                                                                                                                                                                                       | Required response                                                                                                          |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `ready`     | `selected_target`, `frontier_reason`, `selection_reason`, `question_to_answer`, role/name/prototype support, `replacement_boundary`, `fallback_strategy`, and the compare log are all reviewable. | Proceed with Stage 6 planning and keep the same constraints visible in the generated outputs.                              |
| `qualified` | The step is usable, but one dependency remains caveated, such as a qualified upstream reference or constrained compare setup.                                                                     | Proceed only if the caveat is carried into the Stage 6 output and later audit finding.                                     |
| `blocked`   | Required selection fields are missing, evidence for role/name/prototype is still weak, or the compare boundary/log is not yet supportable.                                                        | Route back to the missing prerequisite artifact or earlier phase instead of treating exploratory decompilation as allowed. |

## How To Use This Phase Skill

1. Re-open the Stage 6 source surfaces and confirm the current step is
   `ready`, `qualified`, or `blocked`.
2. Fill in [`./planning-brief.md`](./planning-brief.md) with the current
   selected target, compare boundary, and output expectations.
3. Hand that brief into `speckit` as a file or inline paste.
4. Re-open this skill and audit generated `spec.md`, `plan.md`, and `tasks.md`
   with the same Stage 6 checklist.
5. If any blocking mismatch appears, refine or regenerate the planning
   artifacts rather than weakening the contract.

## Stage 6 Output Expectations

Every Stage 6 step that proceeds must leave a reviewable output that keeps the
selected target, selection reason, current interpretation, incremental compare
posture, and remaining uncertainty explicit.

## Worked Examples

- Happy-path handoff:
  [`./examples/progressive-decomp-speckit-handoff.md`](./examples/progressive-decomp-speckit-handoff.md)
- Blocked direct invocation and audit findings:
  [`./examples/blocked-direct-invocation.md`](./examples/blocked-direct-invocation.md)

## Audit Checklist For Generated Artifacts

- `spec.md` still describes the full Stage 6 scope rather than decompilation
  alone.
- `plan.md` keeps direct invocation states, prerequisite artifacts, and the
  compare boundary visible.
- `tasks.md` includes work for the happy path, blocked path, audit flow, and
  reviewer evidence.
- Generated outputs keep the selected target, selection reason, current
  interpretation, incremental compare posture, and remaining uncertainty
  explicit.
- Outside-in ordering and route-back behavior remain visible instead of being
  flattened into generic late-stage analysis.
- If any generated artifact drops the compare obligation or hides a caveat, the
  result is a blocking finding that must be corrected by refinement or
  regeneration.

### Audit Finding Format

- `artifact_name`
- `severity`
- `violated_rule`
- `evidence`
- `required_correction` when the finding is blocking

## Post-Speckit Audit Routing

- Re-open the generated artifact that violated the contract.
- Refine or regenerate the planning output so the Stage 6 rule becomes visible
  again.
- Do not accept a local shortcut that weakens Stage 6 in order to make the
  artifact pass.

## Next Step Routing

- Use the umbrella skill when the real question is still which phase applies.
- Return to intake when target identity, archive normalization, or planning
  scope is still unstable.
- Return to evidence when the replay or artifact surface is not reviewable yet.
- Return to source-comparison artifacts when a source-derived claim still lacks
  a reviewable upstream posture.
- Return to semantic reconstruction when role, name, or prototype evidence is
  not strong enough for Stage 6.

## Local Rule Policy

- Local repository rules may require extra review evidence, stricter naming, or
  additional compare notes.
- Local repository rules may not remove selected-only gating, compare
  obligations, or Markdown-visible audit expectations.
- Treat stricter local rules as additive overlays.
- Treat any local rule that weakens this Stage 6 contract as invalid.
