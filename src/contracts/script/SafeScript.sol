// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {Script} from "forge-std/Script.sol";

interface IScriptSafe {
    function nonce() external view returns (uint256);
    function getTransactionHash(
        address,
        uint256,
        bytes calldata,
        uint8,
        uint256,
        uint256,
        uint256,
        address,
        address,
        uint256
    ) external view returns (bytes32);
    function execTransaction(
        address,
        uint256,
        bytes calldata,
        uint8,
        uint256,
        uint256,
        uint256,
        address,
        address payable,
        bytes calldata
    ) external payable returns (bool);
    function enableModule(address module) external;
    function isModuleEnabled(address module) external view returns (bool);
}

abstract contract SafeScript is Script {
    function _safeTx(address safe, address to, bytes memory data, uint8 operation) internal {
        IScriptSafe account = IScriptSafe(safe);
        bytes32 digest =
            account.getTransactionHash(to, 0, data, operation, 0, 0, 0, address(0), address(0), account.nonce());
        uint256 ownerKey = vm.envUint("OWNER_PRIVATE_KEY");
        uint256 approverKey = vm.envUint("APPROVER_PRIVATE_KEY");
        require(vm.addr(ownerKey) == vm.envAddress("OWNER_ADDRESS"), "owner key/address mismatch");
        require(vm.addr(approverKey) == vm.envAddress("APPROVER_ADDRESS"), "approver key/address mismatch");
        (uint8 v1, bytes32 r1, bytes32 s1) = vm.sign(ownerKey, digest);
        (uint8 v2, bytes32 r2, bytes32 s2) = vm.sign(approverKey, digest);
        bytes memory sig1 = abi.encodePacked(r1, s1, v1);
        bytes memory sig2 = abi.encodePacked(r2, s2, v2);
        bytes memory signatures =
            vm.addr(ownerKey) < vm.addr(approverKey) ? bytes.concat(sig1, sig2) : bytes.concat(sig2, sig1);
        vm.broadcast(vm.envUint("DEPLOYER_PRIVATE_KEY"));
        require(
            account.execTransaction(to, 0, data, operation, 0, 0, 0, address(0), payable(address(0)), signatures),
            "Safe execution failed"
        );
    }

    function _entry(address target, bytes memory data) internal pure returns (bytes memory) {
        return abi.encodePacked(uint8(0), target, uint256(0), uint256(data.length), data);
    }
}
