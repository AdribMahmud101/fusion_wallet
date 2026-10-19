#!/usr/bin/env bash
set -euo pipefail

# scripts/agent/run_golden.sh
# Reproduces the autonomous agent golden path on Base Sepolia from scratch.
# Signs strictly with SESSION_PRIVATE_KEY.
# Anchors each decision's canonical decisionHash on-chain.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKTREE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "${WORKTREE_DIR}"

echo "=========================================================="
echo " Starting Autonomous Agent Golden Path (Card K9)"
echo " Worktree: ${WORKTREE_DIR}"
echo " Time:     $(date -u +"%Y-%m-%d %H:%M:%SZ")"
echo "=========================================================="

# 1. Environment & Keys
if [ -f "/root/projects/fusion_wallet/.env.local" ]; then
    set -a
    . /root/projects/fusion_wallet/.env.local
    set +a
fi

if [ -z "${SESSION_PRIVATE_KEY:-}" ]; then
    echo "ERROR: SESSION_PRIVATE_KEY not set!" >&2
    exit 1
fi

BIN="${WORKTREE_DIR}/src/fusion-core/target/release/agent-runner"
PLANNER_BIN="${WORKTREE_DIR}/src/fusion-planner/target/release/planner-host"
MODEL_DIR="/root/projects/feros/sdk/demo-app/app/src/main/assets/models/lfm_230m"
MANIFEST="${WORKTREE_DIR}/deployments/84532.json"
OBLIGATIONS="${WORKTREE_DIR}/evidence/agent/obligations-84532.json"
GOLDEN_JSON="${WORKTREE_DIR}/evidence/agent/golden.json"
PASSPORTS_DIR="${WORKTREE_DIR}/evidence/agent/passports"
RUNS_DIR="${WORKTREE_DIR}/evidence/agent/runs"
CHECK_LOG="${WORKTREE_DIR}/evidence/agent/check.log"

if [ ! -x "${BIN}" ]; then
    echo "Building release agent-runner..."
    cargo build --release --manifest-path src/fusion-core/Cargo.toml --bin agent-runner
fi

mkdir -p "${PASSPORTS_DIR}" "${RUNS_DIR}" "${WORKTREE_DIR}/evidence/agent/dryrun"

# Initialize golden.json
echo "[]" > "${GOLDEN_JSON}"

TMP_LOG="/tmp/agent_runner_step.log"

extract_and_save_run_log() {
    local tmp_file="$1"
    local tx_hash
    tx_hash=$(grep -oE "Tx submitted: 0x[0-9a-fA-F]{64}" "${tmp_file}" | awk '{print $3}' | head -n 1 || true)
    if [ -n "${tx_hash}" ]; then
        echo "Saving full execution log for ${tx_hash}..."
        cat "${tmp_file}" >> "${RUNS_DIR}/${tx_hash}.log"
        cat "${tmp_file}" >> "${RUNS_DIR}/${tx_hash,,}.log"
    fi
}

# -----------------------------------------------------------------------------
# Tx 1: Instant PAY (Bill 0: 8.00 USDC to Acme)
# -----------------------------------------------------------------------------
echo ""
echo ">>> [Tx 1/3] Executing Instant PAY (8.00 USDC)..."
"${BIN}" \
    --manifest "${MANIFEST}" \
    --obligations "${OBLIGATIONS}" \
    --planner host \
    --planner-bin "${PLANNER_BIN}" \
    --model-dir "${MODEL_DIR}" \
    --choice B \
    --user-goal "Pay small bill B immediately" \
    --evidence-out "${GOLDEN_JSON}" \
    --evidence-id "E7-AGENT" \
    --passport-dir "${PASSPORTS_DIR}" \
    --runs-dir "${RUNS_DIR}" \
    --source "AGENT" 2>&1 | tee "${TMP_LOG}"

extract_and_save_run_log "${TMP_LOG}"

# -----------------------------------------------------------------------------
# Tx 2: Queue Delayed PAY (Bill 1: 50.00 USDC to Payroll)
# -----------------------------------------------------------------------------
echo ""
echo ">>> [Tx 2/3] Queueing Delayed PAY (50.00 USDC)..."
"${BIN}" \
    --manifest "${MANIFEST}" \
    --obligations "${OBLIGATIONS}" \
    --planner host \
    --planner-bin "${PLANNER_BIN}" \
    --model-dir "${MODEL_DIR}" \
    --choice A \
    --user-goal "Pay payroll bill A" \
    --evidence-out "${GOLDEN_JSON}" \
    --evidence-id "E8-QUEUE" \
    --passport-dir "${PASSPORTS_DIR}" \
    --runs-dir "${RUNS_DIR}" \
    --source "AGENT" 2>&1 | tee "${TMP_LOG}"

extract_and_save_run_log "${TMP_LOG}"

# -----------------------------------------------------------------------------
# Wait for Timelock (delaySeconds = 180s)
# -----------------------------------------------------------------------------
echo ""
echo ">>> Waiting 185s for timelock delay to elapse..."
for remaining in $(seq 185 -15 1); do
    echo "    Timelock countdown: ${remaining}s remaining..."
    sleep 15
done
echo "    Timelock elapsed. Proceeding to execution."

# -----------------------------------------------------------------------------
# Tx 3: Execute Queued Delayed PAY (Bill 1: 50.00 USDC to Payroll)
# -----------------------------------------------------------------------------
echo ""
echo ">>> [Tx 3/3] Executing Queued Delayed PAY (50.00 USDC)..."
"${BIN}" \
    --manifest "${MANIFEST}" \
    --obligations "${OBLIGATIONS}" \
    --planner host \
    --planner-bin "${PLANNER_BIN}" \
    --model-dir "${MODEL_DIR}" \
    --choice A \
    --user-goal "Execute queued payroll bill A" \
    --evidence-out "${GOLDEN_JSON}" \
    --evidence-id "E8-EXEC" \
    --passport-dir "${PASSPORTS_DIR}" \
    --runs-dir "${RUNS_DIR}" \
    --source "AGENT" 2>&1 | tee "${TMP_LOG}"

extract_and_save_run_log "${TMP_LOG}"

# -----------------------------------------------------------------------------
# Step 4: HOLD Decision (no remaining unpaid obligations)
# -----------------------------------------------------------------------------
echo ""
echo ">>> [Step 4] Evaluating Autonomous HOLD Decision..."
"${BIN}" \
    --manifest "${MANIFEST}" \
    --obligations "${OBLIGATIONS}" \
    --planner host \
    --planner-bin "${PLANNER_BIN}" \
    --model-dir "${MODEL_DIR}" \
    --choice H \
    --user-goal "Maintain runway and hold" \
    --evidence-out "${GOLDEN_JSON}" \
    --evidence-id "HOLD-AGENT" \
    --passport-dir "${PASSPORTS_DIR}" \
    --runs-dir "${RUNS_DIR}" \
    --source "AGENT" 2>&1 | tee "${TMP_LOG}"

# -----------------------------------------------------------------------------
# Verification (check_golden.py)
# -----------------------------------------------------------------------------
echo ""
echo ">>> Running Mechanical Verification (check_golden.py)..."
python3 "${SCRIPT_DIR}/check_golden.py" 2>&1 | tee "${CHECK_LOG}"

echo ""
echo "=========================================================="
echo " Golden Path Execution & Verification Complete!"
echo " Verification Log: ${CHECK_LOG}"
echo "=========================================================="
