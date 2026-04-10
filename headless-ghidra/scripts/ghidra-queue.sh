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

case "$ACTION" in
  acquire)
    [[ -z "$HOLDER" ]] && { echo "error: --holder required for acquire" >&2; exit 1; }
    lock_dir="$(dirname "$LOCK_FILE")"
    mkdir -p "$lock_dir"

    elapsed=0
    while ! (set -o noclobber; echo "$HOLDER" > "$LOCK_FILE") 2>/dev/null; do
      current=$(cat "$LOCK_FILE" 2>/dev/null || echo "unknown")
      if [[ $elapsed -ge $TIMEOUT ]]; then
        echo "{\"status\":\"timeout\",\"holder\":\"$current\",\"waited\":$elapsed}"
        exit 1
      fi
      sleep 1
      elapsed=$((elapsed + 1))
    done
    echo "{\"status\":\"acquired\",\"holder\":\"$HOLDER\",\"lock_file\":\"$LOCK_FILE\"}"
    ;;

  release)
    [[ -z "$HOLDER" ]] && { echo "error: --holder required for release" >&2; exit 1; }
    if [[ -f "$LOCK_FILE" ]]; then
      current=$(cat "$LOCK_FILE" 2>/dev/null || echo "")
      if [[ "$current" == "$HOLDER" ]]; then
        rm -f "$LOCK_FILE"
        echo "{\"status\":\"released\",\"holder\":\"$HOLDER\"}"
      else
        echo "{\"status\":\"error\",\"message\":\"lock held by $current, not $HOLDER\"}"
        exit 1
      fi
    else
      echo "{\"status\":\"released\",\"message\":\"lock was not held\"}"
    fi
    ;;

  status)
    if [[ -f "$LOCK_FILE" ]]; then
      current=$(cat "$LOCK_FILE" 2>/dev/null || echo "unknown")
      echo "{\"status\":\"locked\",\"holder\":\"$current\"}"
    else
      echo "{\"status\":\"free\"}"
    fi
    ;;

  *)
    echo "Usage: ghidra-queue.sh {acquire|release|status} --lock-file <path> [--holder <id>]" >&2
    exit 1
    ;;
esac
