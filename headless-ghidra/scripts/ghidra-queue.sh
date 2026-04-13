#!/usr/bin/env bash
# ghidra-queue.sh — FIFO lock manager for serializing Ghidra project operations.
# Usage:
#   ghidra-queue.sh acquire --lock-file <path> --holder <agent-id> [--timeout <seconds>]
#   ghidra-queue.sh release --lock-file <path> --holder <agent-id>
#   ghidra-queue.sh status  --lock-file <path>
set -euo pipefail

ACTION="${1:-}"
LOCK_FILE=""
HOLDER=""
TIMEOUT=300

shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --lock-file) LOCK_FILE="$2"; shift 2 ;;
    --holder) HOLDER="$2"; shift 2 ;;
    --timeout) TIMEOUT="$2"; shift 2 ;;
    *) shift ;;
  esac
done

[[ -z "$LOCK_FILE" ]] && { echo "error: --lock-file required" >&2; exit 1; }

# Format: holder:PID
lock_value() { printf '%s:%s' "$1" "$$"; }

is_pid_alive() {
  local pid="$1"
  [[ -n "$pid" && "$pid" -gt 0 ]] && kill -0 "$pid" 2>/dev/null
}

is_stale_lock() {
  local current="$1"
  [[ "$current" != *":" ]] && return 1
  local stored_holder="${current%%:*}"
  local stored_pid="${current##*:}"
  if [[ -z "$stored_pid" || "$stored_pid" == "$stored_holder" ]]; then
    return 1
  fi
  if ! is_pid_alive "$stored_pid"; then
    return 0
  fi
  return 1
}

acquire_lock() {
  local holder="$1"
  local lock_value_str="$(lock_value "$holder")"
  if (set -o noclobber; echo "$lock_value_str" > "$LOCK_FILE") 2>/dev/null; then
    echo "{\"status\":\"acquired\",\"holder\":\"$holder\",\"pid\":$$,\"lock_file\":\"$LOCK_FILE\"}"
    return 0
  fi
  return 1
}

case "$ACTION" in
  acquire)
    [[ -z "$HOLDER" ]] && { echo "error: --holder required for acquire" >&2; exit 1; }
    lock_dir="$(dirname "$LOCK_FILE")"
    mkdir -p "$lock_dir"

    elapsed=0
    while true; do
      if [[ -f "$LOCK_FILE" ]]; then
        current=$(cat "$LOCK_FILE" 2>/dev/null || echo "")
        if [[ -n "$current" ]] && is_stale_lock "$current"; then
          echo "{\"status\":\"stale_recovered\",\"previous\":\"$current\"}" >&2
          rm -f "$LOCK_FILE"
        fi
      fi

      if acquire_lock "$HOLDER"; then
        exit 0
      fi

      if [[ $elapsed -ge $TIMEOUT ]]; then
        echo "{\"status\":\"timeout\",\"holder\":\"$(cat "$LOCK_FILE" 2>/dev/null || echo 'unknown')\",\"waited\":$elapsed}"
        exit 1
      fi
      sleep 1
      elapsed=$((elapsed + 1))
    done
    ;;

  release)
    [[ -z "$HOLDER" ]] && { echo "error: --holder required for release" >&2; exit 1; }
    if [[ -f "$LOCK_FILE" ]]; then
      current=$(cat "$LOCK_FILE" 2>/dev/null || echo "")
      stored_holder="${current%%:*}"
      if [[ "$stored_holder" == "$HOLDER" ]]; then
        rm -f "$LOCK_FILE"
        echo "{\"status\":\"released\",\"holder\":\"$HOLDER\"}"
      else
        echo "{\"status\":\"error\",\"message\":\"lock held by $stored_holder, not $HOLDER\"}"
        exit 1
      fi
    else
      echo "{\"status\":\"released\",\"message\":\"lock was not held\"}"
    fi
    ;;

  status)
    if [[ -f "$LOCK_FILE" ]]; then
      current=$(cat "$LOCK_FILE" 2>/dev/null || echo "unknown")
      echo "{\"status\":\"locked\",\"holder\":\"${current%%:*}\"}"
    else
      echo "{\"status\":\"free\"}"
    fi
    ;;

  *)
    echo "Usage: ghidra-queue.sh {acquire|release|status} --lock-file <path> [--holder <id>]" >&2
    exit 1
    ;;
esac
