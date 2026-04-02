---
name: "headless-ghidra-frida-evidence"
description: "Phase skill for import-only Frida dynamic evidence normalization, replay expectations, and audit of generated planning artifacts."
phase: "frida_dynamic_evidence_import"
---

# Headless Ghidra Frida Evidence

Use this phase skill when planning must integrate externally captured
Frida-derived dynamic evidence without making live Frida execution part of the
supported `headless-ghidra` workflow.

The canonical contract is [`./planning-brief.md`](./planning-brief.md). Use it
to shape planning outputs and to audit generated artifacts for weakened Frida
evidence requirements.

## Phase Focus

This phase covers:

- imported Frida traces, logs, and hook context
- runtime-capture manifest consumption and evidence-manifest normalization
- provenance, replay, and verification expectations for dynamic observations
- explicit static-vs-dynamic conflict adjudication
- audit expectations for generated planning artifacts

## Non-Negotiable Constraints

- Import-only scope. This skill does not execute Frida, provide live hook
  commands, or depend on GUI-driven capture activity.
- Runtime evidence must link back to a reviewable runtime-capture manifest from
  [`../headless-ghidra-frida-runtime-injection/`](../headless-ghidra-frida-runtime-injection/).
- Headless-only workflow. Imported evidence must remain compatible with
  headless Ghidra planning and review.
- Evidence-backed claims. Dynamic observations must trace to captured artifacts,
  provenance notes, or explicit verification surfaces.
- Observed claims stay distinct from inferred or unresolved claims.
- Conflicts with static analysis preserve both evidence surfaces until a
  reviewer records an explicit adjudication decision.
- Reproducible workflow expectations. Reviewers must be able to see what was
  captured, when it was captured, and how it maps back to the target.
- Reviewable Markdown outputs. Generated planning artifacts and findings remain
  inspectable as Markdown.
- Runtime artifacts stay under `.work/ghidra-artifacts/` and are referenced
  explicitly rather than copied into the skill package.
- No downstream `speckit` extension or constitution change is required.

## Required Inputs

- existing intake summary or normalized target context
- linked runtime-capture manifest and produced artifact references
- externally captured Frida evidence bundle
- provenance details for the capture, including source and timing context
- replay or verification expectations that a reviewer must confirm
- static evidence context when the runtime evidence might disagree with existing
  analysis
- known evidence gaps, ambiguities, or follow-up questions
- optional local overlays that only tighten the contract

## How To Use This Skill

1. Fill in [`./planning-brief.md`](./planning-brief.md) with the imported
   Frida evidence bundle, linked runtime-capture manifest, provenance, and
   replay expectations.
2. Pass that brief into `speckit` as a file or inline paste.
3. Review the generated planning artifacts against the same Frida evidence
   checklist before treating them as ready for implementation.
4. Summarize the imported runtime outputs in
   [`./templates/frida-evidence-manifest.md`](./templates/frida-evidence-manifest.md),
   keeping observed claims, inferred claims, and open questions separate.
5. If Frida-derived evidence conflicts with static analysis, record the
   conflict and an explicit reviewer decision instead of silently choosing one
   side.
6. If the plan introduces reusable normalization, translation, or
   manifest-generation scripts, route the follow-up through script review
   rather than weakening this import-only boundary.

## Examples

- Frida evidence handoff example:
  [`./examples/frida-trace-handoff.md`](./examples/frida-trace-handoff.md)
- Frida request rejection example:
  [`./examples/contract-violation-example.md`](./examples/contract-violation-example.md)
- Frida conflict-preservation example:
  [`./examples/frida-trace-contract-violation.md`](./examples/frida-trace-contract-violation.md)

## Next Step Routing

- Use this phase after intake is stable and the planning request already has
  externally captured Frida-derived evidence plus a runtime-capture manifest.
- Return to the runtime-injection phase when the request still needs capture
  planning, reusable script selection, or runtime audit gates before evidence
  can be trusted.
- Return to generic evidence when the source is no longer Frida-specific or the
  dynamic capture format does not matter to the planning contract.
- Move to script authoring and review when the plan introduces reusable
  normalization, translation, manifest-generation scripts, or common-library
  behavior changes.
- Return to intake if target identity, provenance, or scope is still unstable.
