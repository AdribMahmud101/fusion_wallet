#!/usr/bin/env bash
set -euo pipefail

# scripts/demo/live_demo.sh
# One-command live demo for Fusion Wallet (Card K19).
# Runs end-to-end on Base Sepolia under the mandatory session-key flock:
#   1) Snapshot balances, buckets, module state
#   2) Owners (2-of-2) commit fresh instant-tier bill to active payee
#   3) Agent tick with real MNN+AHEM-G planner
#   4) Broadcast payment tx and wait for receipt
#   5) Verify on-chain status, session signer, passport anchoring, isPaid, duplicate pay revert
#   6) Record evidence row in evidence/demo/runs.json

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${WORKTREE_DIR}"

LOCK_FILE="/root/projects/fusion_wallet/.session.lock"
mkdir -p "$(dirname "${LOCK_FILE}")"

echo "========================================================================"
echo " Acquiring exclusive session lock (${LOCK_FILE})..."
echo "========================================================================"

# File descriptor 200 holds the lock for the entire demo execution
exec 200>"${LOCK_FILE}"
flock -x 200
echo "[Lock] Exclusive lock acquired. Executing live demo..."

# Run demo engine
python3 "${SCRIPT_DIR}/demo_engine.py" "$@"

echo "[Lock] Releasing session lock."
flock -u 200
