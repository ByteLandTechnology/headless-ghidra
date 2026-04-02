# Example: Blocked Progressive Decompilation Direct Invocation

## Scenario

A user asks to start Stage 6 immediately, but the current selection or compare
boundary is not yet reviewable enough to support direct invocation.

## Blocking Checks

| Missing or weak item                                                                                   | Surface                                                 | Result    | Why it blocks                                                                        |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------- | --------- | ------------------------------------------------------------------------------------ |
| `selection_reason` or `question_to_answer` is missing                                                  | `input-inventory.md`                                    | `blocked` | Stage 6 cannot start until the current step has an explicit reason and question.     |
| `replacement_boundary` or `fallback_strategy` is still unresolved                                      | `analysis-selection-playbook.md` + `input-inventory.md` | `blocked` | The workflow cannot move deeper without a reviewed boundary and route-back strategy. |
| `comparison-command-log.md` is missing, stale, or only caveated without an explicit carry-forward plan | `comparison-command-log.md`                             | `blocked` | Incremental compare is part of the same Stage 6 contract, not an optional follow-up. |
| A source-derived claim still depends on a deferred or stale upstream review                            | source-comparison artifacts                             | `blocked` | The caveat is too incomplete to support the semantic claim the user wants to make.   |

Direct invocation decision: `blocked`

## Required Route-Back

- Return to target-selection or reconstruction work when the current boundary
  is not selected strongly enough.
- Return to compare planning when the command log, `replacement_boundary`, or
  `fallback_strategy` is not reviewable yet.
- Return to source-comparison review when the requested claim depends on an
  upstream posture that is still deferred or stale.
- Do not keep going by treating exploratory decompilation as good enough.

## Example Blocking Audit Findings

| artifact_name | severity   | violated_rule                                                | evidence                                                                                    | required_correction                                                               |
| ------------- | ---------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| `spec.md`     | `blocking` | `incremental compare remains mandatory`                      | The artifact describes Stage 6 as decompilation only.                                       | Regenerate the spec so incremental compare is restored as part of the same step.  |
| `plan.md`     | `blocking` | `direct invocation states are tied to visible prerequisites` | The plan lets Stage 6 start without `selection_reason` or compare-boundary review.          | Reintroduce the prerequisite artifacts and readiness states.                      |
| `tasks.md`    | `blocking` | `outside-in route-back remains explicit`                     | The tasks jump from selection directly to deep decompilation with no blocked-path evidence. | Add the blocked-path example and route-back tasks before implementation proceeds. |

## Reviewer Next Steps

- Record the blocking finding in reviewable Markdown.
- Refine or regenerate the planning artifact that weakened the gate.
- Re-run the direct invocation check only after the missing prerequisite is
  explicitly visible.
