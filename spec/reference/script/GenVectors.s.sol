// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import "forge-std/Script.sol";
import {IKavachiModule} from "../src/IKavachiModule.sol";
import {HashV41} from "../src/HashV41.sol";

contract GenVectors is Script {
    // Pinned test constants
    uint256 internal constant CHAIN_ID = 84532;
    address internal constant SAFE = 0x000000000000000000000000000000000000dEaD;
    address internal constant MODULE = 0x00000000000000000000000000000000000000B0;
    address internal constant VAULT_A = 0x00000000000000000000000000000000000000A1;
    address internal constant VAULT_B = 0x00000000000000000000000000000000000000b2;
    address internal constant GUARDIAN = 0x0000000000000000000000000000000000000007;
    address internal constant SESSION_KEY = 0x000000000000000000000000000000000000005E;
    address internal constant RECIPIENT = 0x00000000000000000000000000000000000000f1;

    string internal constant CANONICAL_PASSPORT_JSON =
        '{"candidateSet":[{"assets":"10000000","id":"0x6666666666666666666666666666666666666666666666666666666666666666","kind":"DEPOSIT","label":"A","reason":"SURPLUS_ABOVE_THRESHOLD"}],"candidateSetDigest":"0x5555555555555555555555555555555555555555555555555555555555555555","chainId":84532,"choiceId":"0x6666666666666666666666666666666666666666666666666666666666666666","choiceLabel":"A","createdAtChain":1792239375,"mandateHash":"0x4444444444444444444444444444444444444444444444444444444444444444","module":"0x00000000000000000000000000000000000000B0","planner":{"ahem":{"g":{"forcedTokens":0,"mode":"OFF","retries":0,"vetoes":0},"l":{"mode":"OFF"}},"host":"linux-host","kind":"deterministic","libMnnSha256":"0x0000000000000000000000000000000000000000000000000000000000000000","libllmSha256":"0x0000000000000000000000000000000000000000000000000000000000000000","modelId":"none","modelSha256":"0x0000000000000000000000000000000000000000000000000000000000000000"},"reasonCode":"SURPLUS_ABOVE_THRESHOLD","route":"INSTANT","safe":"0x000000000000000000000000000000000000dEaD","snapshot":{"block":48238481,"hash":"0x3333333333333333333333333333333333333333333333333333333333333333"},"userGoalSha256":"0x7777777777777777777777777777777777777777777777777777777777777777","version":"4.1"}';

    function run() external {
        string memory outputPath = vm.envOr("VECTOR_OUT_PATH", string("../vectors/v4_1.json"));
        string memory json = generateJson();
        vm.writeFile(outputPath, json);
        console.log("Wrote vectors to:", outputPath);
    }

    function generateJson() public pure returns (string memory) {
        // 1. Policy & Vault Limits
        IKavachiModule.VaultLimit[] memory vaults = new IKavachiModule.VaultLimit[](2);
        vaults[0] = IKavachiModule.VaultLimit({vault: VAULT_A, maxExposureRaw: 6000000000});
        vaults[1] = IKavachiModule.VaultLimit({vault: VAULT_B, maxExposureRaw: 6000000000});

        IKavachiModule.Policy memory policy = IKavachiModule.Policy({
            permissionTier: IKavachiModule.PermissionTier.AUTONOMOUS,
            operatingBufferRaw: 500000000,
            microMaxRaw: 10000000,
            delayedMaxRaw: 100000000,
            hardMaxRaw: 2500000000,
            dailyLimitRaw: 4000000000,
            delaySeconds: 180,
            maxShareSlippageBps: 50,
            guardian: GUARDIAN,
            vaults: vaults,
            payeeDelaySeconds: 3600,
            payEarlyWindowSeconds: 259200
        });

        bytes32 vaultLimitsHash = HashV41.computeVaultLimitsHash(vaults);
        bytes32 policyHash = HashV41.computePolicyHash(policy);

        // 2. SessionId (with and without session)
        uint256 sessionStartEpoch = 1;
        uint64 sessionExpiresAt = 1792325775;
        uint256 sessionLimitRaw = 8000000000;

        bytes32 sessionIdWithSession = HashV41.computeSessionId(
            CHAIN_ID,
            SAFE,
            MODULE,
            sessionStartEpoch,
            SESSION_KEY,
            sessionExpiresAt,
            sessionLimitRaw
        );
        bytes32 sessionIdWithoutSession = HashV41.computeSessionId(
            CHAIN_ID,
            SAFE,
            MODULE,
            0,
            address(0),
            0,
            0
        );

        // 3. DecisionHash
        bytes32 decisionHash = HashV41.computeDecisionHash(CANONICAL_PASSPORT_JSON);

        // 4. Obligations commitments
        // 4a. Empty commitment
        bytes32 emptyRoot = HashV41.computeMerkleRoot(new bytes32[](0));

        // 4b. Single-leaf proof
        bytes32 singleObligationIdHash = keccak256(bytes("bill_single"));
        bytes32 singlePayeeIdHash = keccak256(bytes("payee_single"));
        uint256 singleAmountRaw = 100000000;
        uint64 singleDueAt = 1792241000;
        bytes32 singleLeaf = HashV41.computeObligationLeaf(
            singleObligationIdHash,
            singlePayeeIdHash,
            singleAmountRaw,
            singleDueAt
        );
        bytes32[] memory singleLeaves = new bytes32[](1);
        singleLeaves[0] = singleLeaf;
        bytes32 singleRoot = HashV41.computeMerkleRoot(singleLeaves);
        bytes32[] memory singleProof = HashV41.computeMerkleProof(singleLeaves, 0);

        // 4c. Three-leaf proof
        bytes32 b1ObligationIdHash = keccak256(bytes("bill_001"));
        bytes32 b1PayeeIdHash = keccak256(bytes("payee_acme"));
        uint256 b1AmountRaw = 75000000;
        uint64 b1DueAt = 1792241000;
        bytes32 leaf1 = HashV41.computeObligationLeaf(b1ObligationIdHash, b1PayeeIdHash, b1AmountRaw, b1DueAt);

        bytes32 b2ObligationIdHash = keccak256(bytes("bill_002"));
        bytes32 b2PayeeIdHash = keccak256(bytes("payee_beta"));
        uint256 b2AmountRaw = 150000000;
        uint64 b2DueAt = 1792242000;
        bytes32 leaf2 = HashV41.computeObligationLeaf(b2ObligationIdHash, b2PayeeIdHash, b2AmountRaw, b2DueAt);

        bytes32 b3ObligationIdHash = keccak256(bytes("bill_003"));
        bytes32 b3PayeeIdHash = keccak256(bytes("payee_gamma"));
        uint256 b3AmountRaw = 300000000;
        uint64 b3DueAt = 1792243000;
        bytes32 leaf3 = HashV41.computeObligationLeaf(b3ObligationIdHash, b3PayeeIdHash, b3AmountRaw, b3DueAt);

        bytes32[] memory threeLeavesUnsorted = new bytes32[](3);
        threeLeavesUnsorted[0] = leaf1;
        threeLeavesUnsorted[1] = leaf2;
        threeLeavesUnsorted[2] = leaf3;
        bytes32[] memory threeLeaves = HashV41.sortLeaves(threeLeavesUnsorted);

        bytes32 threeRoot = HashV41.computeMerkleRoot(threeLeaves);
        bytes32[] memory proof0 = HashV41.computeMerkleProof(threeLeaves, 0);
        bytes32[] memory proof1 = HashV41.computeMerkleProof(threeLeaves, 1);
        bytes32[] memory proof2 = HashV41.computeMerkleProof(threeLeaves, 2);

        // 5. MandateHash
        uint256 policyEpoch = 1;
        uint256 reservedRaw = 1500000000;
        bytes32 mandateHash = HashV41.computeMandateHash(
            CHAIN_ID,
            SAFE,
            MODULE,
            policyEpoch,
            policyHash,
            threeRoot,
            reservedRaw,
            sessionIdWithSession,
            SESSION_KEY,
            sessionExpiresAt,
            sessionLimitRaw,
            false
        );

        // 6. Actions (one per S kind + REBALANCE)
        // 6a. DEPOSIT
        IKavachiModule.Action memory depositAction = IKavachiModule.Action({
            kind: IKavachiModule.ActionKind.DEPOSIT,
            sourceVault: address(0),
            destinationVault: VAULT_A,
            assets: 10000000,
            minSharesOut: 9950000,
            maxSharesBurn: 0,
            obligationIdHash: bytes32(0),
            payeeIdHash: bytes32(0),
            recipient: address(0),
            dueAt: 0,
            expectedNonce: 0,
            policyEpoch: 1,
            deadline: 1792240275,
            decisionHash: decisionHash
        });
        bytes32 depositDigest = HashV41.computeActionDigest(
            CHAIN_ID,
            SAFE,
            MODULE,
            mandateHash,
            sessionIdWithSession,
            depositAction
        );

        // 6b. WITHDRAW
        IKavachiModule.Action memory withdrawAction = IKavachiModule.Action({
            kind: IKavachiModule.ActionKind.WITHDRAW,
            sourceVault: VAULT_A,
            destinationVault: address(0),
            assets: 5000000,
            minSharesOut: 0,
            maxSharesBurn: 5050000,
            obligationIdHash: bytes32(0),
            payeeIdHash: bytes32(0),
            recipient: address(0),
            dueAt: 0,
            expectedNonce: 1,
            policyEpoch: 1,
            deadline: 1792240275,
            decisionHash: keccak256(bytes("withdraw_decision"))
        });
        bytes32 withdrawDigest = HashV41.computeActionDigest(
            CHAIN_ID,
            SAFE,
            MODULE,
            mandateHash,
            sessionIdWithSession,
            withdrawAction
        );

        // 6c. REBALANCE
        IKavachiModule.Action memory rebalanceAction = IKavachiModule.Action({
            kind: IKavachiModule.ActionKind.REBALANCE,
            sourceVault: VAULT_A,
            destinationVault: VAULT_B,
            assets: 8000000,
            minSharesOut: 7960000,
            maxSharesBurn: 8040000,
            obligationIdHash: bytes32(0),
            payeeIdHash: bytes32(0),
            recipient: address(0),
            dueAt: 0,
            expectedNonce: 2,
            policyEpoch: 1,
            deadline: 1792240275,
            decisionHash: keccak256(bytes("rebalance_decision"))
        });
        bytes32 rebalanceDigest = HashV41.computeActionDigest(
            CHAIN_ID,
            SAFE,
            MODULE,
            mandateHash,
            sessionIdWithSession,
            rebalanceAction
        );

        // 6d. PAY_OBLIGATION
        IKavachiModule.Action memory payAction = IKavachiModule.Action({
            kind: IKavachiModule.ActionKind.PAY_OBLIGATION,
            sourceVault: address(0),
            destinationVault: address(0),
            assets: 75000000,
            minSharesOut: 0,
            maxSharesBurn: 0,
            obligationIdHash: b1ObligationIdHash,
            payeeIdHash: b1PayeeIdHash,
            recipient: RECIPIENT,
            dueAt: b1DueAt,
            expectedNonce: 3,
            policyEpoch: 1,
            deadline: 1792240275,
            decisionHash: keccak256(bytes("pay_decision"))
        });
        bytes32 payDigest = HashV41.computeActionDigest(
            CHAIN_ID,
            SAFE,
            MODULE,
            mandateHash,
            sessionIdWithSession,
            payAction
        );

        // 7. CandidateId (from DEPOSIT action with decisionHash = 0)
        bytes32 snapshotBlockHash = 0x3333333333333333333333333333333333333333333333333333333333333333;
        IKavachiModule.Action memory candidateAction = IKavachiModule.Action({
            kind: depositAction.kind,
            sourceVault: depositAction.sourceVault,
            destinationVault: depositAction.destinationVault,
            assets: depositAction.assets,
            minSharesOut: depositAction.minSharesOut,
            maxSharesBurn: depositAction.maxSharesBurn,
            obligationIdHash: depositAction.obligationIdHash,
            payeeIdHash: depositAction.payeeIdHash,
            recipient: depositAction.recipient,
            dueAt: depositAction.dueAt,
            expectedNonce: depositAction.expectedNonce,
            policyEpoch: depositAction.policyEpoch,
            deadline: depositAction.deadline,
            decisionHash: bytes32(0)
        });
        bytes32 actionDigestWithoutDecision = HashV41.computeActionDigest(
            CHAIN_ID,
            SAFE,
            MODULE,
            mandateHash,
            sessionIdWithSession,
            candidateAction
        );
        bytes32 candidateId = HashV41.computeCandidateId(snapshotBlockHash, actionDigestWithoutDecision);

        // String serialization
        string memory buf = "{\n";
        buf = string.concat(buf, '  "version": "4.1",\n');
        buf = string.concat(buf, '  "description": "Kavachi V4.1 reference hashing and interface test vectors",\n');

        // Constants
        buf = string.concat(buf, '  "constants": {\n');
        buf = string.concat(buf, '    "TAG_POLICY": "', vm.toString(HashV41.TAG_POLICY), '",\n');
        buf = string.concat(buf, '    "TAG_SESSION": "', vm.toString(HashV41.TAG_SESSION), '",\n');
        buf = string.concat(buf, '    "TAG_MANDATE": "', vm.toString(HashV41.TAG_MANDATE), '",\n');
        buf = string.concat(buf, '    "TAG_ACTION": "', vm.toString(HashV41.TAG_ACTION), '",\n');
        buf = string.concat(buf, '    "TAG_CANDIDATE": "', vm.toString(HashV41.TAG_CANDIDATE), '",\n');
        buf = string.concat(buf, '    "EMPTY_OBLIGATIONS_ROOT": "', vm.toString(HashV41.EMPTY_OBLIGATIONS_ROOT), '"\n');
        buf = string.concat(buf, '  },\n');

        // Policy
        buf = string.concat(buf, '  "policy_hash": {\n');
        buf = string.concat(buf, '    "input": {\n');
        buf = string.concat(buf, '      "permissionTier": "2",\n');
        buf = string.concat(buf, '      "permissionTierName": "AUTONOMOUS",\n');
        buf = string.concat(buf, '      "operatingBufferRaw": "', vm.toString(policy.operatingBufferRaw), '",\n');
        buf = string.concat(buf, '      "microMaxRaw": "', vm.toString(policy.microMaxRaw), '",\n');
        buf = string.concat(buf, '      "delayedMaxRaw": "', vm.toString(policy.delayedMaxRaw), '",\n');
        buf = string.concat(buf, '      "hardMaxRaw": "', vm.toString(policy.hardMaxRaw), '",\n');
        buf = string.concat(buf, '      "dailyLimitRaw": "', vm.toString(policy.dailyLimitRaw), '",\n');
        buf = string.concat(buf, '      "delaySeconds": "', vm.toString(policy.delaySeconds), '",\n');
        buf = string.concat(buf, '      "maxShareSlippageBps": "', vm.toString(policy.maxShareSlippageBps), '",\n');
        buf = string.concat(buf, '      "guardian": "', vm.toString(policy.guardian), '",\n');
        buf = string.concat(buf, '      "vaults": [\n');
        buf = string.concat(buf, '        {"vault": "', vm.toString(vaults[0].vault), '", "maxExposureRaw": "', vm.toString(vaults[0].maxExposureRaw), '"},\n');
        buf = string.concat(buf, '        {"vault": "', vm.toString(vaults[1].vault), '", "maxExposureRaw": "', vm.toString(vaults[1].maxExposureRaw), '"}\n');
        buf = string.concat(buf, '      ],\n');
        buf = string.concat(buf, '      "payeeDelaySeconds": "', vm.toString(policy.payeeDelaySeconds), '",\n');
        buf = string.concat(buf, '      "payEarlyWindowSeconds": "', vm.toString(policy.payEarlyWindowSeconds), '"\n');
        buf = string.concat(buf, '    },\n');
        buf = string.concat(buf, '    "expected": {\n');
        buf = string.concat(buf, '      "vaultLimitsHash": "', vm.toString(vaultLimitsHash), '",\n');
        buf = string.concat(buf, '      "policyHash": "', vm.toString(policyHash), '"\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  },\n');

        // Session ID
        buf = string.concat(buf, '  "session_id": {\n');
        buf = string.concat(buf, '    "with_session": {\n');
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "chainId": "', vm.toString(CHAIN_ID), '",\n');
        buf = string.concat(buf, '        "safe": "', vm.toString(SAFE), '",\n');
        buf = string.concat(buf, '        "module": "', vm.toString(MODULE), '",\n');
        buf = string.concat(buf, '        "sessionStartEpoch": "', vm.toString(sessionStartEpoch), '",\n');
        buf = string.concat(buf, '        "sessionKey": "', vm.toString(SESSION_KEY), '",\n');
        buf = string.concat(buf, '        "sessionExpiresAt": "', vm.toString(sessionExpiresAt), '",\n');
        buf = string.concat(buf, '        "sessionLimitRaw": "', vm.toString(sessionLimitRaw), '"\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "sessionId": "', vm.toString(sessionIdWithSession), '"\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    },\n');
        buf = string.concat(buf, '    "without_session": {\n');
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "chainId": "', vm.toString(CHAIN_ID), '",\n');
        buf = string.concat(buf, '        "safe": "', vm.toString(SAFE), '",\n');
        buf = string.concat(buf, '        "module": "', vm.toString(MODULE), '",\n');
        buf = string.concat(buf, '        "sessionStartEpoch": "0",\n');
        buf = string.concat(buf, '        "sessionKey": "0x0000000000000000000000000000000000000000",\n');
        buf = string.concat(buf, '        "sessionExpiresAt": "0",\n');
        buf = string.concat(buf, '        "sessionLimitRaw": "0"\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "sessionId": "', vm.toString(sessionIdWithoutSession), '"\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  },\n');

        // MandateHash
        buf = string.concat(buf, '  "mandate_hash": {\n');
        buf = string.concat(buf, '    "input": {\n');
        buf = string.concat(buf, '      "chainId": "', vm.toString(CHAIN_ID), '",\n');
        buf = string.concat(buf, '      "safe": "', vm.toString(SAFE), '",\n');
        buf = string.concat(buf, '      "module": "', vm.toString(MODULE), '",\n');
        buf = string.concat(buf, '      "policyEpoch": "', vm.toString(policyEpoch), '",\n');
        buf = string.concat(buf, '      "policyHash": "', vm.toString(policyHash), '",\n');
        buf = string.concat(buf, '      "obligationsRoot": "', vm.toString(threeRoot), '",\n');
        buf = string.concat(buf, '      "reservedRaw": "', vm.toString(reservedRaw), '",\n');
        buf = string.concat(buf, '      "sessionId": "', vm.toString(sessionIdWithSession), '",\n');
        buf = string.concat(buf, '      "sessionKey": "', vm.toString(SESSION_KEY), '",\n');
        buf = string.concat(buf, '      "sessionExpiresAt": "', vm.toString(sessionExpiresAt), '",\n');
        buf = string.concat(buf, '      "sessionLimitRaw": "', vm.toString(sessionLimitRaw), '",\n');
        buf = string.concat(buf, '      "paused": false\n');
        buf = string.concat(buf, '    },\n');
        buf = string.concat(buf, '    "expected": {\n');
        buf = string.concat(buf, '      "mandateHash": "', vm.toString(mandateHash), '"\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  },\n');

        // Actions
        buf = string.concat(buf, '  "actions": {\n');
        buf = string.concat(buf, '    "deposit": ', formatActionJson(depositAction, mandateHash, sessionIdWithSession, depositDigest), ',\n');
        buf = string.concat(buf, '    "withdraw": ', formatActionJson(withdrawAction, mandateHash, sessionIdWithSession, withdrawDigest), ',\n');
        buf = string.concat(buf, '    "rebalance": ', formatActionJson(rebalanceAction, mandateHash, sessionIdWithSession, rebalanceDigest), ',\n');
        buf = string.concat(buf, '    "pay_obligation": ', formatActionJson(payAction, mandateHash, sessionIdWithSession, payDigest), '\n');
        buf = string.concat(buf, '  },\n');

        // CandidateId
        buf = string.concat(buf, '  "candidate_id": {\n');
        buf = string.concat(buf, '    "input": {\n');
        buf = string.concat(buf, '      "snapshotBlockHash": "', vm.toString(snapshotBlockHash), '",\n');
        buf = string.concat(buf, '      "actionDigestWithoutDecision": "', vm.toString(actionDigestWithoutDecision), '"\n');
        buf = string.concat(buf, '    },\n');
        buf = string.concat(buf, '    "expected": {\n');
        buf = string.concat(buf, '      "candidateId": "', vm.toString(candidateId), '"\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  },\n');

        // Commitments
        buf = string.concat(buf, '  "commitments": {\n');
        buf = string.concat(buf, '    "empty_commitment": {\n');
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "leaves": [],\n');
        buf = string.concat(buf, '        "reservedRaw": "0",\n');
        buf = string.concat(buf, '        "count": "0"\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "root": "', vm.toString(emptyRoot), '"\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    },\n');

        // Single leaf
        buf = string.concat(buf, '    "single_leaf": {\n');
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "obligationId": "bill_single",\n');
        buf = string.concat(buf, '        "obligationIdHash": "', vm.toString(singleObligationIdHash), '",\n');
        buf = string.concat(buf, '        "payeeIdHash": "', vm.toString(singlePayeeIdHash), '",\n');
        buf = string.concat(buf, '        "amountRaw": "', vm.toString(singleAmountRaw), '",\n');
        buf = string.concat(buf, '        "dueAt": "', vm.toString(singleDueAt), '"\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "leaf": "', vm.toString(singleLeaf), '",\n');
        buf = string.concat(buf, '        "root": "', vm.toString(singleRoot), '",\n');
        buf = string.concat(buf, '        "proof": ', formatProofArray(singleProof), ',\n');
        buf = string.concat(buf, '        "verified": true\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    },\n');

        // Three leaves
        buf = string.concat(buf, '    "three_leaves": {\n');
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "obligations": [\n');
        buf = string.concat(buf, '          {\n');
        buf = string.concat(buf, '            "obligationId": "bill_001",\n');
        buf = string.concat(buf, '            "obligationIdHash": "', vm.toString(b1ObligationIdHash), '",\n');
        buf = string.concat(buf, '            "payeeIdHash": "', vm.toString(b1PayeeIdHash), '",\n');
        buf = string.concat(buf, '            "amountRaw": "', vm.toString(b1AmountRaw), '",\n');
        buf = string.concat(buf, '            "dueAt": "', vm.toString(b1DueAt), '"\n');
        buf = string.concat(buf, '          },\n');
        buf = string.concat(buf, '          {\n');
        buf = string.concat(buf, '            "obligationId": "bill_002",\n');
        buf = string.concat(buf, '            "obligationIdHash": "', vm.toString(b2ObligationIdHash), '",\n');
        buf = string.concat(buf, '            "payeeIdHash": "', vm.toString(b2PayeeIdHash), '",\n');
        buf = string.concat(buf, '            "amountRaw": "', vm.toString(b2AmountRaw), '",\n');
        buf = string.concat(buf, '            "dueAt": "', vm.toString(b2DueAt), '"\n');
        buf = string.concat(buf, '          },\n');
        buf = string.concat(buf, '          {\n');
        buf = string.concat(buf, '            "obligationId": "bill_003",\n');
        buf = string.concat(buf, '            "obligationIdHash": "', vm.toString(b3ObligationIdHash), '",\n');
        buf = string.concat(buf, '            "payeeIdHash": "', vm.toString(b3PayeeIdHash), '",\n');
        buf = string.concat(buf, '            "amountRaw": "', vm.toString(b3AmountRaw), '",\n');
        buf = string.concat(buf, '            "dueAt": "', vm.toString(b3DueAt), '"\n');
        buf = string.concat(buf, '          }\n');
        buf = string.concat(buf, '        ]\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "sortedLeaves": [\n');
        buf = string.concat(buf, '          "', vm.toString(threeLeaves[0]), '",\n');
        buf = string.concat(buf, '          "', vm.toString(threeLeaves[1]), '",\n');
        buf = string.concat(buf, '          "', vm.toString(threeLeaves[2]), '"\n');
        buf = string.concat(buf, '        ],\n');
        buf = string.concat(buf, '        "root": "', vm.toString(threeRoot), '",\n');
        buf = string.concat(buf, '        "proofs": [\n');
        buf = string.concat(buf, '          ', formatProofArray(proof0), ',\n');
        buf = string.concat(buf, '          ', formatProofArray(proof1), ',\n');
        buf = string.concat(buf, '          ', formatProofArray(proof2), '\n');
        buf = string.concat(buf, '        ],\n');
        buf = string.concat(buf, '        "verified": [true, true, true]\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  },\n');

        // DecisionHash
        buf = string.concat(buf, '  "decision_hash": {\n');
        buf = string.concat(buf, '    "input": {\n');
        buf = string.concat(buf, '      "canonicalPassportJson": ', escapeJsonString(CANONICAL_PASSPORT_JSON), '\n');
        buf = string.concat(buf, '    },\n');
        buf = string.concat(buf, '    "expected": {\n');
        buf = string.concat(buf, '      "decisionHash": "', vm.toString(decisionHash), '"\n');
        buf = string.concat(buf, '    }\n');
        buf = string.concat(buf, '  }\n');
        buf = string.concat(buf, '}\n');

        return buf;
    }

    function formatActionJson(
        IKavachiModule.Action memory action,
        bytes32 mandateHash,
        bytes32 sessionId,
        bytes32 digest
    ) internal pure returns (string memory) {
        string memory buf = "{\n";
        buf = string.concat(buf, '      "input": {\n');
        buf = string.concat(buf, '        "chainId": "', vm.toString(CHAIN_ID), '",\n');
        buf = string.concat(buf, '        "safe": "', vm.toString(SAFE), '",\n');
        buf = string.concat(buf, '        "module": "', vm.toString(MODULE), '",\n');
        buf = string.concat(buf, '        "mandateHash": "', vm.toString(mandateHash), '",\n');
        buf = string.concat(buf, '        "sessionId": "', vm.toString(sessionId), '",\n');
        buf = string.concat(buf, '        "action": {\n');
        buf = string.concat(buf, '          "kind": "', vm.toString(uint8(action.kind)), '",\n');
        buf = string.concat(buf, '          "sourceVault": "', vm.toString(action.sourceVault), '",\n');
        buf = string.concat(buf, '          "destinationVault": "', vm.toString(action.destinationVault), '",\n');
        buf = string.concat(buf, '          "assets": "', vm.toString(action.assets), '",\n');
        buf = string.concat(buf, '          "minSharesOut": "', vm.toString(action.minSharesOut), '",\n');
        buf = string.concat(buf, '          "maxSharesBurn": "', vm.toString(action.maxSharesBurn), '",\n');
        buf = string.concat(buf, '          "obligationIdHash": "', vm.toString(action.obligationIdHash), '",\n');
        buf = string.concat(buf, '          "payeeIdHash": "', vm.toString(action.payeeIdHash), '",\n');
        buf = string.concat(buf, '          "recipient": "', vm.toString(action.recipient), '",\n');
        buf = string.concat(buf, '          "dueAt": "', vm.toString(action.dueAt), '",\n');
        buf = string.concat(buf, '          "expectedNonce": "', vm.toString(action.expectedNonce), '",\n');
        buf = string.concat(buf, '          "policyEpoch": "', vm.toString(action.policyEpoch), '",\n');
        buf = string.concat(buf, '          "deadline": "', vm.toString(action.deadline), '",\n');
        buf = string.concat(buf, '          "decisionHash": "', vm.toString(action.decisionHash), '"\n');
        buf = string.concat(buf, '        }\n');
        buf = string.concat(buf, '      },\n');
        buf = string.concat(buf, '      "expected": {\n');
        buf = string.concat(buf, '        "actionDigest": "', vm.toString(digest), '"\n');
        buf = string.concat(buf, '      }\n');
        buf = string.concat(buf, '    }');
        return buf;
    }

    function formatProofArray(bytes32[] memory proof) internal pure returns (string memory) {
        if (proof.length == 0) return "[]";
        string memory buf = "[";
        for (uint256 i = 0; i < proof.length; i++) {
            if (i > 0) buf = string.concat(buf, ", ");
            buf = string.concat(buf, '"', vm.toString(proof[i]), '"');
        }
        buf = string.concat(buf, "]");
        return buf;
    }

    function escapeJsonString(string memory s) internal pure returns (string memory) {
        bytes memory b = bytes(s);
        bytes memory escaped = new bytes(b.length * 2 + 2);
        uint256 idx = 0;
        escaped[idx++] = '"';
        for (uint256 i = 0; i < b.length; i++) {
            bytes1 char = b[i];
            if (char == '"') {
                escaped[idx++] = "\\";
                escaped[idx++] = '"';
            } else if (char == "\\") {
                escaped[idx++] = "\\";
                escaped[idx++] = "\\";
            } else {
                escaped[idx++] = char;
            }
        }
        escaped[idx++] = '"';
        bytes memory result = new bytes(idx);
        for (uint256 i = 0; i < idx; i++) {
            result[i] = escaped[i];
        }
        return string(result);
    }
}
