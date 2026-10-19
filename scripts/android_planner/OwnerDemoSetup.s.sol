// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {SafeScript} from "../../src/contracts/script/SafeScript.sol";
import {IKavachiModule as I} from "../../src/contracts/src/IKavachiModule.sol";

/// @notice Explicit owner setup only, never APP agent evidence. Exact prepared payload guard.
contract OwnerDemoSetup is SafeScript {
    function run() external {
        I module = I(vm.envAddress("MODULE_ADDRESS"));
        address safe = vm.envAddress("SAFE_ADDRESS");
        I.ModuleState memory beforeState = module.getState();
        require(beforeState.configured && !beforeState.paused && !module.getQueue().exists, "setup lifecycle changed");
        require(beforeState.reservedRaw == 0, "unpaid commitment must never be replaced");
        require(beforeState.policyEpoch == vm.envUint("K17_EPOCH"), "setup epoch changed");
        require(beforeState.obligationsRoot == vm.envBytes32("K17_PREVIOUS_ROOT"), "setup root changed");
        require(beforeState.sessionKey == vm.envAddress("SESSION_ADDRESS"), "setup session changed");
        require(beforeState.sessionExpiresAt == vm.envUint("K17_EXPECTED_EXPIRY"), "setup expiry changed");
        uint256 remaining = beforeState.sessionLimitRaw - beforeState.sessionUsedRaw;
        require(remaining == vm.envUint("K17_REMAINING"), "setup allowance changed");
        bytes32 root = vm.envBytes32("K17_NEW_ROOT");
        bytes memory batch = bytes.concat(
            _entry(address(module), abi.encodeCall(I.commitObligations, (root, 8e6, 1))),
            _entry(address(module), abi.encodeCall(I.startSession, (beforeState.sessionKey, beforeState.sessionExpiresAt, remaining))),
            _entry(address(module), abi.encodeCall(I.resume, (beforeState.policyEpoch + 1)))
        );
        bytes memory data = abi.encodeWithSignature("multiSend(bytes)", batch);
        require(keccak256(data) == vm.envBytes32("K17_PAYLOAD_HASH"), "prepared setup bytes differ");
        _safeTx(safe, vm.envAddress("K17_MULTISEND"), data, 1);
        I.ModuleState memory afterState = module.getState();
        require(!afterState.paused && afterState.policyEpoch == beforeState.policyEpoch + 2, "setup epoch/lifecycle failed");
        require(afterState.reservedRaw == 8e6 && afterState.obligationCount == 1 && afterState.obligationsRoot == root, "setup commitment failed");
        require(afterState.sessionKey == beforeState.sessionKey && afterState.sessionExpiresAt == beforeState.sessionExpiresAt
            && afterState.sessionLimitRaw == remaining, "setup changed authority");
    }
}
