#!/usr/bin/env bash
# gate-check.sh — Programmatic gate validation for the headless Ghidra pipeline.
# Returns: 0 = pass, 1 = fail (blocking), 2 = warn (conditional)
# Output: JSON to stdout
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  gate-check.sh --gate <P0|P1|P2|P3|P5|P6> --artifact-root <path> [--iteration <NNN>] [--function <fn_id>]

Gates P0–P2 are phase-level. P3 accepts either the current target-selection.md
surface (no --iteration) or the legacy iterations/<NNN>/batch-manifest.yaml
surface (--iteration required). P5/P6 require --iteration and --function.

Exit codes: 0 = pass, 1 = fail, 2 = warn
EOF
  exit 1
}

GATE=""
ARTIFACT_ROOT=""
ITERATION=""
FUNCTION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --gate) GATE="$2"; shift 2 ;;
    --artifact-root) ARTIFACT_ROOT="$2"; shift 2 ;;
    --iteration) ITERATION="$2"; shift 2 ;;
    --function) FUNCTION="$2"; shift 2 ;;
    *) usage ;;
  esac
done

[[ -z "$GATE" || -z "$ARTIFACT_ROOT" ]] && usage

PASS=0
FAIL=1
WARN=2

checks=()
failures=()
warnings=()
exit_code=$PASS

check() {
  local id="$1" desc="$2" type="$3" result="$4"
  checks+=("{\"id\":\"$id\",\"desc\":\"$desc\",\"type\":\"$type\",\"result\":\"$result\"}")
  if [[ "$result" == "fail" ]]; then
    if [[ "$type" == "blocking" ]]; then
      failures+=("$id: $desc")
      exit_code=$FAIL
    else
      warnings+=("$id: $desc")
      [[ $exit_code -eq $PASS ]] && exit_code=$WARN
    fi
  fi
}

file_exists() { [[ -f "$1" ]] && echo "pass" || echo "fail"; }
dir_exists() { [[ -d "$1" ]] && echo "pass" || echo "fail"; }
file_executable() { [[ -x "$1" ]] && echo "pass" || echo "fail"; }
file_nonempty() { [[ -s "$1" ]] && echo "pass" || echo "fail"; }
text_contains() {
  local file="$1"
  local pattern="$2"
  if [[ -f "$file" ]] && rg -n "$pattern" "$file" >/dev/null 2>&1; then
    echo "pass"
  else
    echo "fail"
  fi
}

