#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REPO_URL="https://github.com/apple-oss-distributions/file_cmds.git"
DEFAULT_FILE_CMDS_COMMIT="afe96c16b2d4e6c0b52f46277a3d8c8433bce057"

WORKSPACE_ROOT=""
VALIDATION_ROOT=""
SOURCE_DIR=""
BUILD_DIR=""
REPO_URL="${DEFAULT_REPO_URL}"
FILE_CMDS_COMMIT="${DEFAULT_FILE_CMDS_COMMIT}"
FORCE_CLEAN=0
REUSE_EXISTING_BUILD=0
OUTPUT_JSON=0
RUN_SMOKE_TESTS=0
RUN_FRIDA_SMOKE_TEST=0

usage() {
  cat <<'EOF'
Usage: build-mock-ls-for-verify.sh [options]

Build the mock verification `ls` binary used by verification-only runtime
checks. This script does not write to the formal skill artifact contract.

Options:
  --workspace-root PATH      Workspace root. Defaults to the git repo root,
                             then the current working directory.
  --verification-root PATH   Verification root. Defaults to
                             <workspace-root>/.work/verification/mock-ls.
  --validation-root PATH     Backward-compatible alias for --verification-root.
  --source-dir PATH          Checkout dir. Defaults to
                             <verification-root>/src/file_cmds.
  --build-dir PATH           Build dir. Defaults to <verification-root>/build.
  --repo-url URL             Source repo URL. Defaults to the Apple OSS mirror.
  --commit SHA               Fixed file_cmds commit to build.
  --force-clean              Reset a dirty validation checkout before building.
  --reuse-existing-build     Return an existing executable build if present.
  --smoke-test               Run deterministic fixture cases against the mock ls.
  --frida-smoke-test         Attempt a minimal Frida spawn->attach smoke test.
  --output-json              Print a JSON object instead of a bare path.
  -h, --help                 Show this message.

Output:
  Prints the mock verification binary path on stdout, or JSON when
  --output-json is present.
EOF
}

log() {
  printf '[build-mock-ls-for-verify] %s\n' "$*" >&2
}

