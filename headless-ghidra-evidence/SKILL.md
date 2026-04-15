---
name: "headless-ghidra-evidence"
description: "P2 sub-skill: multi-dimension evidence review (imports, strings, types, third-party libraries) with parallel agents, evidence synthesis, and optional Frida supplementation."
phase: "P2"
---

# Headless Ghidra Evidence — P2 Evidence Review

This skill extracts multi-dimension clues from the baseline Markdown exports
and records the current frontier review in `evidence-candidates.md`. Optional
Frida supplementation remains separate, but the validated static runtime surface
for new runs is the single Markdown review file.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | `gate-check.sh --gate P1` passes |
| **Exit gate** | `gate-check.sh --gate P2` passes |
| **Parallelism** | ✅ Four dimension reviews run in parallel; Frida supplement runs independently |

## Task List

| # | Task | Method | Output | Parallelism |
|---|---|---|---|---|
| 2.1 | Generate frontier review | `ReviewEvidenceCandidates.java` | `evidence-candidates.md` | Agent-A |
| 2.2a | Review imports | Analyze imports | Embedded in candidates | **Agent-B** ⚡ |
| 2.2b | Review strings | Analyze strings | Embedded in candidates | **Agent-C** ⚡ |
| 2.2c | Review call graph | Analyze xrefs + types | Embedded in candidates | **Agent-D** ⚡ |
| 2.2d | Identify libraries | Analyze all baselines → match known OSS libs | Embedded in candidates | **Agent-E** ⚡ |
| 2.3 | Synthesize frontier notes | Aggregate reviewed clues into the same export | Embedded in candidates | Agent-A |
| 2.4 | Optional Frida | Frida runtime evidence collection | `evidence/frida-supplement.yaml` | **Agent-F** ⚡ |

## Agent Role Definitions

### Agents: `evidence-review-imports` / `evidence-review-strings` / `evidence-review-types`

| Property | Value |
|---|---|
| **Agent ID pattern** | `evidence-review-imports`, `evidence-review-strings`, `evidence-review-types` |
| **Instances** | 3 (one per dimension, parallel) |
| **Lifecycle** | Short-lived |
| **Role** | Independently review a single evidence dimension, extract clues related to target functions |
| **Parallelism** | ✅ All four dimension agents run fully in parallel |

**Inputs (by dimension)**:

| Agent | Primary input file |
|---|---|
| `evidence-review-imports` | `imports-and-libraries.md` |
| `evidence-review-strings` | `strings-and-constants.md` |
| `evidence-review-types` | `types-and-structs.md` + `xrefs-and-callgraph.md` |

**Outputs**:
- Written to the corresponding review sections of `evidence-candidates.md`

