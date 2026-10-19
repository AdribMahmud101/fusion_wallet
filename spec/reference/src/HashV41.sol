// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {IKavachiModule} from "./IKavachiModule.sol";

/// @title HashV41
/// @notice Reference library implementing Kavachi V4.1 hashing formulas (§9).
library HashV41 {
    bytes32 public constant TAG_POLICY = keccak256(bytes("KAVACHI_POLICY_V4_1"));
    bytes32 public constant TAG_SESSION = keccak256(bytes("KAVACHI_SESSION_V4_1"));
    bytes32 public constant TAG_MANDATE = keccak256(bytes("KAVACHI_MANDATE_V4_1"));
    bytes32 public constant TAG_ACTION = keccak256(bytes("KAVACHI_ACTION_V4_1"));
    bytes32 public constant TAG_CANDIDATE = keccak256(bytes("KAVACHI_CANDIDATE_V4_1"));

    /// @dev Empty set obligations root (§8.2)
    bytes32 public constant EMPTY_OBLIGATIONS_ROOT = keccak256(bytes("KAVACHI_EMPTY_OBLIGATIONS_V4"));

    /// @notice Computes keccak256(UTF8(tag))
    function tag(string memory name) internal pure returns (bytes32) {
        return keccak256(bytes(name));
    }

    /// @notice Sorts vault limits array ascending by vault address
    function sortVaultLimits(IKavachiModule.VaultLimit[] memory vaults)
        internal
        pure
        returns (IKavachiModule.VaultLimit[] memory)
    {
        uint256 len = vaults.length;
        IKavachiModule.VaultLimit[] memory sorted = new IKavachiModule.VaultLimit[](len);
        for (uint256 i = 0; i < len; i++) {
            sorted[i] = vaults[i];
        }
        for (uint256 i = 0; i < len; i++) {
            for (uint256 j = i + 1; j < len; j++) {
                if (sorted[j].vault < sorted[i].vault) {
                    IKavachiModule.VaultLimit memory temp = sorted[i];
                    sorted[i] = sorted[j];
                    sorted[j] = temp;
                }
            }
        }
        return sorted;
    }

    /// @notice Computes vaultLimitsHash = keccak256(abi.encode((address,uint256)[] sortedVaultLimits))
    function computeVaultLimitsHash(IKavachiModule.VaultLimit[] memory vaults)
        internal
        pure
        returns (bytes32)
    {
        IKavachiModule.VaultLimit[] memory sorted = sortVaultLimits(vaults);
        return keccak256(abi.encode(sorted));
    }

    /// @notice Computes PolicyHash (§9)
    function computePolicyHash(IKavachiModule.Policy memory policy) internal pure returns (bytes32) {
        bytes32 vLimitsHash = computeVaultLimitsHash(policy.vaults);
        return computePolicyHash(policy, vLimitsHash);
    }

    /// @notice Computes PolicyHash with explicit vaultLimitsHash
    function computePolicyHash(IKavachiModule.Policy memory policy, bytes32 vaultLimitsHash)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(
            abi.encode(
                TAG_POLICY,
                uint8(policy.permissionTier),
                policy.operatingBufferRaw,
                policy.microMaxRaw,
                policy.delayedMaxRaw,
                policy.hardMaxRaw,
                policy.dailyLimitRaw,
                policy.delaySeconds,
                policy.maxShareSlippageBps,
                policy.guardian,
                vaultLimitsHash,
                policy.payeeDelaySeconds,
                policy.payEarlyWindowSeconds
            )
        );
    }

    /// @notice Computes SessionId (§9). Returns bytes32(0) if sessionKey == address(0) && expiry == 0 && limit == 0.
    function computeSessionId(
        uint256 chainId,
        address safe,
        address module,
        uint256 sessionStartEpoch,
        address sessionKey,
        uint64 sessionExpiresAt,
        uint256 sessionLimitRaw
    ) internal pure returns (bytes32) {
        if (sessionKey == address(0) && sessionExpiresAt == 0 && sessionLimitRaw == 0) {
            return bytes32(0);
        }
        return computeSessionIdRaw(
            chainId,
            safe,
            module,
            sessionStartEpoch,
            sessionKey,
            sessionExpiresAt,
            sessionLimitRaw
        );
    }

    /// @notice Computes raw SessionId using the TAG formula unconditionally
    function computeSessionIdRaw(
        uint256 chainId,
        address safe,
        address module,
        uint256 sessionStartEpoch,
        address sessionKey,
        uint64 sessionExpiresAt,
        uint256 sessionLimitRaw
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                TAG_SESSION,
                chainId,
                safe,
                module,
                sessionStartEpoch,
                sessionKey,
                sessionExpiresAt,
                sessionLimitRaw
            )
        );
    }

    /// @notice Computes MandateHash (§9)
    function computeMandateHash(
        uint256 chainId,
        address safe,
        address module,
        uint256 policyEpoch,
        bytes32 policyHash,
        bytes32 obligationsRoot,
        uint256 reservedRaw,
        bytes32 sessionId,
        address sessionKey,
        uint64 sessionExpiresAt,
        uint256 sessionLimitRaw,
        bool paused
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                TAG_MANDATE,
                chainId,
                safe,
                module,
                policyEpoch,
                policyHash,
                obligationsRoot,
                reservedRaw,
                sessionId,
                sessionKey,
                sessionExpiresAt,
                sessionLimitRaw,
                paused
            )
        );
    }

    /// @notice Computes MandateHash from ModuleState (§9)
    function computeMandateHash(
        uint256 chainId,
        address safe,
        address module,
        IKavachiModule.ModuleState memory state
    ) internal pure returns (bytes32) {
        return computeMandateHash(
            chainId,
            safe,
            module,
            state.policyEpoch,
            state.policyHash,
            state.obligationsRoot,
            state.reservedRaw,
            state.sessionId,
            state.sessionKey,
            state.sessionExpiresAt,
            state.sessionLimitRaw,
            state.paused
        );
    }

    /// @notice Computes ActionDigest (§9)
    function computeActionDigest(
        uint256 chainId,
        address safe,
        address module,
        bytes32 mandateHash,
        bytes32 sessionId,
        IKavachiModule.Action memory action
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                TAG_ACTION,
                chainId,
                safe,
                module,
                mandateHash,
                sessionId,
                action
            )
        );
    }

    /// @notice Computes CandidateId (§9)
    function computeCandidateId(
        bytes32 snapshotBlockHash,
        bytes32 actionDigestWithoutDecision
    ) internal pure returns (bytes32) {
        return keccak256(
            abi.encode(
                TAG_CANDIDATE,
                snapshotBlockHash,
                actionDigestWithoutDecision
            )
        );
    }

    /// @notice Computes CandidateId directly from action by zeroing decisionHash
    function computeCandidateId(
        uint256 chainId,
        address safe,
        address module,
        bytes32 mandateHash,
        bytes32 sessionId,
        bytes32 snapshotBlockHash,
        IKavachiModule.Action memory action
    ) internal pure returns (bytes32) {
        IKavachiModule.Action memory actionWithoutDecision = IKavachiModule.Action({
            kind: action.kind,
            sourceVault: action.sourceVault,
            destinationVault: action.destinationVault,
            assets: action.assets,
            minSharesOut: action.minSharesOut,
            maxSharesBurn: action.maxSharesBurn,
            obligationIdHash: action.obligationIdHash,
            payeeIdHash: action.payeeIdHash,
            recipient: action.recipient,
            dueAt: action.dueAt,
            expectedNonce: action.expectedNonce,
            policyEpoch: action.policyEpoch,
            deadline: action.deadline,
            decisionHash: bytes32(0)
        });
        bytes32 actionDigestWithoutDecision = computeActionDigest(
            chainId,
            safe,
            module,
            mandateHash,
            sessionId,
            actionWithoutDecision
        );
        return computeCandidateId(snapshotBlockHash, actionDigestWithoutDecision);
    }

    /// @notice Computes ObligationLeaf (§9): double hash via bytes.concat
    function computeObligationLeaf(
        bytes32 obligationIdHash,
        bytes32 payeeIdHash,
        uint256 amountRaw,
        uint64 dueAt
    ) internal pure returns (bytes32) {
        bytes32 innerHash = keccak256(
            abi.encode(
                obligationIdHash,
                payeeIdHash,
                amountRaw,
                dueAt
            )
        );
        return keccak256(bytes.concat(innerHash));
    }

    /// @notice Computes DecisionHash (§9): keccak256 of UTF-8 RFC 8785 JCS canonical JSON
    function computeDecisionHash(string memory canonicalPassportJson) internal pure returns (bytes32) {
        return keccak256(bytes(canonicalPassportJson));
    }

    /// @notice Hashes two 32-byte nodes in sorted order (§8.2, compatible with OpenZeppelin)
    function hashPair(bytes32 a, bytes32 b) internal pure returns (bytes32) {
        return a < b ? keccak256(bytes.concat(a, b)) : keccak256(bytes.concat(b, a));
    }

    /// @notice Processes a sorted-pair Merkle proof against a leaf
    function processProof(bytes32[] memory proof, bytes32 leaf) internal pure returns (bytes32) {
        bytes32 computed = leaf;
        for (uint256 i = 0; i < proof.length; i++) {
            computed = hashPair(computed, proof[i]);
        }
        return computed;
    }

    /// @notice Verifies a sorted-pair Merkle proof
    function verifyObligationProof(
        bytes32[] memory proof,
        bytes32 root,
        bytes32 leaf
    ) internal pure returns (bool) {
        return processProof(proof, leaf) == root;
    }

    /// @notice Sorts leaf hashes ascending
    function sortLeaves(bytes32[] memory leaves) internal pure returns (bytes32[] memory) {
        uint256 len = leaves.length;
        bytes32[] memory sorted = new bytes32[](len);
        for (uint256 i = 0; i < len; i++) {
            sorted[i] = leaves[i];
        }
        for (uint256 i = 0; i < len; i++) {
            for (uint256 j = i + 1; j < len; j++) {
                if (sorted[j] < sorted[i]) {
                    bytes32 temp = sorted[i];
                    sorted[i] = sorted[j];
                    sorted[j] = temp;
                }
            }
        }
        return sorted;
    }

    /// @notice Computes complete binary tree Merkle root from sorted leaves (OpenZeppelin StandardMerkleTree)
    function computeMerkleRoot(bytes32[] memory sortedLeaves) internal pure returns (bytes32) {
        uint256 n = sortedLeaves.length;
        if (n == 0) {
            return EMPTY_OBLIGATIONS_ROOT;
        }
        if (n == 1) {
            return sortedLeaves[0];
        }
        bytes32[] memory tree = new bytes32[](2 * n - 1);
        for (uint256 i = 0; i < n; i++) {
            tree[tree.length - 1 - i] = sortedLeaves[i];
        }
        uint256 start = tree.length - 1 - n;
        for (uint256 i = start + 1; i > 0; i--) {
            uint256 u = i - 1;
            tree[u] = hashPair(tree[2 * u + 1], tree[2 * u + 2]);
        }
        return tree[0];
    }

    /// @notice Generates Merkle proof for leaf at `leafIndex` from sorted leaves (OpenZeppelin StandardMerkleTree)
    function computeMerkleProof(bytes32[] memory sortedLeaves, uint256 leafIndex)
        internal
        pure
        returns (bytes32[] memory)
    {
        uint256 n = sortedLeaves.length;
        require(leafIndex < n, "HashV41: leaf index out of bounds");
        if (n <= 1) {
            return new bytes32[](0);
        }
        bytes32[] memory tree = new bytes32[](2 * n - 1);
        for (uint256 i = 0; i < n; i++) {
            tree[tree.length - 1 - i] = sortedLeaves[i];
        }
        uint256 start = tree.length - 1 - n;
        for (uint256 i = start + 1; i > 0; i--) {
            uint256 u = i - 1;
            tree[u] = hashPair(tree[2 * u + 1], tree[2 * u + 2]);
        }

        uint256 index = tree.length - 1 - leafIndex;
        uint256 depth = 0;
        uint256 temp = index;
        while (temp > 0) {
            depth++;
            temp = (temp - 1) / 2;
        }

        bytes32[] memory proof = new bytes32[](depth);
        uint256 p = 0;
        while (index > 0) {
            uint256 pairIndex = (index % 2 == 0) ? index - 1 : index + 1;
            proof[p++] = tree[pairIndex];
            index = (index - 1) / 2;
        }
        return proof;
    }
}
