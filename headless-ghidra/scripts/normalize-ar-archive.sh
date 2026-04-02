#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

ARCHIVE_PATH=""
ARCHIVE_ID=""
WORKSPACE_ROOT=""
ARTIFACT_ROOT=""
MEMBER_OUTPUT_ROOT=""
REVIEW_OUTPUT_ROOT=""
INTAKE_RECORD=""
INVENTORY_RECORD=""
HANDOFF_RECORD=""
REPLAY_RECORD=""
SELECTION_POLICY=""
EXTRACTOR_BIN=""
EXTRACTOR_LABEL=""
ORIGINAL_ARGS=("$@")

LIST_STDERR=""
ARCHIVE_FILE_OBSERVATION=""
OVERALL_STATUS=""
STOP_CONDITION=""

declare -a MEMBERS=()
declare -a ACCEPTED_MEMBER_IDS=()
declare -a DEFERRED_MEMBER_IDS=()
declare -a UNSUPPORTED_MEMBER_IDS=()
declare -a FAILED_MEMBER_IDS=()
declare -a INVENTORY_ROWS=()
declare -a INPUT_ARGUMENT_ROWS=()
declare -a OUTPUT_PATH_ROWS=()
declare -a EXPECTED_OBSERVATION_ROWS=()
declare -a FAILURE_SIGNAL_ROWS=()
declare -a EXTRACTION_COMMAND_ROWS=()

usage() {
  cat <<'EOF'
Usage: normalize-ar-archive.sh [options]

Options:
  --archive PATH            Source archive to normalize.
  --archive-id ID           Stable archive identifier. Defaults to the archive
                            basename without the trailing .a suffix.
  --workspace-root PATH     Workspace root. Defaults to the git repo root, then
                            the current working directory.
  --artifact-root PATH      Runtime artifact root. Defaults to
                            <workspace-root>/.work/ghidra-artifacts/<archive-id>/.
  --member-output-root PATH Runtime member extraction root. Defaults to
                            <artifact-root>/normalized-members/.
  --review-output-root PATH Runtime review-surface root. Defaults to
                            <artifact-root>/review/.
  --intake-record PATH      Runtime archive intake Markdown output.
  --inventory-record PATH   Runtime member inventory Markdown output.
  --handoff-record PATH     Runtime normalization handoff Markdown output.
  --replay-record PATH      Runtime replay-command Markdown output.
  --selection-policy VALUE  Optional member filter. Supported forms:
                            accepted-all (default),
                            exact:<member-name>,
                            regex:<extended-regex>.
  --extractor PATH          Preferred local archive extractor binary.
  -h, --help                Show this message.

Notes:
  - This wrapper is headless-only and prepares archive members before the
    standard Ghidra workflow begins.
  - Runtime outputs default under .work/ghidra-artifacts/.
  - The wrapper refuses to write runtime-generated review surfaces under the
    tracked skill package.
  - Sample files under examples/artifacts/ are tracked review surfaces, not
    default live output destinations.
EOF
}

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

quote_command() {
  local out=""
  local arg=""
  local quoted=""
  for arg in "$@"; do
    printf -v quoted '%q' "${arg}"
    if [[ -n "${out}" ]]; then
      out+=" "
    fi
    out+="${quoted}"
  done
  printf '%s\n' "${out}"
}

safe_id_component() {
  local value="${1:-}"
  local lowered=""
  lowered="$(printf '%s' "${value}" | tr '[:upper:]' '[:lower:]')"
  lowered="$(printf '%s' "${lowered}" | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')"
  if [[ -z "${lowered}" ]]; then
    lowered="unknown"
  fi
  printf '%s\n' "${lowered}"
}

