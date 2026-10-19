// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {IKavachiModule as I, ISafeMinimal} from "./IKavachiModule.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {HashV41} from "./HashV41.sol";

/// @notice §7.14 first size split. Stateless validation and hashing only; no financial calls.
library KavachiPolicyLib {
    function validateAndHash(I.Policy calldata p, address safe, address sessionKey, address vaultA, address vaultB)
        external
        view
        returns (bytes32)
    {
        if (
            p.microMaxRaw == 0 || p.delayedMaxRaw < p.microMaxRaw || p.hardMaxRaw < p.delayedMaxRaw
                || p.dailyLimitRaw < p.hardMaxRaw || p.delaySeconds < 120 || p.delaySeconds > 900
                || p.maxShareSlippageBps > 1000 || p.guardian == address(0) || p.guardian == safe
                || p.guardian == sessionKey || ISafeMinimal(safe).isOwner(p.guardian) || p.payeeDelaySeconds < 3600
                || p.payeeDelaySeconds > 604800 || p.payEarlyWindowSeconds > 2592000 || p.vaults.length > 2
        ) revert I.InvalidPolicy();
        for (uint256 i; i < p.vaults.length; ++i) {
            address v = p.vaults[i].vault;
            if (
                (v != vaultA && v != vaultB) || p.vaults[i].maxExposureRaw == 0 || (i > 0 && p.vaults[i - 1].vault >= v)
            ) revert I.InvalidPolicy();
        }
        return HashV41.computePolicyHash(p);
    }

    // Stateless envelope checks use storage references to avoid copying the full policy/state ABI.
    function validateAction(
        I.Action memory a,
        I.Policy storage policy,
        I.ModuleState storage state,
        I.Queue storage queued,
        mapping(bytes32 => uint64) storage approvalValidUntil,
        bool queueing,
        uint256 chainId,
        address safe
    ) external view returns (bytes32 digest) {
        digest = HashV41.computeActionDigest(chainId, safe, address(this), state.mandateHash, state.sessionId, a);
        if (queued.exists && queued.actionDigest != digest) revert I.QueueBusy();
        if (a.expectedNonce != state.actionNonce) revert I.BadNonce();
        if (a.policyEpoch != state.policyEpoch) revert I.StalePolicy();
        if (queued.exists && !queueing && block.timestamp > queued.expiresAt) revert I.QueueExpired();
        if (a.deadline < block.timestamp) revert I.ActionExpired();
        if (a.deadline > state.sessionExpiresAt) revert I.InvalidDeadline();
        if (queueing) {
            if (
                a.deadline < block.timestamp + policy.delaySeconds + 30
                    || a.deadline > block.timestamp + policy.delaySeconds + 900
            ) revert I.InvalidDeadline();
        } else if (!queued.exists && a.deadline > block.timestamp + (_needsApproval(a, policy) ? 900 : 120)) {
            revert I.InvalidDeadline();
        }
        if (a.decisionHash == 0) revert I.MissingDecisionHash();
        _shape(a);
        if (a.assets == 0) revert I.InvalidAmount();
        if (a.assets > policy.hardMaxRaw) revert I.HardLimitExceeded();
        uint256 used = state.dailyDayIndex == block.timestamp / 1 days ? state.dailyUsedRaw : 0;
        if (a.assets > policy.dailyLimitRaw || used > policy.dailyLimitRaw - a.assets) revert I.DailyLimitExceeded();
        if (a.assets > state.sessionLimitRaw || state.sessionUsedRaw > state.sessionLimitRaw - a.assets) {
            revert I.SessionLimitExceeded();
        }
        if (_delayed(a, policy)) {
            if (!queueing) {
                if (!queued.exists) revert I.QueueRequired();
                if (queued.sessionId != state.sessionId) revert I.QueueMismatch();
                if (block.timestamp < queued.readyAt) revert I.QueueNotReady();
            }
        } else if (queueing) {
            revert I.QueueRequired();
        }
        if (_needsApproval(a, policy)) {
            uint64 until = approvalValidUntil[digest];
            if (until == 0) revert I.ApprovalRequired();
            if (until < block.timestamp) revert I.ApprovalExpired();
        }
    }

    function _delayed(I.Action memory a, I.Policy storage policy) private view returns (bool) {
        return a.assets > policy.microMaxRaw && a.assets <= policy.delayedMaxRaw;
    }

    function _needsApproval(I.Action memory a, I.Policy storage policy) private view returns (bool) {
        return policy.permissionTier == I.PermissionTier.OWNER_APPROVED || a.assets > policy.delayedMaxRaw;
    }

    function _shape(I.Action memory a) private pure {
        if (a.kind == I.ActionKind.REBALANCE) revert I.UnsupportedAction();
        if (a.kind == I.ActionKind.PAY_OBLIGATION) {
            if (
                a.sourceVault != address(0) || a.destinationVault != address(0) || a.minSharesOut != 0
                    || a.maxSharesBurn != 0 || a.obligationIdHash == 0 || a.payeeIdHash == 0
                    || a.recipient == address(0) || a.dueAt == 0
            ) revert I.UnsupportedAction();
        } else {
            if (a.obligationIdHash != 0 || a.payeeIdHash != 0 || a.recipient != address(0) || a.dueAt != 0) {
                revert I.UnsupportedAction();
            }
            if (a.kind == I.ActionKind.DEPOSIT) {
                if (
                    a.sourceVault != address(0) || a.destinationVault == address(0) || a.minSharesOut == 0
                        || a.maxSharesBurn != 0
                ) revert I.UnsupportedAction();
            } else if (
                a.destinationVault != address(0) || a.sourceVault == address(0) || a.minSharesOut != 0
                    || a.maxSharesBurn == 0
            ) {
                revert I.UnsupportedAction();
            }
        }
    }

    function shareFloor(uint256 shares, uint16 bps) external pure returns (uint256) {
        return Math.mulDiv(shares, 10000 - bps, 10000);
    }

    function shareCeiling(uint256 shares, uint16 bps) external pure returns (uint256) {
        return Math.mulDiv(shares, 10000 + bps, 10000, Math.Rounding.Ceil);
    }

    function redeemMatches(address asset, address safe, address vault, uint256 cashBefore, bytes memory returned)
        external
        view
        returns (bool)
    {
        if (returned.length != 32) return false;
        uint256 cashAfter = IERC20(asset).balanceOf(safe);
        (bool ok, bytes memory shares) = vault.staticcall{gas: 30000}(abi.encodeCall(IERC20.balanceOf, (safe)));
        return ok && shares.length == 32 && abi.decode(shares, (uint256)) == 0 && cashAfter >= cashBefore
            && cashAfter - cashBefore == abi.decode(returned, (uint256));
    }
}
