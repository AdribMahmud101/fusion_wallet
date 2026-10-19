#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== Kavachi V4.1 Freeze Check ==="
echo "Worktree root: ${WORKTREE_ROOT}"
echo "Script dir:    ${SCRIPT_DIR}"

TMPDIR="$(mktemp -d /tmp/k0_freeze_check_XXXXXX)"
trap 'rm -rf "${TMPDIR}"' EXIT

echo "Created temp workspace: ${TMPDIR}"

# 1. Rebuild ABIs into temp dir
echo "--- Rebuilding ABIs from spec/reference ---"
forge inspect --root "${SCRIPT_DIR}/reference" IKavachiModule abi --json > "${TMPDIR}/IKavachiModule.json"
forge inspect --root "${SCRIPT_DIR}/reference" ISafeMinimal abi --json > "${TMPDIR}/ISafeMinimal.json"
forge inspect --root "${SCRIPT_DIR}/reference" ISanctionsList abi --json > "${TMPDIR}/ISanctionsList.json"

echo "Diffing generated ABIs against spec/abi/..."
diff -u "${SCRIPT_DIR}/abi/IKavachiModule.json" "${TMPDIR}/IKavachiModule.json"
diff -u "${SCRIPT_DIR}/abi/ISafeMinimal.json" "${TMPDIR}/ISafeMinimal.json"
diff -u "${SCRIPT_DIR}/abi/ISanctionsList.json" "${TMPDIR}/ISanctionsList.json"
echo "✔ ABI files match byte-for-byte."

# 2. Rebuild vectors into temp dir
echo "--- Rebuilding vectors from spec/reference ---"
(
  cd "${SCRIPT_DIR}/reference"
  VECTOR_OUT_PATH="${TMPDIR}/v4_1.json" forge script script/GenVectors.s.sol:GenVectors
)

echo "Diffing generated vectors against spec/vectors/v4_1.json..."
diff -u "${SCRIPT_DIR}/vectors/v4_1.json" "${TMPDIR}/v4_1.json"
echo "✔ Vector file matches byte-for-byte."

# 3. Run Foundry test suite in spec/reference
echo "--- Running test suite in spec/reference ---"
(
  cd "${SCRIPT_DIR}/reference"
  forge test -vv
)
echo "✔ All reference tests and vector re-checks passed."

echo "=== All freeze checks passed successfully ==="
exit 0
