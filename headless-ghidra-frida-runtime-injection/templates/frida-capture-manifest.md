# Frida Capture Manifest Template

Use this template to record a bounded runtime-capture handoff before evidence
import begins.

## Capture Identity

- `manifest_id`:
- `target_id`:
- `target_scope`:
- `requested_scenario`:

## Reusable Script Selection

| Field                 | Summary |
| --------------------- | ------- |
| `selected_script_ids` |         |
| `selection_rationale` |         |
| `coverage_notes`      |         |
| `escalation_needed`   |         |

## Reproducible Capture Commands

- `capture_commands`:
- `capture_environment`:
- `operator_notes`:

## Runtime Artifact References

- `artifact_root`: `.work/ghidra-artifacts/<target-id>/`
- `produced_artifacts`:
- `artifact_summary`:

## Audit Gates

- `audit_gates`:
- `headless_only_confirmed`:
- `manifest_complete`:
- `handoff_ready_for_evidence_import`:
- `unresolved_gaps`:

## Next Step Routing

- `next_phase`: `headless-ghidra-frida-evidence`
- `script_review_required`:
- `script_review_reason`:
