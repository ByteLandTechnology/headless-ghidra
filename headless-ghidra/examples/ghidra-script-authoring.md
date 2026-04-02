# Ghidra Script Authoring

This guide defines how to add or change reusable headless Ghidra scripts for
this repository without introducing hidden state, GUI assumptions, or
non-replayable outputs.

## Validation Status

The workflow in this guide is intended for headless Ghidra execution. As of
2026-03-29, the sample validation record includes a maintainer-run local replay
against a maintainer-local Ghidra 12.0.4 install root. Runtime-specific
behavior is captured in
`./artifacts/sample-target/latest-version-validation.md`.

Operational constraint:

- run actions for the same `target-id` sequentially
- parallel headless runs against the same project can fail with a Ghidra
  project lock before the script logic starts

## Support Boundary

Before writing or revising a script, classify it as one of:

- **Supported reusable script**:
  clear purpose, replayable inputs, reviewable outputs or mutation notes,
  registration surfaces, and evidence
- **Sample-specific helper**:
  useful local helper that still depends on narrow sample scope, hardcoded
  assumptions, or missing review surfaces
- **Unsupported candidate**:
  not ready for either reusable support or even sample-specific review

Do not present a nearby implementation as supported until the contract and
review evidence say so explicitly.

## Baseline Guardrails

Every supported reusable script must preserve:

- headless-only execution
- evidence-backed claims
- reproducible commands, inputs, and destinations
- reviewable outputs or mutation notes

If a proposed script breaks any of these baselines, downgrade or defer it.

## Reusable Script Categories

| Category                | Typical Purpose                                            | Default Side-Effect Class |
| ----------------------- | ---------------------------------------------------------- | ------------------------- |
| `analysis_export`       | Export reviewable evidence from current analysis state.    | `export_only`             |
| `verification_audit`    | Check whether a reviewable claim still holds.              | `read_only`               |
| `metadata_updating`     | Apply justified names, types, or related metadata changes. | `metadata_updating`       |
| `orchestration_wrapper` | Route explicit inputs and outputs across replay stages.    | `mixed_wrapper`           |

## When To Reuse, Extend, Replace, Or Create

Reuse an existing script when:

- the task matches its current purpose
- the existing inputs already cover the needed selection parameters
- the output can remain in the same artifact family
- the side-effect class does not change

Extend the shared pattern when:

- the task is similar to an existing export or metadata pass
- the new behavior can still be deterministic
- the new output belongs beside the current tracked artifact set

Replace an existing script only when:

- the workflow role stays the same
- the registration surfaces are updated in the same review set
- the validation reference moves with the replacement

Create a new script when:

- the analysis task has a distinct purpose
- the output contract is different enough to deserve a new artifact
- the script changes a different class of program metadata
- reusing an old script would hide a materially different workflow

Downgrade to sample-specific helper when:

- the script only fits one binary, one rename set, or one captured scope
- replay inputs remain implicit
- runtime output depends on the installed skill package path
- mutation reasoning is not reviewable

## Placement Rules

- Reusable headless Ghidra scripts belong under
  `<installed-skill-root>/ghidra-scripts/`.
- Headless wrapper or orchestration scripts belong under
  `<installed-skill-root>/scripts/`.
- Guidance and examples belong under
  `<installed-skill-root>/examples/`.
- Runtime-generated outputs belong under `.work/ghidra-artifacts/<target-id>/`.
- Runtime-generated helper scripts belong under a workspace path such as
  `.work/ghidra-artifacts/<target-id>/generated-scripts/`.
- Files under `<installed-skill-root>/examples/artifacts/`
  are tracked sample surfaces and must not become the default runtime output
  path.

## Naming Rules

Use script names that describe intent, not a temporary experiment.

Good patterns:

- `ExportAnalysisArtifacts.java`
- `ApplyRenames.java`
- `VerifyRenames.java`

Avoid:

- `test.py`
- `tmp_export.py`
- `new_script_final2.py`

## Script Contract

Every reusable script should document:

- purpose
- expected inputs
- output files it may write
- side-effect class
- baseline guardrails
- deterministic replay expectations
- how the headless runner passes parameters

Suggested header fields:

```text
Purpose:
Inputs:
Outputs:
Side Effects: read-only | export-only | metadata-updating | mixed-wrapper
Baselines: headless-only | evidence-backed | reproducible | reviewable outputs
Replay Notes:
```

For planned manifest-driven additions, prefer a fixed input or output filename
pair that makes the contract obvious from tracked docs alone. Current planned
candidate surfaces include:

- `ExportCallGraph.java` -> `call-graph-detail.md`
- `ReviewEvidenceCandidates.java` -> `evidence-candidates.md`
- `PlanTargetSelection.java` -> `target-selection.md`
- `ApplyFunctionSignatures.java` -> `signature-log.md` plus
  `signature-apply-report.md`