first_existing_file() {
  local candidate=""
  for candidate in "$@"; do
    if [[ -f "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

markdown_table_data_row_count() {
  local file="$1"
  local heading="${2:-}"
  local total=0
  if [[ ! -f "$file" ]]; then
    echo "0"
    return
  fi
  if [[ -n "$heading" ]]; then
    total=$(awk -v heading="$heading" '
      $0 == heading { in_section=1; next }
      in_section && /^## / { exit }
      in_section && /^\| / { count++ }
      END { print count + 0 }
    ' "$file")
  else
    total=$(grep -c '^| ' "$file" 2>/dev/null || echo "0")
  fi
  if [[ "$total" -ge 2 ]]; then
    echo $((total - 2))
  else
    echo "0"
  fi
}

markdown_section_contains() {
  local file="$1" heading="$2" pattern="$3"
  if [[ ! -f "$file" ]]; then
    echo "fail"
    return
  fi
  awk -v heading="$heading" '
    $0 == heading { in_section=1; next }
    in_section && /^## / { exit }
    in_section { print }
  ' "$file" | rg -q --pcre2 "$pattern" && echo "pass" || echo "fail"
}

derive_reconstruction_root() {
  local artifact_root="$1"
  local parent_root=""

  if [[ "${artifact_root}" == *"/ghidra-artifacts/"* ]]; then
    printf '%s\n' "${artifact_root/\/ghidra-artifacts\//\/reconstruction\/}"
    return 0
  fi

  parent_root="$(dirname "$(dirname "${artifact_root}")")"
  printf '%s\n' "${parent_root}/reconstruction/$(basename "${artifact_root}")"
}

yaml_field_nonempty() {
  local file="$1" field="$2"
  if command -v yq &>/dev/null; then
    local val
    val=$(yq -r "$field // \"\"" "$file" 2>/dev/null || echo "")
    [[ -n "$val" && "$val" != "null" ]] && echo "pass" || echo "fail"
  else
    echo "fail"
  fi
}

yaml_parseable() {
  local file="$1"
  if command -v yq &>/dev/null; then
    yq '.' "$file" >/dev/null 2>&1 && echo "pass" || echo "fail"
  else
    echo "fail"
  fi
}

yaml_array_length() {
  local file="$1" field="$2"
  if command -v yq &>/dev/null; then
    yq -r "$field | length" "$file" 2>/dev/null || echo "0"
  else
    echo "0"
  fi
}

yaml_array_all_have_fields() {
  local file="$1" array_path="$2"
  shift 2
  local fields=("$@")
  if ! command -v yq &>/dev/null; then
    echo "fail"
    return
  fi
  local len
  len=$(yq -r "$array_path | length" "$file" 2>/dev/null || echo "0")
  [[ "$len" -eq 0 ]] && { echo "fail"; return; }
  for field in "${fields[@]}"; do
    local missing
    missing=$(yq -r "$array_path | map(select(.$field == null or .$field == \"\")) | length" "$file" 2>/dev/null || echo "1")
    [[ "$missing" -ne 0 ]] && { echo "fail"; return; }
  done
  echo "pass"
}

yaml_object_fields_nonempty() {
  local file="$1"
  shift
  local fields=("$@")
  if ! command -v yq &>/dev/null || [[ ! -f "$file" ]]; then
    echo "fail"
    return
  fi
  for field in "${fields[@]}"; do
    local val
    val=$(yq -r "$field // \"\"" "$file" 2>/dev/null || echo "")
    [[ -n "$val" && "$val" != "null" ]] || {
      echo "fail"
      return
    }
  done
  echo "pass"
}

yaml_no_duplicate_values() {
  local file="$1" array_path="$2" field="$3"
  if ! command -v yq &>/dev/null; then
    echo "fail"
    return
  fi
  local total unique
  total=$(yq -r "$array_path | map(.$field) | length" "$file" 2>/dev/null || echo "0")
  unique=$(yq -r "$array_path | map(.$field) | unique | length" "$file" 2>/dev/null || echo "0")
  [[ "$total" == "$unique" ]] && echo "pass" || echo "fail"
}

yaml_all_field_equals() {
  local file="$1" array_path="$2" field="$3" expected="$4"
  if ! command -v yq &>/dev/null; then
    echo "fail"
    return
  fi
  local mismatched
  mismatched=$(yq -r "$array_path | map(select(.$field != \"$expected\")) | length" "$file" 2>/dev/null || echo "1")
  [[ "$mismatched" -eq 0 ]] && echo "pass" || echo "fail"
}

yaml_field_equals() {
  local file="$1" field="$2" expected="$3"
  if command -v yq &>/dev/null; then
    local val
    val=$(yq -r "$field // \"\"" "$file" 2>/dev/null || echo "")
    [[ "$val" == "$expected" ]] && echo "pass" || echo "fail"
  else
    echo "fail"
  fi
}

yaml_field_numeric_equals() {
  local file="$1" field="$2" expected="$3"
  if command -v yq &>/dev/null; then
    local val
    val=$(yq -r "$field // \"-1\"" "$file" 2>/dev/null || echo "-1")
    [[ "$val" == "$expected" ]] && echo "pass" || echo "fail"
  else
    echo "fail"
  fi
}

AR="$ARTIFACT_ROOT"

case "$GATE" in
  P0)
    check P0_01 "intake/target-identity.yaml exists" blocking "$(file_exists "$AR/intake/target-identity.yaml")"
    check P0_02 "target_id field non-empty" blocking "$(yaml_field_nonempty "$AR/intake/target-identity.yaml" '.target_id')"
    check P0_03 "binary_path field non-empty" blocking "$(yaml_field_nonempty "$AR/intake/target-identity.yaml" '.binary_path')"

    local_bp=""
    if command -v yq &>/dev/null && [[ -f "$AR/intake/target-identity.yaml" ]]; then
      local_bp=$(yq -r '.binary_path // ""' "$AR/intake/target-identity.yaml" 2>/dev/null || echo "")
    fi
    check P0_04 "binary_path file exists" blocking "$(file_exists "${local_bp:-/nonexistent}")"

    check P0_05 "workspace-manifest.yaml exists" blocking "$(file_exists "$AR/intake/workspace-manifest.yaml")"
    check P0_06 "artifact_root directory exists" blocking "$(dir_exists "$AR")"
    check P0_07 "ghidra-discovery.yaml exists" blocking "$(file_exists "$AR/intake/ghidra-discovery.yaml")"
    check P0_08 "install_dir non-empty" blocking "$(yaml_field_nonempty "$AR/intake/ghidra-discovery.yaml" '.install_dir')"
    check P0_09 "analyze_headless_path non-empty" blocking "$(yaml_field_nonempty "$AR/intake/ghidra-discovery.yaml" '.analyze_headless_path')"

    local_ah=""
    if command -v yq &>/dev/null && [[ -f "$AR/intake/ghidra-discovery.yaml" ]]; then
      local_ah=$(yq -r '.analyze_headless_path // ""' "$AR/intake/ghidra-discovery.yaml" 2>/dev/null || echo "")
    fi
    check P0_10 "analyzeHeadless executable" blocking "$(file_executable "${local_ah:-/nonexistent}")"

    # Derive reconstruction root from artifact root
    recon_root="$(derive_reconstruction_root "$AR")"
    check P0_11 "reconstruction directory exists" blocking "$(dir_exists "$recon_root")"
    check P0_12 "reconstruction-manifest.yaml exists" blocking "$(file_exists "$recon_root/reconstruction-manifest.yaml")"
    ;;

  P1)
    fn_file="$(first_existing_file "$AR/baseline/function-names.md" "$AR/function-names.md" || true)"
    imports_file="$(first_existing_file "$AR/baseline/imports-and-libraries.md" "$AR/imports-and-libraries.md" || true)"
    strings_file="$(first_existing_file "$AR/baseline/strings-and-constants.md" "$AR/strings-and-constants.md" || true)"
    types_file="$(first_existing_file "$AR/baseline/types-and-structs.md" "$AR/types-and-structs.md" || true)"
    xrefs_file="$(first_existing_file "$AR/baseline/xrefs-and-callgraph.md" "$AR/xrefs-and-callgraph.md" || true)"
    decomp_file="$(first_existing_file "$AR/baseline/decompiled-output.md" "$AR/decompiled-output.md" || true)"

    [[ -n "$fn_file" ]] && r="pass" || r="fail"
    check P1_01 "function-names.md exists" blocking "$r"
    [[ -n "$imports_file" ]] && r="pass" || r="fail"
    check P1_02 "imports-and-libraries.md exists" blocking "$r"
    [[ -n "$strings_file" ]] && r="pass" || r="fail"
    check P1_03 "strings-and-constants.md exists" blocking "$r"
    [[ -n "$types_file" ]] && r="pass" || r="fail"
    check P1_04 "types-and-structs.md exists" blocking "$r"
    [[ -n "$xrefs_file" ]] && r="pass" || r="fail"
    check P1_05 "xrefs-and-callgraph.md exists" blocking "$r"
    if [[ -n "$decomp_file" ]] && rg -n 'Decompiled bodies are intentionally blocked in this stage\.' "$decomp_file" >/dev/null 2>&1; then
      r="pass"
    else
      r="fail"
    fi
    check P1_06 "decompiled-output.md retains blocked placeholder" blocking "$r"

    for f in "$fn_file" "$imports_file" "$strings_file" "$types_file" "$xrefs_file" "$decomp_file"; do
      [[ -n "$f" ]] || continue
      check "P1_07_$(basename "$f" .md)" "$(basename "$f") is non-empty" blocking "$(file_nonempty "$f")"
    done

    fn_count=$(markdown_table_data_row_count "${fn_file:-/nonexistent}" '## Observed Functions')
    [[ "$fn_count" -ge 1 ]] && r="pass" || r="fail"
    check P1_08 "function-names has >= 1 function row" warning "$r"
    ;;

  P2)
    evidence_md="$(first_existing_file "$AR/evidence/evidence-candidates.md" "$AR/evidence-candidates.md" || true)"
    evidence_yaml="$(first_existing_file "$AR/evidence/evidence-candidates.yaml" || true)"
    library_yaml="$(first_existing_file "$AR/evidence/library-identification.yaml" || true)"
    anchor_yaml="$(first_existing_file "$AR/evidence/anchor-summary.yaml" || true)"

    if [[ -n "$evidence_md" ]]; then
      check P2_01 "evidence-candidates.md exists" blocking "$(file_exists "$evidence_md")"
      check P2_02 "evidence-candidates.md is non-empty" blocking "$(file_nonempty "$evidence_md")"
      check P2_03 "frontier candidate table present" blocking "$(text_contains "$evidence_md" '^## Frontier Candidate Rows$')"
      candidate_rows=$(markdown_table_data_row_count "$evidence_md" '## Frontier Candidate Rows')
      [[ "$candidate_rows" -ge 1 ]] && r="pass" || r="fail"
      check P2_04 "evidence review exports at least 1 candidate row" blocking "$r"
      check P2_05 "candidate table includes frontier reasoning columns" blocking "$(markdown_section_contains "$evidence_md" '## Frontier Candidate Rows' 'Frontier Basis .* Triggering Evidence')"
      check P2_06 "review prompts recorded" warning "$(text_contains "$evidence_md" '^## Recommended Review Prompts$')"
    else
      check P2_01 "evidence-candidates.yaml exists" blocking "$(file_exists "${evidence_yaml:-/nonexistent}")"
      check P2_02 "library-identification.yaml exists" blocking "$(file_exists "${library_yaml:-/nonexistent}")"
      check P2_03 "anchor-summary.yaml exists" blocking "$(file_exists "${anchor_yaml:-/nonexistent}")"

      anchor_count=$(yaml_array_length "${anchor_yaml:-/nonexistent}" '.anchors')
      [[ "$anchor_count" -ge 1 ]] && r="pass" || r="fail"
      check P2_04 "anchor-summary has >= 1 anchor" blocking "$r"
      check P2_05 "every anchor has address + frontier_reason" blocking "$(yaml_array_all_have_fields "${anchor_yaml:-/nonexistent}" '.anchors' address frontier_reason)"
      check P2_06 "every library has confidence + evidence" blocking "$(yaml_array_all_have_fields "${library_yaml:-/nonexistent}" '.libraries' confidence evidence)"
    fi
    ;;

  P3)
    target_selection_md="$(first_existing_file "$AR/target-selection.md" "$AR/evidence/target-selection.md" || true)"
    if [[ -n "$target_selection_md" && -z "$ITERATION" ]]; then
      check P3_01 "target-selection.md exists" blocking "$(file_exists "$target_selection_md")"
      check P3_02 "automatic default selection recorded" blocking "$(text_contains "$target_selection_md" '^## Automatic Default Selection$')"
      check P3_03 "selection fields include selected target, frontier reason, and question" blocking "$(
        if [[ "$(markdown_section_contains "$target_selection_md" '## Automatic Default Selection' '^\| Selected Target \|')" == "pass" && \
              "$(markdown_section_contains "$target_selection_md" '## Automatic Default Selection' '^\| Frontier Reason \|')" == "pass" && \
              "$(markdown_section_contains "$target_selection_md" '## Automatic Default Selection' '^\| Question To Answer \|')" == "pass" ]]; then
          echo pass
        else
          echo fail
        fi
      )"
      check P3_04 "candidate selection rows table present" blocking "$(text_contains "$target_selection_md" '^## Candidate Selection Rows$')"
      check P3_05 "at least one row is marked ready or selected as default" blocking "$(
        if [[ "$(markdown_section_contains "$target_selection_md" '## Candidate Selection Rows' '^\| yes \|')" == "pass" || \
              "$(markdown_section_contains "$target_selection_md" '## Candidate Selection Rows' '\| ready \|$')" == "pass" ]]; then
          echo pass
        else
          echo fail
        fi
      )"
    else
      [[ -z "$ITERATION" ]] && { echo '{"error":"--iteration required for legacy P3 batch-manifest validation"}'; exit 1; }
      manifest="$AR/iterations/$ITERATION/batch-manifest.yaml"
      check P3_01 "batch-manifest.yaml exists" blocking "$(file_exists "$manifest")"

      fn_count=$(yaml_array_length "$manifest" '.functions')
      [[ "$fn_count" -ge 1 ]] && r="pass" || r="fail"
      check P3_02 "functions list non-empty" blocking "$r"
      check P3_03 "every function has address + frontier_reason + question_to_answer" blocking "$(yaml_array_all_have_fields "$manifest" '.functions' address frontier_reason question_to_answer)"
      check P3_04 "no duplicate addresses" blocking "$(yaml_no_duplicate_values "$manifest" '.functions' address)"
      check P3_05 "every function status == pending" blocking "$(yaml_all_field_equals "$manifest" '.functions' status pending)"
    fi
    ;;

  P5)
    [[ -z "$ITERATION" || -z "$FUNCTION" ]] && { echo '{"error":"--iteration and --function required for P5"}'; exit 1; }
    fn_dir="$AR/iterations/$ITERATION/functions/$FUNCTION"

    has_c=$(find "$fn_dir/decompiled-output" -name '*.c' 2>/dev/null | head -1)
    [[ -n "$has_c" ]] && r="pass" || r="fail"
    check P5_01 "decompiled-output/ contains .c file" blocking "$r"
    check P5_02 "decompilation-record.yaml exists" blocking "$(file_exists "$fn_dir/decompilation-record.yaml")"
    check P5_03 "decompilation-record has all required fields" blocking "$(yaml_object_fields_nonempty "$fn_dir/decompilation-record.yaml" '.function_id' '.function_name' '.address' '.decompiled_source' '.decompilation_backend' '.decompilation_action')"

    backend_status="$(yaml_field_equals "$fn_dir/decompilation-record.yaml" '.decompilation_backend' 'ghidra_headless')"
    action_status="$(yaml_field_equals "$fn_dir/decompilation-record.yaml" '.decompilation_action' 'decompile-selected')"
    if [[ "$backend_status" == "pass" && "$action_status" == "pass" ]]; then
      r="pass"
    else
      r="fail"
    fi
    check P5_04 "decompilation provenance is ghidra_headless via decompile-selected" blocking "$r"

    check P5_05 "semantic-record.yaml exists" blocking "$(file_exists "$fn_dir/semantic-record.yaml")"
    check P5_06 "role/name/prototype evidence has >= 2 items" blocking "$(
      if command -v yq &>/dev/null && [[ -f "$fn_dir/semantic-record.yaml" ]]; then
        role=$(yq -r '.role_evidence // [] | length' "$fn_dir/semantic-record.yaml" 2>/dev/null || echo "0")
        name=$(yq -r '.name_evidence // [] | length' "$fn_dir/semantic-record.yaml" 2>/dev/null || echo "0")
        proto=$(yq -r '.prototype_evidence // [] | length' "$fn_dir/semantic-record.yaml" 2>/dev/null || echo "0")
        total=$((role + name + proto))
        [[ "$total" -ge 2 ]] && echo "pass" || echo "fail"
      else
        echo "fail"
      fi
    )"
    check P5_07 "source-comparison.yaml exists" blocking "$(file_exists "$fn_dir/source-comparison.yaml")"
    check P5_08 "reference_status is set" blocking "$(yaml_field_nonempty "$fn_dir/source-comparison.yaml" '.reference_status')"
    check P5_09 "verify-report has no failed entries" blocking "$(
      if [[ -f "$fn_dir/verify-report.yaml" ]] && command -v yq &>/dev/null; then
        failed=$(yq -r '.results // [] | map(select(.status == "failed")) | length' "$fn_dir/verify-report.yaml" 2>/dev/null || echo "1")
        [[ "$failed" -eq 0 ]] && echo "pass" || echo "fail"
      else
        echo "fail"
      fi
    )"

    # P5_09: reconstruction project .c + .h written
    recon_root="$(derive_reconstruction_root "$AR")"
    fn_name=""
    if command -v yq &>/dev/null && [[ -f "$fn_dir/decompilation-record.yaml" ]]; then
      fn_name=$(yq -r '.function_name // ""' "$fn_dir/decompilation-record.yaml" 2>/dev/null || echo "")
    fi
    if [[ -n "$fn_name" ]]; then
      has_src="fail"; has_hdr="fail"
      [[ -f "$recon_root/src/${fn_name}.c" ]] && has_src="pass"
      [[ -f "$recon_root/include/${fn_name}.h" ]] && has_hdr="pass"
      [[ "$has_src" == "pass" && "$has_hdr" == "pass" ]] && r="pass" || r="fail"
    else
      r="fail"
    fi
    check P5_10 "reconstruction project .c + .h written" blocking "$r"
    check P5_11 "reconstruction-manifest.yaml updated" blocking "$(file_exists "$recon_root/reconstruction-manifest.yaml")"
    ;;

  P6)
    [[ -z "$ITERATION" || -z "$FUNCTION" ]] && { echo '{"error":"--iteration and --function required for P6"}'; exit 1; }
    fn_dir="$AR/iterations/$ITERATION/functions/$FUNCTION"

    has_inputs=$(find "$fn_dir/test-inputs" -name '*.yaml' 2>/dev/null | head -1)
    [[ -n "$has_inputs" ]] && r="pass" || r="fail"
    check P6_01 "test-inputs/ has >= 1 source file" blocking "$r"
    check P6_02 "frida-io-recording.yaml exists" blocking "$(file_exists "$fn_dir/frida-io-recording.yaml")"
    check P6_03 "verification-result.yaml exists" blocking "$(file_exists "$fn_dir/verification-result.yaml")"
    check P6_04 "status == verified" blocking "$(yaml_field_equals "$fn_dir/verification-result.yaml" '.status' 'verified')"
    check P6_05 "test_summary.overall.failed == 0" blocking "$(yaml_field_numeric_equals "$fn_dir/verification-result.yaml" '.test_summary.overall.failed' '0')"
    check P6_06 "gate_verdict == pass" blocking "$(yaml_field_equals "$fn_dir/verification-result.yaml" '.gate_verdict' 'pass')"
    ;;

  *)
    echo "{\"error\":\"unknown gate: $GATE\"}"
    exit 1
    ;;
esac

# Output JSON
checks_json=""
failures_json=""
warnings_json=""

if [[ ${#checks[@]} -gt 0 ]]; then
  checks_json=$(printf '%s,' "${checks[@]}" | sed 's/,$//')
fi
if [[ ${#failures[@]} -gt 0 ]]; then
  failures_json=$(printf '\"%s\",' "${failures[@]}" | sed 's/,$//')
fi
if [[ ${#warnings[@]} -gt 0 ]]; then
  warnings_json=$(printf '\"%s\",' "${warnings[@]}" | sed 's/,$//')
fi

echo "{\"gate\":\"$GATE\",\"exit_code\":$exit_code,\"checks\":[$checks_json],\"failures\":[$failures_json],\"warnings\":[$warnings_json]}"
exit $exit_code
