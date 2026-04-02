phase_id: selected_decomp_incremental_compare
skill_name: Progressive Decompilation
canonical_stage_name: Selected Decompilation And Incremental Compare
artifact_purpose: Define the portable Stage 6 contract for selected decompilation, incremental compare, and audit of generated planning artifacts.
primary_consumer: downstream_user
constrained_outputs:

- spec.md
- plan.md
- tasks.md
  audit_scope:
- spec.md
- plan.md
- tasks.md
  example_path: .agents/skills/headless-ghidra-progressive-decompilation/examples/progressive-decomp-speckit-handoff.md

# When To Use This Phase Skill

Use this brief when the request has already reached Stage 6 and the current
selected target, compare boundary, and replay posture are visible enough to
plan one selected decompilation step plus incremental compare.

# Canonical Stage 6 Name

- **Skill entrypoint**: `Progressive Decompilation`
- **Canonical workflow stage**:
  `Selected Decompilation And Incremental Compare`

Use the short skill name as the entrypoint. Preserve the canonical Stage 6 name
when describing the workflow contract in generated planning artifacts.

# Non-Negotiable Stage 6 Constraints

- Stage 6 includes selected decompilation and incremental compare together.
- The workflow remains late-stage and selected-only.
- Outside-in frontier order remains the default.
- `selected_target`, `frontier_reason`, `selection_reason`, and
  `question_to_answer` must already be visible before the step is `ready`.
- `replacement_boundary`, `fallback_strategy`, and the compare command record
  must remain explicit.
- Direct invocation does not waive headless-only, evidence-backed,
  reproducible, or Markdown-reviewable expectations.
- No downstream extension or constitution change is required.

# Required Prerequisite Artifacts

- Stage rules and Stage 6 entry criteria:
  [`../headless-ghidra/examples/analysis-selection-playbook.md`](../headless-ghidra/examples/analysis-selection-playbook.md)
- Current selection and compare-input surface:
  [`../headless-ghidra/examples/artifacts/sample-target/input-inventory.md`](../headless-ghidra/examples/artifacts/sample-target/input-inventory.md)
- Replayable Stage 6 command surface:
  [`../headless-ghidra/examples/artifacts/sample-target/command-manifest.md`](../headless-ghidra/examples/artifacts/sample-target/command-manifest.md)
- Reviewable compare record:
  [`../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md`](../headless-ghidra/examples/artifacts/sample-target/comparison-command-log.md)

When source-derived reasoning is part of the step, ensure the upstream
reference posture is already reviewable before treating the invocation as
`ready`.

# Direct Invocation Readiness Model

| State       | Required inputs                                                                                                                              | Output expectation                                                                | Escalation target                                        |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `ready`     | explicit current target, populated selection fields, supported role/name/prototype evidence, reviewed compare boundary, and compare log path | generated outputs may proceed and must keep the same Stage 6 fields visible       | none                                                     |
| `qualified` | same as `ready`, but one supporting dependency remains caveated                                                                              | generated outputs must carry the caveat into the Stage 6 output and audit surface | source-comparison or compare setup review                |
| `blocked`   | one or more required selection or compare inputs is missing or unsupported                                                                   | no Stage 6 planning handoff is accepted                                           | route back to the missing prerequisite artifact or phase |

# Planning Brief Body

Use this body directly or adapt it with the same meaning:

```md
Prepare planning artifacts for the Stage 6 headless Ghidra workflow
`Selected Decompilation And Incremental Compare`.

Current Stage 6 target:

- `selected_target`:
- `frontier_reason`:
- `selection_reason`:
- `question_to_answer`:

Current compare boundary:

- `replacement_boundary`:
- `fallback_strategy`:
- `comparison_command_log_path`:
- `compare_status`:

Current evidence posture:

- role evidence:
- candidate name evidence:
- candidate prototype evidence:
- source-comparison caveat, if any:

Non-negotiable constraints:

- selected-only, outside-in Stage 6 progression
- incremental compare remains mandatory
- headless-only, evidence-backed, reproducible workflow
- reviewable Markdown outputs for `spec.md`, `plan.md`, and `tasks.md`
- no downstream `speckit` extension or constitution change required

Required reviewable output:

- selected target and why it won the current step
- current interpretation of the boundary being replaced
- incremental compare posture and any caveat
- remaining uncertainty or next unresolved gate
```

# How To Supply The Brief To Speckit

- Supply [`./planning-brief.md`](./planning-brief.md) directly.
- Or paste the `Planning Brief Body` inline into the `speckit` request.
- The transport mode must preserve the same Stage 6 constraints, readiness
  state, and output expectations.

# Audit Checklist For Generated Artifacts

- `spec.md` still treats Stage 6 as selected decompilation plus incremental
  compare.
- `plan.md` still names the prerequisite artifacts and readiness model.
- `tasks.md` still includes the happy path, blocked path, reviewer evidence,
  and final review.
- Generated outputs keep the selected target, selection reason, compare
  boundary, current interpretation, and remaining uncertainty visible.
- Any qualified caveat remains explicit rather than being flattened away.
- Any contract violation is handled by refinement or regeneration, not by
  weakening Stage 6.

### Audit Finding Format

| Field                 | Meaning                                                        |
| --------------------- | -------------------------------------------------------------- |
| `artifact_name`       | Reviewed artifact such as `spec.md`, `plan.md`, or `tasks.md`. |
| `severity`            | `info`, `warning`, or `blocking`.                              |
| `violated_rule`       | The Stage 6 checklist rule that failed.                        |
| `evidence`            | Observable artifact facts supporting the finding.              |
| `required_correction` | Required when the finding is blocking.                         |

### Required Response When Audit Fails

- Record the finding in reviewable Markdown.
- Refine or regenerate the artifact that drifted.
- Do not accept a workaround that weakens the Stage 6 contract.

# Local Rule Policy

- Local repository rules may add stricter evidence, naming, or compare-review
  requirements.
- Local repository rules may not remove selected-only gating, compare
  obligations, or reviewable Markdown outputs.
- Treat stricter local rules as additive overlays.
