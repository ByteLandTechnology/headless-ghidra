#!/usr/bin/env bash
# gate-check.sh — Programmatic gate validation for the headless Ghidra pipeline.
# Returns: 0 = pass, 1 = fail (blocking), 2 = warn (conditional)
# Output: JSON to stdout
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  gate-check.sh --gate <P0|P1|P2|P3|P5|P6> --artifact-root <path> [--iteration <NNN>] [--function <fn_id>]

Gates P0–P2 are phase-level. P3 requires --iteration. P5/P6 require --iteration and --function.

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
    recon_root="${AR/ghidra-artifacts/reconstruction}"
    check P0_11 "reconstruction directory exists" blocking "$(dir_exists "$recon_root")"
    check P0_12 "reconstruction-manifest.yaml exists" blocking "$(file_exists "$recon_root/reconstruction-manifest.yaml")"
    ;;

  P1)
    for f in function-names imports-and-libraries strings-and-constants types-and-structs xrefs-and-callgraph; do
      check "P1_$(printf '%02d' $((++_p1_idx)))" "baseline/${f}.yaml exists" blocking "$(file_exists "$AR/baseline/${f}.yaml")"
    done
    _p1_idx=5
    check P1_06 "decompiled-output.yaml exists with empty functions" blocking "$(file_exists "$AR/baseline/decompiled-output.yaml")"

    for f in function-names imports-and-libraries strings-and-constants types-and-structs xrefs-and-callgraph decompiled-output; do
      check "P1_07_${f}" "baseline/${f}.yaml parseable" blocking "$(yaml_parseable "$AR/baseline/${f}.yaml")"
    done

    fn_count=$(yaml_array_length "$AR/baseline/function-names.yaml" '.functions')
    [[ "$fn_count" -ge 1 ]] && r="pass" || r="fail"
    check P1_08 "function-names has >= 1 function" warning "$r"
    ;;

  P2)
    check P2_01 "evidence-candidates.yaml exists" blocking "$(file_exists "$AR/evidence/evidence-candidates.yaml")"
    check P2_02 "library-identification.yaml exists" blocking "$(file_exists "$AR/evidence/library-identification.yaml")"
    check P2_03 "anchor-summary.yaml exists" blocking "$(file_exists "$AR/evidence/anchor-summary.yaml")"

    anchor_count=$(yaml_array_length "$AR/evidence/anchor-summary.yaml" '.anchors')
    [[ "$anchor_count" -ge 1 ]] && r="pass" || r="fail"
    check P2_04 "anchor-summary has >= 1 anchor" blocking "$r"
    check P2_05 "every anchor has address + frontier_reason" blocking "$(yaml_array_all_have_fields "$AR/evidence/anchor-summary.yaml" '.anchors' address frontier_reason)"
    check P2_06 "every library has confidence + evidence" blocking "$(yaml_array_all_have_fields "$AR/evidence/library-identification.yaml" '.libraries' confidence evidence)"
    ;;

  P3)
    [[ -z "$ITERATION" ]] && { echo '{"error":"--iteration required for P3"}'; exit 1; }
    manifest="$AR/iterations/$ITERATION/batch-manifest.yaml"
    check P3_01 "batch-manifest.yaml exists" blocking "$(file_exists "$manifest")"

    fn_count=$(yaml_array_length "$manifest" '.functions')
    [[ "$fn_count" -ge 1 ]] && r="pass" || r="fail"
    check P3_02 "functions list non-empty" blocking "$r"
    check P3_03 "every function has address + frontier_reason + question_to_answer" blocking "$(yaml_array_all_have_fields "$manifest" '.functions' address frontier_reason question_to_answer)"
    check P3_04 "no duplicate addresses" blocking "$(yaml_no_duplicate_values "$manifest" '.functions' address)"
    check P3_05 "every function status == pending" blocking "$(yaml_all_field_equals "$manifest" '.functions' status pending)"
    ;;

  P5)
    [[ -z "$ITERATION" || -z "$FUNCTION" ]] && { echo '{"error":"--iteration and --function required for P5"}'; exit 1; }
    fn_dir="$AR/iterations/$ITERATION/functions/$FUNCTION"

    has_c=$(find "$fn_dir/decompiled-output" -name '*.c' 2>/dev/null | head -1)
    [[ -n "$has_c" ]] && r="pass" || r="fail"
    check P5_01 "decompiled-output/ contains .c file" blocking "$r"
    check P5_02 "decompilation-record.yaml exists" blocking "$(file_exists "$fn_dir/decompilation-record.yaml")"
    check P5_03 "decompilation-record has all required fields" blocking "$(yaml_array_all_have_fields "$fn_dir/decompilation-record.yaml" '.' function_id function_name address decompiled_source)"
    check P5_04 "semantic-record.yaml exists" blocking "$(file_exists "$fn_dir/semantic-record.yaml")"
    check P5_05 "role/name/prototype evidence has >= 2 items" blocking "$(
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
    check P5_06 "source-comparison.yaml exists" blocking "$(file_exists "$fn_dir/source-comparison.yaml")"
    check P5_07 "reference_status is set" blocking "$(yaml_field_nonempty "$fn_dir/source-comparison.yaml" '.reference_status')"
    check P5_08 "verify-report has no failed entries" blocking "$(
      if [[ -f "$fn_dir/verify-report.yaml" ]] && command -v yq &>/dev/null; then
        failed=$(yq -r '.results // [] | map(select(.status == "failed")) | length' "$fn_dir/verify-report.yaml" 2>/dev/null || echo "1")
        [[ "$failed" -eq 0 ]] && echo "pass" || echo "fail"
      else
        echo "fail"
      fi
    )"

    # P5_09: reconstruction project .c + .h written
    recon_root="${AR/ghidra-artifacts/reconstruction}"
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
    check P5_09 "reconstruction project .c + .h written" blocking "$r"
    check P5_10 "reconstruction-manifest.yaml updated" blocking "$(file_exists "$recon_root/reconstruction-manifest.yaml")"
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
checks_json=$(printf '%s,' "${checks[@]}" | sed 's/,$//')
echo "{\"gate\":\"$GATE\",\"exit_code\":$exit_code,\"checks\":[$checks_json],\"failures\":[$(printf '\"%s\",' "${failures[@]}" | sed 's/,$//')],\"warnings\":[$(printf '\"%s\",' "${warnings[@]}" | sed 's/,$//')]}"
exit $exit_code
