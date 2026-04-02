---
name: "headless-ghidra-intake"
description: "Phase skill for target intake and project initialization before speckit planning or audit."
phase: "intake_init"
---

# Headless Ghidra Intake

Use this phase skill when the work is still defining the target, project
layout, analyst intent, and the minimum planning inputs that must survive a
`speckit` handoff.

This skill is the contract surface for intake and initialization. The canonical
artifact is [`./planning-brief.md`](./planning-brief.md). Use that file to
carry constraints into `speckit`, then use the same file to review generated
planning artifacts.

## Phase Focus

This phase covers:

- target identity and source description
- project/workspace initialization assumptions
- analysis scope boundaries
- initial analyst questions and deliverable types
- prerequisites needed before evidence extraction or script work

This phase does not replace later evidence or script-review guidance. It
prepares the baseline that those later phases build on.

## Non-Negotiable Constraints

- Keep the workflow headless-only. GUI-only steps are out of scope.
- Keep claims evidence-backed. Intake facts should cite observed inputs,
  provided binaries, manifests, or already-recorded repository evidence.
- Keep the workflow reproducible. The plan must preserve project naming,
  workspace assumptions, and replayable setup expectations.
- Keep outputs reviewable in Markdown. `spec.md`, `plan.md`, and `tasks.md`
  should remain inspectable without hidden tools or downstream hooks.
- Do not require downstream `speckit` extensions or constitution edits to use
  this phase contract.

## Required Inputs

Prepare the planning brief with:

- target name and binary or sample identity
- source or provenance notes for the target
- intended reverse-engineering scope and out-of-scope areas
- expected deliverables for planning
- existing repository constraints or local overlays that only tighten the
  contract

## Runtime Choice UX

When the running skill genuinely needs the user to choose between scope
boundaries, deliverable types, or other discrete intake options:

1. If the runtime exposes a structured choice input tool (for example
   `request_user_input`), use it instead of a plain-text list.
2. Keep each option short, mutually exclusive, and user-facing.
3. Put the recommended or default option first whenever the current intake
   evidence clearly favors one, and state that recommendation briefly.
4. Fall back to Markdown or plain-text lists only when no structured choice
   input is available.
5. If only one reviewed scope or deliverable path remains, do not force a
   dialog; state the automatic default and the intake evidence supporting it.

## How To Use This Skill

1. Fill in [`./planning-brief.md`](./planning-brief.md) with the intake facts
   that are already known.
2. Provide that brief to `speckit` either as the file itself or as an inline
   paste.
3. Check the generated `spec.md`, `plan.md`, and `tasks.md` against the same
   intake contract before moving deeper into evidence or script work.
4. If the generated artifacts drop an intake constraint, refine or regenerate
   the planning artifacts rather than weakening this phase contract.

## Example

- Portable handoff example:
  [`./examples/intake-speckit-handoff.md`](./examples/intake-speckit-handoff.md)

## Next Step Routing

- Stay in intake when target identity, scope, or deliverables are still fuzzy.
- Move to the evidence phase after intake is stable and planning needs replay,
  extraction, or artifact expectations.
- Move to script authoring and review when the plan introduces reusable
  headless Ghidra scripts or checklist-governed script changes.
