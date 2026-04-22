#!/usr/bin/env bash
set -euo pipefail

SOURCE_DIR=""
OUTPUT_DIR=""
GHIDRA_DIR=""

usage() {
  cat <<'EOF'
Usage: build-bundle.sh --ghidra-dir PATH --source-dir PATH --output-dir PATH

Compiles Ghidra Java script implementations into a jar and emits a single
stable `-postScript` entrypoint script for headless Ghidra.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ghidra-dir)
      GHIDRA_DIR="${2:-}"
      shift 2
      ;;
    --source-dir)
      SOURCE_DIR="${2:-}"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="${2:-}"
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

if [[ -z "${GHIDRA_DIR}" || -z "${SOURCE_DIR}" || -z "${OUTPUT_DIR}" ]]; then
  usage >&2
  exit 1
fi

if [[ ! -d "${SOURCE_DIR}" ]]; then
  printf 'Source dir not found: %s\n' "${SOURCE_DIR}" >&2
  exit 1
fi

if [[ ! -d "${GHIDRA_DIR}" ]]; then
  printf 'Ghidra dir not found: %s\n' "${GHIDRA_DIR}" >&2
  exit 1
fi

SOURCE_DIR="$(cd "${SOURCE_DIR}" && pwd -P)"
mkdir -p "${OUTPUT_DIR}"
OUTPUT_DIR="$(cd "${OUTPUT_DIR}" && pwd -P)"

BUNDLE_JAR="${OUTPUT_DIR}/ghidra-agent-cli-ghidra-scripts.jar"
STAMP_FILE="${OUTPUT_DIR}/.bundle.stamp"
ENTRY_SCRIPT_NAME="GhidraAgentCliEntry.java"

choose_javac() {
  local candidates=()
  if [[ -n "${GHIDRA_JAVA_HOME:-}" ]]; then
    candidates+=("${GHIDRA_JAVA_HOME}/bin/javac")
  fi
  if [[ -n "${JAVA_HOME:-}" ]]; then
    candidates+=("${JAVA_HOME}/bin/javac")
  fi
  candidates+=(
    "${GHIDRA_DIR}/jdk/bin/javac"
    "${GHIDRA_DIR}/jre/bin/javac"
  )

  local candidate=""
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  if command -v javac >/dev/null 2>&1; then
    command -v javac
    return 0
  fi

  return 1
}

choose_jar() {
  local candidates=()
  if [[ -n "${GHIDRA_JAVA_HOME:-}" ]]; then
    candidates+=("${GHIDRA_JAVA_HOME}/bin/jar")
  fi
  if [[ -n "${JAVA_HOME:-}" ]]; then
    candidates+=("${JAVA_HOME}/bin/jar")
  fi
  candidates+=(
    "${GHIDRA_DIR}/jdk/bin/jar"
    "${GHIDRA_DIR}/jre/bin/jar"
  )

  local candidate=""
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  if command -v jar >/dev/null 2>&1; then
    command -v jar
    return 0
  fi

  return 1
}

