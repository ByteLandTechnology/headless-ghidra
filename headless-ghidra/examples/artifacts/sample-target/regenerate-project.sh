#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
RUNNER="${SKILL_DIR}/scripts/run-headless-analysis.sh"

TARGET_ID="${TARGET_ID:-sample-target}"
TARGET_BINARY="${TARGET_BINARY:-}"
GHIDRA_INSTALL_DIR="${GHIDRA_INSTALL_DIR:-}"
WORKFLOW_ACTION="${WORKFLOW_ACTION:-baseline}"
UPSTREAM_PROJECT_SLUG="${UPSTREAM_PROJECT_SLUG:-}"
SELECTED_FUNCTIONS="${SELECTED_FUNCTIONS:-}"
WORKSPACE_ROOT_ARG="${GHIDRA_WORKSPACE_ROOT:-}"
WORKSPACE_ROOT=""
ARTIFACT_ROOT="${ARTIFACT_ROOT:-}"

resolve_existing_dir() {
  local candidate="$1"
  if [[ "${candidate}" != /* ]]; then
    candidate="${PWD}/${candidate}"
  fi
  if [[ ! -d "${candidate}" ]]; then
    return 1
  fi
  cd "${candidate}" && pwd -P
}

detect_workspace_root() {
  local explicit_root="$1"
  local resolved=""
  local git_root=""

  if [[ -n "${explicit_root}" ]]; then
    resolved="$(resolve_existing_dir "${explicit_root}" || true)"
    if [[ -z "${resolved}" ]]; then
      printf 'Workspace root not found: %s\n' "${explicit_root}" >&2
      exit 1
    fi
    printf '%s\n' "${resolved}"
    return 0
  fi

  git_root="$(git -C "${PWD}" rev-parse --show-toplevel 2>/dev/null || true)"
  if [[ -n "${git_root}" ]]; then
    printf '%s\n' "${git_root}"
    return 0
  fi

  cd "${PWD}" && pwd -P
}

usage() {
  cat <<'EOF'
Environment-driven wrapper around run-headless-analysis.sh.

Supported WORKFLOW_ACTION values:
  baseline
  plan-call-graph
  call-graph
  plan-compare
  compare-prep
  plan-decompile
  decompile-selected

Required environment:
  TARGET_BINARY for baseline, plan-call-graph, call-graph, plan-decompile, and
    decompile-selected
  UPSTREAM_PROJECT_SLUG for plan-compare and compare-prep
  SELECTED_FUNCTIONS as a comma-separated list for plan-decompile and
    decompile-selected

Optional environment:
  GHIDRA_WORKSPACE_ROOT to override the default workspace-root detection
EOF
}

require_binary() {
  if [[ -n "${TARGET_BINARY}" ]]; then
    return 0
  fi
  cat <<'EOF' >&2
TARGET_BINARY is not set.
Provide the absolute path to the binary you want to analyze, for example:
  export TARGET_BINARY=/absolute/path/to/binary
EOF
  exit 1
}

require_slug() {
  if [[ -n "${UPSTREAM_PROJECT_SLUG}" ]]; then
    return 0
  fi
  cat <<'EOF' >&2
UPSTREAM_PROJECT_SLUG is not set.
Provide the probable upstream project slug before Source Comparison replay, for
example:
  export UPSTREAM_PROJECT_SLUG=zlib
EOF
  exit 1
}

append_selected_functions() {
  local raw="${1}"
  local item=""
  local trimmed=""
  SELECTED_ARGS=()
  IFS=',' read -r -a items <<< "${raw}"
  for item in "${items[@]}"; do
    trimmed="${item#"${item%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"
    if [[ -n "${trimmed}" ]]; then
      SELECTED_ARGS+=(--selected-function "${trimmed}")
    fi
  done
}

if [[ "${WORKFLOW_ACTION}" == "--help" || "${WORKFLOW_ACTION}" == "-h" ]]; then
  usage
  exit 0
fi

WORKSPACE_ROOT="$(detect_workspace_root "${WORKSPACE_ROOT_ARG}")"
if [[ -z "${ARTIFACT_ROOT}" ]]; then
  ARTIFACT_ROOT="${WORKSPACE_ROOT}/.work/ghidra-artifacts/${TARGET_ID}"
fi

COMMAND=(
  "${RUNNER}"
  --action "${WORKFLOW_ACTION}"
  --workspace-root "${WORKSPACE_ROOT}"
  --target-id "${TARGET_ID}"
)
COMMAND+=(--artifacts-dir "${ARTIFACT_ROOT}")

case "${WORKFLOW_ACTION}" in
  baseline|plan-call-graph|call-graph)
    require_binary
    COMMAND+=(--binary "${TARGET_BINARY}")
    ;;
  plan-compare|compare-prep)
    require_slug
    COMMAND+=(--project-slug "${UPSTREAM_PROJECT_SLUG}")
    ;;
  plan-decompile|decompile-selected)
    require_binary
    if [[ -z "${SELECTED_FUNCTIONS}" ]]; then
      cat <<'EOF' >&2
SELECTED_FUNCTIONS is not set.
Provide a comma-separated outer-to-inner function list, for example:
  export SELECTED_FUNCTIONS='outer_fn@00102140,inner_fn@00101890'
EOF
      exit 1
    fi
    COMMAND+=(--binary "${TARGET_BINARY}")
    append_selected_functions "${SELECTED_FUNCTIONS}"
    COMMAND+=("${SELECTED_ARGS[@]}")
    ;;
  *)
    printf 'Unsupported WORKFLOW_ACTION: %s\n' "${WORKFLOW_ACTION}" >&2
    usage >&2
    exit 1
    ;;
esac

if [[ -n "${GHIDRA_INSTALL_DIR}" ]]; then
  COMMAND+=(--install-dir "${GHIDRA_INSTALL_DIR}")
fi

printf 'Running: %q ' "${COMMAND[@]}"
printf '\n'
"${COMMAND[@]}"

if [[ "${WORKFLOW_ACTION}" == "compare-prep" ]]; then
  cat <<'EOF'

Remember to update:
- upstream-reference.md with reference_mode, reference_path, and fallback_reason
- third-party-diff.md with inherited, modified, and unresolved findings
EOF
fi
