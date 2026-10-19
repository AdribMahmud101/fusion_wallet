#!/usr/bin/env bash
set -euo pipefail

# scripts/judge/verify_all.sh
# Keyless, read-only judge verification suite for Fusion Wallet (Card K19).
# Runs verify_evidence.py, check_golden.py, checks evidence/app/*.json,
# checks evidence/demo/runs.json, checks Sourcify contract links, and prints
# a one-screen PASS/FAIL summary table with counts.
# Exits 0 only if all checks pass.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${WORKTREE_DIR}"

python3 "${SCRIPT_DIR}/verify_all.py" "$@"
