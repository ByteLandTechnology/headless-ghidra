# Ghidra Script Review Checklist

Use this checklist before adding or changing a reusable headless Ghidra script.
If any required item is not satisfied, do not treat the script as part of the
supported workflow yet.

## Validation Context

- [ ] The newest repository-supported Ghidra version is identified from
      `./artifacts/sample-target/latest-version-validation.md`.
- [ ] Any normative runtime claim made by this script or its wrapper has been
      checked against that recorded version.
- [ ] If local validation was unavailable, the documentation says so plainly and
      does not imply a successful run.

## Classification Gate

- [ ] The script is classified as either supported reusable, sample-specific
      helper, or unsupported candidate.
- [ ] The classification decision is recorded in a review or evidence artifact.
- [ ] A nearby local helper is not being presented as supported only because it
      exists in the worktree.

## Baseline Guardrails

- [ ] The workflow remains headless-only.
- [ ] The support claim is evidence-backed.
- [ ] Commands, inputs, and runtime destinations are reproducible.
- [ ] Outputs or mutation notes remain reviewable.

## Purpose and Placement

- [ ] The script has one clear purpose.
- [ ] The script belongs in the correct tracked package path.
- [ ] The filename describes durable intent rather than a temporary experiment.
- [ ] The script is not duplicating an existing supported script without a
      strong reason.

## Inputs and Invocation

- [ ] Required inputs are explicit.
- [ ] The headless invocation path is documented.
- [ ] The expected binary, target identifier, and artifact root are derivable
      from tracked commands.
- [ ] Archive-oriented wrappers expose archive path, archive id, artifact root,
      and any member or review output roots explicitly.
- [ ] The script does not require GUI setup or hidden local state.

## Outputs And Runtime Boundary

- [ ] Every tracked output path is explicit.
- [ ] Outputs are deterministic enough to diff across reruns.
- [ ] Empty or unavailable result sets are reported honestly.
- [ ] Output content is useful to a reviewer without opening the GUI.
- [ ] Runtime-generated content does not default into `.agents/skills/`.
- [ ] Sample paths under `examples/artifacts/` are not being reused as live
      output destinations.
- [ ] Archive-oriented wrappers emit reviewable intake, inventory, handoff, and
      replay surfaces rather than hiding stop or failure details in shell logs
      only.

## Side Effects

- [ ] The script declares whether it is read-only, export-only,
      metadata-updating, or mixed-wrapper.
- [ ] If it updates names, prototypes, types, or related metadata, the replay
      notes explain what changes and why.
- [ ] Meaningful mutations are justified in reviewable notes rather than hidden
      in local state.
- [ ] Any supported rename automation consumes a reviewable `renaming-log.md`
      manifest and writes reports under `.work/ghidra-artifacts/`.
- [ ] Any supported signature automation consumes a reviewable
      `signature-log.md` manifest and writes
      `signature-apply-report.md` or
      `signature-verification-report.md` under `.work/ghidra-artifacts/`.
- [ ] Any supported evidence-review or selection export keeps fixed
      output filenames such as `evidence-candidates.md` and
      `target-selection.md` rather than inventing one-off sample names.
- [ ] Any supported call-graph detail export keeps a fixed
      `call-graph-detail.md` output and remains export-only.
- [ ] Any supported lint surface writes a reviewable `artifact-lint-report.md`
      without treating tracked sample docs as runtime outputs.
- [ ] Any supported archive-normalization wrapper records exact wrapper and
      extractor commands, preserves duplicate-member visibility, and hands off
      only accepted extracted member paths.
- [ ] The rename-manifest schema matches the currently supported Java scripts.
      The current validated schema accepts `function`, `symbol`, and `label`,
      with executable `function` and `symbol` rows explicitly replayed in the
      latest local run. Do not advertise any wider item-kind set unless runtime
      support and the validation record both move with it.
- [ ] The script uses the weakest side-effect class that still solves the task.

## Program Data Access

- [ ] The script clearly states which program objects it reads.
- [ ] It handles stripped or partially recovered binaries without pretending the
      missing data exists.
- [ ] It does not overstate confidence for decompiler or type-recovery output.

## Workflow Registration

- [ ] The runner or command manifest references the script when invocation is
      required.
- [ ] Matching review guidance exists for the script's support claim.
- [ ] A matching evidence or downgrade note exists for the script's results.
- [ ] The script fits the outside-in analysis flow instead of bypassing it.
- [ ] Archive normalization, when applicable, is registered as the gate before
      Stage 1 baseline evidence begins.
- [ ] Follow-up prompts or next-step guidance are updated if the script changes
      analysis choices.
- [ ] If the script is only planned, the docs say so plainly and do not present
      the sample surface as already registered runtime support.

## Downgrade Triggers

- [ ] Downgrade if the behavior only fits one captured sample or one hardcoded
      address set.
- [ ] Downgrade if replay inputs are not explicit enough for another reviewer.
- [ ] Downgrade if the runtime destination remains inside `.agents/skills/`.
- [ ] Downgrade if mutation reasoning is not reviewable.

## Final Gate

- [ ] The script author can explain why this is reuse, extension, replacement,
      or a justified new script.
- [ ] A reviewer can replay the documented command path from the repository.
- [ ] No part of the script documentation fabricates runtime help, install
      layout, or successful execution that was not actually observed.