build_classpath() {
  local jars=()
  local root=""
  for root in "${GHIDRA_DIR}/Ghidra" "${GHIDRA_DIR}/Framework" "${GHIDRA_DIR}/Extensions"; do
    [[ -d "${root}" ]] || continue
    while IFS= read -r jar_path; do
      jars+=("${jar_path}")
    done < <(find "${root}" -type f -name '*.jar' | sort)
  done

  if [[ ${#jars[@]} -eq 0 ]]; then
    return 1
  fi

  local classpath=""
  local jar_path=""
  for jar_path in "${jars[@]}"; do
    if [[ -n "${classpath}" ]]; then
      classpath="${classpath}:"
    fi
    classpath="${classpath}${jar_path}"
  done
  printf '%s\n' "${classpath}"
}

needs_rebuild() {
  [[ -f "${BUNDLE_JAR}" ]] || return 0
  [[ -f "${STAMP_FILE}" ]] || return 0
  if find "${SOURCE_DIR}" -maxdepth 1 -type f \( -name '*.java' -o -name 'build-bundle.sh' \) -newer "${STAMP_FILE}" | grep -q .; then
    return 0
  fi
  return 1
}

JAVAC="$(choose_javac || true)"
JAR_TOOL="$(choose_jar || true)"
CLASSPATH="$(build_classpath || true)"

if [[ -z "${JAVAC}" || -z "${JAR_TOOL}" || -z "${CLASSPATH}" ]]; then
  printf 'Skipping bundle build: javac/jar or Ghidra jars not available.\n' >&2
  exit 1
fi

if ! needs_rebuild; then
  exit 0
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

IMPL_SRC_DIR="${TMP_DIR}/impl-src/ghidra_agent_cli/bundle/impl"
CLASS_DIR="${TMP_DIR}/classes"
mkdir -p "${IMPL_SRC_DIR}" "${CLASS_DIR}"

# The output directory is treated as a dedicated generated bundle directory.
# Clear previous generated artifacts before writing the refreshed jar/wrappers.
find "${OUTPUT_DIR}" -maxdepth 1 -type f \
  \( -name '*.java' -o -name '*.jar' -o -name '.bundle.stamp' \) \
  -delete

while IFS= read -r source_file; do
  base_name="$(basename "${source_file}" .java)"
  impl_file="${IMPL_SRC_DIR}/${base_name}Impl.java"

  {
    printf 'package ghidra_agent_cli.bundle.impl;\n\n'
    awk -v class_name="${base_name}" '
      {
        line = $0
        if (line ~ ("public class " class_name "[[:space:]]+extends[[:space:]]+GhidraScript")) {
          sub("public class " class_name, "public class " class_name "Impl", line)
        }
        print line
      }
    ' "${source_file}"
  } > "${impl_file}"
done < <(find "${SOURCE_DIR}" -maxdepth 1 -type f -name '*.java' | sort)

cat > "${OUTPUT_DIR}/${ENTRY_SCRIPT_NAME}" <<'EOF'
import generic.jar.ResourceFile;
import ghidra.app.script.GhidraScript;

import java.io.File;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.util.Arrays;

public class GhidraAgentCliEntry extends GhidraScript {
    private static final String BUNDLE_JAR_NAME = "ghidra-agent-cli-ghidra-scripts.jar";

    @Override
    protected void run() throws Exception {
        String[] wrapperArgs = getScriptArgs();
        if (wrapperArgs.length < 1) {
            throw new IllegalArgumentException(
                "Usage: GhidraAgentCliEntry <LogicalScript.java> [script args...]");
        }

        String logicalScriptName = wrapperArgs[0];
        String[] implArgs = Arrays.copyOfRange(wrapperArgs, 1, wrapperArgs.length);
        String implClassName = resolveImplementationClassName(logicalScriptName);
        runBundled(implClassName, implArgs);
    }

    private File resolveBundleJar() throws IOException {
        ResourceFile currentSourceFile = getSourceFile();
        if (currentSourceFile == null) {
            throw new IllegalStateException("Bundled wrapper source file is unavailable");
        }

        ResourceFile parentDir = currentSourceFile.getParentFile();
        if (parentDir == null) {
            throw new IllegalStateException(
                "Bundled wrapper parent directory is unavailable: " + currentSourceFile);
        }

        return new File(parentDir.getCanonicalPath(), BUNDLE_JAR_NAME);
    }

    private String resolveImplementationClassName(String logicalScriptName) {
        String scriptName = logicalScriptName.replace('\\', '/');
        int lastSlash = scriptName.lastIndexOf('/');
        if (lastSlash >= 0) {
            scriptName = scriptName.substring(lastSlash + 1);
        }

        if (scriptName.endsWith(".java")) {
            scriptName = scriptName.substring(0, scriptName.length() - ".java".length());
        }

        if (scriptName.isEmpty()) {
            throw new IllegalArgumentException("Logical script name must not be empty");
        }

        return "ghidra_agent_cli.bundle.impl." + scriptName + "Impl";
    }

    private void runBundled(String implClassName, String[] implArgs) throws Exception {
        File bundleJar = resolveBundleJar();
        if (!bundleJar.isFile()) {
            throw new IllegalStateException("Bundled script jar not found: " + bundleJar);
        }

        ClassLoader parentLoader = Thread.currentThread().getContextClassLoader();
        if (parentLoader == null) {
            parentLoader = GhidraAgentCliEntry.class.getClassLoader();
        }

        try (URLClassLoader loader = new URLClassLoader(
                new URL[] { bundleJar.toURI().toURL() },
                parentLoader)) {
            Class<?> implClass = Class.forName(implClassName, true, loader);
            Object impl = implClass.getDeclaredConstructor().newInstance();
            if (!(impl instanceof GhidraScript)) {
                throw new IllegalStateException(
                    implClassName + " must extend ghidra.app.script.GhidraScript");
            }

            GhidraScript implScript = (GhidraScript) impl;
            copyScriptState(this, implScript);
            implScript.setScriptArgs(implArgs);
            invokeRun(implClass, implScript);
        }
    }

    private void invokeRun(Class<?> implClass, Object impl) throws Exception {
        Method runMethod = implClass.getDeclaredMethod("run");
        runMethod.setAccessible(true);
        try {
            runMethod.invoke(impl);
        } catch (InvocationTargetException error) {
            Throwable cause = error.getCause();
            if (cause instanceof Exception) {
                throw (Exception) cause;
            }
            if (cause instanceof Error) {
                throw (Error) cause;
            }
            throw error;
        }
    }

    private void copyScriptState(Object source, Object target) throws IllegalAccessException {
        Class<?> type = GhidraScript.class;
        while (type != null && type != Object.class) {
            for (Field field : type.getDeclaredFields()) {
                if (Modifier.isStatic(field.getModifiers())) {
                    continue;
                }
                field.setAccessible(true);
                field.set(target, field.get(source));
            }
            type = type.getSuperclass();
        }
    }

}
EOF

"${JAVAC}" \
  -classpath "${CLASSPATH}" \
  -d "${CLASS_DIR}" \
  $(find "${TMP_DIR}/impl-src" -type f -name '*.java' | sort)

"${JAR_TOOL}" cf "${BUNDLE_JAR}" -C "${CLASS_DIR}" .
touch "${STAMP_FILE}"
