phase_id: script_authoring_review
artifact_purpose: Define the portable script authoring and review contract used to shape speckit planning and review generated planning artifacts.
primary_consumer: downstream_user
constrained_outputs:

- spec.md
- plan.md
- tasks.md
  audit_scope:
- spec.md
- plan.md
- tasks.md
  example_path: .agents/skills/headless-ghidra-script-review/examples/script-authoring-review-audit.md

# When To Use This Phase Skill

Use this brief when planning includes reusable headless Ghidra scripts,
registration expectations, or checklist-driven review requirements that must be
preserved through `speckit`, including reusable Frida capture helpers,
common-library changes, or manifest-generation scripts escalated from the
runtime and evidence phases.

# Non-Negotiable Reverse-Engineering Constraints

- Headless-only scope. Script work must stay compatible with headless analysis
  and replay workflows.
- Evidence-backed claims. Script expectations and review findings must be tied
  to tracked examples, observable outputs, or documented workflow evidence.
- Reproducible workflow expectations. Script commands, parameters, naming, and
  registration steps must be replayable.
- Reviewable Markdown outputs. `spec.md`, `plan.md`, `tasks.md`, and review
  findings must remain inspectable in Markdown.
- Reusable Frida helper, coverage-change, and manifest-generation work must be
  made explicit here rather than hidden inside runtime-injection or
  evidence-import notes.
- No downstream extension or constitution change is required. The contract is
  portable across repositories and only allows stricter overlays.

# Required Planning Inputs

- `script_scope`: what the reusable script must do and where it fits in the
  workflow
- `frida_governance_scope`: whether the work adds script coverage, changes
  output behavior, or introduces reusable capture or manifest helpers
- `deterministic_expectations`: required inputs, outputs, and replay behavior
- `registration_requirements`: repository-relative location, naming, and review
  expectations
- `manifest_obligations`: which capture or evidence manifests the reusable
  script must create, update, or keep aligned
- `non_negotiable_constraints`: headless-only, evidence-backed,
  reproducibility, and Markdown reviewability requirements
- `validation_expectations`: how a reviewer confirms script obligations
  survived planning
- `local_rule_overlay`: optional stricter local rules

# Planning Brief Body

Use this body directly or adapt it with the same meaning:

```md
Prepare planning artifacts for reusable headless Ghidra script authoring and
review.

Script scope:

- [fill in the script objective]
- [fill in where the script fits in the workflow]

Frida governance scope:

- [fill in whether this is new coverage, changed behavior, or a reusable
  helper]
- [fill in which phase escalated the work here]

Deterministic expectations:

- [fill in required inputs]
- [fill in expected outputs]
- [fill in replay or parameter expectations]

Registration requirements:

- repository-relative script location
- review checklist requirement
- naming or manifest expectations

Manifest obligations:

- [fill in capture manifest updates or generation rules]
- [fill in evidence manifest updates or generation rules]

Non-negotiable constraints:

- Headless-only workflow. No GUI dependency.
- Evidence-backed claims only.
- Reproducible script authoring and review expectations.
- Reviewable Markdown outputs for spec.md, plan.md, tasks.md, and findings.
- No downstream speckit extension or constitution change required.

Validation expectations:

- A reviewer can identify script scope, deterministic behavior, and review
  requirements in one pass.
- A reviewer can tell whether runtime-injection or evidence-import was correct
  to escalate the work here.
- A reviewer can confirm the script's output and manifest obligations still
  align with the Frida runtime and evidence contracts.
- Generated artifacts preserve those obligations without weakening them.
```

# How To Supply The Brief To Speckit

- Supply [`./planning-brief.md`](./planning-brief.md) directly.
- Or paste the `Planning Brief Body` inline into the `speckit` request.
- The transport mode must not weaken script-authoring or review obligations.

# Audit Checklist For Generated Artifacts

- `spec.md` identifies the script objective, deterministic expectations, and
  review surface.
- `plan.md` preserves repository-relative script placement, registration, and
  replayable execution expectations.
- `plan.md` identifies whether the work is a Frida library-coverage change, a
  behavior change, or a reusable manifest-generation helper.
- `tasks.md` includes checklist-based authoring and review work rather than
  implying ad hoc or hidden review.
- `tasks.md` keeps capture-helper and manifest-generation follow-up explicit
  when runtime or evidence phases escalate them here.
- Generated artifacts keep headless-only, evidence-backed, reproducible, and
  Markdown-reviewable expectations explicit.
- If a generated artifact weakens this contract, refine or regenerate the
  planning artifacts instead of weakening the contract.

# Local Rule Policy

- Local repository rules may add stricter script registration, naming, or
  review requirements.
- Local repository rules may add stricter Frida script metadata, coverage-note,
  or manifest-generation requirements.
- Local repository rules may not allow non-deterministic script behavior, GUI
  dependency, or hidden non-Markdown review surfaces.
- Treat stricter local rules as additive overlays.
- Treat any local rule that weakens this contract as invalid.
