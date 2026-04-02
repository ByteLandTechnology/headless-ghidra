#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEFAULT_EXPORT_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/ExportAnalysisArtifacts.java"
DEFAULT_CALL_GRAPH_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/ExportCallGraph.java"
DEFAULT_REVIEW_EVIDENCE_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/ReviewEvidenceCandidates.java"
DEFAULT_TARGET_SELECTION_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/PlanTargetSelection.java"
DEFAULT_APPLY_RENAMES_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/ApplyRenames.java"
DEFAULT_VERIFY_RENAMES_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/VerifyRenames.java"
DEFAULT_APPLY_SIGNATURES_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/ApplyFunctionSignatures.java"
DEFAULT_VERIFY_SIGNATURES_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/VerifyFunctionSignatures.java"
DEFAULT_LINT_REVIEW_ARTIFACTS_SCRIPT_PATH="${SKILL_DIR}/ghidra-scripts/LintReviewArtifacts.java"
WORKSPACE_ROOT=""
DEFAULT_PROJECT_ROOT=""
DEFAULT_ARTIFACTS_ROOT=""
DEFAULT_GHIDRA_RUNTIME_USER_HOME=""
DEFAULT_UPSTREAM_SOURCES_ROOT=""
LOG_DIR=""
ORIGINAL_HOME="${HOME}"

usage() {
  cat <<'EOF'
Usage: run-headless-analysis.sh [options]

Options:
  --action ACTION            One of: discover, plan-baseline, baseline,
                             plan-call-graph, call-graph,
                             plan-review-evidence, review-evidence,
                             plan-target-selection, target-selection,
                             plan-compare, compare-prep, plan-decompile,
                             decompile-selected, plan-apply-renames,
                             apply-renames, plan-verify-renames,
                             verify-renames, plan-apply-signatures,
                             apply-signatures, plan-verify-signatures,
                             verify-signatures, plan-lint-review-artifacts,
                             lint-review-artifacts.
                             Aliases: plan -> plan-baseline,
                             regenerate -> baseline.
  --binary PATH              Binary to import or process.
  --target-id ID             Stable target identifier. Defaults to the binary basename.
  --workspace-root PATH      Root of the analysis workspace. Defaults to
                             GHIDRA_WORKSPACE_ROOT, then the git repo root from
                             the current working directory, then the current
                             working directory itself.
  --project-root PATH        Root for generated Ghidra projects.
  --artifacts-dir PATH       Directory for tracked Markdown artifacts.
  --script-path PATH         Ghidra post-script to run after import or process.
  --rename-log PATH          Reviewable Markdown rename plan. Defaults to
                             <artifacts-dir>/renaming-log.md.
  --signature-log PATH       Reviewable Markdown signature plan. Defaults to
                             <artifacts-dir>/signature-log.md.
  --review-artifact PATH     Artifact path for lint-review-artifacts
                             (repeatable). Defaults to renaming-log.md and
                             signature-log.md under <artifacts-dir>.
  --install-dir PATH         Preferred Ghidra install directory.
  --project-slug SLUG        Upstream project slug for Source Comparison planning.
  --selected-function VALUE  Selected function name, address, or name@address
                             for Selected Decompilation (repeatable).
  --extra-arg VALUE          Extra arg forwarded to analyzeHeadless (repeatable).
  -h, --help                 Show this message.

Notes:
  - Skill package paths are resolved from the script's own location.
  - This wrapper is headless-only and intentionally avoids GUI fallback.
  - Baseline Evidence and Selected Decompilation are separate actions.
  - Runtime Java prefers GHIDRA_JAVA_HOME, then the recorded Ghidra JDK,
    then JAVA_HOME, then java on PATH.
  - Runtime artifacts default to <workspace-root>/.work/ghidra-artifacts/<target-id>/.
  - Writing generated artifacts under tracked skill directories such as
    .agents/skills/ or .claude/skills/ is rejected.
  - Source Comparison planning records paths and next commands but does not
    guess an upstream repository or version for you.
EOF
}

fail_missing_binary() {
  cat <<'EOF' >&2
Missing --binary PATH.

Provide a local binary and then rerun, for example:
  bash <skill-root>/scripts/run-headless-analysis.sh \
    --action baseline \
    --binary /path/to/binary \
    --target-id sample-target
EOF
  exit 1
}

fail_missing_project_slug() {
  cat <<'EOF' >&2
Missing --project-slug SLUG.

Provide the upstream project identifier before Source Comparison planning, for
example:
  bash <skill-root>/scripts/run-headless-analysis.sh \
    --action plan-compare \
    --binary /path/to/binary \
    --target-id sample-target \
    --project-slug zlib
EOF
  exit 1
}

