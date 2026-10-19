// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {SafeScript, IScriptSafe} from "./SafeScript.sol";
import {IKavachiModule as I} from "../src/IKavachiModule.sol";
import {HashV41} from "../src/HashV41.sol";

/// @notice D17: threshold-two MultiSendCallOnly setup, then immediately start payee cooling.
contract Setup is SafeScript {
    function run() external {
        address safe = vm.envAddress("SAFE_ADDRESS");
        I module = I(vm.envAddress("MODULE_ADDRESS"));
        if (!IScriptSafe(safe).isModuleEnabled(address(module))) {
            _safeTx(safe, safe, abi.encodeCall(IScriptSafe.enableModule, (address(module))), 0);
        }
        uint256 scale = vm.envOr("AMOUNT_DIVISOR", uint256(1));
        address a = vm.envAddress("VAULT_A");
        address b = vm.envAddress("VAULT_B");
        I.Policy memory policy;
        policy.permissionTier = I.PermissionTier.AUTONOMOUS;
        policy.operatingBufferRaw = 500e6 / scale;
        policy.microMaxRaw = 10e6 / scale;
        policy.delayedMaxRaw = 100e6 / scale;
        policy.hardMaxRaw = 2500e6 / scale;
        policy.dailyLimitRaw = 4000e6 / scale;
        policy.delaySeconds = 180;
        policy.maxShareSlippageBps = 50;
        policy.guardian = vm.envAddress("GUARDIAN_ADDRESS");
        policy.vaults = new I.VaultLimit[](2);
        policy.vaults[0] = I.VaultLimit(a < b ? a : b, 6000e6 / scale);
        policy.vaults[1] = I.VaultLimit(a < b ? b : a, 6000e6 / scale);
        policy.payeeDelaySeconds = 3600;
        policy.payEarlyWindowSeconds = 3 days;
        uint64 due = uint64(vm.envUint("OBLIGATIONS_DUE_AT"));
        bytes32[] memory leaves = new bytes32[](5);
        uint256[5] memory amounts = [uint256(8e6), 75e6, 150e6, 300e6, 50e6];
        for (uint256 i; i < 5; i++) {
            bytes32 payee = i < 2
                ? keccak256("fusion.k2.payee.acme")
                : i < 4 ? keccak256("fusion.k2.payee.payroll") : keccak256("fusion.k2.payee.newbank");
            leaves[i] = HashV41.computeObligationLeaf(
                keccak256(abi.encodePacked("fusion.k2.bill.", bytes1(uint8(i)))), payee, amounts[i] / scale, due
            );
        }
        bytes32 root = HashV41.computeMerkleRoot(HashV41.sortLeaves(leaves));
        uint256 epoch = module.getState().policyEpoch;
        bytes memory batch = bytes.concat(
            _entry(address(module), abi.encodeCall(I.configurePolicy, (policy))),
            _entry(address(module), abi.encodeCall(I.commitObligations, (root, 583e6 / scale, 5))),
            _entry(
                address(module),
                abi.encodeCall(
                    I.startSession,
                    (vm.envAddress("SESSION_ADDRESS"), uint64(block.timestamp + 23 hours), 8000e6 / scale)
                )
            ),
            _entry(address(module), abi.encodeCall(I.resume, (epoch + 2)))
        );
        // The Safe delegatecalls the trusted official MultiSend; every inner operation is Call.
        _safeTx(safe, vm.envAddress("SAFE_MULTISEND"), abi.encodeWithSignature("multiSend(bytes)", batch), 1);
        _safeTx(
            safe,
            address(module),
            abi.encodeCall(
                I.addPayee,
                (keccak256("fusion.k2.payee.acme"), vm.envAddress("OWNER_ADDRESS"), keccak256("Acme Hosting"))
            ),
            0
        );
        _safeTx(
            safe,
            address(module),
            abi.encodeCall(
                I.addPayee,
                (keccak256("fusion.k2.payee.payroll"), vm.envAddress("APPROVER_ADDRESS"), keccak256("Payroll Co."))
            ),
            0
        );
        require(!module.getState().paused && module.getState().reservedRaw == 583e6 / scale, "setup state mismatch");
    }
}
