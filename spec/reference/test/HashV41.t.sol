// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "forge-std/Test.sol";
import {IKavachiModule} from "../src/IKavachiModule.sol";
import {HashV41} from "../src/HashV41.sol";

contract HashV41Test is Test {
    using stdJson for string;

    string internal jsonContent;

    function setUp() public {
        // Read frozen vector file from ../vectors/v4_1.json
        jsonContent = vm.readFile("../vectors/v4_1.json");
    }

    // 1. Acceptance check: TAG strings and constants match §9 and §8.2 literally
    function test_tag_strings() public pure {
        assertEq(HashV41.TAG_POLICY, keccak256(bytes("KAVACHI_POLICY_V4_1")));
        assertEq(HashV41.TAG_SESSION, keccak256(bytes("KAVACHI_SESSION_V4_1")));
        assertEq(HashV41.TAG_MANDATE, keccak256(bytes("KAVACHI_MANDATE_V4_1")));
        assertEq(HashV41.TAG_ACTION, keccak256(bytes("KAVACHI_ACTION_V4_1")));
        assertEq(HashV41.TAG_CANDIDATE, keccak256(bytes("KAVACHI_CANDIDATE_V4_1")));
        assertEq(HashV41.EMPTY_OBLIGATIONS_ROOT, keccak256(bytes("KAVACHI_EMPTY_OBLIGATIONS_V4")));
    }

    // 2. PolicyHash & VaultLimitsHash verification
    function test_vector_policy_hash() public view {
        bytes32 expectedVaultLimitsHash = jsonContent.readBytes32(".policy_hash.expected.vaultLimitsHash");
        bytes32 expectedPolicyHash = jsonContent.readBytes32(".policy_hash.expected.policyHash");

        address vault0 = jsonContent.readAddress(".policy_hash.input.vaults[0].vault");
        uint256 cap0 = jsonContent.readUint(".policy_hash.input.vaults[0].maxExposureRaw");
        address vault1 = jsonContent.readAddress(".policy_hash.input.vaults[1].vault");
        uint256 cap1 = jsonContent.readUint(".policy_hash.input.vaults[1].maxExposureRaw");

        IKavachiModule.VaultLimit[] memory vaults = new IKavachiModule.VaultLimit[](2);
        vaults[0] = IKavachiModule.VaultLimit({vault: vault0, maxExposureRaw: cap0});
        vaults[1] = IKavachiModule.VaultLimit({vault: vault1, maxExposureRaw: cap1});

        bytes32 calculatedVaultLimitsHash = HashV41.computeVaultLimitsHash(vaults);
        assertEq(calculatedVaultLimitsHash, expectedVaultLimitsHash, "vaultLimitsHash mismatch");

        IKavachiModule.Policy memory policy = IKavachiModule.Policy({
            permissionTier: IKavachiModule.PermissionTier(uint8(jsonContent.readUint(".policy_hash.input.permissionTier"))),
            operatingBufferRaw: jsonContent.readUint(".policy_hash.input.operatingBufferRaw"),
            microMaxRaw: jsonContent.readUint(".policy_hash.input.microMaxRaw"),
            delayedMaxRaw: jsonContent.readUint(".policy_hash.input.delayedMaxRaw"),
            hardMaxRaw: jsonContent.readUint(".policy_hash.input.hardMaxRaw"),
            dailyLimitRaw: jsonContent.readUint(".policy_hash.input.dailyLimitRaw"),
            delaySeconds: uint32(jsonContent.readUint(".policy_hash.input.delaySeconds")),
            maxShareSlippageBps: uint16(jsonContent.readUint(".policy_hash.input.maxShareSlippageBps")),
            guardian: jsonContent.readAddress(".policy_hash.input.guardian"),
            vaults: vaults,
            payeeDelaySeconds: uint32(jsonContent.readUint(".policy_hash.input.payeeDelaySeconds")),
            payEarlyWindowSeconds: uint32(jsonContent.readUint(".policy_hash.input.payEarlyWindowSeconds"))
        });

        bytes32 calculatedPolicyHash = HashV41.computePolicyHash(policy);
        assertEq(calculatedPolicyHash, expectedPolicyHash, "policyHash mismatch");

        // Verify sorting idempotency and sorting when input is reversed
        IKavachiModule.VaultLimit[] memory reversedVaults = new IKavachiModule.VaultLimit[](2);
        reversedVaults[0] = vaults[1];
        reversedVaults[1] = vaults[0];
        assertEq(HashV41.computeVaultLimitsHash(reversedVaults), expectedVaultLimitsHash, "sortVaultLimits failed to sort");
    }

    // 3. SessionId verification (with and without session)
    function test_vector_session_id() public view {
        // With session
        uint256 chainId = jsonContent.readUint(".session_id.with_session.input.chainId");
        address safe = jsonContent.readAddress(".session_id.with_session.input.safe");
        address module = jsonContent.readAddress(".session_id.with_session.input.module");
        uint256 sessionStartEpoch = jsonContent.readUint(".session_id.with_session.input.sessionStartEpoch");
        address sessionKey = jsonContent.readAddress(".session_id.with_session.input.sessionKey");
        uint64 sessionExpiresAt = uint64(jsonContent.readUint(".session_id.with_session.input.sessionExpiresAt"));
        uint256 sessionLimitRaw = jsonContent.readUint(".session_id.with_session.input.sessionLimitRaw");

        bytes32 expectedWithSession = jsonContent.readBytes32(".session_id.with_session.expected.sessionId");
        bytes32 computedWithSession = HashV41.computeSessionId(
            chainId,
            safe,
            module,
            sessionStartEpoch,
            sessionKey,
            sessionExpiresAt,
            sessionLimitRaw
        );
        assertEq(computedWithSession, expectedWithSession, "sessionId (with session) mismatch");

        // Without session
        bytes32 expectedWithoutSession = jsonContent.readBytes32(".session_id.without_session.expected.sessionId");
        bytes32 computedWithoutSession = HashV41.computeSessionId(
            chainId,
            safe,
            module,
            0,
            address(0),
            0,
            0
        );
        assertEq(computedWithoutSession, bytes32(0), "sessionId without session must be 0");
        assertEq(computedWithoutSession, expectedWithoutSession, "sessionId (without session) mismatch");
    }

    // 4. MandateHash verification
    function test_vector_mandate_hash() public view {
        uint256 chainId = jsonContent.readUint(".mandate_hash.input.chainId");
        address safe = jsonContent.readAddress(".mandate_hash.input.safe");
        address module = jsonContent.readAddress(".mandate_hash.input.module");
        uint256 policyEpoch = jsonContent.readUint(".mandate_hash.input.policyEpoch");
        bytes32 policyHash = jsonContent.readBytes32(".mandate_hash.input.policyHash");
        bytes32 obligationsRoot = jsonContent.readBytes32(".mandate_hash.input.obligationsRoot");
        uint256 reservedRaw = jsonContent.readUint(".mandate_hash.input.reservedRaw");
        bytes32 sessionId = jsonContent.readBytes32(".mandate_hash.input.sessionId");
        address sessionKey = jsonContent.readAddress(".mandate_hash.input.sessionKey");
        uint64 sessionExpiresAt = uint64(jsonContent.readUint(".mandate_hash.input.sessionExpiresAt"));
        uint256 sessionLimitRaw = jsonContent.readUint(".mandate_hash.input.sessionLimitRaw");
        bool paused = jsonContent.readBool(".mandate_hash.input.paused");

        bytes32 expectedMandateHash = jsonContent.readBytes32(".mandate_hash.expected.mandateHash");
        bytes32 computedMandateHash = HashV41.computeMandateHash(
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
        );
        assertEq(computedMandateHash, expectedMandateHash, "mandateHash mismatch");
    }

    // 5. Actions verification: DEPOSIT, WITHDRAW, REBALANCE, PAY_OBLIGATION
    function test_vector_actions() public view {
        _verifyAction(".actions.deposit");
        _verifyAction(".actions.withdraw");
        _verifyAction(".actions.rebalance");
        _verifyAction(".actions.pay_obligation");
    }

    function _verifyAction(string memory prefix) internal view {
        uint256 chainId = jsonContent.readUint(string.concat(prefix, ".input.chainId"));
        address safe = jsonContent.readAddress(string.concat(prefix, ".input.safe"));
        address module = jsonContent.readAddress(string.concat(prefix, ".input.module"));
        bytes32 mandateHash = jsonContent.readBytes32(string.concat(prefix, ".input.mandateHash"));
        bytes32 sessionId = jsonContent.readBytes32(string.concat(prefix, ".input.sessionId"));

        IKavachiModule.Action memory action = IKavachiModule.Action({
            kind: IKavachiModule.ActionKind(uint8(jsonContent.readUint(string.concat(prefix, ".input.action.kind")))),
            sourceVault: jsonContent.readAddress(string.concat(prefix, ".input.action.sourceVault")),
            destinationVault: jsonContent.readAddress(string.concat(prefix, ".input.action.destinationVault")),
            assets: jsonContent.readUint(string.concat(prefix, ".input.action.assets")),
            minSharesOut: jsonContent.readUint(string.concat(prefix, ".input.action.minSharesOut")),
            maxSharesBurn: jsonContent.readUint(string.concat(prefix, ".input.action.maxSharesBurn")),
            obligationIdHash: jsonContent.readBytes32(string.concat(prefix, ".input.action.obligationIdHash")),
            payeeIdHash: jsonContent.readBytes32(string.concat(prefix, ".input.action.payeeIdHash")),
            recipient: jsonContent.readAddress(string.concat(prefix, ".input.action.recipient")),
            dueAt: uint64(jsonContent.readUint(string.concat(prefix, ".input.action.dueAt"))),
            expectedNonce: jsonContent.readUint(string.concat(prefix, ".input.action.expectedNonce")),
            policyEpoch: jsonContent.readUint(string.concat(prefix, ".input.action.policyEpoch")),
            deadline: uint64(jsonContent.readUint(string.concat(prefix, ".input.action.deadline"))),
            decisionHash: jsonContent.readBytes32(string.concat(prefix, ".input.action.decisionHash"))
        });

        bytes32 expectedDigest = jsonContent.readBytes32(string.concat(prefix, ".expected.actionDigest"));
        bytes32 computedDigest = HashV41.computeActionDigest(chainId, safe, module, mandateHash, sessionId, action);
        assertEq(computedDigest, expectedDigest, string.concat("ActionDigest mismatch for ", prefix));
    }

    // 6. CandidateId verification
    function test_vector_candidate_id() public view {
        bytes32 snapshotBlockHash = jsonContent.readBytes32(".candidate_id.input.snapshotBlockHash");
        bytes32 actionDigestWithoutDecision = jsonContent.readBytes32(".candidate_id.input.actionDigestWithoutDecision");

        bytes32 expectedCandidateId = jsonContent.readBytes32(".candidate_id.expected.candidateId");
        bytes32 computedCandidateId = HashV41.computeCandidateId(snapshotBlockHash, actionDigestWithoutDecision);
        assertEq(computedCandidateId, expectedCandidateId, "candidateId mismatch");
    }

    // 7. Commitments verification: empty, single leaf, three leaves
    function test_commitments_empty() public view {
        bytes32 expectedRoot = jsonContent.readBytes32(".commitments.empty_commitment.expected.root");
        bytes32 computedRoot = HashV41.computeMerkleRoot(new bytes32[](0));
        assertEq(computedRoot, HashV41.EMPTY_OBLIGATIONS_ROOT, "empty root must equal EMPTY_OBLIGATIONS_ROOT");
        assertEq(computedRoot, expectedRoot, "empty commitment root mismatch");
    }

    function test_commitments_single_leaf() public view {
        bytes32 obligationIdHash = jsonContent.readBytes32(".commitments.single_leaf.input.obligationIdHash");
        bytes32 payeeIdHash = jsonContent.readBytes32(".commitments.single_leaf.input.payeeIdHash");
        uint256 amountRaw = jsonContent.readUint(".commitments.single_leaf.input.amountRaw");
        uint64 dueAt = uint64(jsonContent.readUint(".commitments.single_leaf.input.dueAt"));

        bytes32 expectedLeaf = jsonContent.readBytes32(".commitments.single_leaf.expected.leaf");
        bytes32 expectedRoot = jsonContent.readBytes32(".commitments.single_leaf.expected.root");

        bytes32 computedLeaf = HashV41.computeObligationLeaf(obligationIdHash, payeeIdHash, amountRaw, dueAt);
        assertEq(computedLeaf, expectedLeaf, "single leaf hash mismatch");

        bytes32[] memory leaves = new bytes32[](1);
        leaves[0] = computedLeaf;
        bytes32 computedRoot = HashV41.computeMerkleRoot(leaves);
        assertEq(computedRoot, expectedRoot, "single leaf root mismatch");
        assertEq(computedRoot, computedLeaf, "single leaf root must equal leaf");

        bytes32[] memory computedProof = HashV41.computeMerkleProof(leaves, 0);
        assertEq(computedProof.length, 0, "single leaf proof must be empty");
        assertTrue(HashV41.verifyObligationProof(computedProof, computedRoot, computedLeaf), "single leaf proof verify failed");
    }

    function test_commitments_three_leaves() public view {
        bytes32[] memory expectedSortedLeaves = jsonContent.readBytes32Array(".commitments.three_leaves.expected.sortedLeaves");
        bytes32 expectedRoot = jsonContent.readBytes32(".commitments.three_leaves.expected.root");

        // Recompute leaves
        bytes32[] memory rawLeaves = new bytes32[](3);
        for (uint256 i = 0; i < 3; i++) {
            string memory base = string.concat(".commitments.three_leaves.input.obligations[", vm.toString(i), "]");
            bytes32 obIdHash = jsonContent.readBytes32(string.concat(base, ".obligationIdHash"));
            bytes32 payeeIdHash = jsonContent.readBytes32(string.concat(base, ".payeeIdHash"));
            uint256 amountRaw = jsonContent.readUint(string.concat(base, ".amountRaw"));
            uint64 dueAt = uint64(jsonContent.readUint(string.concat(base, ".dueAt")));
            rawLeaves[i] = HashV41.computeObligationLeaf(obIdHash, payeeIdHash, amountRaw, dueAt);
        }

        bytes32[] memory sortedLeaves = HashV41.sortLeaves(rawLeaves);
        assertEq(sortedLeaves.length, 3);
        for (uint256 i = 0; i < 3; i++) {
            assertEq(sortedLeaves[i], expectedSortedLeaves[i], "sorted leaf mismatch");
        }

        bytes32 computedRoot = HashV41.computeMerkleRoot(sortedLeaves);
        assertEq(computedRoot, expectedRoot, "three leaves Merkle root mismatch");

        // Verify proofs for all 3 leaves
        for (uint256 i = 0; i < 3; i++) {
            string memory proofPath = string.concat(".commitments.three_leaves.expected.proofs[", vm.toString(i), "]");
            bytes32[] memory expectedProof = jsonContent.readBytes32Array(proofPath);
            bytes32[] memory computedProof = HashV41.computeMerkleProof(sortedLeaves, i);

            assertEq(computedProof.length, expectedProof.length, "proof length mismatch");
            for (uint256 p = 0; p < computedProof.length; p++) {
                assertEq(computedProof[p], expectedProof[p], "proof element mismatch");
            }

            assertTrue(
                HashV41.verifyObligationProof(computedProof, computedRoot, sortedLeaves[i]),
                "valid proof must verify"
            );

            // Negative test: tampered leaf must fail verification
            bytes32 tamperedLeaf = bytes32(uint256(sortedLeaves[i]) ^ 1);
            assertFalse(
                HashV41.verifyObligationProof(computedProof, computedRoot, tamperedLeaf),
                "tampered leaf must not verify"
            );
        }
    }

    // 8. DecisionHash verification
    function test_vector_decision_hash() public view {
        string memory canonicalJson = jsonContent.readString(".decision_hash.input.canonicalPassportJson");
        bytes32 expectedDecisionHash = jsonContent.readBytes32(".decision_hash.expected.decisionHash");

        bytes32 computedDecisionHash = HashV41.computeDecisionHash(canonicalJson);
        assertEq(computedDecisionHash, expectedDecisionHash, "decisionHash mismatch");
        assertEq(computedDecisionHash, keccak256(bytes(canonicalJson)), "decisionHash must equal keccak256 of UTF8 string");
    }

    // 9. Merkle property tests (pair commutativity, double hash)
    function testFuzz_hashPair_commutativity(bytes32 a, bytes32 b) public pure {
        assertEq(HashV41.hashPair(a, b), HashV41.hashPair(b, a), "hashPair must be commutative");
    }

    function test_double_hash_leaf_safety() public pure {
        // Inner hash
        bytes32 inner = keccak256(abi.encode(bytes32(0), bytes32(0), uint256(1), uint64(2)));
        // Outer hash
        bytes32 outer = keccak256(bytes.concat(inner));
        assertEq(HashV41.computeObligationLeaf(bytes32(0), bytes32(0), 1, 2), outer);
    }
}
