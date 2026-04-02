---
name: "headless-ghidra-script-review"
description: "Phase skill for reusable headless Ghidra script authoring, review, registration, and contract-based audit."
phase: "script_authoring_review"
---

# Headless Ghidra Script Review

Use this phase skill when planning includes reusable headless Ghidra scripts,
script registration, or review expectations that must survive `speckit`
planning, including reusable Frida runtime-capture helpers, common-library
coverage changes, or manifest-generation logic that should not stay hidden in
phase-specific notes.

The canonical contract is [`./planning-brief.md`](./planning-brief.md). Use it
to shape planning outputs and to audit generated artifacts for missing or
weakened script-review requirements.

## Phase Focus

This phase covers:

- reusable headless Ghidra script authoring
- reusable Frida runtime-capture helper and common-library governance
- deterministic script inputs and outputs
- manifest-generation and normalization helper review
- script registration and naming expectations
- review checklist obligations for generated plans

## Non-Negotiable Constraints

- Headless-only workflow. Script usage must stay compatible with headless
  analysis flows.
- Evidence-backed claims. Script behavior and review findings must point to
  observable output or tracked examples.
- Reproducible workflow expectations. Script invocation, parameters,
  registration, and review steps must be replayable.
- Reviewable Markdown outputs. Generated planning artifacts and findings remain
  inspectable as Markdown.
- Runtime and evidence phases must escalate reusable helper, behavior-change,
  coverage-gap, or manifest-generation work here instead of weakening their own
  phase contracts.
- No downstream `speckit` extension or constitution change is required.

## Required Inputs

- planned script purpose and target workflow stage
- whether the request changes reusable Frida script-library coverage,
  invocation behavior, or manifest expectations
- expected script inputs, outputs, and deterministic behavior
- registration or naming expectations for reusable scripts
- capture-helper or manifest-generation obligations when Frida phases are in
  scope
- required review checklist items and failure handling
- optional local overlays that only tighten the contract

## How To Use This Skill

1. Fill in [`./planning-brief.md`](./planning-brief.md) with the script scope,
   deterministic expectations, and review obligations.
2. Provide the brief to `speckit` as a file or inline paste.
3. Review the generated planning artifacts with the same contract.
4. If runtime-injection or evidence-import work introduces reusable Frida
   helpers, output-shape changes, or missing common-library coverage, keep the
   change in this phase rather than pushing it back into the runtime or
   evidence contracts.
5. If a required script rule is missing, refine or regenerate the artifacts
   rather than weakening the contract.

## Examples

- Audit-oriented example:
  [`./examples/script-authoring-review-audit.md`](./examples/script-authoring-review-audit.md)
- Contract violation example:
  [`./examples/contract-violation-example.md`](./examples/contract-violation-example.md)

## Next Step Routing

- Use this phase when the plan introduces new reusable scripts or changes to
  how scripts are reviewed and registered.
- Use this phase when `headless-ghidra-frida-runtime-injection` needs a new
  reusable capture helper, a changed common-script behavior, or updated
  coverage notes for one of the five supported Frida runtime scenarios.
- Use this phase when `headless-ghidra-frida-evidence` needs reusable
  normalization or manifest-generation logic rather than a one-off evidence
  note.
- Return to evidence when a missing detail is really about replay surfaces,
  artifact capture, or validation rather than script obligations.
- Return to this phase after `speckit` generates artifacts so the same
  checklist can catch weakened script-review requirements.
