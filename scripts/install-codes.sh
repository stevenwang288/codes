#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"

NO_BUILD=0
if [[ "${1:-}" == "--no-build" ]]; then
  NO_BUILD=1
  shift || true
fi

if [[ "${NO_BUILD}" != "1" ]]; then
  (cd "${REPO_ROOT}" && ./build-fast.sh)
fi

SRC_BIN="${REPO_ROOT}/code-rs/bin/codes"
if [[ ! -x "${SRC_BIN}" ]]; then
  echo "[codes] ERROR: missing built binary at ${SRC_BIN}" >&2
  echo "[codes] Try: ./build-fast.sh" >&2
  exit 1
fi

DEST_HOME="${HOME}/.codes"
DEST_BIN="${DEST_HOME}/bin"
mkdir -p "${DEST_BIN}"

cp -f "${SRC_BIN}" "${DEST_BIN}/codes"
chmod +x "${DEST_BIN}/codes" || true

echo "[codes] Installed: ${DEST_BIN}/codes"
echo "[codes] Add to PATH (pick one):"
echo "  - bash:  echo 'export PATH=\"${DEST_BIN}:$PATH\"' >> ~/.bashrc"
echo "  - zsh:   echo 'export PATH=\"${DEST_BIN}:$PATH\"' >> ~/.zshrc"
echo "  - fish:  set -Ux PATH \"${DEST_BIN}\" \$PATH"