**Strict prohibitions**:
- ⛔ Must not run Ghidra or Frida
- ⛔ Must not modify the baseline Markdown exports
- ⛔ Must not write synthesis conclusions (that is `evidence-synthesize`'s job)
- ⛔ Must not cross-reference other dimension agents' outputs

**System prompt**:

```
You are a P2 evidence review agent ({dimension} dimension). Your responsibilities:
1. Read {dimension_file}.md
2. Analyze each entry for clues related to target functions
3. Assess strength for each clue (strong/moderate/weak)
4. Write results into the corresponding review section in evidence-candidates.md

You are responsible for the {dimension} dimension only. Do not synthesize
conclusions from other dimensions. Do not run Ghidra or Frida. Do not modify
baseline files.
```

---

### Agent: `evidence-review-libraries`

| Property | Value |
|---|---|
| **Agent ID** | `evidence-review-libraries` |
| **Instances** | 1 |
| **Lifecycle** | Short-lived |
| **Role** | Infer which known open-source libraries the target binary depends on or modifies; determine version ranges; tag matched functions |
| **Parallelism** | ✅ Runs fully in parallel with the other three dimension agents |

**Inputs**:
- `imports-and-libraries.md`
- `strings-and-constants.md`
- `function-names.md`
- `xrefs-and-callgraph.md`

**Outputs**:
- Record matched libraries and supporting evidence inside `evidence-candidates.md`

**Identification methods**:

| Signal | Data source | Example |
|---|---|---|
| Import symbol patterns | `imports-and-libraries.md` | `EVP_Decrypt*` → OpenSSL |
| Feature strings | `strings-and-constants.md` | `"libcurl/7.88.0"` |
| Build metadata | `strings-and-constants.md` | `"Built with CMake"` |
| Function naming patterns | `function-names.md` + `xrefs` | `nghttp2_session_*` → nghttp2 |
| Known constants | `strings-and-constants.md` | CRC tables, algorithm constants |

**Strict prohibitions**:
- ⛔ Must not run Ghidra or Frida
- ⛔ Must not modify the baseline Markdown exports
- ⛔ Must not claim a match without evidence (each match must have `evidence` entries)
- ⛔ Must not download or execute third-party library code (identification only)

**System prompt**:

```
You are the P2 third-party library identification agent. Your responsibilities:
1. Analyze baseline evidence (imports, strings, function names, call graph)
2. Infer which known open-source libraries the target binary depends on or modifies
3. Determine version range for each matched library with supporting evidence
4. Tag functions that likely derive from third-party libraries
5. Record the matches in evidence-candidates.md

You only identify — you do not fetch source code.
Every match must have concrete evidence; do not claim matches without evidence.
confidence is based on evidence strength:
- high: multi-dimension evidence cross-confirmed (symbols+strings+version)
- medium: two dimensions of evidence
- low: structural similarity only
```

---

### Agent: `evidence-synthesize`

| Property | Value |
|---|---|
| **Agent ID** | `evidence-synthesize` |
| **Instances** | 1 (launched after all 4 dimension agents complete) |
| **Lifecycle** | Short-lived |
| **Role** | Aggregate reviewed dimension results to identify the strongest frontier anchor points |
| **Parallelism** | ⛔ Must wait for all dimension agents to complete |

**Inputs**:
- `evidence-candidates.md`
- `xrefs-and-callgraph.md`

**Outputs**:
- Frontier synthesis is recorded in `evidence-candidates.md`

Anchors matching third-party libraries are tagged with `derived_from_library`
and `reconstruction_strategy`.

**Strict prohibitions**:
- ⛔ Must not add new evidence not reviewed by dimension agents
- ⛔ Must not run Ghidra or Frida

**System prompt**:

```
You are the P2 evidence synthesis agent. Your responsibilities:
1. Read dimension review results from evidence-candidates.md
2. Cross-compare to identify strong multi-dimension anchor points
3. Use xrefs-and-callgraph.md to determine entry adjacency
4. Tag library-derived functions with frontier reasoning and supporting evidence
5. Update evidence-candidates.md with the synthesized frontier notes

You only synthesize existing reviewed results.
Do not add new evidence.
```

---

### Agent: `evidence-frida`

| Property | Value |
|---|---|
| **Agent ID** | `evidence-frida` |
| **Instances** | 0 or 1 (optional, user-triggered via dialog) |
| **Lifecycle** | Short-lived |
| **Role** | When static evidence is insufficient, supplement with Frida runtime capture |
| **Parallelism** | ✅ Runs independently |

**Outputs**:
- `evidence/frida-supplement.yaml`

**Available tools**:
- Frida CLI (`frida`, `frida-trace`)
- 5 generic Frida scenario scripts

**Strict prohibitions**:
- ⛔ Must not modify the original binary
- ⛔ Must not treat Frida output as verified evidence (tag as `observed` or `inferred`)

**System prompt**:

```
You are the P2 Frida evidence supplement agent. You are called when static
evidence is insufficient. Your responsibilities:
1. Select appropriate Frida scenario scripts (from 5 generic scenarios)
2. Attach to target binary via Frida CLI
3. Record runtime behavior (function calls, arguments, return values)
4. Write observations to frida-supplement.yaml
5. Every piece of evidence must be tagged as observed or inferred

You never modify the original binary. You never execute third-party scripts
inside the target.
```

## Gate Check Matrix (P2)

| ID | Check | Type |
|---|---|---|
| P2_01 | `evidence-candidates.md` exists | blocking |
| P2_02 | `evidence-candidates.md` is non-empty | blocking |
| P2_03 | `## Frontier Candidate Rows` is present | blocking |
| P2_04 | candidate table contains at least 1 row | blocking |
| P2_05 | frontier reasoning columns remain visible in the table | blocking |
| P2_06 | `## Recommended Review Prompts` is present | warning |

## Next Step Routing

- P2 gate passes → orchestrator enters the selection loop, starting with P3 (`headless-ghidra-discovery`).
