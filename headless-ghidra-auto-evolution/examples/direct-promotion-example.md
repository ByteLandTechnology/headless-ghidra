# Example: Direct Promotion From One Real Task

## When To Use This Example

Use this example when one completed real task already contains enough evidence
to justify creating a new tracked child-skill entry and updating the umbrella
skill routing without waiting for a second validating task.

## Source Corpus

- Completed repository work that introduced the
  `headless-ghidra-auto-evolution` child skill and its review template.
- [`../SKILL.md`](../SKILL.md)
- [`../templates/auto-evolution-review-record.md`](../templates/auto-evolution-review-record.md)
- [`../../headless-ghidra/SKILL.md`](../../headless-ghidra/SKILL.md)

## Example Context

- Real task scope: feature `006-skill-auto-evolution`
- Observed reusable candidate: an explicit post-task workflow for extracting
  reusable improvements from real work
- Requested outcome: direct tracked-asset creation if the evidence is complete

## Review Snapshot

- `task_context`: the repository already completed real planning and analysis
  work for feature `006`, and that work repeatedly referenced the need for an
  explicit auto-evolution child skill rather than a hidden maintainer habit.
- `reusable_part_summary`: promote the reviewable workflow for mining reusable
  improvements from completed work into a dedicated child-skill entry with its
  own template and examples.
- `benefit_statement`: future maintainers and agents can invoke the same
  governed workflow directly instead of rediscovering the process from scratch.
- `non_sample_specific_reasoning`: the feature is defined around repository
  workflow governance rather than any one binary, address map, or target quirk;
  the promoted asset is a reusable child-skill surface for the whole skill
  family.
- `input_trust_status`: reviewed notes and evidence stay untrusted inputs and
  are reduced to repo-authored summaries before promotion.
- `embedded_instruction_handling`: no embedded instruction from the reviewed
  corpus drives credentials, permissions, or unrelated actions.
- `maintainer_approval_status`: `approved` for the `child_skill_entry`
  promotion, recorded in the review record before direct creation.

## Candidate Review

| candidate_id                       | candidate_kind      | reusable_part_summary                                                                                        | sample_specific_details                                                        | expected_benefit                                                        | non_sample_specific_reasoning                                                                       | overlap_status | evidence_status | classification |
| ---------------------------------- | ------------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------ | ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- | -------------- | --------------- | -------------- |
| `auto-evolution-child-skill-entry` | `child_skill_entry` | Add a dedicated child skill, review template, and worked examples for post-task reusable-improvement review. | No target-specific addresses, binaries, or one-off runtime paths are promoted. | Makes reusable-improvement capture explicit, reviewable, and teachable. | The workflow governs repository skill evolution and applies across completed tasks, not one sample. | `new_path`     | `complete`      | `accepted`     |

## Promotion Decision Log

| decision_id                      | candidate_id                       | final_action    | target_assets                                                                                                                                                                                                                                                                                                                                                                | overlap_resolution                                                                                                                                                                                                     | justification                                                                                                              | runtime_boundary_note                                                                                                                                                         | approval_gate_status |
| -------------------------------- | ---------------------------------- | --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------- |
| `decision-auto-evolution-create` | `auto-evolution-child-skill-entry` | `direct_create` | `.agents/skills/headless-ghidra-auto-evolution/SKILL.md`; `.agents/skills/headless-ghidra-auto-evolution/templates/auto-evolution-review-record.md`; `.agents/skills/headless-ghidra-auto-evolution/examples/direct-promotion-example.md`; `.agents/skills/headless-ghidra-auto-evolution/examples/deferred-candidate-example.md`; `.agents/skills/headless-ghidra/SKILL.md` | Existing phase skills cover intake, evidence, and script authoring, but none owns post-task auto evolution. A new child-skill entry is justified, while the umbrella skill is updated instead of duplicated elsewhere. | One real task produced a complete, reviewable evidence set and named tracked paths for the supported new workflow surface. | Any future runtime helper or export discovered during auto evolution still remains under `.work/ghidra-artifacts/`; this example promotes only tracked Markdown skill assets. | `approved`           |

## Asset Target Summary

| asset_path                                                                                | asset_type          | parent_surface                                             | change_mode       | review_visibility     |
| ----------------------------------------------------------------------------------------- | ------------------- | ---------------------------------------------------------- | ----------------- | --------------------- |
| `.agents/skills/headless-ghidra-auto-evolution/SKILL.md`                                  | `child_skill_entry` | `.agents/skills/headless-ghidra-auto-evolution/`           | `create_new`      | `child_skill_visible` |
| `.agents/skills/headless-ghidra-auto-evolution/templates/auto-evolution-review-record.md` | `template`          | `.agents/skills/headless-ghidra-auto-evolution/templates/` | `create_new`      | `child_skill_visible` |
| `.agents/skills/headless-ghidra-auto-evolution/examples/direct-promotion-example.md`      | `example`           | `.agents/skills/headless-ghidra-auto-evolution/examples/`  | `create_new`      | `child_skill_visible` |
| `.agents/skills/headless-ghidra-auto-evolution/examples/deferred-candidate-example.md`    | `example`           | `.agents/skills/headless-ghidra-auto-evolution/examples/`  | `create_new`      | `child_skill_visible` |
| `.agents/skills/headless-ghidra/SKILL.md`                                                 | `skill_file`        | `.agents/skills/headless-ghidra/`                          | `update_existing` | `umbrella_visible`    |

## Why Direct Promotion Is Allowed

- The example is grounded in one completed real task with repository-tracked
  artifacts.
- All four proof elements are explicit in one review surface.
- The promoted asset type is a `child_skill_entry`, which is one of the
  highest-risk tracked asset types in this feature.
- The resulting tracked paths are named directly and do not depend on hidden
  manual approval logic.

## Reviewer Conclusion

A reviewer can confirm from one task record that direct promotion is justified,
that the promoted asset type is `child_skill_entry`, and that the resulting
tracked paths are explicit and bounded.
