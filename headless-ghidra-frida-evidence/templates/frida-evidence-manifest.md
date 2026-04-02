# Frida Evidence Manifest Template

Use this template to summarize imported Frida-derived evidence in a Markdown
surface that remains reviewable, replayable, and separate from live
instrumentation steps.

## Target Summary

- `manifest_id`:
- `target_id`:
- `target_scope`:
- `linked_intake_artifact`:
- `artifact_root`: `.work/ghidra-artifacts/<target-id>/`
- `linked_capture_manifest`:

## Evidence Bundle

| Field                  | Summary |
| ---------------------- | ------- |
| `trace_summary`        |         |
| `hook_profile_summary` |         |
| `supporting_outputs`   |         |
| `known_gaps`           |         |

## Provenance

| Field              | Summary |
| ------------------ | ------- |
| `capture_source`   |         |
| `captured_at`      |         |
| `target_linkage`   |         |
| `provenance_notes` |         |
| `integrity_notes`  |         |

## Runtime Capture Linkage

| Field                 | Summary |
| --------------------- | ------- |
| `selected_script_ids` |         |
| `capture_commands`    |         |
| `produced_artifacts`  |         |
| `audit_gate_status`   |         |

## Replay And Verification Notes

- `review_surface`:
- `verification_expectations`:
- `follow_up_needed`:

## Analyst Interpretation Boundaries

- `observed_claims`:
- `inferred_claims`:
- `open_questions`:

## Conflict Adjudication Record

| Field                | Summary |
| -------------------- | ------- |
| `target_subject`     |         |
| `static_evidence`    |         |
| `dynamic_evidence`   |         |
| `reviewer_decision`  |         |
| `decision_rationale` |         |

- `conflict_record_ids`:

## Reviewer Decision

- `ready_for_planning`:
- `blocking_issues`:
