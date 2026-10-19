// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {SafeScript} from "./SafeScript.sol";
import {IKavachiModule as I} from "../src/IKavachiModule.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

/// @notice K2 operational evidence only. Golden*.s.sol remains owned by K9.
contract EvidenceActions is SafeScript {
    function run() external {
        I module = I(vm.envAddress("MODULE_ADDRESS"));
        address safe = vm.envAddress("SAFE_ADDRESS");
        bytes32 operation = keccak256(bytes(vm.envString("EVIDENCE_OPERATION")));
        if (operation == keccak256("approve")) {
            I.Action memory action = abi.decode(vm.envBytes("ACTION_DATA"), (I.Action));
            _safeTx(safe, address(module), abi.encodeCall(I.approveAction, (action, uint64(block.timestamp + 900))), 0);
        } else if (operation == keccak256("execute") || operation == keccak256("queue")) {
            I.Action memory action = abi.decode(vm.envBytes("ACTION_DATA"), (I.Action));
            bytes32[] memory proof = abi.decode(vm.envBytes("PROOF_DATA"), (bytes32[]));
            vm.broadcast(vm.envUint("SESSION_PRIVATE_KEY"));
            if (operation == keccak256("execute")) module.execute(action, proof);
            else module.queueAction(action, proof);
        } else if (operation == keccak256("cancel")) {
            bytes32 digest = module.getQueue().actionDigest;
            vm.broadcast(vm.envUint("GUARDIAN_PRIVATE_KEY"));
            module.cancelQueuedAction(digest);
        } else if (operation == keccak256("pause")) {
            vm.broadcast(vm.envUint("GUARDIAN_PRIVATE_KEY"));
            module.pause(keccak256("K2_SCRIPT_STOP"));
        } else if (operation == keccak256("unwind")) {
            vm.broadcast(vm.envUint("GUARDIAN_PRIVATE_KEY"));
            module.emergencyUnwind(keccak256("K2_SCRIPT_UNWIND"));
        } else if (operation == keccak256("resume")) {
            _safeTx(safe, address(module), abi.encodeCall(I.resume, (module.getState().policyEpoch)), 0);
        } else if (operation == keccak256("ownerPay")) {
            I.Action memory action = abi.decode(vm.envBytes("ACTION_DATA"), (I.Action));
            bytes32[] memory proof = abi.decode(vm.envBytes("PROOF_DATA"), (bytes32[]));
            _safeTx(
                safe,
                address(module),
                abi.encodeCall(
                    I.ownerPayObligation,
                    (action.obligationIdHash, action.payeeIdHash, action.assets, action.dueAt, proof)
                ),
                0
            );
        } else if (operation == keccak256("newPayee")) {
            _safeTx(
                safe,
                address(module),
                abi.encodeCall(
                    I.addPayee,
                    (
                        keccak256("fusion.k2.payee.newbank"),
                        vm.envAddress("OPS_ADDRESS"),
                        keccak256("Acme new bank details")
                    )
                ),
                0
            );
        } else if (operation == keccak256("seedVaults")) {
            uint256 scale = vm.envOr("AMOUNT_DIVISOR", uint256(1));
            address asset = vm.envAddress("TREASURY_ASSET");
            address a = vm.envAddress("VAULT_A");
            address b = vm.envAddress("VAULT_B");
            I.Policy memory policy = module.getPolicy();
            require(
                IERC20(asset).balanceOf(safe)
                    >= 7000e6 / scale + module.getState().reservedRaw + policy.operatingBufferRaw,
                "owner allocation must preserve reserve and buffer"
            );
            require(
                IERC4626(a).convertToAssets(IERC20(a).balanceOf(safe)) + 3000e6 / scale <= 6000e6 / scale,
                "vault A exposure cap"
            );
            require(
                IERC4626(b).convertToAssets(IERC20(b).balanceOf(safe)) + 4000e6 / scale <= 6000e6 / scale,
                "vault B exposure cap"
            );
            bytes memory batch = bytes.concat(
                _entry(asset, abi.encodeCall(IERC20.approve, (a, 3000e6 / scale))),
                _entry(a, abi.encodeCall(IERC4626.deposit, (3000e6 / scale, safe))),
                _entry(asset, abi.encodeCall(IERC20.approve, (a, 0))),
                _entry(asset, abi.encodeCall(IERC20.approve, (b, 4000e6 / scale))),
                _entry(b, abi.encodeCall(IERC4626.deposit, (4000e6 / scale, safe))),
                _entry(asset, abi.encodeCall(IERC20.approve, (b, 0)))
            );
            _safeTx(safe, vm.envAddress("SAFE_MULTISEND"), abi.encodeWithSignature("multiSend(bytes)", batch), 1);
        } else {
            revert("unknown evidence operation");
        }
    }
}