json_escape() {
  python3 -c 'import json, sys; print(json.dumps(sys.argv[1]))' "$1"
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

detect_workspace_root() {
  local explicit_root="${1:-}"
  local git_root=""

  if [[ -n "${explicit_root}" ]]; then
    mkdir -p "${explicit_root}"
    cd "${explicit_root}" && pwd -P
    return 0
  fi

  if git_root="$(git -C "${PWD}" rev-parse --show-toplevel 2>/dev/null)"; then
    printf '%s\n' "${git_root}"
    return 0
  fi

  pwd -P
}

mock_ls_patch() {
  cat <<'EOF'
diff --git a/ls/ls.c b/ls/ls.c
index 74435a5..08e1caa 100644
--- a/ls/ls.c
+++ b/ls/ls.c
@@ -80,7 +80,6 @@ __FBSDID("$FreeBSD$");
 #include <sys/param.h>
 #include <get_compat.h>
 #include <sys/sysctl.h>
-#include <System/sys/fsctl.h>
 #include <TargetConditionals.h>
 #endif /* __APPLE__ */
 #include "ls.h"
@@ -871,8 +870,8 @@ display(const FTSENT *p, FTSENT *list, int options)
 	u_int64_t btotal, labelstrlen, maxlen, maxnlink;
 #ifndef __APPLE__
 	u_int64_t maxlattr;
-#endif // ! __APPLE__
 	u_int64_t maxlabelstr;
+#endif // ! __APPLE__
 	u_int sizelen;
 	int maxflags;
 	gid_t maxgroup;
@@ -881,7 +880,10 @@ display(const FTSENT *p, FTSENT *list, int options)
 	char *initmax;
 	int entries, needstats;
 	const char *user, *group;
-	char *flags, *labelstr = NULL;
+	char *flags;
+#ifndef __APPLE__
+	char *labelstr = NULL;
+#endif
 #ifndef __APPLE__
 	char buf[STRBUF_SIZEOF(u_quad_t) + 1];
 #endif // ! __APPLE__
@@ -938,7 +940,9 @@ display(const FTSENT *p, FTSENT *list, int options)
 	maxflags = width[5];
 	maxsize = width[6];
 	maxlen = width[7];
+#ifndef __APPLE__
 	maxlabelstr = width[8];
+#endif
 
 	MAKENINES(maxinode);
 	MAKENINES(maxblock);
@@ -1036,8 +1040,8 @@ display(const FTSENT *p, FTSENT *list, int options)
 						maxflags = flen;
 				} else
 					flen = 0;
-				labelstr = NULL;
 #ifndef __APPLE__
+				labelstr = NULL;
 				if (f_label) {
 					char name[PATH_MAX + 1];
 					mac_t label;
@@ -1091,9 +1095,10 @@ label_out:
 						maxlabelstr = labelstrlen;
 				} else
 					labelstrlen = 0;
-#else
-				labelstrlen = 0;
 #endif /* !__APPLE__ */
+#ifdef __APPLE__
+				labelstrlen = 0;
+#endif
 
 				if ((np = calloc(1, sizeof(NAMES) + labelstrlen +
 				    ulen + glen + flen + 4)) == NULL)
diff --git a/ls/print.c b/ls/print.c
index a72277c..2c66c68 100644
--- a/ls/print.c
+++ b/ls/print.c
@@ -50,7 +50,6 @@ __FBSDID("$FreeBSD$");
 #include <pwd.h>
 #include <TargetConditionals.h>
 #include <membership.h>
-#include <membershipPriv.h>
 #include <uuid/uuid.h>
 #endif
 
@@ -58,7 +57,7 @@ __FBSDID("$FreeBSD$");
 #include <errno.h>
 #include <fts.h>
 #include <langinfo.h>
-#include <libutil.h>
+#include <util.h>
 #include <limits.h>
 #include <stdio.h>
 #include <stdint.h>
@@ -78,6 +77,12 @@ __FBSDID("$FreeBSD$");
 #include "ls.h"
 #include "extern.h"
 
+static int
+compat_humanize_number(char *buf, size_t len, int64_t bytes)
+{
+	return snprintf(buf, len, "%lld", (long long)bytes);
+}
+
 static int	printaname(const FTSENT *, u_long, u_long);
 static void	printdev(size_t, dev_t);
 static void	printlink(const FTSENT *);
@@ -276,32 +281,17 @@ static struct {
 static char *
 uuid_to_name(uuid_t *uu) 
 {
-	int type;
 	char *name = NULL;
-	char *recname = NULL;
-	
-#define MAXNAMETAG (MAXLOGNAME + 6) /* + strlen("group:") */
-	name = (char *) malloc(MAXNAMETAG);
+	size_t max_name_len;
+
+	max_name_len = 37;
+	name = (char *) malloc(max_name_len);
 	
 	if (NULL == name) {
 		err(1, "malloc");
 	}
 	
-	if (f_numericonly) {
-		goto errout;
-	}
-	
-	if (mbr_identifier_translate(ID_TYPE_UUID, *uu, sizeof(*uu), ID_TYPE_NAME, (void **) &recname, &type)) {
-		goto errout;
-	}
-	
-	snprintf(name, MAXNAMETAG, "%s:%s", (type == MBR_REC_TYPE_USER ? "user" : "group"), recname);
-	free(recname);
-	
-	return name;
-errout:
 	uuid_unparse_upper(*uu, name);
-	
 	return name;
 }
 
@@ -999,8 +989,7 @@ printsize(size_t width, off_t bytes)
 		 */
 		char buf[HUMANVALSTR_LEN - 1 + 1];
 
-		humanize_number(buf, sizeof(buf), (int64_t)bytes, "",
-		    HN_AUTOSCALE, HN_B | HN_NOSPACE | HN_DECIMAL);
+		compat_humanize_number(buf, sizeof(buf), (int64_t)bytes);
 		(void)printf("%*s ", (u_int)width, buf);
 	} else if (f_thousands) {		/* with commas */
 		/* This format assignment needed to work round gcc bug. */
EOF
}

ensure_commit_available() {
  if git -C "${SOURCE_DIR}" rev-parse --verify "${FILE_CMDS_COMMIT}^{commit}" >/dev/null 2>&1; then
    return 0
  fi
  git -C "${SOURCE_DIR}" fetch --depth 1 origin "${FILE_CMDS_COMMIT}"
}

ensure_checkout() {
  mkdir -p "$(dirname "${SOURCE_DIR}")"

  if [[ ! -d "${SOURCE_DIR}/.git" ]]; then
    git clone "${REPO_URL}" "${SOURCE_DIR}"
  fi

  ensure_commit_available

  local current_commit=""
  current_commit="$(git -C "${SOURCE_DIR}" rev-parse HEAD)"

  if [[ "${current_commit}" != "${FILE_CMDS_COMMIT}" ]]; then
    if [[ -n "$(git -C "${SOURCE_DIR}" status --porcelain)" && "${FORCE_CLEAN}" -ne 1 ]]; then
      printf 'Validation checkout is dirty: %s\n' "${SOURCE_DIR}" >&2
      printf 'Re-run with --force-clean if this checkout is disposable.\n' >&2
      exit 1
    fi
    if [[ "${FORCE_CLEAN}" -eq 1 ]]; then
      git -C "${SOURCE_DIR}" reset --hard HEAD
      git -C "${SOURCE_DIR}" clean -fdx
    fi
    git -C "${SOURCE_DIR}" checkout --detach "${FILE_CMDS_COMMIT}"
  fi
}

apply_compat_patch() {
  local patch_path="${VALIDATION_ROOT}/mock-ls-compat.patch"
  mkdir -p "${VALIDATION_ROOT}"
  mock_ls_patch > "${patch_path}"

  if git -C "${SOURCE_DIR}" apply --reverse --check "${patch_path}" >/dev/null 2>&1; then
    return 0
  fi

  if ! git -C "${SOURCE_DIR}" apply --check "${patch_path}" >/dev/null 2>&1; then
    printf 'Compatibility patch did not apply cleanly at %s\n' "${FILE_CMDS_COMMIT}" >&2
    exit 1
  fi

  git -C "${SOURCE_DIR}" apply "${patch_path}"
}

build_mock_ls() {
  local derived_data_dir="${VALIDATION_ROOT}/derived-data"

  mkdir -p \
    "${BUILD_DIR}" \
    "${derived_data_dir}" \
    "${VALIDATION_ROOT}/home" \
    "${VALIDATION_ROOT}/tmp" \
    "${VALIDATION_ROOT}/module-cache"

  local output_path="${BUILD_DIR}/ls"
  if [[ "${REUSE_EXISTING_BUILD}" -eq 1 && -x "${output_path}" ]]; then
    printf '%s\n' "${output_path}"
    return 0
  fi

  (
    export HOME="${VALIDATION_ROOT}/home"
    export TMPDIR="${VALIDATION_ROOT}/tmp"
    cd "${SOURCE_DIR}"
    xcodebuild \
      -project "${SOURCE_DIR}/file_cmds.xcodeproj" \
      -scheme ls \
      -configuration Release \
      -sdk macosx \
      -derivedDataPath "${derived_data_dir}" \
      CONFIGURATION_BUILD_DIR="${BUILD_DIR}" \
      CLANG_MODULE_CACHE_PATH="${VALIDATION_ROOT}/module-cache" \
      CODE_SIGNING_ALLOWED=NO \
      CODE_SIGNING_REQUIRED=NO \
      GCC_TREAT_WARNINGS_AS_ERRORS=NO \
      build >&2
  )

  if [[ ! -x "${output_path}" ]]; then
    printf 'Expected mock verification binary was not produced: %s\n' "${output_path}" >&2
    exit 1
  fi

  printf '%s\n' "${output_path}"
}

run_output_smoke_test() {
  local candidate_path="$1"

  log "Running mock ls fixture smoke test"
  python3 - "${candidate_path}" >&2 <<'PY'
from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile

candidate = sys.argv[1]
env = os.environ.copy()
env["LC_ALL"] = "C"
env["LANG"] = "C"

with tempfile.TemporaryDirectory(prefix="mock-ls-smoke-") as tmp:
    root = Path(tmp)
    (root / "beta").write_text("beta\n", encoding="utf-8")
    (root / "gamma").write_text("gamma\n", encoding="utf-8")
    (root / ".alpha").write_text("alpha\n", encoding="utf-8")
    expected_cases = [
        {
            "argv": [],
            "stdout": "beta\ngamma\n",
            "stderr": "",
            "returncode": 0,
        },
        {
            "argv": ["."],
            "stdout": "beta\ngamma\n",
            "stderr": "",
            "returncode": 0,
        },
        {
            "argv": ["-a"],
            "stdout": ".\n..\n.alpha\nbeta\ngamma\n",
            "stderr": "",
            "returncode": 0,
        },
    ]

    for case in expected_cases:
        completed = subprocess.run(
            [candidate, *case["argv"]],
            cwd=str(root),
            capture_output=True,
            text=True,
            env=env,
        )
        same = (
            completed.returncode == case["returncode"]
            and completed.stdout == case["stdout"]
            and completed.stderr == case["stderr"]
        )
        print(
            {
                "argv": case["argv"],
                "match": same,
                "returncode": completed.returncode,
            }
        )
        if not same:
            print({"expected_stdout": case["stdout"], "actual_stdout": completed.stdout})
            print({"expected_stderr": case["stderr"], "actual_stderr": completed.stderr})
            raise SystemExit(1)
PY
}

run_frida_smoke_test() {
  local candidate_path="$1"

  log "Running Frida spawn->attach smoke test"
  python3 - "${candidate_path}" >&2 <<'PY'
from __future__ import annotations

import subprocess
import sys

candidate = sys.argv[1]
child_code = r"""
from __future__ import annotations

import sys

import frida

candidate = sys.argv[1]
pid = frida.spawn([candidate, "."])
session = None

try:
    session = frida.attach(pid)
    print({"spawned_pid": pid, "attach": "ok"})
finally:
    if session is not None:
        session.detach()
    try:
        frida.kill(pid)
    except Exception:
        pass
"""

completed = subprocess.run(
    [sys.executable, "-c", child_code, candidate],
    capture_output=True,
    text=True,
)
if completed.returncode != 0:
    raise SystemExit(
        "Frida smoke test failed"
        + f" (exit_code={completed.returncode})"
        + ("\nstdout:\n" + completed.stdout if completed.stdout else "")
        + ("\nstderr:\n" + completed.stderr if completed.stderr else "")
    )
if completed.stdout:
    print(completed.stdout.strip())
if completed.stderr:
    print(completed.stderr.strip(), file=sys.stderr)
PY
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verification-root|--validation-root)
      VALIDATION_ROOT="${2:-}"
      shift 2
      ;;
    --workspace-root)
      WORKSPACE_ROOT="${2:-}"
      shift 2
      ;;
    --source-dir)
      SOURCE_DIR="${2:-}"
      shift 2
      ;;
    --build-dir)
      BUILD_DIR="${2:-}"
      shift 2
      ;;
    --repo-url)
      REPO_URL="${2:-}"
      shift 2
      ;;
    --commit)
      FILE_CMDS_COMMIT="${2:-}"
      shift 2
      ;;
    --force-clean)
      FORCE_CLEAN=1
      shift
      ;;
    --reuse-existing-build)
      REUSE_EXISTING_BUILD=1
      shift
      ;;
    --smoke-test)
      RUN_SMOKE_TESTS=1
      shift
      ;;
    --frida-smoke-test)
      RUN_FRIDA_SMOKE_TEST=1
      shift
      ;;
    --output-json)
      OUTPUT_JSON=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      printf 'Unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

