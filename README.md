# Headless Ghidra Skill Family

End-to-end decompilation pipeline skill family. A global orchestrator manages
seven phases (P0–P6), implementing a fully traceable, gate-checked workflow
from target intake through Frida I/O verification.

## Architecture

```
headless-ghidra                       ← global orchestrator
├── headless-ghidra-intake            ← P0 target intake
├── headless-ghidra-baseline          ← P1 baseline extraction
├── headless-ghidra-evidence          ← P2 evidence review (incl. library identification)
├── headless-ghidra-discovery         ← P3 batch discovery
├── headless-ghidra-batch-decompile   ← P4+P5 batch decompilation
└── headless-ghidra-frida-verify      ← P6 Frida I/O verification
```

## Pipeline

```
P0 Intake → P1 Baseline → P2 Evidence → [P3 Discovery → P4+P5 Decompile → P6 Verify]*
```

- **P0–P2**: One-time initialization
- **P3–P6**: Iteration loop, each round processes a batch of frontier functions
- The orchestrator manages global state via `pipeline-state.yaml`
- Each phase transition is validated by `gate-check.sh` programmatic gate checks

## Skill Map

| Skill | Phase | Responsibility | Agent Count |
|---|---|---|---|
| [`headless-ghidra`](./headless-ghidra/) | Orchestrator | Read state, dispatch sub-agents, run gates, show dialogs | 1 |
| [`headless-ghidra-intake`](./headless-ghidra-intake/) | P0 | Target identity, workspace, Ghidra discovery | 2 (parallel) |
| [`headless-ghidra-baseline`](./headless-ghidra-baseline/) | P1 | Ghidra headless baseline export (6 YAMLs) | 1 |
| [`headless-ghidra-evidence`](./headless-ghidra-evidence/) | P2 | 4-dimension review + library ID + synthesis + Frida | 4–6 (parallel) |
| [`headless-ghidra-discovery`](./headless-ghidra-discovery/) | P3 | Frontier batch discovery | 1/round |
| [`headless-ghidra-batch-decompile`](./headless-ghidra-batch-decompile/) | P4+P5 | Source comparison → semantic rebuild → decompile | N/round (fn-parallel) |
| [`headless-ghidra-frida-verify`](./headless-ghidra-frida-verify/) | P6 | Frida I/O recording → comparison → gate verdict | N/round (fn-parallel) |

## Core Constraints

- **Headless-only workflows**. GUI operations are out of scope.
- **Evidence-driven**. All decisions reference observable evidence.
- **Reproducible**. Commands, inputs, and expected results are explicitly replayable.
- **All-YAML artifacts**. All artifacts are in YAML format (except code).
- **Gate-checked**. Every phase transition is validated by `gate-check.sh`.

## Artifact Path Conventions

```
.work/
├── ghidra-artifacts/<target-id>/          ← analysis artifacts
│   ├── pipeline-state.yaml               ← single source of truth
│   ├── intake/                           ← P0 artifacts
│   ├── baseline/                         ← P1 artifacts (6 YAML files)
│   ├── evidence/                         ← P2 artifacts
│   └── iterations/<NNN>/                 ← P3–P6 iteration artifacts
│       └── functions/<fn_id>/            ← per-function artifacts
├── ghidra-projects/<target-id>/          ← Ghidra project (single instance)
└── reconstruction/<target-id>/           ← CMake reconstruction project
    ├── CMakeLists.txt
    ├── include/
    ├── src/
    ├── third_party/
    ├── stubs/
    └── tests/
```

## Scripts

| Script | Location | Purpose |
|---|---|---|
| `gate-check.sh` | `headless-ghidra/scripts/` | Programmatic gate validation |
| `ghidra-queue.sh` | `headless-ghidra/scripts/` | Ghidra operation serial lock |
| `reconstruction-init.sh` | `headless-ghidra/scripts/` | Reconstruction project initialization |
| `run-headless-analysis.sh` | `headless-ghidra/scripts/` | Ghidra headless analysis |
| `discover-ghidra.sh` | `headless-ghidra/scripts/` | Ghidra installation discovery |
| `normalize-ar-archive.sh` | `headless-ghidra/scripts/` | Archive normalization |

## Frida Scripts

| Script | Purpose |
|---|---|
| `io-capture.js` | Generic function I/O recording |
| `io-compare.js` | Original vs reconstructed I/O comparison |
| `fuzz-input-gen.js` | Signature-based fuzz input generation |
| `signature-analysis.js` | Runtime function signature analysis |
| `decomp-compare.js` | Decompilation comparison |
| `call-tree-trace.js` | Call tree tracing |
| `dispatch-vtable-trace.js` | Dispatch/vtable tracing |
| `hotpath-coverage.js` | Hot path coverage |

## Quick Start

1. The orchestrator reads target path, checks for an in-progress `pipeline-state.yaml`
2. If none exists, starts a fresh pipeline from P0
3. If one exists, shows a dialog asking to resume or restart
4. Executes P0 → P1 → P2 → [P3 → P4+P5 → P6]* in order
5. Runs `gate-check.sh` at each phase transition; continues on pass