fail_missing_selected_functions() {
  cat <<'EOF' >&2
Selected Decompilation requires at least one --selected-function VALUE.

Record the function selection first, then rerun, for example:
  bash <skill-root>/scripts/run-headless-analysis.sh \
    --action decompile-selected \
    --binary /path/to/binary \
    --target-id sample-target \
    --selected-function FUN_00123456@00123456
EOF
  exit 1
}

fail_missing_rename_log() {
  cat <<'EOF' >&2
Rename application and verification require a reviewable rename plan.

Provide --rename-log PATH or create the default:
  <artifacts-dir>/renaming-log.md

The file must be a Markdown table with rows ready for replayable rename or
verification work.
EOF
  exit 1
}

fail_missing_signature_log() {
  cat <<'EOF' >&2
Signature application and verification require a reviewable signature plan.

Provide --signature-log PATH or create the default:
  <artifacts-dir>/signature-log.md

The file must be a Markdown table with rows ready for replayable signature
application or verification work.
EOF
  exit 1
}

normalize_action() {
  case "$1" in
    plan) printf 'plan-baseline\n' ;;
    regenerate) printf 'baseline\n' ;;
    *) printf '%s\n' "$1" ;;
  esac
}

resolve_path() {
  local raw_path="$1"
  local parent=""
  local base=""
  if [[ "${raw_path}" != /* ]]; then
    raw_path="${PWD}/${raw_path}"
  fi
  parent="$(dirname "${raw_path}")"
  base="$(basename "${raw_path}")"
  mkdir -p "${parent}"
  parent="$(cd "${parent}" && pwd -P)"
  printf '%s/%s\n' "${parent}" "${base}"
}

append_forbidden_skill_root_if_present() {
  local candidate="$1"
  local resolved=""
  local existing=""

  if [[ ! -d "${candidate}" ]]; then
    return 0
  fi

  resolved="$(cd "${candidate}" && pwd -P)"
  for existing in "${FORBIDDEN_SKILL_ROOTS[@]}"; do
    if [[ "${existing}" == "${resolved}" ]]; then
      return 0
    fi
  done

  FORBIDDEN_SKILL_ROOTS+=("${resolved}")
}

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

fail_forbidden_artifacts_dir() {
  cat <<EOF >&2
Refusing to write runtime-generated artifacts under the skill directory.

Requested artifacts dir:
  $1

Choose a workspace path under:
  ${DEFAULT_ARTIFACTS_ROOT}/<target-id>/

The files under the installed skill package (for example
'.agents/skills/headless-ghidra/' or
'.claude/skills/headless-ghidra/') are tracked sample
documentation only, not the runtime output location.
EOF
  exit 1
}

print_running_command() {
  printf 'Running: %q ' "$@"
  printf '\n'
}

resolve_runtime_java_home() {
  local candidate="${GHIDRA_JAVA_HOME:-}"
  local save_file=""
  local java_path=""
  local derived_home=""

  if [[ -n "${candidate}" && -x "${candidate}/bin/java" ]]; then
    printf '%s\n' "${candidate}"
    return 0
  fi

  save_file="$(
    find "${ORIGINAL_HOME}/Library/ghidra" -maxdepth 3 -type f -name java_home.save 2>/dev/null \
      | sort \
      | tail -n 1
  )"
  if [[ -n "${save_file}" ]]; then
    candidate="$(tr -d '\r' < "${save_file}")"
    if [[ -n "${candidate}" && -x "${candidate}/bin/java" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  fi

  candidate="${JAVA_HOME:-}"
  if [[ -n "${candidate}" && -x "${candidate}/bin/java" ]]; then
    printf '%s\n' "${candidate}"
    return 0
  fi

  java_path="$(command -v java || true)"
  if [[ -n "${java_path}" ]]; then
    derived_home="$(cd "$(dirname "${java_path}")/.." && pwd -P)"
    if [[ -x "${derived_home}/bin/java" ]]; then
      printf '%s\n' "${derived_home}"
      return 0
    fi
  fi

  return 1
}

build_java_options() {
  local redirected_user_home="$1"
  if [[ -n "${_JAVA_OPTIONS:-}" ]]; then
    printf '%s %s\n' "${_JAVA_OPTIONS}" "-Duser.home=${redirected_user_home}"
    return 0
  fi
  printf '%s\n' "-Duser.home=${redirected_user_home}"
}

require_file() {
  local path="$1"
  local reason="$2"
  if [[ ! -f "${path}" ]]; then
    printf 'Missing expected artifact: %s\nReason: %s\n' "${path}" "${reason}" >&2
    return 1
  fi
}

check_log_for_script_errors() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    printf 'Expected Ghidra log was not written: %s\n' "${path}" >&2
    return 1
  fi

  if rg -n \
    -e 'REPORT SCRIPT ERROR' \
    -e 'GhidraScriptLoadException' \
    -e 'The class could not be found' \
    -e 'Failed to get OSGi bundle containing script' \
    -e 'Traceback \\(most recent call last\\)' \
    "${path}" >/dev/null 2>&1; then
    printf 'Detected Ghidra script failure markers in %s\n' "${path}" >&2
    return 1
  fi
}

check_report_failed_count() {
  local report_path="$1"
  local label="$2"

  require_file "${report_path}" "${label} requires a reviewable report." || return 1
  if ! rg -n '^- Failed: `0`$' "${report_path}" >/dev/null 2>&1; then
    printf '%s report did not end cleanly: %s\n' "${label}" "${report_path}" >&2
    return 1
  fi
}

check_baseline_artifacts() {
  local base="${ARTIFACTS_DIR}"
  require_file "${base}/function-names.md" "Baseline export must emit observed functions." || return 1
  require_file "${base}/imports-and-libraries.md" "Baseline export must emit import evidence." || return 1
  require_file "${base}/strings-and-constants.md" "Baseline export must emit string evidence." || return 1
  require_file "${base}/types-and-structs.md" "Baseline export must emit type evidence." || return 1
  require_file "${base}/xrefs-and-callgraph.md" "Baseline export must emit xref evidence." || return 1
  require_file "${base}/decompiled-output.md" "Baseline export must emit the blocked decompilation placeholder." || return 1
  require_file "${base}/renaming-log.md" "Baseline export must emit the reviewable rename schema." || return 1
  require_file "${base}/signature-log.md" "Baseline export must emit the reviewable signature schema." || return 1
  if ! rg -n '^## Status$' "${base}/decompiled-output.md" >/dev/null 2>&1; then
    printf 'Baseline placeholder is malformed: %s\n' "${base}/decompiled-output.md" >&2
    return 1
  fi
}

check_decompile_artifacts() {
  local path="${ARTIFACTS_DIR}/decompiled-output.md"
  require_file "${path}" "Selected decompilation must emit reviewable output." || return 1
  if ! rg -n '^### Function ' "${path}" >/dev/null 2>&1; then
    printf 'Selected decompilation did not export any function sections: %s\n' "${path}" >&2
    return 1
  fi
}

check_export_artifact() {
  local path="$1"
  local label="$2"
  require_file "${path}" "${label} must emit a reviewable Markdown artifact." || return 1
}

run_checked_action() {
  local action_name="$1"
  shift
  local run_log="${LOG_DIR}/${action_name}.run.log"
  local script_log="${LOG_DIR}/${action_name}.script.log"
  local runtime_user_home="${DEFAULT_GHIDRA_RUNTIME_USER_HOME}"
  local runtime_java_home=""
  local java_options=""
  local -a env_prefix=()
  local command_status=0
  local validation_status=0

  mkdir -p "${runtime_user_home}"
  runtime_java_home="$(resolve_runtime_java_home || true)"
  java_options="$(build_java_options "${runtime_user_home}")"
  env_prefix=(env)
  if [[ -n "${runtime_java_home}" ]]; then
    env_prefix+=("JAVA_HOME=${runtime_java_home}")
  fi
  env_prefix+=("_JAVA_OPTIONS=${java_options}")
  rm -f "${run_log}" "${script_log}"
  print_running_command "${env_prefix[@]}" "$@"
  "${env_prefix[@]}" "$@" || command_status=$?

  check_log_for_script_errors "${run_log}" || validation_status=$?
  check_log_for_script_errors "${script_log}" || validation_status=$?

  case "${action_name}" in
    baseline)
      check_baseline_artifacts || validation_status=$?
      ;;
    call-graph)
      check_export_artifact "${ARTIFACTS_DIR}/call-graph-detail.md" "Detailed call graph export" || validation_status=$?
      ;;
    review-evidence)
      check_export_artifact "${ARTIFACTS_DIR}/evidence-candidates.md" "Evidence review" || validation_status=$?
      ;;
    target-selection)
      check_export_artifact "${ARTIFACTS_DIR}/target-selection.md" "Target selection" || validation_status=$?
      ;;
    decompile-selected)
      check_decompile_artifacts || validation_status=$?
      ;;
    apply-renames)
      check_report_failed_count "${ARTIFACTS_DIR}/rename-apply-report.md" "Rename application" || validation_status=$?
      ;;
    verify-renames)
      check_report_failed_count "${ARTIFACTS_DIR}/rename-verification-report.md" "Rename verification" || validation_status=$?
      ;;
    apply-signatures)
      check_report_failed_count "${ARTIFACTS_DIR}/signature-apply-report.md" "Signature application" || validation_status=$?
      ;;
    verify-signatures)
      check_report_failed_count "${ARTIFACTS_DIR}/signature-verification-report.md" "Signature verification" || validation_status=$?
      ;;
    lint-review-artifacts)
      check_report_failed_count "${ARTIFACTS_DIR}/artifact-lint-report.md" "Review artifact lint" || validation_status=$?
      ;;
  esac

  if [[ "${command_status}" -ne 0 ]]; then
    return "${command_status}"
  fi

  if [[ "${validation_status}" -ne 0 ]]; then
    return "${validation_status}"
  fi
}

ACTION="plan-baseline"
BINARY_PATH=""
TARGET_ID=""
WORKSPACE_ROOT_ARG="${GHIDRA_WORKSPACE_ROOT:-}"
PROJECT_ROOT=""
ARTIFACTS_DIR=""
SCRIPT_PATH=""
SCRIPT_PATH_EXPLICIT=0
RENAME_LOG=""
SIGNATURE_LOG=""
INSTALL_DIR=""
PROJECT_SLUG=""
EXTRA_ARGS=()
SELECTED_FUNCTIONS=()
REVIEW_ARTIFACTS=()
ARTIFACTS_DIR_EXPLICIT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --action)
      ACTION="$(normalize_action "${2:-}")"
      shift 2
      ;;
    --binary)
      BINARY_PATH="${2:-}"
      shift 2
      ;;
    --target-id)
      TARGET_ID="${2:-}"
      shift 2
      ;;
    --workspace-root)
      WORKSPACE_ROOT_ARG="${2:-}"
      shift 2
      ;;
    --project-root)
      PROJECT_ROOT="${2:-}"
      shift 2
      ;;
    --artifacts-dir)
      ARTIFACTS_DIR="${2:-}"
      ARTIFACTS_DIR_EXPLICIT=1
      shift 2
      ;;
    --script-path)
      SCRIPT_PATH="${2:-}"
      SCRIPT_PATH_EXPLICIT=1
      shift 2
      ;;
    --rename-log)
      RENAME_LOG="${2:-}"
      shift 2
      ;;
    --signature-log)
      SIGNATURE_LOG="${2:-}"
      shift 2
      ;;
    --review-artifact)
      REVIEW_ARTIFACTS+=("${2:-}")
      shift 2
      ;;
    --install-dir)
      INSTALL_DIR="${2:-}"
      shift 2
      ;;
    --project-slug)
      PROJECT_SLUG="${2:-}"
      shift 2
      ;;
    --selected-function)
      SELECTED_FUNCTIONS+=("${2:-}")
      shift 2
      ;;
    --extra-arg)
      EXTRA_ARGS+=("${2:-}")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown option: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "${ACTION}" == "discover" ]]; then
  if [[ -n "${INSTALL_DIR}" ]]; then
    exec "${SCRIPT_DIR}/discover-ghidra.sh" --install-dir "${INSTALL_DIR}"
  fi
  exec "${SCRIPT_DIR}/discover-ghidra.sh"
fi

WORKSPACE_ROOT="$(detect_workspace_root "${WORKSPACE_ROOT_ARG}")"
DEFAULT_PROJECT_ROOT="${WORKSPACE_ROOT}/.work/ghidra-projects"
DEFAULT_ARTIFACTS_ROOT="${WORKSPACE_ROOT}/.work/ghidra-artifacts"
DEFAULT_GHIDRA_RUNTIME_USER_HOME="${WORKSPACE_ROOT}/.work/ghidra-user-home"
DEFAULT_UPSTREAM_SOURCES_ROOT="${WORKSPACE_ROOT}/.work/upstream-sources"

if [[ -z "${PROJECT_ROOT}" ]]; then
  PROJECT_ROOT="${DEFAULT_PROJECT_ROOT}"
fi

if [[ -z "${TARGET_ID}" ]]; then
  if [[ -n "${BINARY_PATH}" ]]; then
    TARGET_ID="$(basename "${BINARY_PATH}")"
    TARGET_ID="${TARGET_ID//[^A-Za-z0-9._-]/-}"
  else
    TARGET_ID="sample-target"
  fi
fi

if [[ "${ARTIFACTS_DIR_EXPLICIT}" -ne 1 ]]; then
  ARTIFACTS_DIR="${DEFAULT_ARTIFACTS_ROOT}/${TARGET_ID}"
fi

ARTIFACTS_DIR="$(resolve_path "${ARTIFACTS_DIR}")"
if [[ -z "${RENAME_LOG}" ]]; then
  RENAME_LOG="${ARTIFACTS_DIR}/renaming-log.md"
fi
RENAME_LOG="$(resolve_path "${RENAME_LOG}")"
if [[ -z "${SIGNATURE_LOG}" ]]; then
  SIGNATURE_LOG="${ARTIFACTS_DIR}/signature-log.md"
fi
SIGNATURE_LOG="$(resolve_path "${SIGNATURE_LOG}")"
LOG_DIR="${ARTIFACTS_DIR}/logs"
SKILL_DIR_REAL="$(cd "${SKILL_DIR}" && pwd -P)"
FORBIDDEN_SKILL_ROOTS=("${SKILL_DIR_REAL}")
SKILL_PARENT_REAL="$(cd "${SKILL_DIR}/.." && pwd -P)"
if [[ "$(basename "${SKILL_PARENT_REAL}")" == "skills" ]]; then
  case "$(basename "$(cd "${SKILL_PARENT_REAL}/.." && pwd -P)")" in
    .agents|.claude)
      append_forbidden_skill_root_if_present "${SKILL_PARENT_REAL}"
      ;;
  esac
fi
append_forbidden_skill_root_if_present "${WORKSPACE_ROOT}/.agents/skills"
append_forbidden_skill_root_if_present "${WORKSPACE_ROOT}/.claude/skills"

for forbidden_root in "${FORBIDDEN_SKILL_ROOTS[@]}"; do
  case "${ARTIFACTS_DIR}" in
    "${forbidden_root}"|${forbidden_root}/*)
      fail_forbidden_artifacts_dir "${ARTIFACTS_DIR}"
      ;;
  esac
done

if [[ ${#REVIEW_ARTIFACTS[@]} -gt 0 ]]; then
  for i in "${!REVIEW_ARTIFACTS[@]}"; do
    REVIEW_ARTIFACTS[$i]="$(resolve_path "${REVIEW_ARTIFACTS[$i]}")"
  done
fi

if [[ "${SCRIPT_PATH_EXPLICIT}" -ne 1 ]]; then
  case "${ACTION}" in
    plan-baseline|baseline|plan-decompile|decompile-selected)
      SCRIPT_PATH="${DEFAULT_EXPORT_SCRIPT_PATH}"
      ;;
    plan-call-graph|call-graph)
      SCRIPT_PATH="${DEFAULT_CALL_GRAPH_SCRIPT_PATH}"
      ;;
    plan-review-evidence|review-evidence)
      SCRIPT_PATH="${DEFAULT_REVIEW_EVIDENCE_SCRIPT_PATH}"
      ;;
    plan-target-selection|target-selection)
      SCRIPT_PATH="${DEFAULT_TARGET_SELECTION_SCRIPT_PATH}"
      ;;
    plan-apply-renames|apply-renames)
      SCRIPT_PATH="${DEFAULT_APPLY_RENAMES_SCRIPT_PATH}"
      ;;
    plan-verify-renames|verify-renames)
      SCRIPT_PATH="${DEFAULT_VERIFY_RENAMES_SCRIPT_PATH}"
      ;;
    plan-apply-signatures|apply-signatures)
      SCRIPT_PATH="${DEFAULT_APPLY_SIGNATURES_SCRIPT_PATH}"
      ;;
    plan-verify-signatures|verify-signatures)
      SCRIPT_PATH="${DEFAULT_VERIFY_SIGNATURES_SCRIPT_PATH}"
      ;;
    plan-lint-review-artifacts|lint-review-artifacts)
      SCRIPT_PATH="${DEFAULT_LINT_REVIEW_ARTIFACTS_SCRIPT_PATH}"
      ;;
    *)
      SCRIPT_PATH="${DEFAULT_EXPORT_SCRIPT_PATH}"
      ;;
  esac
fi

PROJECT_DIR="${PROJECT_ROOT}/${TARGET_ID}"

case "${ACTION}" in
  plan-compare|compare-prep)
    if [[ -z "${PROJECT_SLUG}" ]]; then
      fail_missing_project_slug
    fi

    TRACKED_UPSTREAM_PATH="third_party/upstream/${PROJECT_SLUG}"
    FALLBACK_UPSTREAM_PATH="${DEFAULT_UPSTREAM_SOURCES_ROOT}/${PROJECT_SLUG}"

    if [[ "${ACTION}" == "compare-prep" ]]; then
      mkdir -p "${DEFAULT_UPSTREAM_SOURCES_ROOT}/${PROJECT_SLUG}"
    fi

    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
PROJECT_SLUG=${PROJECT_SLUG}
TRACKED_UPSTREAM_PATH=${TRACKED_UPSTREAM_PATH}
FALLBACK_UPSTREAM_PATH=${FALLBACK_UPSTREAM_PATH}

Preferred tracked path:
  git submodule add <repo-url> ${TRACKED_UPSTREAM_PATH}

Fallback local clone path:
  git clone <repo-url> ${FALLBACK_UPSTREAM_PATH}

Required follow-up:
  Record \`reference_mode\`, \`reference_path\`, and \`fallback_reason\` in:
    ${ARTIFACTS_DIR}/upstream-reference.md
  Record inherited, modified, and unresolved findings in:
    ${ARTIFACTS_DIR}/third-party-diff.md
EOF
    exit 0
    ;;
  plan-baseline|baseline|plan-call-graph|call-graph|plan-review-evidence|review-evidence|plan-target-selection|target-selection|plan-decompile|decompile-selected|plan-apply-renames|apply-renames|plan-verify-renames|verify-renames|plan-apply-signatures|apply-signatures|plan-verify-signatures|verify-signatures|plan-lint-review-artifacts|lint-review-artifacts)
    if [[ -z "${BINARY_PATH}" ]]; then
      fail_missing_binary
    fi
    if [[ ! -f "${BINARY_PATH}" ]]; then
      printf 'Binary not found: %s\n' "${BINARY_PATH}" >&2
      exit 1
    fi
    ;;
  *)
    printf 'Unsupported action: %s\n' "${ACTION}" >&2
    usage >&2
    exit 1
    ;;
esac

if [[ "${ACTION}" == "plan-decompile" || "${ACTION}" == "decompile-selected" ]]; then
  if [[ ${#SELECTED_FUNCTIONS[@]} -eq 0 ]]; then
    fail_missing_selected_functions
  fi
fi

if [[ "${ACTION}" == "apply-renames" || "${ACTION}" == "verify-renames" ]]; then
  if [[ ! -f "${RENAME_LOG}" ]]; then
    fail_missing_rename_log
  fi
fi

if [[ "${ACTION}" == "apply-signatures" || "${ACTION}" == "verify-signatures" ]]; then
  if [[ ! -f "${SIGNATURE_LOG}" ]]; then
    fail_missing_signature_log
  fi
fi

if [[ -n "${INSTALL_DIR}" ]]; then
  ANALYZE_HEADLESS="$("${SCRIPT_DIR}/discover-ghidra.sh" --install-dir "${INSTALL_DIR}" --print-analyze-headless)"
else
  ANALYZE_HEADLESS="$("${SCRIPT_DIR}/discover-ghidra.sh" --print-analyze-headless)"
fi

PROGRAM_NAME=""
if [[ -n "${BINARY_PATH}" ]]; then
  PROGRAM_NAME="$(basename "${BINARY_PATH}")"
fi

BASELINE_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -import "${BINARY_PATH}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" baseline
)

DECOMPILE_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" decompile
)

REVIEW_EVIDENCE_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}"
)

TARGET_SELECTION_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}"
)

CALL_GRAPH_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}"
)

if [[ ${#SELECTED_FUNCTIONS[@]} -gt 0 ]]; then
  DECOMPILE_COMMAND+=("${SELECTED_FUNCTIONS[@]}")
fi

if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  BASELINE_COMMAND+=("${EXTRA_ARGS[@]}")
  DECOMPILE_COMMAND+=("${EXTRA_ARGS[@]}")
  CALL_GRAPH_COMMAND+=("${EXTRA_ARGS[@]}")
  REVIEW_EVIDENCE_COMMAND+=("${EXTRA_ARGS[@]}")
  TARGET_SELECTION_COMMAND+=("${EXTRA_ARGS[@]}")
fi

BASELINE_COMMAND+=(
  -log "${LOG_DIR}/baseline.run.log"
  -scriptlog "${LOG_DIR}/baseline.script.log"
)

DECOMPILE_COMMAND+=(
  -log "${LOG_DIR}/decompile-selected.run.log"
  -scriptlog "${LOG_DIR}/decompile-selected.script.log"
)

REVIEW_EVIDENCE_COMMAND+=(
  -log "${LOG_DIR}/review-evidence.run.log"
  -scriptlog "${LOG_DIR}/review-evidence.script.log"
)

TARGET_SELECTION_COMMAND+=(
  -log "${LOG_DIR}/target-selection.run.log"
  -scriptlog "${LOG_DIR}/target-selection.script.log"
)

CALL_GRAPH_COMMAND+=(
  -log "${LOG_DIR}/call-graph.run.log"
  -scriptlog "${LOG_DIR}/call-graph.script.log"
)

APPLY_RENAMES_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" "${RENAME_LOG}"
)

VERIFY_RENAMES_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" "${RENAME_LOG}"
)

APPLY_SIGNATURES_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" "${SIGNATURE_LOG}"
)

VERIFY_SIGNATURES_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}" "${SIGNATURE_LOG}"
)

LINT_REVIEW_ARTIFACTS_COMMAND=(
  "${ANALYZE_HEADLESS}"
  "${PROJECT_ROOT}"
  "${TARGET_ID}"
  -process "${PROGRAM_NAME}"
  -scriptPath "$(dirname "${SCRIPT_PATH}")"
  -postScript "$(basename "${SCRIPT_PATH}")" "${ARTIFACTS_DIR}" "${TARGET_ID}"
)

if [[ ${#REVIEW_ARTIFACTS[@]} -gt 0 ]]; then
  LINT_REVIEW_ARTIFACTS_COMMAND+=("${REVIEW_ARTIFACTS[@]}")
fi

if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  APPLY_RENAMES_COMMAND+=("${EXTRA_ARGS[@]}")
  VERIFY_RENAMES_COMMAND+=("${EXTRA_ARGS[@]}")
  APPLY_SIGNATURES_COMMAND+=("${EXTRA_ARGS[@]}")
  VERIFY_SIGNATURES_COMMAND+=("${EXTRA_ARGS[@]}")
  LINT_REVIEW_ARTIFACTS_COMMAND+=("${EXTRA_ARGS[@]}")
fi

APPLY_RENAMES_COMMAND+=(
  -log "${LOG_DIR}/apply-renames.run.log"
  -scriptlog "${LOG_DIR}/apply-renames.script.log"
)

VERIFY_RENAMES_COMMAND+=(
  -log "${LOG_DIR}/verify-renames.run.log"
  -scriptlog "${LOG_DIR}/verify-renames.script.log"
)

APPLY_SIGNATURES_COMMAND+=(
  -log "${LOG_DIR}/apply-signatures.run.log"
  -scriptlog "${LOG_DIR}/apply-signatures.script.log"
)

VERIFY_SIGNATURES_COMMAND+=(
  -log "${LOG_DIR}/verify-signatures.run.log"
  -scriptlog "${LOG_DIR}/verify-signatures.script.log"
)

LINT_REVIEW_ARTIFACTS_COMMAND+=(
  -log "${LOG_DIR}/lint-review-artifacts.run.log"
  -scriptlog "${LOG_DIR}/lint-review-artifacts.script.log"
)

case "${ACTION}" in
  plan-baseline)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
COMMAND=$(printf '%q ' "${BASELINE_COMMAND[@]}")

Stage contract:
  - This action is for \`Baseline Evidence\` only.
  - It must not export decompiled bodies.
  - Review the resulting artifacts before \`Selected Decompilation\`.
EOF
    exit 0
    ;;
  baseline)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action baseline "${BASELINE_COMMAND[@]}"
    ;;
  plan-call-graph)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
CALL_GRAPH_DETAIL=${ARTIFACTS_DIR}/call-graph-detail.md
COMMAND=$(printf '%q ' "${CALL_GRAPH_COMMAND[@]}")

Stage contract:
  - This action is for \`Baseline Evidence Follow-Up\`.
  - It exports focused caller/callee detail without mutating program metadata.
  - Use it when \`xrefs-and-callgraph.md\` is too coarse for outside-in target selection.
EOF
    exit 0
    ;;
  call-graph)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action call-graph "${CALL_GRAPH_COMMAND[@]}"
    ;;
  plan-review-evidence)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
EVIDENCE_CANDIDATES=${ARTIFACTS_DIR}/evidence-candidates.md
COMMAND=$(printf '%q ' "${REVIEW_EVIDENCE_COMMAND[@]}")

Stage contract:
  - This action is for \`Evidence Review\`.
  - It exports a reviewable candidate surface without mutating program metadata.
  - Keep metric-style fields secondary and confirm frontier eligibility before promoting any row into \`target-selection.md\`.
EOF
    exit 0
    ;;
  review-evidence)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action review-evidence "${REVIEW_EVIDENCE_COMMAND[@]}"
    ;;
  plan-target-selection)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
TARGET_SELECTION=${ARTIFACTS_DIR}/target-selection.md
COMMAND=$(printf '%q ' "${TARGET_SELECTION_COMMAND[@]}")

Stage contract:
  - This action is for \`Target Selection\`.
  - It exports a reviewable selection surface with one automatic default target.
  - Confirm the frontier basis and matched-only gate before moving inward.
EOF
    exit 0
    ;;
  target-selection)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action target-selection "${TARGET_SELECTION_COMMAND[@]}"
    ;;
  plan-decompile)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
SELECTED_FUNCTIONS=$(printf '%q ' "${SELECTED_FUNCTIONS[@]}")
COMMAND=$(printf '%q ' "${DECOMPILE_COMMAND[@]}")

Stage contract:
  - This action is for \`Selected Decompilation\` only.
  - Record selection rationale and role/name/prototype evidence before running it.
  - Preserve outside-in order when choosing the selected functions.
EOF
    exit 0
    ;;
  decompile-selected)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action decompile-selected "${DECOMPILE_COMMAND[@]}"
    ;;
  plan-apply-renames)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
RENAME_LOG=${RENAME_LOG}
APPLY_REPORT=${ARTIFACTS_DIR}/rename-apply-report.md
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
COMMAND=$(printf '%q ' "${APPLY_RENAMES_COMMAND[@]}")

Stage contract:
  - This action applies reviewable rename entries from \`renaming-log.md\`.
  - Only rows with executable status should mutate the project.
  - The apply report must remain under \`${ARTIFACTS_DIR}\`.
EOF
    exit 0
    ;;
  apply-renames)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action apply-renames "${APPLY_RENAMES_COMMAND[@]}"
    ;;
  plan-verify-renames)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
RENAME_LOG=${RENAME_LOG}
VERIFY_REPORT=${ARTIFACTS_DIR}/rename-verification-report.md
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
COMMAND=$(printf '%q ' "${VERIFY_RENAMES_COMMAND[@]}")

Stage contract:
  - This action verifies rename-plan rows against the current project state.
  - It does not invent missing rename results.
  - The verification report must remain under \`${ARTIFACTS_DIR}\`.
EOF
    exit 0
    ;;
  verify-renames)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action verify-renames "${VERIFY_RENAMES_COMMAND[@]}"
    ;;
  plan-apply-signatures)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
SIGNATURE_LOG=${SIGNATURE_LOG}
APPLY_REPORT=${ARTIFACTS_DIR}/signature-apply-report.md
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
COMMAND=$(printf '%q ' "${APPLY_SIGNATURES_COMMAND[@]}")

Stage contract:
  - This action applies reviewable signature entries from \`signature-log.md\`.
  - Only executable rows should mutate function names, return types, parameters, or calling conventions.
  - The apply report must remain under \`${ARTIFACTS_DIR}\`.
EOF
    exit 0
    ;;
  apply-signatures)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action apply-signatures "${APPLY_SIGNATURES_COMMAND[@]}"
    ;;
  plan-verify-signatures)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
SIGNATURE_LOG=${SIGNATURE_LOG}
VERIFY_REPORT=${ARTIFACTS_DIR}/signature-verification-report.md
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
COMMAND=$(printf '%q ' "${VERIFY_SIGNATURES_COMMAND[@]}")

Stage contract:
  - This action verifies signature-plan rows against current project state.
  - It does not invent missing signature results.
  - The verification report must remain under \`${ARTIFACTS_DIR}\`.
EOF
    exit 0
    ;;
  verify-signatures)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action verify-signatures "${VERIFY_SIGNATURES_COMMAND[@]}"
    ;;
  plan-lint-review-artifacts)
    cat <<EOF
ACTION=${ACTION}
TARGET_ID=${TARGET_ID}
BINARY_PATH=${BINARY_PATH}
PROJECT_DIR=${PROJECT_DIR}
ARTIFACTS_DIR=${ARTIFACTS_DIR}
SCRIPT_PATH=${SCRIPT_PATH}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
LINT_REPORT=${ARTIFACTS_DIR}/artifact-lint-report.md
REVIEW_ARTIFACTS=$(printf '%q ' "${REVIEW_ARTIFACTS[@]}")
COMMAND=$(printf '%q ' "${LINT_REVIEW_ARTIFACTS_COMMAND[@]}")

Stage contract:
  - This action lints reviewable manifests such as \`renaming-log.md\` and \`signature-log.md\`.
  - Parse failures must still produce a reviewable lint report.
  - The lint report must remain under \`${ARTIFACTS_DIR}\`.
EOF
    exit 0
    ;;
  lint-review-artifacts)
    mkdir -p "${PROJECT_ROOT}"
    mkdir -p "${ARTIFACTS_DIR}"
    mkdir -p "${LOG_DIR}"
    run_checked_action lint-review-artifacts "${LINT_REVIEW_ARTIFACTS_COMMAND[@]}"
    ;;
esac
