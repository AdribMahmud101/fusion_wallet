// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {Script, console2} from "forge-std/Script.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {HashV41} from "../src/HashV41.sol";
import {IKavachiModule as I} from "../src/IKavachiModule.sol";

/// @notice Read-only regeneration evidence. Prints computed hashes; never writes frozen spec files.
/// Run: forge script script/GenVectors.s.sol:GenVectors
contract GenVectors is Script {
    using stdJson for string;

    function run() external view {
        string memory j = vm.readFile("../../spec/vectors/v4_1.json");
        string[4] memory names = [string("deposit"), "withdraw", "rebalance", "pay_obligation"];
        for (uint256 i; i < 4; i++) {
            string memory base = string.concat(".actions.", names[i]);
            string memory path = string.concat(base, ".input.action");
            I.Action memory a;
            a.kind = I.ActionKind(uint8(j.readUint(string.concat(path, ".kind"))));
            a.sourceVault = j.readAddress(string.concat(path, ".sourceVault"));
            a.destinationVault = j.readAddress(string.concat(path, ".destinationVault"));
            a.assets = j.readUint(string.concat(path, ".assets"));
            a.minSharesOut = j.readUint(string.concat(path, ".minSharesOut"));
            a.maxSharesBurn = j.readUint(string.concat(path, ".maxSharesBurn"));
            a.obligationIdHash = j.readBytes32(string.concat(path, ".obligationIdHash"));
            a.payeeIdHash = j.readBytes32(string.concat(path, ".payeeIdHash"));
            a.recipient = j.readAddress(string.concat(path, ".recipient"));
            a.dueAt = uint64(j.readUint(string.concat(path, ".dueAt")));
            a.expectedNonce = j.readUint(string.concat(path, ".expectedNonce"));
            a.policyEpoch = j.readUint(string.concat(path, ".policyEpoch"));
            a.deadline = uint64(j.readUint(string.concat(path, ".deadline")));
            a.decisionHash = j.readBytes32(string.concat(path, ".decisionHash"));
            bytes32 digest = HashV41.computeActionDigest(
                j.readUint(string.concat(base, ".input.chainId")),
                j.readAddress(string.concat(base, ".input.safe")),
                j.readAddress(string.concat(base, ".input.module")),
                j.readBytes32(string.concat(base, ".input.mandateHash")),
                j.readBytes32(string.concat(base, ".input.sessionId")),
                a
            );
            require(digest == j.readBytes32(string.concat(base, ".expected.actionDigest")), "frozen vector mismatch");
            console2.log(names[i]);
            console2.logBytes32(digest);
        }
        console2.log("empty commitment");
        console2.logBytes32(HashV41.EMPTY_OBLIGATIONS_ROOT);
        console2.log("decision");
        console2.logBytes32(HashV41.computeDecisionHash(j.readString(".decision_hash.input.canonicalPassportJson")));
    }
}
