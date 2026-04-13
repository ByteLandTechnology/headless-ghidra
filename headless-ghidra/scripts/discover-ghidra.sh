#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF'
Usage: discover-ghidra.sh [options]

Options:
  --install-dir PATH         Prefer a specific Ghidra install directory.
  --print-install-dir        Print only the resolved install directory.
  --print-analyze-headless   Print only the resolved analyzeHeadless path.
  --show-help                Run the discovered analyzeHeadless help command.
  -h, --help                 Show this message.

Discovery order:
  1. --install-dir
  2. GHIDRA_INSTALL_DIR / GHIDRA_HOME
  3. analyzeHeadless on PATH
  4. Common local install roots under /Applications and $HOME
EOF
}

emit_missing_guidance() {
  cat <<'EOF' >&2
Ghidra was not found.

Next steps:
1. Install Ghidra locally, or
2. Provide a path with --install-dir /path/to/ghidra, or
3. Export GHIDRA_INSTALL_DIR=/path/to/ghidra

After installation, rerun:
  <installed-skill-root>/scripts/discover-ghidra.sh --show-help
EOF
}

normalize_install_dir() {
  local candidate="$1"
  local resolved=""
  if [[ -z "${candidate}" ]]; then
    return 1
  fi

  if [[ -d "${candidate}" ]]; then
    resolved="$(cd "${candidate}" && pwd -P)"
  else
    return 1
  fi

  case "${resolved}" in
    "${HOME}/Library/ghidra"|${HOME}/Library/ghidra/*)
      return 1
      ;;
  esac

  if [[ -d "${resolved}/support" ]]; then
    printf '%s\n' "${resolved}"
    return 0
  fi

  if [[ -d "${resolved}/libexec/support" ]]; then
    printf '%s\n' "${resolved}/libexec"
    return 0
  fi

  if [[ -d "${resolved}/Contents/Resources/Java/support" ]]; then
    printf '%s\n' "${resolved}/Contents/Resources/Java"
    return 0
  fi

  return 1
}

analyze_headless_from_install() {
  local install_dir="$1"
  if [[ -x "${install_dir}/support/analyzeHeadless" ]]; then
    printf '%s\n' "${install_dir}/support/analyzeHeadless"
    return 0
  fi

  if [[ -x "${install_dir}/libexec/support/analyzeHeadless" ]]; then
    printf '%s\n' "${install_dir}/libexec/support/analyzeHeadless"
    return 0
  fi

  return 1
}

search_common_installs() {
  local roots=(
    "/Applications"
    "$HOME/Applications"
    "$HOME/tools"
    "$HOME/Tools"
    "$HOME/Downloads"
    "$HOME/opt"
    "/opt/homebrew/opt"
    "/opt/homebrew/Cellar"
    "/usr/local/opt"
    "/usr/local/Cellar"
  )
  local root=""
  local candidate=""
  for root in "${roots[@]}"; do
    [[ -d "${root}" ]] || continue
    while IFS= read -r candidate; do
      resolved="$(normalize_install_dir "${candidate}" || true)"
      if [[ -n "${resolved}" ]]; then
        printf '%s\n' "${resolved}"
        return 0
      fi
    done < <(find "${root}" -maxdepth 3 \( -name 'ghidra*' -o -name 'Ghidra.app' \) 2>/dev/null | sort)
  done

  return 1
}

run_help() {
  local analyze_headless="$1"
  local help_output=""
  local help_status=0
  local flag=""

  for flag in -help --help -h; do
    help_output="$("${analyze_headless}" "${flag}" 2>&1)" && help_status=0 || help_status=$?
    if [[ "${help_output}" == *"Headless Analyzer Usage"* ]]; then
      printf '%s\n' "${help_output}"
      exit 0
    fi
  done

  printf '%s\n' "${help_output}" >&2
  exit "${help_status}"
}

INSTALL_DIR_ARG=""
PRINT_INSTALL_DIR=0
PRINT_ANALYZE_HEADLESS=0
SHOW_HELP=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-dir)
      INSTALL_DIR_ARG="${2:-}"
      shift 2
      ;;
    --print-install-dir)
      PRINT_INSTALL_DIR=1
      shift
      ;;
    --print-analyze-headless)
      PRINT_ANALYZE_HEADLESS=1
      shift
      ;;
    --show-help)
      SHOW_HELP=1
      shift
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

INSTALL_DIR=""
ANALYZE_HEADLESS=""

if [[ -n "${INSTALL_DIR_ARG}" ]]; then
  INSTALL_DIR="$(normalize_install_dir "${INSTALL_DIR_ARG}" || true)"
fi

if [[ -z "${INSTALL_DIR}" && -n "${GHIDRA_INSTALL_DIR:-}" ]]; then
  INSTALL_DIR="$(normalize_install_dir "${GHIDRA_INSTALL_DIR}" || true)"
fi

if [[ -z "${INSTALL_DIR}" && -n "${GHIDRA_HOME:-}" ]]; then
  INSTALL_DIR="$(normalize_install_dir "${GHIDRA_HOME}" || true)"
fi

if [[ -z "${INSTALL_DIR}" ]]; then
  ANALYZE_HEADLESS="$(command -v analyzeHeadless || true)"
  if [[ -n "${ANALYZE_HEADLESS}" ]]; then
    INSTALL_DIR="$(cd "$(dirname "${ANALYZE_HEADLESS}")/.." && pwd)"
  fi
fi

if [[ -z "${INSTALL_DIR}" ]]; then
  INSTALL_DIR="$(search_common_installs || true)"
fi

if [[ -n "${INSTALL_DIR}" ]]; then
  ANALYZE_HEADLESS="$(analyze_headless_from_install "${INSTALL_DIR}" || true)"
fi

if [[ -z "${ANALYZE_HEADLESS}" ]]; then
  emit_missing_guidance
  exit 1
fi

if [[ "${PRINT_INSTALL_DIR}" -eq 1 ]]; then
  printf '%s\n' "${INSTALL_DIR}"
  exit 0
fi

if [[ "${PRINT_ANALYZE_HEADLESS}" -eq 1 ]]; then
  printf '%s\n' "${ANALYZE_HEADLESS}"
  exit 0
fi

if [[ "${SHOW_HELP}" -eq 1 ]]; then
  run_help "${ANALYZE_HEADLESS}"
fi

cat <<EOF
GHIDRA_INSTALL_DIR=${INSTALL_DIR}
ANALYZE_HEADLESS=${ANALYZE_HEADLESS}
NEXT_HELP_COMMAND=${ANALYZE_HEADLESS} -help
EOF
