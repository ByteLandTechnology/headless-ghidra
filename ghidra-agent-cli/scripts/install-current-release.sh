#!/usr/bin/env bash
# Clone-first install helper. Downloads the GitHub Release archive for the
# currently checked-out tag (or a version passed explicitly) and installs the
# binary to INSTALL_DIR (default: .local/bin).
#
# Most users should prefer `npm install -g <package>` instead. This script is
# the fallback for users who already have the repository checked out and want
# a local-filesystem install without going through the npm registry. It
# requires Node.js to parse release/config.json.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
CONFIG_PATH="${REPO_ROOT}/release/config.json"

if [[ ! -f "${CONFIG_PATH}" ]]; then
  echo "Missing ${CONFIG_PATH}." >&2
  exit 1
fi

if ! command -v node >/dev/null 2>&1; then
  echo "Node.js is required to read ${CONFIG_PATH}." >&2
  exit 1
fi

CLI_NAME=$(node -e "console.log(require('${CONFIG_PATH}').cliName)")
OWNER_REPO=$(node -e "console.log(require('${CONFIG_PATH}').sourceRepository)")
OWNER_REPO="${GITHUB_REPOSITORY:-$OWNER_REPO}"

VERSION="${1:-}"
if [[ -z "${VERSION}" ]] && command -v git >/dev/null 2>&1; then
  VERSION="$(git -C "${REPO_ROOT}" describe --tags --exact-match 2>/dev/null || true)"
  VERSION="${VERSION#v}"
fi
if [[ -z "${VERSION}" ]]; then
  echo "Pass a version explicitly or check out a released tag." >&2
  exit 1
fi

RUST_TARGET=$(node -e "
  const c = require('${CONFIG_PATH}');
  const osMap = { Darwin: 'darwin', Linux: 'linux', Windows_NT: 'win32' };
  const archMap = { arm64: 'arm64', aarch64: 'arm64', x86_64: 'x64', AMD64: 'x64' };
  const rawOs = '$(uname -s)';
  const os = osMap[rawOs] || (/^MINGW|^MSYS/.test(rawOs) ? 'win32' : undefined);
  const cpu = archMap['$(uname -m)'];
  if (!os || !cpu) { process.stderr.write('Unsupported platform.\n'); process.exit(1); }
  const t = c.targets.find(e => e.os === os && e.cpu === cpu);
  if (!t) { process.stderr.write('No matching target for ' + os + '-' + cpu + '.\n'); process.exit(1); }
  process.stdout.write(t.rustTarget);
")
if [[ -z "${RUST_TARGET}" ]]; then
  echo "Could not resolve target from ${CONFIG_PATH}." >&2
  exit 1
fi

INSTALL_DIR="${INSTALL_DIR:-${REPO_ROOT}/.local/bin}"
ARCHIVE="${CLI_NAME}-${RUST_TARGET}.tar.gz"
CHECKSUM="${ARCHIVE}.sha256"
BASE_URL="https://github.com/${OWNER_REPO}/releases/download/v${VERSION}"
# Windows targets ship binaries with .exe suffix.
BINARY_NAME="${CLI_NAME}"
if echo "${RUST_TARGET}" | grep -q "windows"; then
  BINARY_NAME="${CLI_NAME}.exe"
fi
TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

mkdir -p "${INSTALL_DIR}"
echo "Downloading ${BASE_URL}/${ARCHIVE}"
curl --fail --location --silent --show-error "${BASE_URL}/${ARCHIVE}" -o "${TMP}/${ARCHIVE}"

echo "Verifying checksum..."
curl --fail --location --silent --show-error "${BASE_URL}/${CHECKSUM}" -o "${TMP}/${CHECKSUM}"
# Normalize checksum file for sha256sum (Linux) if needed.
if command -v sha256sum >/dev/null 2>&1; then
  (cd "${TMP}" && sha256sum -c "${CHECKSUM}" >/dev/null 2>&1 || {
    echo "Checksum verification failed. The archive may be corrupted or tampered." >&2
    exit 1
  })
else
  (cd "${TMP}" && shasum -a 256 -c "${CHECKSUM}" >/dev/null 2>&1 || {
    echo "Checksum verification failed. The archive may be corrupted or tampered." >&2
    exit 1
  })
fi

tar -xzf "${TMP}/${ARCHIVE}" -C "${TMP}"
install -m 0755 "${TMP}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

echo "Installed ${CLI_NAME} ${VERSION} to ${INSTALL_DIR}/${BINARY_NAME}"
