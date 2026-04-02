# Headless Ghidra Skill Family

This repository is the source for a standalone skill family for planning and
auditing headless Ghidra reverse-engineering work. It is built for workflows
that need explicit phase boundaries, reproducible handoffs, and reviewable
Markdown outputs instead of ad hoc analyst notes.

Use this family when you want to:

- route a reverse-engineering task to the right planning phase
- hand a phase contract into `speckit` without downstream-only hooks
- audit generated `spec.md`, `plan.md`, and `tasks.md` against the same rules
- keep dynamic evidence, script changes, and late-stage decompilation work
  inside clear review boundaries

## What This Skill Family Provides

- One umbrella entrypoint in [`headless-ghidra/`](./headless-ghidra/) that
  explains routing across the full workflow.
- Phase-specific skills for intake, evidence, Frida runtime capture, Frida
  evidence import, progressive decompilation, and script review.
- A canonical `planning-brief.md` in each planning phase directory. That file
  is both the portable handoff surface and the audit checklist for generated
  planning artifacts.
- Worked examples and templates in each skill directory so the contract stays
  concrete instead of implicit.
- A reusable Frida common script library in
  [`headless-ghidra-frida-runtime-injection/frida-scripts/`](./headless-ghidra-frida-runtime-injection/frida-scripts/)
  covering five supported runtime evidence scenarios.
- An explicit auto-evolution skill for turning completed real-task learnings
  into tracked reusable assets when the evidence is strong enough.

## Core Guarantees

Every skill in this family keeps the same baseline:

- Headless-only workflows. GUI-only Ghidra guidance stays out of scope.
- Evidence-backed claims. Planning and review must point back to observable
  inputs, artifacts, or recorded findings.
- Reproducible execution. Commands, manifests, and review expectations stay
  explicit.
- Markdown-first outputs. The important contract surfaces remain readable and
  auditable in version control.
- Portable contracts. The family is designed to work without requiring
  downstream `speckit` extensions or custom constitution edits.

Runtime artifacts remain outside the tracked skill package. When Frida runtime
capture is in scope, produced outputs stay under `.work/ghidra-artifacts/` and
are referenced from manifests rather than copied into skill directories.

## Skill Map

| Skill | Use it when | Key surfaces |
| --- | --- | --- |
| [`headless-ghidra`](./headless-ghidra/SKILL.md) | You need the family entrypoint, routing help, or the shared collaboration sequence. | `SKILL.md`, `examples/`, `templates/`, `ghidra-scripts/`, `scripts/` |
| [`headless-ghidra-intake`](./headless-ghidra-intake/SKILL.md) | You are defining target identity, provenance, scope, deliverables, and initialization assumptions. | `planning-brief.md`, `examples/` |
| [`headless-ghidra-evidence`](./headless-ghidra-evidence/SKILL.md) | You need evidence extraction, replay expectations, and audit surfaces after intake is stable. | `planning-brief.md`, `examples/` |
| [`headless-ghidra-frida-runtime-injection`](./headless-ghidra-frida-runtime-injection/SKILL.md) | You need reproducible CLI/headless Frida capture planning and reusable script selection. | `planning-brief.md`, `templates/frida-capture-manifest.md`, `frida-scripts/`, `examples/` |
| [`headless-ghidra-frida-evidence`](./headless-ghidra-frida-evidence/SKILL.md) | You already have captured Frida outputs and need import-only evidence normalization and review. | `planning-brief.md`, `templates/frida-evidence-manifest.md`, `examples/` |
| [`headless-ghidra-progressive-decompilation`](./headless-ghidra-progressive-decompilation/SKILL.md) | You are at Stage 6 and need the standalone contract for selected decompilation and incremental compare. | `planning-brief.md`, `examples/` |
| [`headless-ghidra-script-review`](./headless-ghidra-script-review/SKILL.md) | The plan introduces reusable scripts, manifest-generation logic, or Frida helper coverage changes. | `planning-brief.md`, `examples/` |
| [`headless-ghidra-auto-evolution`](./headless-ghidra-auto-evolution/SKILL.md) | A completed real task exposed a reusable improvement that should be accepted, deferred, or rejected explicitly. | `SKILL.md`, `templates/auto-evolution-review-record.md`, `examples/` |

## Frida Runtime Support

The Frida runtime branch of the family is intentionally split into two phases:

- [`headless-ghidra-frida-runtime-injection`](./headless-ghidra-frida-runtime-injection/SKILL.md)
  owns capture planning, reusable script selection, and capture-manifest
  generation.
- [`headless-ghidra-frida-evidence`](./headless-ghidra-frida-evidence/SKILL.md)
  owns import-only review of captured dynamic evidence.

The shipped common Frida script library supports five first-class scenarios:

- function signature analysis
- decompilation-to-original comparison
- runtime call-tree tracing
- dynamic dispatch or vtable observation
- hot-path or coverage observation

If a request falls outside the documented coverage or changes a reusable
script's behavior or output shape, route the follow-up into
[`headless-ghidra-script-review`](./headless-ghidra-script-review/SKILL.md)
instead of extending the runtime phase ad hoc.

## Typical Usage

1. Start with [`headless-ghidra/SKILL.md`](./headless-ghidra/SKILL.md) if the
   correct phase is not obvious.
2. Move into the phase-specific skill directory that matches the current stage
   of work.
3. Fill in that phase's `planning-brief.md`.
4. Hand the brief to `speckit` as a file or inline paste.
5. Re-open the same skill and audit the generated `spec.md`, `plan.md`, and
   `tasks.md` against the same phase contract.
6. Use the local `examples/` and `templates/` to keep manifests, review notes,
   and audit findings in the expected shape.
7. After a real task finishes, use
   [`headless-ghidra-auto-evolution`](./headless-ghidra-auto-evolution/SKILL.md)
   when a reusable improvement should be reviewed for promotion into the
   tracked workflow.

## Common Routes

- New target or unclear scope:
  [`headless-ghidra-intake`](./headless-ghidra-intake/SKILL.md)
- Stable scope and need replayable evidence planning:
  [`headless-ghidra-evidence`](./headless-ghidra-evidence/SKILL.md)
- Need bounded dynamic capture before evidence import:
  [`headless-ghidra-frida-runtime-injection`](./headless-ghidra-frida-runtime-injection/SKILL.md)
- Already have Frida outputs and a capture manifest:
  [`headless-ghidra-frida-evidence`](./headless-ghidra-frida-evidence/SKILL.md)
- Need standalone Stage 6 decompilation and compare planning:
  [`headless-ghidra-progressive-decompilation`](./headless-ghidra-progressive-decompilation/SKILL.md)
- Need reusable helper or script-governance review:
  [`headless-ghidra-script-review`](./headless-ghidra-script-review/SKILL.md)
- Want to promote a completed-task learning into a tracked asset:
  [`headless-ghidra-auto-evolution`](./headless-ghidra-auto-evolution/SKILL.md)

## Repository Layout

- [`headless-ghidra/`](./headless-ghidra/) is the umbrella skill and shared
  support surface. It includes routing guidance, walkthroughs, example
  artifacts, reusable headless Ghidra scripts, and helper shell scripts.
- Each phase directory contains the phase skill, a fixed-name
  `planning-brief.md`, and phase-specific examples or templates.
- The Frida runtime phase also includes the tracked common script library and
  its manifest.
- The auto-evolution phase includes the review template used to decide whether
  a reusable improvement becomes a tracked asset.

If you are opening the repository for the first time, start in
[`headless-ghidra/SKILL.md`](./headless-ghidra/SKILL.md), then follow the phase
links above.