- `VerifyFunctionSignatures.java` -> `signature-log.md` plus
  `signature-verification-report.md`
- `LintReviewArtifacts.java` -> `artifact-lint-report.md`

## Side-Effect Classes

### Read-Only

- Inspects `currentProgram` and related data only.
- Does not write tracked files directly unless the wrapper captures output.
- Does not modify names, types, or prototypes.

### Export-Only

- Reads program state and writes deterministic tracked artifacts.
- Does not mutate analysis metadata inside the project.

### Metadata-Updating

- May rename functions, refine prototypes, update types, or add structure to
  the analysis state.
- Must log the reason for each meaningful update in tracked notes so replay
  remains reviewable.
- For the supported rename workflow in this repository, the reviewable input is
  a Markdown `renaming-log.md` manifest and the runtime outputs are apply and
  verification reports under `.work/ghidra-artifacts/<target-id>/`.
- For planned signature replay, keep the schema conservative and text-based:
  function address, expected current name, optional expected current signature,
  optional new function name, return type, parameter list, calling convention,
  evidence fields, and status.
- Do not imply that a full C prototype parser is required before the manifest
  becomes reusable. A conservative schema that can be applied and verified
  honestly is preferable.
- Supported rename rows must declare an item kind that matches the intended
  target class. The current active Java schema accepts `function`, `symbol`,
  and `label`. The latest recorded local replay explicitly validated
  executable `function` and `symbol` rows; keep the validation artifact in sync
  if label-specific replay claims are added later.

### Mixed Wrapper

- Coordinates replay steps, paths, and helper invocation.
- May create disposable project or artifact directories in `.work/`.
- Must still keep outputs explicit and reviewable.

Archive-specific expectation for this repository:

- `normalize-ar-archive.sh` is a supported `orchestration_wrapper`.
- It must emit `archive-intake-record.md`,
  `archive-member-inventory.md`, `archive-normalization-handoff.md`, and
  `archive-replay-command-record.md` for every reviewed normalization run.
- It must hand off only accepted extracted member paths, never the raw archive
  itself, to later Ghidra stages.
- It must keep live outputs under
  `.work/ghidra-artifacts/<archive-id>/`.
- It must reject unsupported `--selection-policy` values rather than silently
  widening scope.

Prefer the weakest side-effect class that solves the task.

## Inputs and Parameters

Do not bury configuration in local machine state.

Pass or derive explicitly:

- target identifier
- artifact root
- archive path and archive identifier when the input is an archive
- member output root and review output root for archive normalization
- selected function or address scope
- optional category filters
- optional output file names

The headless runner should be able to show, from a tracked command alone, which
binary and which parameters a script used.

When documenting `artifact-root`, use a workspace path under
`.work/ghidra-artifacts/` rather than any path inside `.agents/skills/`.

## Interacting With Program Data

Reusable scripts may inspect:

- functions and entrypoints
- symbols and namespaces
- strings and constants
- imports, exports, and external libraries
- types, structs, enums, and pointers
- references, xrefs, and call relationships
- decompiler results, if the workflow has validated that usage locally

Rules:

- Do not assume every program exposes every category.
- Write defensive logic for stripped binaries and partial recovery.
- When a script depends on a category that may be absent, make the absence
  explicit in the output.

## Output Rules

Outputs should be:

- deterministic
- repository-relative by contract
- reviewable as Markdown or other tracked text
- stable enough to diff across reruns

Each output should answer:

- what was inspected
- what was found
- what remains ambiguous
- which follow-up step the analyst should take next

## Registration Into The Workflow

Before treating a script as supported:

1. Place it in `ghidra-scripts/` or `scripts/` with a durable name.
2. Define its purpose, inputs, outputs, side-effect class, and baseline
   guardrails.
3. Wire it into the invocation guidance, review guidance, and evidence
   guidance.
4. Add or update the affected artifact path.
5. Review it with the checklist in
   `./ghidra-script-review-checklist.md`.
6. Re-check its normative instructions against the newest
   repository-supported Ghidra version before finalizing them.

## No Fabricated Runtime Claims

Do not write phrases like:

- "this flag definitely works on all installs"
- "the decompiler always returns usable output"
- "the help text is as follows"

Instead, write:

- "obtain help from the discovered local `analyzeHeadless` binary and record the
  exact command in the validation artifact"
- "if the local run differs, update the validation record and runner contract"

## Minimal Review Questions

- Is this task already covered by an existing script?
- Does the script expose all inputs needed for replay?
- Are its outputs deterministic and reviewable?
- Does it avoid GUI-only assumptions?
- Is the side-effect class clearly declared?
- Does it stay out of `.agents/skills/` for runtime-generated content?
- Is latest-version validation still accurate for the normative claims it makes?
