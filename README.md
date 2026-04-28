# Headless Ghidra Skill Family

Headless Ghidra is a Ghidra reverse-engineering skill family for agents. Install
the skill family, then ask your agent to use `headless-ghidra`; the bundled
`ghidra-agent-cli` is invoked by the skills and is not something normal users
install, build, or run manually.

Translations: [简体中文](./README.zh-CN.md) | [日本語](./README.ja-JP.md)

## Install

Recommended in Codex:

```text
$skill-installer install all skills from https://github.com/ByteLandTechnology/headless-ghidra
```

This should install 7 sibling skills: `headless-ghidra`, the 5 P0-P4 phase
skills, and the bundled helper skill `ghidra-agent-cli`. Restart Codex after
installation.

Use the `skills` CLI to install every skill in this skill family to every
supported agent at once:

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --all
```

Here, `--all` is shorthand for `--skill '*' --agent '*' --yes`.

To install every skill for one agent only:

```sh
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent codex --skill '*' --yes
npx --yes skills add https://github.com/ByteLandTechnology/headless-ghidra --agent claude-code --skill '*' --yes
```

## Use

Prerequisites:

- Ghidra is installed locally.
- The target binary is in a workspace path the agent can read.
- Frida is optional and only needed for runtime observation.

Start a new analysis:

```text
Use the headless-ghidra skill to analyze ./sample-target. Start at P0 intake,
choose a stable target id, and stop after each phase gate so I can review the
artifacts.
```

Resume an existing target:

```text
Resume the same target and continue through P1 baseline.
Show me the current pipeline state and the artifacts I should review.
Continue with P2 evidence, but do not classify uncertain third-party code
without showing me the evidence first.
Run the next P3/P4 iteration for the selected hotpath functions.
```

Runtime output belongs in the active workspace under `targets/<target-id>/` and
`artifacts/<target-id>/`, not in the installed skill directory.

## Documentation Map

| If you want to...                                  | Read                                                                                                                                                                                |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Install the skill family and start a run           | This README                                                                                                                                                                         |
| Understand how the entry skill routes work         | [Orchestrator Skill Guide](./headless-ghidra/README.md)                                                                                                                             |
| Run or review a specific phase                     | The matching P0-P4 phase README below                                                                                                                                               |
| Choose walkthroughs, playbooks, or script guidance | [Examples And Guides](./headless-ghidra/examples/README.md)                                                                                                                         |
| Choose what to analyze first                       | [Analysis Selection Playbook](./headless-ghidra/examples/analysis-selection-playbook.md)                                                                                            |
| See a complete analysis narrative                  | [Reverse Engineering Walkthrough](./headless-ghidra/examples/reverse-engineering-walkthrough.md)                                                                                    |
| Author or review custom Ghidra scripts             | [Ghidra Script Authoring](./headless-ghidra/examples/ghidra-script-authoring.md) and [Ghidra Script Review Checklist](./headless-ghidra/examples/ghidra-script-review-checklist.md) |
| Debug agent command syntax or output               | [CLI Tool Reference](./ghidra-agent-cli/README.md)                                                                                                                                  |

## Phases

| Phase | README                                                         | Purpose                                                                     |
| ----- | -------------------------------------------------------------- | --------------------------------------------------------------------------- |
| P0    | [Intake](./headless-ghidra-intake/README.md)                   | Confirm the target, initialize the workspace, and set scope.                |
| P1    | [Baseline](./headless-ghidra-baseline/README.md)               | Import into Ghidra, export baseline artifacts, and record runtime evidence. |
| P2    | [Evidence](./headless-ghidra-evidence/README.md)               | Identify third-party code and evidence sources.                             |
| P3    | [Discovery](./headless-ghidra-discovery/README.md)             | Enrich names, signatures, types, constants, and strings.                    |
| P4    | [Batch Decompile](./headless-ghidra-batch-decompile/README.md) | Apply metadata and decompile selected functions.                            |