WORKSPACE_ROOT="$(detect_workspace_root "${WORKSPACE_ROOT}")"
if [[ -z "${VALIDATION_ROOT}" ]]; then
  VALIDATION_ROOT="${WORKSPACE_ROOT}/.work/verification/mock-ls"
fi
if [[ -z "${SOURCE_DIR}" ]]; then
  SOURCE_DIR="${VALIDATION_ROOT}/src/file_cmds"
fi
if [[ -z "${BUILD_DIR}" ]]; then
  BUILD_DIR="${VALIDATION_ROOT}/build"
fi

VALIDATION_ROOT="$(resolve_path "${VALIDATION_ROOT}")"
SOURCE_DIR="$(resolve_path "${SOURCE_DIR}")"
BUILD_DIR="$(resolve_path "${BUILD_DIR}")"

ensure_checkout
apply_compat_patch
VERIFICATION_BINARY_PATH="$(build_mock_ls)"

if [[ "${RUN_SMOKE_TESTS}" -eq 1 ]]; then
  run_output_smoke_test "${VERIFICATION_BINARY_PATH}"
fi

if [[ "${RUN_FRIDA_SMOKE_TEST}" -eq 1 ]]; then
  run_frida_smoke_test "${VERIFICATION_BINARY_PATH}"
fi

if [[ "${OUTPUT_JSON}" -eq 1 ]]; then
  printf '{\n'
  printf '  "verification_binary": %s,\n' "$(json_escape "${VERIFICATION_BINARY_PATH}")"
  printf '  "capture_binary": %s,\n' "$(json_escape "${VERIFICATION_BINARY_PATH}")"
  printf '  "validation_root": %s,\n' "$(json_escape "${VALIDATION_ROOT}")"
  printf '  "source_dir": %s,\n' "$(json_escape "${SOURCE_DIR}")"
  printf '  "build_dir": %s,\n' "$(json_escape "${BUILD_DIR}")"
  printf '  "repo_url": %s,\n' "$(json_escape "${REPO_URL}")"
  printf '  "commit": %s\n' "$(json_escape "${FILE_CMDS_COMMIT}")"
  printf '}\n'
else
  printf '%s\n' "${VERIFICATION_BINARY_PATH}"
fi
