---
name: "headless-ghidra-evidence"
description: "Phase skill for evidence extraction, replay expectations, and audit of generated planning artifacts."
phase: "evidence_replay"
---

# Headless Ghidra Evidence

Use this phase skill when planning needs to preserve how evidence will be
extracted, replayed, and reviewed after the target and scope are already
normalized.

The canonical contract is [`./planning-brief.md`](./planning-brief.md). That
brief carries evidence and replay constraints into `speckit`, then acts as the
checklist for reviewing generated planning artifacts.

## Phase Focus

This phase covers:

- evidence sources and extraction expectations
- replayable command or manifest requirements
- artifact capture and review surfaces
- validation expectations for generated planning artifacts

This phase is generic evidence planning. It does not own active Frida runtime
capture. When the request needs new Frida capture planning rather than review
of already captured outputs, route first to
[`../headless-ghidra-frida-runtime-injection/SKILL.md`](../headless-ghidra-frida-runtime-injection/SKILL.md).
When a runtime-capture manifest already exists and the remaining work is
provenance review, observed-versus-inferred claim labeling, or
static-vs-dynamic conflict recording, route to
[`../headless-ghidra-frida-evidence/SKILL.md`](../headless-ghidra-frida-evidence/SKILL.md).

## Non-Negotiable Constraints

- Headless-only workflow. Evidence collection must not depend on GUI-only
  activity.
- Evidence-backed claims. Reverse-engineering conclusions must trace to
  observable exports, manifests, or recorded outputs.
- Reproducible workflow expectations. Replay commands, inputs, and outputs must
  be explicit enough to regenerate.
- Reviewable Markdown outputs. The planning and audit surfaces remain readable
  as Markdown.
- No downstream `speckit` extension or constitution change is required.

## Required Inputs

- existing intake summary or normalized target context
- expected evidence sources and artifact types
- replay expectations, including command, manifest, or export surfaces
- validation gates a reviewer must confirm after planning
- optional local overlays that only tighten the contract

## How To Use This Skill

1. Fill in [`./planning-brief.md`](./planning-brief.md) with the evidence and
   replay expectations for the target.
2. Pass that brief into `speckit` as a file or inline paste.
3. Review the generated planning artifacts against the same evidence checklist
   before treating them as ready for implementation.
4. If a generated artifact weakens replay or evidence requirements, refine or
   regenerate the planning artifacts rather than weakening this phase contract.

## Example

- Evidence handoff example:
  [`./examples/evidence-speckit-handoff.md`](./examples/evidence-speckit-handoff.md)

## Next Step Routing

- Use this phase after intake is stable and before script-specific planning.
- Move to the Frida runtime-injection phase when the planning request still
  needs reproducible CLI/headless Frida capture, common script selection, or a
  capture manifest before imported evidence can be reviewed.
- Move to the Frida evidence phase when the planning request depends on
  externally captured Frida traces, hook logs, or session notes that need their
  own provenance and replayable handoff contract.
- Return to intake if the real gap is still target identity, initial scope, or
  setup normalization rather than evidence design.
- Move to script authoring and review when the plan introduces reusable Ghidra
  scripts, registration work, or checklist-based script review.
