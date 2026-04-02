# Common Frida Script Library

This directory holds the tracked reusable Frida runtime-capture scripts for the
`headless-ghidra-frida-runtime-injection` phase.

## Discovery Surface

Use [`./manifest.md`](./manifest.md) first. It maps each first-class runtime
evidence scenario to one reusable script, its invocation shape, expected
outputs, and escalation boundary.

## Supported Scripts

| Script                                                   | Scenario                               | Invocation Shape                                                        | Expected Outputs                                        | Escalate When                                            |
| -------------------------------------------------------- | -------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------- | -------------------------------------------------------- |
| [`signature-analysis.js`](./signature-analysis.js)       | function signature analysis            | `frida -f <target> -l signature-analysis.js --runtime=v8 --no-pause`    | call observations, parameter samples, signature notes   | capture needs type recovery outside the configured hooks |
| [`decomp-compare.js`](./decomp-compare.js)               | decompilation-to-original comparison   | `frida -f <target> -l decomp-compare.js --runtime=v8 --no-pause`        | branch observations, call ordering, return summaries    | comparison needs new helper logic or output fields       |
| [`call-tree-trace.js`](./call-tree-trace.js)             | runtime call-tree tracing              | `frida -f <target> -l call-tree-trace.js --runtime=v8 --no-pause`       | parent-child call edges, depth summaries, edge counters | tracing depth, edge rules, or aggregation logic changes  |
| [`dispatch-vtable-trace.js`](./dispatch-vtable-trace.js) | dynamic dispatch or vtable observation | `frida -f <target> -l dispatch-vtable-trace.js --runtime=v8 --no-pause` | receiver/target mapping, dispatch-site observations     | target resolution or receiver classification changes     |
| [`hotpath-coverage.js`](./hotpath-coverage.js)           | hot-path or coverage observation       | `frida -f <target> -l hotpath-coverage.js --runtime=v8 --no-pause`      | counters, hot-path ranking, branch-hit summaries        | ranking logic or coverage output shape changes           |

## Output Expectations

Every reusable script must produce outputs that can be summarized in a capture
manifest:

- `selected_script_ids`
- `capture_commands`
- `produced_artifacts`
- scenario-specific observation summaries
- coverage notes and unresolved gaps

Runtime artifacts stay under `.work/ghidra-artifacts/<target-id>/`. Do not copy
raw runtime output into tracked skill files.

## Reuse-First Rule

Before proposing any new helper:

1. match the request to a supported scenario in [`./manifest.md`](./manifest.md)
2. confirm the existing script's `coverage_notes`
3. record the selected script identifier in the capture manifest
4. escalate to script review only if the requested behavior or outputs exceed
   documented coverage

## Escalation Boundary

Route the change to
[`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
when:

- no script covers the requested supported scenario
- a script's behavior or expected outputs need to change
- a reusable capture helper is needed
- a script must generate or normalize new manifest fields
