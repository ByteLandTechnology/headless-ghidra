# Example: Progressive Decompilation Speckit Handoff

## Scenario

An analyst has already completed evidence review and target selection for the
current Stage 6 frontier step. They want to invoke `Progressive
Decompilation` directly without reopening the umbrella skill as the only source
of truth.

## Source Surfaces Used

- Stage rules:
  [`../../headless-ghidra/examples/analysis-selection-playbook.md`](../../headless-ghidra/examples/analysis-selection-playbook.md)
- Selection and compare-input surface:
  [`../../headless-ghidra/examples/artifacts/sample-target/input-inventory.md`](../../headless-ghidra/examples/artifacts/sample-target/input-inventory.md)
- Replayable Stage 6 commands:
  [`../../headless-ghidra/examples/artifacts/sample-target/command-manifest.md`](../../headless-ghidra/examples/artifacts/sample-target/command-manifest.md)
- Compare record:
  [`../../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md`](../../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md)

## Readiness Check

| Check                                                           | Surface                                                 | Result  | Why it matters                                                                  |
| --------------------------------------------------------------- | ------------------------------------------------------- | ------- | ------------------------------------------------------------------------------- |
| Current target and frontier context are already selected        | `input-inventory.md` + `analysis-selection-playbook.md` | `ready` | Stage 6 is selected-only and cannot start from a generic candidate list.        |
| `selection_reason` and `question_to_answer` are already visible | `input-inventory.md`                                    | `ready` | The next decompilation step must explain why this boundary is the current step. |
| The compare record is named and ready to capture the step       | `comparison-command-log.md`                             | `ready` | Incremental compare is mandatory for Stage 6.                                   |
| Replay commands exist for selected decompilation planning       | `command-manifest.md`                                   | `ready` | The planned step stays reproducible instead of becoming an informal note.       |

Direct invocation decision: `ready`

## Planning Brief Excerpt For Speckit

```md
Prepare planning artifacts for one Stage 6 step in
`Selected Decompilation And Incremental Compare`.

Current Stage 6 target:

- `selected_target`: use the currently recorded frontier function
- `frontier_reason`: carry forward the recorded outside-in reason
- `selection_reason`: explain why this boundary won the current step
- `question_to_answer`: state what the step should resolve next

Current compare boundary:

- `replacement_boundary`: replace only the reviewed current boundary
- `fallback_strategy`: keep the route for unresolved callees explicit
- `comparison_command_log_path`: use the tracked compare log
- `compare_status`: preserve the current clean or caveated posture

Required reviewable output:

- selected target and why it won the step
- current interpretation of the replaced boundary
- incremental compare posture
- remaining uncertainty before the workflow moves inward
```

## Expected Reviewable Output

A compliant generated output keeps all of the following visible:

- `selected_target`
- `selection_reason`
- current interpretation of the boundary being replaced
- incremental compare posture for the same step
- remaining uncertainty or next unresolved gate

## Qualified Variation

If one dependency is only `qualified`, such as a source-comparison posture that
remains caveated, the invocation may still proceed only when that caveat is
carried into the generated output and later audit note.

Expected qualified carry-forward:

- the same `selected_target` still anchors the step
- the caveat is named explicitly rather than hidden in reviewer intuition
- the audit checklist can tell why the step was usable but not fully clean

## Reviewer Next Steps

- Re-open the same `planning-brief.md` after `speckit` generation.
- Confirm the generated `spec.md`, `plan.md`, and `tasks.md` still describe
  Stage 6 as selected decompilation plus incremental compare.
- If the generated artifacts drop the compare obligation, hide the caveat, or
  flatten outside-in ordering, treat the result as blocking and regenerate it.