safe_filename_component() {
  local value="${1:-}"
  local sanitized=""
  sanitized="$(printf '%s' "${value}" | sed -E 's#/#--#g; s/[^A-Za-z0-9._-]+/-/g; s/^-+//; s/-+$//')"
  if [[ -z "${sanitized}" ]]; then
    sanitized="unknown"
  fi
  printf '%s\n' "${sanitized}"
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

resolve_existing_dir() {
  local raw_path="$1"
  if [[ "${raw_path}" != /* ]]; then
    raw_path="${PWD}/${raw_path}"
  fi
  if [[ ! -d "${raw_path}" ]]; then
    return 1
  fi
  cd "${raw_path}" && pwd -P
}

detect_workspace_root() {
  local explicit_root="${1:-}"
  local resolved=""
  local git_root=""

  if [[ -n "${explicit_root}" ]]; then
    resolved="$(resolve_existing_dir "${explicit_root}" || true)"
    if [[ -z "${resolved}" ]]; then
      fail "Workspace root not found: ${explicit_root}"
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

ensure_runtime_path() {
  local raw_path="$1"
  local resolved=""
  resolved="$(resolve_path "${raw_path}")"
  case "${resolved}" in
    "${SKILL_DIR}"|${SKILL_DIR}/*)
      fail "Refusing to write runtime output under the tracked skill package: ${resolved}"
      ;;
  esac
  printf '%s\n' "${resolved}"
}

append_markdown_bullet() {
  local array_name="$1"
  local value="$2"
  eval "${array_name}+=(\"\${value}\")"
}

emit_bullets() {
  local array_name="$1"
  local -a items=()
  local line=""
  eval "items=(\"\${${array_name}[@]-}\")"
  for line in "${items[@]}"; do
    if [[ -n "${line}" ]]; then
      printf -- '- %s\n' "${line}"
    fi
  done
}

emit_table_rows() {
  local array_name="$1"
  local -a items=()
  local line=""
  eval "items=(\"\${${array_name}[@]-}\")"
  for line in "${items[@]}"; do
    if [[ -n "${line}" ]]; then
      printf '%s\n' "${line}"
    fi
  done
}

count_member_occurrences() {
  local target_name="$1"
  local count=0
  local current_name=""
  for current_name in "${MEMBERS[@]}"; do
    if [[ "${current_name}" == "${target_name}" ]]; then
      count=$((count + 1))
    fi
  done
  printf '%s\n' "${count}"
}

member_occurrence_index() {
  local target_name="$1"
  local end_index="$2"
  local count=0
  local idx=0
  for (( idx=0; idx<=end_index; idx++ )); do
    if [[ "${MEMBERS[idx]}" == "${target_name}" ]]; then
      count=$((count + 1))
    fi
  done
  printf '%s\n' "${count}"
}

detect_extractor() {
  local explicit_bin="${1:-}"
  if [[ -n "${explicit_bin}" ]]; then
    local explicit_resolved=""
    explicit_resolved="$(command -v "${explicit_bin}" 2>/dev/null || true)"
    if [[ -z "${explicit_resolved}" ]]; then
      fail "Preferred extractor not found: ${explicit_bin}"
    fi
    EXTRACTOR_BIN="${explicit_resolved}"
    EXTRACTOR_LABEL="$(basename "${explicit_resolved}")"
    return 0
  fi

  if command -v ar >/dev/null 2>&1; then
    EXTRACTOR_BIN="$(command -v ar)"
    EXTRACTOR_LABEL="ar"
    return 0
  fi

  if command -v llvm-ar >/dev/null 2>&1; then
    EXTRACTOR_BIN="$(command -v llvm-ar)"
    EXTRACTOR_LABEL="llvm-ar"
    return 0
  fi

  fail "No local archive extractor found. Install or provide ar/llvm-ar via --extractor."
}

member_selected() {
  local member_name="$1"
  if [[ -z "${SELECTION_POLICY}" || "${SELECTION_POLICY}" == "accepted-all" ]]; then
    return 0
  fi
  case "${SELECTION_POLICY}" in
    exact:*)
      [[ "${member_name}" == "${SELECTION_POLICY#exact:}" ]]
      ;;
    regex:*)
      [[ "${member_name}" =~ ${SELECTION_POLICY#regex:} ]]
      ;;
    *)
      fail "Unsupported --selection-policy value: ${SELECTION_POLICY}"
      ;;
  esac
}

validate_selection_policy() {
  local policy="${SELECTION_POLICY:-accepted-all}"
  local pattern=""
  local status=0

  case "${policy}" in
    ""|accepted-all)
      return 0
      ;;
    exact:*)
      if [[ -z "${policy#exact:}" ]]; then
        fail "Invalid --selection-policy value: exact:<member-name> requires a non-empty member name"
      fi
      return 0
      ;;
    regex:*)
      pattern="${policy#regex:}"
      if [[ -z "${pattern}" ]]; then
        fail "Invalid --selection-policy value: regex:<extended-regex> requires a non-empty pattern"
      fi
      # Validate with bash's regex engine without letting `set -e` abort
      # before we can emit a user-facing error.
      if [[ "" =~ ${pattern} ]]; then
        status=0
      else
        status=$?
      fi
      if [[ "${status}" -eq 2 ]]; then
        fail "Invalid regex in --selection-policy: ${pattern}"
      fi
      return 0
      ;;
    *)
      fail "Unsupported --selection-policy value: ${policy}"
      ;;
  esac
}

classify_member_kind() {
  local member_name="$1"
  local file_summary="$2"
  local lowered_name=""
  local lowered_file=""

  lowered_name="$(printf '%s' "${member_name}" | tr '[:upper:]' '[:lower:]')"
  lowered_file="$(printf '%s' "${file_summary}" | tr '[:upper:]' '[:lower:]')"

  case "${lowered_name}" in
    "/"|"//"|"__.symdef"|"__.symdef sorted"|"__.symtab"|"__.strtab")
      printf 'metadata_only\n'
      return 0
      ;;
  esac

  if [[ "${lowered_file}" == *"relocatable"* || "${lowered_file}" == *"object"* ]]; then
    printf 'importable_object\n'
    return 0
  fi

  if [[ "${lowered_name}" == *.o || "${lowered_name}" == *.obj ]]; then
    printf 'importable_object\n'
    return 0
  fi

  if [[ "${lowered_file}" == *"text"* || "${lowered_file}" == *"ascii"* || "${lowered_file}" == *"unicode"* || "${lowered_file}" == *"xml"* || "${lowered_file}" == *"json"* ]]; then
    printf 'unsupported_payload\n'
    return 0
  fi

  printf 'unknown\n'
}

write_intake_record() {
  cat >"${INTAKE_RECORD}" <<EOF
# Archive Intake Record: \`${ARCHIVE_ID}\`

## Status

- Artifact state: Runtime archive-normalization intake surface
- Wrapper surface: \`normalize-ar-archive.sh\` as a supported \`orchestration_wrapper\`
- Direct import status: \`blocked_requires_normalization\`
- Overall outcome: \`${OVERALL_STATUS}\`
- Runtime artifact root: \`${ARTIFACT_ROOT}\`
- Review surface root: \`${REVIEW_OUTPUT_ROOT}\`

## Current Review Record

| Field | Value | Review Notes |
|---|---|---|
| \`archive_id\` | \`${ARCHIVE_ID}\` | Stable identifier for this normalization run. |
| \`archive_path\` | \`${ARCHIVE_PATH}\` | Input archive supplied to the wrapper. |
| \`provenance_notes\` | \`local_archive_input\` | Review the caller-provided archive path and local file metadata before deeper analysis. |
| \`direct_import_status\` | \`blocked_requires_normalization\` | The raw archive is not treated as the downstream program target. |
| \`archive_observation\` | \`${ARCHIVE_FILE_OBSERVATION}\` | Captured from \`file\` or the extractor listing boundary. |
| \`normalization_wrapper_id\` | \`normalize-ar-archive\` | Canonical wrapper entrypoint for archive normalization. |
| \`runtime_artifact_root\` | \`${ARTIFACT_ROOT}\` | Runtime-generated members and review outputs stay under \`.work/\`. |
| \`replay_record_path\` | \`${REPLAY_RECORD}\` | Use this record to replay the same normalization pass. |
| \`overall_status\` | \`${OVERALL_STATUS}\` | Must match the handoff summary and any stop condition. |

## Recognition Evidence

- \`file\` observation: \`${ARCHIVE_FILE_OBSERVATION}\`
- Extractor used for listing and extraction: \`${EXTRACTOR_LABEL}\`
- The wrapper listed \`${#MEMBERS[@]}\` archive member(s) before status classification.

## Required Follow-Up

1. Review \`archive-member-inventory.md\` before choosing any downstream target.
2. Review \`archive-normalization-handoff.md\` to confirm which members may proceed.
3. Use \`archive-replay-command-record.md\` to reproduce the same extractor path and output layout.

## Reviewer Notes

- Runtime review surfaces under \`${REVIEW_OUTPUT_ROOT}\` are live outputs.
- Tracked sample files under \`examples/artifacts/sample-target/\` are review examples, not live output destinations.
- When \`overall_status\` is not \`members_ready\`, stop before any Ghidra import attempt.
EOF
}

write_inventory_record() {
  cat >"${INVENTORY_RECORD}" <<EOF
# Archive Member Inventory: \`${ARCHIVE_ID}\`

## Status

- Artifact state: Runtime archive member inventory
- Wrapper surface: \`normalize-ar-archive.sh\`
- Status vocabulary: \`accepted\`, \`deferred\`, \`unsupported\`, \`failed\`
- Current outcome summary:
  - accepted: \`${#ACCEPTED_MEMBER_IDS[@]}\`
  - deferred: \`${#DEFERRED_MEMBER_IDS[@]}\`
  - unsupported: \`${#UNSUPPORTED_MEMBER_IDS[@]}\`
  - failed: \`${#FAILED_MEMBER_IDS[@]}\`

## Member Inventory

| Member Id | Member Name | Member Kind | Member Status | Normalized Target Id | Collision Key | Extracted Runtime Path | Reason | Architecture Notes |
|---|---|---|---|---|---|---|---|---|
$(emit_table_rows INVENTORY_ROWS)

## Rules

- Duplicate member names must not silently overwrite one another.
- \`accepted\` members become the only candidates for downstream Ghidra intake.
- \`deferred\`, \`unsupported\`, and \`failed\` members remain visible here even when the archive-level workflow continues.
- Runtime-generated extracted members live under \`${MEMBER_OUTPUT_ROOT}\`, never under the tracked skill package.
EOF
}

write_handoff_record() {
  local stop_line="not_applicable"
  if [[ -n "${STOP_CONDITION}" ]]; then
    stop_line="${STOP_CONDITION}"
  fi

  cat >"${HANDOFF_RECORD}" <<EOF
# Archive Normalization Handoff: \`${ARCHIVE_ID}\`

## Status

- Artifact state: Runtime downstream handoff record
- Current archive outcome: \`${OVERALL_STATUS}\`
- Stop condition: \`${stop_line}\`

## Accepted Downstream Targets

EOF

  if [[ ${#ACCEPTED_MEMBER_IDS[@]} -eq 0 ]]; then
    cat >>"${HANDOFF_RECORD}" <<'EOF'
- none

EOF
  else
    emit_bullets ACCEPTED_MEMBER_IDS >>"${HANDOFF_RECORD}"
    printf '\n' >>"${HANDOFF_RECORD}"
  fi

  cat >>"${HANDOFF_RECORD}" <<EOF
## Deferred Members

EOF
  if [[ ${#DEFERRED_MEMBER_IDS[@]} -eq 0 ]]; then
    cat >>"${HANDOFF_RECORD}" <<'EOF'
- none

EOF
  else
    emit_bullets DEFERRED_MEMBER_IDS >>"${HANDOFF_RECORD}"
    printf '\n' >>"${HANDOFF_RECORD}"
  fi

  cat >>"${HANDOFF_RECORD}" <<EOF
## Unsupported Members

EOF
  if [[ ${#UNSUPPORTED_MEMBER_IDS[@]} -eq 0 ]]; then
    cat >>"${HANDOFF_RECORD}" <<'EOF'
- none

EOF
  else
    emit_bullets UNSUPPORTED_MEMBER_IDS >>"${HANDOFF_RECORD}"
    printf '\n' >>"${HANDOFF_RECORD}"
  fi

  cat >>"${HANDOFF_RECORD}" <<EOF
## Failed Members

EOF
  if [[ ${#FAILED_MEMBER_IDS[@]} -eq 0 ]]; then
    cat >>"${HANDOFF_RECORD}" <<'EOF'
- none

EOF
  else
    emit_bullets FAILED_MEMBER_IDS >>"${HANDOFF_RECORD}"
    printf '\n' >>"${HANDOFF_RECORD}"
  fi

  cat >>"${HANDOFF_RECORD}" <<EOF
## Downstream Entry Rule

- One accepted member becomes one downstream target identity.
- Review \`archive-intake-record.md\` and \`archive-member-inventory.md\` before any handoff.
- Do not continue into baseline evidence when the current archive outcome is not \`members_ready\`.

## Provenance Anchors

- \`${INTAKE_RECORD}\`
- \`${INVENTORY_RECORD}\`
- \`${REPLAY_RECORD}\`
EOF
}

write_replay_record() {
  local wrapper_command=""
  local extractor_command=""

  wrapper_command="$(
    quote_command \
      "${SCRIPT_DIR}/normalize-ar-archive.sh" \
      --archive "${ARCHIVE_PATH}" \
      --archive-id "${ARCHIVE_ID}" \
      --workspace-root "${WORKSPACE_ROOT}" \
      --artifact-root "${ARTIFACT_ROOT}" \
      --member-output-root "${MEMBER_OUTPUT_ROOT}" \
      --review-output-root "${REVIEW_OUTPUT_ROOT}" \
      --selection-policy "${SELECTION_POLICY:-accepted-all}" \
      --extractor "${EXTRACTOR_BIN}"
  )"

  extractor_command="$(quote_command "${EXTRACTOR_BIN}" t "${ARCHIVE_PATH}")"

  cat >"${REPLAY_RECORD}" <<EOF
# Archive Replay Command Record: \`${ARCHIVE_ID}\`

## Status

- Artifact state: Runtime replay-command surface for archive normalization
- Wrapper surface: \`normalize-ar-archive.sh\`
- Current outcome: \`${OVERALL_STATUS}\`

## Exact Commands

| Field | Value |
|---|---|
| \`wrapper_command\` | \`${wrapper_command}\` |
| \`extractor_command\` | \`${extractor_command}\` |

## Input Arguments

EOF
  emit_bullets INPUT_ARGUMENT_ROWS >>"${REPLAY_RECORD}"
  printf '\n## Output Paths\n\n' >>"${REPLAY_RECORD}"
  emit_bullets OUTPUT_PATH_ROWS >>"${REPLAY_RECORD}"
  printf '\n## Expected Observations\n\n' >>"${REPLAY_RECORD}"
  emit_bullets EXPECTED_OBSERVATION_ROWS >>"${REPLAY_RECORD}"
  printf '\n## Failure Signals\n\n' >>"${REPLAY_RECORD}"
  emit_bullets FAILURE_SIGNAL_ROWS >>"${REPLAY_RECORD}"

  if [[ ${#EXTRACTION_COMMAND_ROWS[@]} -gt 0 ]]; then
    printf '\n## Extraction Commands Observed\n\n' >>"${REPLAY_RECORD}"
    emit_bullets EXTRACTION_COMMAND_ROWS >>"${REPLAY_RECORD}"
  fi

  cat >>"${REPLAY_RECORD}" <<EOF

## Reviewer Notes

- Replay the wrapper from a clean or clearly described workspace.
- The review surfaces under \`${REVIEW_OUTPUT_ROOT}\` are live runtime outputs.
- The tracked sample replay record under \`examples/artifacts/sample-target/\` is a reviewed example, not the default live destination.
EOF
}

write_outputs() {
  write_intake_record
  write_inventory_record
  write_handoff_record
  write_replay_record
}

ARCHIVE_BASENAME=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --archive)
      ARCHIVE_PATH="${2:-}"
      shift 2
      ;;
    --archive-id)
      ARCHIVE_ID="${2:-}"
      shift 2
      ;;
    --workspace-root)
      WORKSPACE_ROOT="${2:-}"
      shift 2
      ;;
    --artifact-root)
      ARTIFACT_ROOT="${2:-}"
      shift 2
      ;;
    --member-output-root)
      MEMBER_OUTPUT_ROOT="${2:-}"
      shift 2
      ;;
    --review-output-root)
      REVIEW_OUTPUT_ROOT="${2:-}"
      shift 2
      ;;
    --intake-record)
      INTAKE_RECORD="${2:-}"
      shift 2
      ;;
    --inventory-record)
      INVENTORY_RECORD="${2:-}"
      shift 2
      ;;
    --handoff-record)
      HANDOFF_RECORD="${2:-}"
      shift 2
      ;;
    --replay-record)
      REPLAY_RECORD="${2:-}"
      shift 2
      ;;
    --selection-policy)
      SELECTION_POLICY="${2:-}"
      shift 2
      ;;
    --extractor)
      EXTRACTOR_BIN="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "Unknown option: $1"
      ;;
  esac
done

validate_selection_policy

if [[ -z "${ARCHIVE_PATH}" ]]; then
  fail "Missing --archive PATH. Use --help for usage."
fi

ARCHIVE_PATH="$(resolve_path "${ARCHIVE_PATH}")"
if [[ ! -f "${ARCHIVE_PATH}" ]]; then
  fail "Archive not found: ${ARCHIVE_PATH}"
fi

WORKSPACE_ROOT="$(detect_workspace_root "${WORKSPACE_ROOT}")"
ARCHIVE_BASENAME="$(basename "${ARCHIVE_PATH}")"
if [[ -z "${ARCHIVE_ID}" ]]; then
  ARCHIVE_ID="${ARCHIVE_BASENAME%.a}"
  if [[ "${ARCHIVE_ID}" == "${ARCHIVE_BASENAME}" ]]; then
    ARCHIVE_ID="${ARCHIVE_BASENAME}"
  fi
fi
ARCHIVE_ID="$(safe_id_component "${ARCHIVE_ID}")"

if [[ -z "${ARTIFACT_ROOT}" ]]; then
  ARTIFACT_ROOT="${WORKSPACE_ROOT}/.work/ghidra-artifacts/${ARCHIVE_ID}"
fi
ARTIFACT_ROOT="$(ensure_runtime_path "${ARTIFACT_ROOT}")"

if [[ -z "${MEMBER_OUTPUT_ROOT}" ]]; then
  MEMBER_OUTPUT_ROOT="${ARTIFACT_ROOT}/normalized-members"
fi
MEMBER_OUTPUT_ROOT="$(ensure_runtime_path "${MEMBER_OUTPUT_ROOT}")"

if [[ -z "${REVIEW_OUTPUT_ROOT}" ]]; then
  REVIEW_OUTPUT_ROOT="${ARTIFACT_ROOT}/review"
fi
REVIEW_OUTPUT_ROOT="$(ensure_runtime_path "${REVIEW_OUTPUT_ROOT}")"

if [[ -z "${INTAKE_RECORD}" ]]; then
  INTAKE_RECORD="${REVIEW_OUTPUT_ROOT}/archive-intake-record.md"
fi
INTAKE_RECORD="$(ensure_runtime_path "${INTAKE_RECORD}")"

if [[ -z "${INVENTORY_RECORD}" ]]; then
  INVENTORY_RECORD="${REVIEW_OUTPUT_ROOT}/archive-member-inventory.md"
fi
INVENTORY_RECORD="$(ensure_runtime_path "${INVENTORY_RECORD}")"

if [[ -z "${HANDOFF_RECORD}" ]]; then
  HANDOFF_RECORD="${REVIEW_OUTPUT_ROOT}/archive-normalization-handoff.md"
fi
HANDOFF_RECORD="$(ensure_runtime_path "${HANDOFF_RECORD}")"

if [[ -z "${REPLAY_RECORD}" ]]; then
  REPLAY_RECORD="${REVIEW_OUTPUT_ROOT}/archive-replay-command-record.md"
fi
REPLAY_RECORD="$(ensure_runtime_path "${REPLAY_RECORD}")"

mkdir -p "${ARTIFACT_ROOT}" "${MEMBER_OUTPUT_ROOT}" "${REVIEW_OUTPUT_ROOT}" "${ARTIFACT_ROOT}/replay" "${ARTIFACT_ROOT}/logs" "${ARTIFACT_ROOT}/tmp"

LIST_STDERR="${ARTIFACT_ROOT}/logs/archive-list.stderr.log"
touch "${LIST_STDERR}"

detect_extractor "${EXTRACTOR_BIN}"
ARCHIVE_FILE_OBSERVATION="$(file -b "${ARCHIVE_PATH}" 2>/dev/null || true)"
if [[ -z "${ARCHIVE_FILE_OBSERVATION}" ]]; then
  ARCHIVE_FILE_OBSERVATION="file_unavailable"
fi

append_markdown_bullet INPUT_ARGUMENT_ROWS "\`archive_path\`: \`${ARCHIVE_PATH}\`"
append_markdown_bullet INPUT_ARGUMENT_ROWS "\`archive_id\`: \`${ARCHIVE_ID}\`"
append_markdown_bullet INPUT_ARGUMENT_ROWS "\`artifact_root\`: \`${ARTIFACT_ROOT}\`"
append_markdown_bullet INPUT_ARGUMENT_ROWS "\`member_output_root\`: \`${MEMBER_OUTPUT_ROOT}\`"
append_markdown_bullet INPUT_ARGUMENT_ROWS "\`review_output_root\`: \`${REVIEW_OUTPUT_ROOT}\`"
append_markdown_bullet INPUT_ARGUMENT_ROWS "\`selection_policy\`: \`${SELECTION_POLICY:-accepted-all}\`"

append_markdown_bullet OUTPUT_PATH_ROWS "\`${INTAKE_RECORD}\`"
append_markdown_bullet OUTPUT_PATH_ROWS "\`${INVENTORY_RECORD}\`"
append_markdown_bullet OUTPUT_PATH_ROWS "\`${HANDOFF_RECORD}\`"
append_markdown_bullet OUTPUT_PATH_ROWS "\`${REPLAY_RECORD}\`"
append_markdown_bullet OUTPUT_PATH_ROWS "\`${MEMBER_OUTPUT_ROOT}\`"

append_markdown_bullet FAILURE_SIGNAL_ROWS "Extractor listing fails for \`${ARCHIVE_PATH}\`."
append_markdown_bullet FAILURE_SIGNAL_ROWS "The archive lists zero members."
append_markdown_bullet FAILURE_SIGNAL_ROWS "No member reaches \`accepted\`, producing a stop condition instead of a downstream target."
append_markdown_bullet FAILURE_SIGNAL_ROWS "A duplicate member name would overwrite another extracted path without an explicit collision rule."

list_output_file="${ARTIFACT_ROOT}/tmp/archive-members.list"
if "${EXTRACTOR_BIN}" t "${ARCHIVE_PATH}" >"${list_output_file}" 2>"${LIST_STDERR}"; then
  while IFS= read -r member_line; do
    if [[ -n "${member_line}" ]]; then
      MEMBERS+=("${member_line}")
    fi
  done <"${list_output_file}"
else
  OVERALL_STATUS="normalization_failed"
  STOP_CONDITION="extractor could not list archive members"
  append_markdown_bullet EXPECTED_OBSERVATION_ROWS "The wrapper records a failure posture and stops before any downstream import handoff."
  write_outputs
  printf 'Archive normalization failed during listing. Review %s\n' "${REPLAY_RECORD}" >&2
  exit 1
fi

if [[ ${#MEMBERS[@]} -eq 0 ]]; then
  OVERALL_STATUS="stopped_no_eligible_members"
  STOP_CONDITION="archive listed zero members"
  append_markdown_bullet EXPECTED_OBSERVATION_ROWS "The wrapper records a stop condition with no accepted downstream targets."
  write_outputs
  printf 'Archive normalization stopped: no members found. Review %s\n' "${REPLAY_RECORD}"
  exit 0
fi

for member_array_index in "${!MEMBERS[@]}"; do
  member_name="${MEMBERS[member_array_index]}"
  member_index="$(member_occurrence_index "${member_name}" "${member_array_index}")"
  member_total_count="$(count_member_occurrences "${member_name}")"

  member_id="${ARCHIVE_ID}--$(safe_id_component "${member_name}")"
  collision_key="not_applicable"
  member_status=""
  member_kind=""
  normalized_target_id="not_applicable"
  extracted_runtime_path="not_applicable"
  reason=""
  architecture_notes="not_applicable"

  if [[ ${member_total_count} -gt 1 ]]; then
    collision_key="dup$(printf '%02d' "${member_index}")"
    member_status="deferred"
    member_kind="unknown"
    reason="duplicate member name requires explicit collision review before deterministic extraction proceeds"
    member_id="${member_id}-${collision_key}"
    DEFERRED_MEMBER_IDS+=("${member_id}")
    INVENTORY_ROWS+=("| \`${member_id}\` | \`${member_name}\` | \`${member_kind}\` | \`${member_status}\` | \`${normalized_target_id}\` | \`${collision_key}\` | \`${extracted_runtime_path}\` | ${reason} | \`${architecture_notes}\` |")
    continue
  fi

  case "$(printf '%s' "${member_name}" | tr '[:upper:]' '[:lower:]')" in
    "/"|"//"|"__.symdef"|"__.symdef sorted"|"__.symtab"|"__.strtab")
      member_status="unsupported"
      member_kind="metadata_only"
      reason="metadata member is not a downstream Ghidra import target"
      UNSUPPORTED_MEMBER_IDS+=("${member_id}")
      INVENTORY_ROWS+=("| \`${member_id}\` | \`${member_name}\` | \`${member_kind}\` | \`${member_status}\` | \`${normalized_target_id}\` | \`${collision_key}\` | \`${extracted_runtime_path}\` | ${reason} | \`${architecture_notes}\` |")
      continue
      ;;
  esac

  extract_work_dir="${ARTIFACT_ROOT}/tmp/extract-${member_id}"
  rm -rf "${extract_work_dir}"
  mkdir -p "${extract_work_dir}"
  extract_stdout="${ARTIFACT_ROOT}/logs/${member_id}.extract.stdout.log"
  extract_stderr="${ARTIFACT_ROOT}/logs/${member_id}.extract.stderr.log"
  if (
    cd "${extract_work_dir}"
    "${EXTRACTOR_BIN}" x "${ARCHIVE_PATH}" "${member_name}" >"${extract_stdout}" 2>"${extract_stderr}"
  ); then
    extracted_path="$(find "${extract_work_dir}" -type f | head -n 1 || true)"
    if [[ -z "${extracted_path}" ]]; then
      member_status="failed"
      member_kind="unknown"
      reason="extractor reported success but produced no extracted member file"
      FAILED_MEMBER_IDS+=("${member_id}")
      INVENTORY_ROWS+=("| \`${member_id}\` | \`${member_name}\` | \`${member_kind}\` | \`${member_status}\` | \`${normalized_target_id}\` | \`${collision_key}\` | \`${extracted_runtime_path}\` | ${reason} | \`${architecture_notes}\` |")
      continue
    fi

    architecture_notes="$(file -b "${extracted_path}" 2>/dev/null || true)"
    if [[ -z "${architecture_notes}" ]]; then
      architecture_notes="file_unavailable"
    fi
    member_kind="$(classify_member_kind "${member_name}" "${architecture_notes}")"
    extracted_basename="$(safe_filename_component "$(basename "${member_name}")")"
    extracted_dest="${MEMBER_OUTPUT_ROOT}/${ARCHIVE_ID}--${extracted_basename}"
    mv "${extracted_path}" "${extracted_dest}"
    extracted_runtime_path="${extracted_dest}"
    extraction_command="$(quote_command "${EXTRACTOR_BIN}" x "${ARCHIVE_PATH}" "${member_name}")"
    EXTRACTION_COMMAND_ROWS+=("\`${extraction_command}\` -> \`${extracted_runtime_path}\`")

    case "${member_kind}" in
      importable_object)
        normalized_target_id="${ARCHIVE_ID}--$(safe_id_component "${member_name}")"
        if member_selected "${member_name}"; then
          member_status="accepted"
          reason="member classified as an importable object and selected for downstream intake"
          ACCEPTED_MEMBER_IDS+=("${normalized_target_id}")
        else
          member_status="deferred"
          normalized_target_id="not_applicable"
          reason="selection policy deferred this otherwise importable member"
          DEFERRED_MEMBER_IDS+=("${member_id}")
        fi
        ;;
      unsupported_payload)
        member_status="unsupported"
        reason="member extracted successfully but does not present as a supported importable object"
        UNSUPPORTED_MEMBER_IDS+=("${member_id}")
        normalized_target_id="not_applicable"
        ;;
      *)
        member_status="deferred"
        reason="member extracted successfully but needs review before it can be treated as import-ready"
        DEFERRED_MEMBER_IDS+=("${member_id}")
        normalized_target_id="not_applicable"
        ;;
    esac
  else
    member_status="failed"
    member_kind="unknown"
    reason="extractor could not extract this member with the current local command"
    FAILED_MEMBER_IDS+=("${member_id}")
  fi

  INVENTORY_ROWS+=("| \`${member_id}\` | \`${member_name}\` | \`${member_kind}\` | \`${member_status}\` | \`${normalized_target_id}\` | \`${collision_key}\` | \`${extracted_runtime_path}\` | ${reason} | \`${architecture_notes}\` |")
done

if [[ ${#ACCEPTED_MEMBER_IDS[@]} -gt 0 ]]; then
  OVERALL_STATUS="members_ready"
  STOP_CONDITION=""
  append_markdown_bullet EXPECTED_OBSERVATION_ROWS "At least one normalized member target is accepted for downstream Ghidra intake."
  append_markdown_bullet EXPECTED_OBSERVATION_ROWS "The member inventory records accepted, deferred, unsupported, and failed states without silent overwrite."
else
  OVERALL_STATUS="stopped_no_eligible_members"
  STOP_CONDITION="no archive member reached accepted status"
  append_markdown_bullet EXPECTED_OBSERVATION_ROWS "The wrapper records a stop condition because no member may proceed downstream."
fi

write_outputs

printf 'Archive normalization complete: %s\n' "${OVERALL_STATUS}"
printf 'Review surfaces:\n'
printf '  %s\n' "${INTAKE_RECORD}"
printf '  %s\n' "${INVENTORY_RECORD}"
printf '  %s\n' "${HANDOFF_RECORD}"
printf '  %s\n' "${REPLAY_RECORD}"
