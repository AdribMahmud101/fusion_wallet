// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {SafeScript} from "../../src/contracts/script/SafeScript.sol";
import {IKavachiModule as I} from "../../src/contracts/src/IKavachiModule.sol";

/// @notice 2-of-2 owner strategy mandate only. No asset transfer, policy limit
/// increase, session extension, session reset or module call is performed here.
contract OwnerAllocationMandate is SafeScript {
    function run() external {
        require(block.chainid == 84532, "Base Sepolia only");
        I module = I(vm.envAddress("MODULE_ADDRESS"));
        I.ModuleState memory s = module.getState();
        I.Policy memory p = module.getPolicy();
        address safe = vm.envAddress("SAFE_ADDRESS");
        address owner = vm.envAddress("OWNER_ADDRESS");
        uint256 amount = vm.envUint("K38_AMOUNT");
        uint64 expiry = uint64(vm.envUint("K38_EXPIRY"));
        require(owner.code.length == 0, "mandate recipient must be inert EOA");
        require(s.configured && !s.paused && !module.getQueue().exists && s.reservedRaw == 0, "lifecycle changed");
        require(s.actionNonce == vm.envUint("K38_NONCE") && s.sessionId == vm.envBytes32("K38_SESSION_ID")
            && s.policyHash == vm.envBytes32("K38_POLICY_HASH"), "prepared state changed");
        require(amount > 0 && amount <= 10e6 && amount <= p.microMaxRaw, "allocation exceeds micro tier");
        require(expiry > block.timestamp + 600 && expiry <= s.sessionExpiresAt, "invalid expiry");
        bytes memory payload = abi.encodeWithSignature(
            "fusionAllocationMandate(uint256,address,address,address,bytes32,bytes32,uint256,uint256,uint64)",
            block.chainid, safe, address(module), vm.envAddress("K38_VAULT"), s.sessionId, s.policyHash,
            amount, s.actionNonce, expiry
        );
        require(keccak256(payload) == vm.envBytes32("K38_PAYLOAD_HASH"), "prepared payload changed");
        _safeTx(safe, owner, payload, 0);
        I.ModuleState memory afterState = module.getState();
        require(keccak256(abi.encode(afterState)) == keccak256(abi.encode(s)), "owner mandate changed module state");
    }
}
