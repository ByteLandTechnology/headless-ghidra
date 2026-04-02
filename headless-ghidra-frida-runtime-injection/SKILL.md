---
name: "headless-ghidra-frida-runtime-injection"
description: "Phase skill for reproducible CLI/headless Frida runtime capture, reusable common script selection, and capture-manifest handoff into evidence import."
phase: "frida_runtime_injection"
---

# Headless Ghidra Frida Runtime Injection

Use this phase skill when planning must support bounded Frida runtime capture
without weakening the repository's headless-only and Markdown-first contract.

The canonical contract is [`./planning-brief.md`](./planning-brief.md). Use it
to shape planning outputs, select reusable common Frida scripts, and audit
generated artifacts for missing capture-manifest or handoff requirements.

## Phase Focus

This phase covers:

- reproducible CLI/headless Frida runtime capture
- selection of tracked reusable common Frida scripts
- capture-manifest generation and runtime artifact references
- explicit audit gates before evidence import can begin

## Supported Runtime Evidence Scenarios

The first-release common script library supports all five first-class runtime
evidence scenarios:

- function signature analysis
- decompilation-to-original comparison
- runtime call-tree tracing
- dynamic dispatch or vtable observation
- hot-path or coverage observation

## Non-Negotiable Constraints

- CLI/headless only. GUI-driven capture is out of scope.
- Open-ended interactive exploration is out of scope.
- Reusable scripts operate only on the configured targets for the approved
  runtime scenario.
- Runtime artifacts stay under `.work/ghidra-artifacts/` and are referenced
  explicitly rather than copied into tracked skill directories.
- Raw runtime values may remain only in those local `.work` artifacts; tracked
  docs and manifests must redact or generalize them.
- Generated planning artifacts must keep selected scripts, capture commands,
  produced artifacts, and audit gates visible in Markdown.
- Reusable script-library coverage gaps, behavior changes, or helper changes
  route to [`../headless-ghidra-script-review/SKILL.md`](../headless-ghidra-script-review/SKILL.md).
- Successful runtime capture hands off to
  [`../headless-ghidra-frida-evidence/SKILL.md`](../headless-ghidra-frida-evidence/SKILL.md).
- No downstream `speckit` extension or constitution change is required.

## Required Inputs

- normalized target identity and scope
- requested runtime evidence scenario
- reusable script selection or coverage-gap decision
- reproducible CLI/headless command shape
- expected runtime artifacts and capture-manifest fields
- audit gates required before evidence import can proceed
- optional stricter local rule overlays

## Runtime Choice UX

When the running skill genuinely needs the user to choose between runtime
scenarios, reusable scripts, coverage-gap routes, or other discrete options:

1. If the runtime exposes a structured choice input tool (for example
   `request_user_input`), use it instead of a plain-text list.
2. Keep each option short, mutually exclusive, and user-facing.
3. Put the recommended or default option first whenever the current manifest or
   scenario evidence clearly favors one, and state that recommendation briefly.
4. Fall back to Markdown or plain-text lists only when no structured choice
   input is available.
5. If one reviewed runtime scenario or reusable script already stands as the
   justified default, do not force a dialog; record the default path and the
   manifest evidence behind it.

## Common Script Selection Workflow

1. Start in [`./frida-scripts/manifest.md`](./frida-scripts/manifest.md) to
   match the request to one of the five supported scenarios.
2. Confirm the invocation shape, expected outputs, and coverage notes for the
   candidate script in [`./frida-scripts/README.md`](./frida-scripts/README.md).
3. Record the selected script identifier or identifiers in the capture
   manifest.
4. If no script covers the request, if capture scope expands beyond configured
   targets, or if behavior/output expectations change, stop and route the
   change through script review.

## Required Outputs

- a capture plan that stays reproducible through CLI/headless invocation
- a runtime capture manifest using
  [`./templates/frida-capture-manifest.md`](./templates/frida-capture-manifest.md)
- explicit artifact references under `.work/ghidra-artifacts/`
- tracked Markdown summaries that redact or generalize raw runtime values
- a handoff to the Frida evidence-import phase with audit-gate results visible

## How To Use This Skill

1. Fill in [`./planning-brief.md`](./planning-brief.md) with the target,
   scenario, selected reusable script, command shape, and audit gates.
2. Confirm the matching script and coverage notes in
   [`./frida-scripts/manifest.md`](./frida-scripts/manifest.md).
3. Use the same brief to audit generated `spec.md`, `plan.md`, and `tasks.md`.
4. If the generated artifacts weaken CLI/headless capture, hide runtime
   outputs, copy raw runtime values into tracked docs, or skip the
   evidence-import handoff, refine or regenerate the planning artifacts
   instead of weakening this contract.

## Examples

- Runtime capture handoff example:
  [`./examples/frida-runtime-speckit-handoff.md`](./examples/frida-runtime-speckit-handoff.md)
- Runtime capture violation example:
  [`./examples/frida-runtime-contract-violation.md`](./examples/frida-runtime-contract-violation.md)

## Next Step Routing

- Use this phase after intake when Frida runtime capture still needs to be
  planned and executed through a reusable, reviewable script path.
- Move to [`../headless-ghidra-frida-evidence/SKILL.md`](../headless-ghidra-frida-evidence/SKILL.md)
  once capture artifacts and the capture manifest are ready for import and
  adjudication.
- Move to [`../headless-ghidra-script-review/SKILL.md`](../headless-ghidra-script-review/SKILL.md)
  when the request needs new reusable script coverage, changed behavior, or a
  reusable helper that alters manifest-generation or normalization behavior.
