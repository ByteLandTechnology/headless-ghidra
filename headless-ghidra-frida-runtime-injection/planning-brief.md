phase_id: frida_runtime_injection
artifact_purpose: Define the portable runtime-capture and handoff contract used to shape speckit planning and audit generated planning artifacts.
primary_consumer: downstream_user
constrained_outputs:

- spec.md
- plan.md
- tasks.md
  audit_scope:
- spec.md
- plan.md
- tasks.md
  example_path: .agents/skills/headless-ghidra-frida-runtime-injection/examples/frida-runtime-speckit-handoff.md
  script_library_path: .agents/skills/headless-ghidra-frida-runtime-injection/frida-scripts/

# When To Use This Phase Skill

Use this brief when planning must support reproducible CLI/headless Frida
runtime capture, reusable script selection, and a reviewable capture-manifest
handoff into the Frida evidence-import phase.

# Non-Negotiable Runtime Capture Constraints

- Capture remains reproducible through CLI/headless invocation.
- GUI-driven capture is out of scope.
- Open-ended interactive exploration is out of scope.
- Runtime artifacts stay under `.work/ghidra-artifacts/`.
- Runtime outputs remain reviewable in Markdown through explicit manifests.
- Reusable library changes route to
  [`../headless-ghidra-script-review/SKILL.md`](../headless-ghidra-script-review/SKILL.md).
- Successful runtime capture must hand off into
  [`../headless-ghidra-frida-evidence/SKILL.md`](../headless-ghidra-frida-evidence/SKILL.md).
- No downstream extension or constitution change is required.

# Required Planning Inputs

- `target_context`: normalized target identity and scope
- `requested_scenario`: one of the five supported runtime evidence scenarios
- `selected_script_ids`: reusable common Frida scripts chosen for capture
- `capture_commands`: reproducible CLI/headless command forms
- `artifact_root`: runtime artifact root under `.work/ghidra-artifacts/`
- `audit_gates`: checks that must pass before evidence import can begin
- `local_rule_overlay`: optional stricter local rules

# Common Script Selection Rules

- Start in [`./frida-scripts/manifest.md`](./frida-scripts/manifest.md) to map
  the request to the supported scenario.
- Confirm invocation shape, expected outputs, and coverage notes in
  [`./frida-scripts/README.md`](./frida-scripts/README.md).
- Reuse the tracked library before proposing any new helper.
- If no reusable script covers the request, or if a script's behavior or
  outputs need to change, stop and escalate to script review.

# Planning Brief Body

Use this body directly or adapt it with the same meaning:

```md
Prepare planning artifacts for bounded Frida runtime capture inside the
headless-only Ghidra workflow.

Target context:

- [fill in normalized target]
- [fill in agreed capture scope]

Requested runtime evidence scenario:

- [fill in one of: signature analysis, decompilation comparison, call-tree
  tracing, dynamic dispatch or vtable observation, hot-path or coverage
  observation]

Selected reusable common Frida scripts:

- [fill in script id]
- [fill in script id if multiple scripts are required]

Capture commands:

- [fill in reproducible CLI/headless command shape]

Artifact root:

- `.work/ghidra-artifacts/<target-id>/`

Produced artifacts:

- [fill in logs, traces, summaries, or manifest-linked outputs]

Audit gates:

- [fill in what must pass before evidence import]

Non-negotiable constraints:

- CLI/headless capture only.
- No GUI-driven capture or open-ended interactive exploration.
- Runtime artifacts stay under `.work/ghidra-artifacts/`.
- Reviewable Markdown outputs for `spec.md`, `plan.md`, and `tasks.md`.
- Reusable script-library changes route to script review.
- No downstream speckit extension or constitution change required.
```

# How To Supply The Brief To Speckit

- Supply [`./planning-brief.md`](./planning-brief.md) directly.
- Or paste the `Planning Brief Body` inline into the `speckit` request.
- The transport mode must not weaken script selection, capture-manifest, or
  evidence-handoff obligations.

# Audit Checklist For Generated Artifacts

- `spec.md` identifies the runtime scenario, selected reusable script surface,
  and capture-manifest expectations.
- `plan.md` keeps CLI/headless capture, runtime artifact boundaries, and
  evidence-import handoff explicit.
- `tasks.md` includes script selection, manifest generation, validation, and
  blocking-path work.
- Runtime artifacts and capture manifests remain explicit and replayable in the
  generated workflow.
- Generated artifacts preserve the route to evidence import rather than
  implying runtime capture is the last supported phase.
- Any coverage gap or behavior change is routed to script review.

# Local Rule Policy

- Local repository rules may add stricter script metadata, capture-manifest
  fields, or audit gates.
- Local repository rules may not introduce GUI-only capture, hidden runtime
  outputs, or undocumented script-selection behavior.
- Treat stricter local rules as additive overlays.
- Treat any local rule that weakens this contract as invalid.
