---
name: "headless-ghidra-intake"
description: "P0 sub-skill: target identity confirmation, workspace creation, Ghidra discovery, and archive normalization. Dispatches two parallel agents: intake-workspace and intake-ghidra."
phase: "P0"
---

# Headless Ghidra Intake — P0 Target Intake

This skill handles the first phase of the pipeline: confirming target identity,
creating the workspace, and discovering the Ghidra installation. The orchestrator
dispatches two parallel agents to execute this phase.

## Entry / Exit Gates

| Property | Value |
|---|---|
| **Entry gate** | A clearly specified binary file or archive path |
| **Exit gate** | `gate-check.sh --gate P0` passes |

## Agent Role Definitions

### Agent: `intake-workspace`

| Property | Value |
|---|---|
| **Agent ID** | `intake-workspace` |
| **Instances** | 1 |
| **Lifecycle** | Short-lived — terminates when P0 completes |
| **Role** | Create workspace, record target identity, initialize reconstruction project |
| **Parallelism** | ✅ Runs in parallel with `intake-ghidra` |

**Inputs**:
- User-provided target binary/archive path
- Workspace root path convention (`.work/`)

**Outputs**:
- `.work/ghidra-artifacts/<target-id>/intake/target-identity.yaml`
- `.work/ghidra-artifacts/<target-id>/intake/workspace-manifest.yaml`
- `.work/ghidra-artifacts/<target-id>/pipeline-state.yaml` (initial version)
- `.work/ghidra-projects/<target-id>/` directory
- `.work/reconstruction/<target-id>/` directory (with `reconstruction-manifest.yaml`, `CMakeLists.txt`, `.gitignore`)
- (conditional) `.work/ghidra-artifacts/<target-id>/intake/archive-normalization.yaml`

**Available tools**:
- `scripts/normalize-ar-archive.sh` (conditional: when input is an archive)
- `scripts/reconstruction-init.sh`
- `mkdir`, `cp`, filesystem operations
- `yq` — YAML generation

**Strict prohibitions**:
- ⛔ Must not run Ghidra (that is `intake-ghidra`'s responsibility)
- ⛔ Must not analyze binary content
- ⛔ Must not modify any `baseline/` or `evidence/` files

**Termination conditions**:
- `target-identity.yaml` + `workspace-manifest.yaml` written
- All directory structures created
- `reconstruction-manifest.yaml` initialized
- If archive input, `archive-normalization.yaml` status is `members_ready`

**System prompt**:

```
You are the P0 workspace initialization agent. Your responsibilities:
1. Determine target-id from the user-provided path (normalize to lowercase+hyphens)
2. Create .work/ghidra-artifacts/<target-id>/ and subdirectories
3. Create .work/ghidra-projects/<target-id>/
4. Run reconstruction-init.sh to initialize the reconstruction project
5. Fill in target-identity.yaml and workspace-manifest.yaml
6. If input is an ar archive, run normalize-ar-archive.sh

You are not responsible for discovering or running Ghidra. Terminate immediately
when finished.
```

---

### Agent: `intake-ghidra`

| Property | Value |
|---|---|
| **Agent ID** | `intake-ghidra` |
| **Instances** | 1 |
| **Lifecycle** | Short-lived |
| **Role** | Discover local Ghidra installation, verify availability, capture help output |
| **Parallelism** | ✅ Runs in parallel with `intake-workspace` |

**Inputs**:
- Environment variables / common installation paths

**Outputs**:
- `.work/ghidra-artifacts/<target-id>/intake/ghidra-discovery.yaml`

**Available tools**:
- `scripts/discover-ghidra.sh`
- Shell commands (`which`, `find`, `test -x`)
- `$ANALYZE_HEADLESS -help`

**Strict prohibitions**:
- ⛔ Must not run analyzeHeadless for actual analysis
- ⛔ Must not create Ghidra projects
- ⛔ Must not fabricate help output (must capture from real binary)

**Termination conditions**:
- `ghidra-discovery.yaml` written
- `analyze_headless_path` points to an actually executable file

**System prompt**:

```
You are the P0 Ghidra discovery agent. Your sole responsibility:
1. Run discover-ghidra.sh to find local Ghidra installation
2. Run analyzeHeadless -help to capture real help output
3. Write results to ghidra-discovery.yaml

You never fabricate help text. If Ghidra is not found, report failure and
terminate.
```

## Artifact Manifest

| File | Format | Description |
|---|---|---|
| `intake/target-identity.yaml` | YAML | Target identity card |
| `intake/workspace-manifest.yaml` | YAML | Workspace directory manifest |
| `intake/ghidra-discovery.yaml` | YAML | Ghidra installation info |
| `intake/archive-normalization.yaml` | YAML (conditional) | Archive normalization result |

## Gate Check Matrix (P0)

| ID | Check | Type |
|---|---|---|
| P0_01 | `intake/target-identity.yaml` exists | blocking |
| P0_02 | Contains `target_id` field, non-empty | blocking |
| P0_03 | Contains `binary_path` field, non-empty | blocking |
| P0_04 | binary_path points to existing file | blocking |
| P0_05 | `intake/workspace-manifest.yaml` exists | blocking |
| P0_06 | artifact_root directory created | blocking |
| P0_07 | `intake/ghidra-discovery.yaml` exists | blocking |
| P0_08 | Contains `install_dir`, non-empty | blocking |
| P0_09 | Contains `analyze_headless_path`, non-empty | blocking |
| P0_10 | analyzeHeadless is executable | blocking |
| P0_11 | reconstruction directory created | blocking |
| P0_12 | `reconstruction-manifest.yaml` exists | blocking |
| P0_13 | (conditional) If archive, normalization status = `members_ready` | blocking |

## Next Step Routing

- P0 gate passes → orchestrator automatically enters P1 (`headless-ghidra-baseline`).
