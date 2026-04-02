phase_id: frida_dynamic_evidence_import
artifact_purpose: Define the portable Frida evidence import and audit contract used to shape speckit planning and review generated planning artifacts.
primary_consumer: downstream_user
constrained_outputs:

- spec.md
- plan.md
- tasks.md
  audit_scope:
- spec.md
- plan.md
- tasks.md
  example_path: .agents/skills/headless-ghidra-frida-evidence/examples/frida-trace-handoff.md
  input_manifest_type: frida_runtime_capture_manifest

# When To Use This Phase Skill

Use this brief when intake is already stable and planning must incorporate
externally captured Frida evidence that already has a linked runtime-capture
manifest, without turning live instrumentation into a supported workflow step.

# Non-Negotiable Evidence Constraints

- Import-only scope. The contract covers imported Frida evidence and audit
  surfaces, not live capture or hook execution.
- Evidence remains tied to a tracked runtime-capture manifest.
- Headless-only scope. Planning outputs must remain compatible with headless
  Ghidra workflows and review.
- Evidence-backed claims. Generated artifacts must tie Frida-derived
  observations to tracked evidence, provenance notes, or manifest fields.
- Observed claims remain distinct from inferred claims.
- Dynamic-vs-static conflicts preserve both evidence sides until a reviewer
  adjudicates them explicitly.
- Reproducible workflow expectations. The generated plan must preserve enough
  provenance and replay detail for a reviewer to verify what was captured.
- Reviewable Markdown outputs. `spec.md`, `plan.md`, and `tasks.md` must keep
  Frida evidence expectations visible in Markdown.
- Runtime-generated analysis artifacts remain under
  `.work/ghidra-artifacts/` and must be referenced explicitly.
- No downstream extension or constitution change is required. The contract
  remains portable and only allows stricter local overlays.

# Required Planning Inputs

- `target_context`: normalized target identity and scope
- `linked_capture_manifest`: the runtime-capture manifest consumed by this
  phase
- `frida_evidence_bundle`: the imported trace, log, hook, or observation set
- `provenance_surface`: what links the evidence back to a target and capture
  moment
- `observed_claims`: facts directly supported by the imported evidence
- `inferred_claims`: interpretation layered on top of the imported evidence
- `static_evidence_context`: static analysis that may agree or disagree with
  the runtime observations
- `replay_surface`: the review notes or manifest expectations needed for
  verification
- `non_negotiable_constraints`: import-only, headless-only, evidence-backed,
  reproducible, and Markdown-reviewable requirements
- `validation_expectations`: how reviewers confirm the generated artifacts kept
  the Frida evidence contract intact
- `local_rule_overlay`: optional stricter local rules

# Conflict Adjudication Rules

- Record the `target_subject` for each disagreement between runtime and static
  analysis.
- Preserve both `static_evidence` and `dynamic_evidence` in reviewable
  Markdown.
- Record an explicit `reviewer_decision` and `decision_rationale`.
- Use `defer` when the evidence is still inconclusive rather than silently
  choosing a side.

# Planning Brief Body

Use this body directly or adapt it with the same meaning:

```md
Prepare planning artifacts for a headless-only Ghidra workflow that imports
Frida-derived evidence as a reviewable external input.

Target context:

- [fill in normalized target]
- [fill in agreed scope]

Imported Frida evidence bundle:

- [fill in trace, log, or observation summary]
- [fill in hook or capture profile summary]

Linked runtime-capture manifest:

- [fill in manifest path or identifier]
- [fill in produced artifact references]

Provenance surface:

- [fill in how the evidence maps to the target]
- [fill in timing, source, or verification notes]

Observed claims:

- [fill in direct runtime observations]

Inferred claims:

- [fill in interpretation layered on top of the observations]

Static evidence context:

- [fill in any static analysis that agrees or disagrees]

Replay surface:

- [fill in manifest or review notes required for verification]

Non-negotiable constraints:

- Import-only workflow. No live Frida execution in scope.
- Headless-only Ghidra planning and review.
- Evidence-backed claims only.
- Reproducible provenance and replay expectations.
- Reviewable Markdown outputs for spec.md, plan.md, and tasks.md.
- Runtime-generated artifacts stay under `.work/ghidra-artifacts/`.
- No downstream speckit extension or constitution change required.

Validation expectations:

- A reviewer can identify the imported evidence bundle, linked runtime-capture
  manifest, and provenance in one pass.
- Generated artifacts preserve those requirements without weakening them.
- Generated artifacts preserve both sides of dynamic-vs-static conflicts.
- Generated artifacts make gaps or unresolved ambiguity explicit.
```

# How To Supply The Brief To Speckit

- Supply [`./planning-brief.md`](./planning-brief.md) directly.
- Or paste the `Planning Brief Body` inline into the `speckit` request.
- The transport mode must not weaken import-only, manifest-linkage,
  provenance, or replay obligations.

# Audit Checklist For Generated Artifacts

- `spec.md` identifies the Frida evidence bundle, linked runtime-capture
  manifest, provenance surface, and why they matter.
- `plan.md` preserves replayable verification notes, manifest expectations, and
  explicit conflict adjudication rather than implying live capture steps.
- `tasks.md` includes work needed to validate imported evidence, review gaps,
  and preserve dynamic-vs-static conflicts in Markdown-visible form.
- Observed and inferred claims remain separated.
- Generated artifacts never silently default to dynamic or static evidence.
- None of the generated artifacts weaken import-only, headless-only,
  evidence-backed, or reproducibility expectations.
- If the Frida evidence contract is weakened, refine or regenerate the planning
  artifacts instead of weakening the contract.

# Local Rule Policy

- Local repository rules may require extra provenance notes, stricter manifest
  fields, or tighter conflict-recording requirements.
- Local repository rules may not introduce live Frida execution, GUI
  dependency, hidden non-Markdown review surfaces, or default bias in conflict
  handling.
- Treat stricter local rules as additive overlays.
- Treat any local rule that weakens this contract as invalid.
