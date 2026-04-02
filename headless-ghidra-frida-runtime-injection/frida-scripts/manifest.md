# Frida Script Library Manifest

## Purpose

Use this manifest to identify the reusable script that covers a requested
runtime evidence scenario and to decide whether the request stays inside the
shipped library or must escalate to script review.

## Library Inventory

| `script_id`             | File                                                     | Scenario                               | Invocation Shape                                                        | Expected Outputs                                               | Coverage Notes                                                                                    |
| ----------------------- | -------------------------------------------------------- | -------------------------------------- | ----------------------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `signature-analysis`    | [`signature-analysis.js`](./signature-analysis.js)       | function signature analysis            | `frida -f <target> -l signature-analysis.js --runtime=v8 --no-pause`    | signature samples, parameter and return observations           | supports bounded hook sets and parameter/return observation; escalate for new normalization logic |
| `decomp-compare`        | [`decomp-compare.js`](./decomp-compare.js)               | decompilation-to-original comparison   | `frida -f <target> -l decomp-compare.js --runtime=v8 --no-pause`        | branch, call-order, and return observations for compare review | supports targeted functions; escalate for broader behavior-diff aggregation                       |
| `call-tree-trace`       | [`call-tree-trace.js`](./call-tree-trace.js)             | runtime call-tree tracing              | `frida -f <target> -l call-tree-trace.js --runtime=v8 --no-pause`       | call edges, depth, and edge counters                           | supports bounded roots and depth limits; escalate for new graph reduction helpers                 |
| `dispatch-vtable-trace` | [`dispatch-vtable-trace.js`](./dispatch-vtable-trace.js) | dynamic dispatch or vtable observation | `frida -f <target> -l dispatch-vtable-trace.js --runtime=v8 --no-pause` | receiver-to-target observations and dispatch-site notes        | supports configured dispatch sites; escalate for new object-model decoding                        |
| `hotpath-coverage`      | [`hotpath-coverage.js`](./hotpath-coverage.js)           | hot-path or coverage observation       | `frida -f <target> -l hotpath-coverage.js --runtime=v8 --no-pause`      | counters, branch-hit summaries, hot-path ranking               | supports configured counters and ranking output; escalate for new summarization behavior          |

## Selection Rules

- Start from the requested scenario, not from a preferred script file.
- Prefer one tracked reusable script whenever coverage notes say the need is
  already supported.
- Combine multiple tracked scripts only when the capture manifest records each
  selected script identifier explicitly.
- If no inventory row matches, or if the coverage notes do not support the
  requested behavior, stop and escalate.

## Escalation Rules

Escalate to
[`../../headless-ghidra-script-review/SKILL.md`](../../headless-ghidra-script-review/SKILL.md)
when:

- a supported scenario has no matching inventory row
- a matching row exists but the coverage notes exclude the requested behavior
- the request changes output shape, manifest fields, or helper logic
- runtime and evidence phases disagree about how a reusable helper should be
  recorded
