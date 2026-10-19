// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {Script, console2} from "forge-std/Script.sol";

interface IDeploySafe {
    function setup(address[] calldata, uint256, address, bytes calldata, address, address, uint256, address payable)
        external;
}

interface IDeployFactory {
    function createProxyWithNonce(address, bytes calldata, uint256) external returns (address);
}

/// @notice Deploys only; policy/session/payee writes are in Setup.s.sol. Keys are env-only.
contract Deploy is Script {
    function _official(string memory artifact) private returns (address deployed) {
        return _deploy(artifact, bytes(""));
    }

    function _deploy(string memory artifact, bytes memory arguments) private returns (address deployed) {
        bytes memory code = bytes.concat(vm.getCode(artifact), arguments);
        assembly ("memory-safe") { deployed := create(0, add(code, 32), mload(code)) }
        require(deployed.code.length != 0, "official artifact deployment failed");
    }

    function run() external {
        require(block.chainid == 31337 || block.chainid == 84532, "unsupported chain");
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address singleton = vm.envOr("SAFE_SINGLETON", address(0));
        address factory = vm.envOr("SAFE_FACTORY", address(0));
        address fallbackHandler = vm.envOr("SAFE_FALLBACK_HANDLER", address(0));
        address multiSend = vm.envOr("SAFE_MULTISEND", address(0));
        address asset = vm.envOr("TREASURY_ASSET", address(0));
        address vaultA = vm.envOr("VAULT_A", address(0));
        vm.startBroadcast(deployerKey);
        // SL5: official, unmodified v1.4.1 artifacts; Safe compiles in safe-legacy profile.
        if (singleton.code.length == 0) singleton = _official("out/Safe.sol/Safe.json");
        if (factory.code.length == 0) factory = _official("out/SafeProxyFactory.sol/SafeProxyFactory.json");
        if (fallbackHandler.code.length == 0) {
            fallbackHandler = _official("out/CompatibilityFallbackHandler.sol/CompatibilityFallbackHandler.json");
        }
        if (multiSend.code.length == 0) multiSend = _official("out/MultiSendCallOnly.sol/MultiSendCallOnly.json");
        if (asset == address(0)) {
            require(block.chainid == 31337, "MockUSDC is Anvil only");
            asset = _official("out/MockUSDC.sol/MockUSDC.json");
        }
        address[] memory owners = new address[](2);
        owners[0] = vm.envAddress("OWNER_ADDRESS");
        owners[1] = vm.envAddress("APPROVER_ADDRESS");
        bytes memory initializer = abi.encodeCall(
            IDeploySafe.setup, (owners, 2, address(0), bytes(""), fallbackHandler, address(0), 0, payable(address(0)))
        );
        address safe = IDeployFactory(factory).createProxyWithNonce(singleton, initializer, vm.envUint("DEPLOY_SALT"));
        if (vaultA == address(0)) {
            vaultA = _deploy(
                "out/KavachiDemoVault.sol/KavachiDemoVault.json", abi.encode(asset, "Kavachi Simulated A", "kSIM-A")
            );
        }
        address vaultB = _deploy(
            "out/KavachiDemoVault.sol/KavachiDemoVault.json", abi.encode(asset, "Kavachi Simulated B", "kSIM-B")
        );
        address sanctions = _deploy("out/KavachiTestSanctionsList.sol/KavachiTestSanctionsList.json", abi.encode(safe));
        address module =
            _deploy("out/KavachiModule.sol/KavachiModule.json", abi.encode(safe, asset, vaultA, vaultB, sanctions));
        vm.stopBroadcast();
        console2.log("DEPLOY_SAFE", safe);
        console2.log("DEPLOY_ASSET", asset);
        console2.log("DEPLOY_VAULT_A", vaultA);
        console2.log("DEPLOY_VAULT_B", vaultB);
        console2.log("DEPLOY_SANCTIONS", sanctions);
        console2.log("DEPLOY_MODULE", module);
        console2.log("DEPLOY_SINGLETON", singleton);
        console2.log("DEPLOY_FACTORY", factory);
        console2.log("DEPLOY_FALLBACK", fallbackHandler);
        console2.log("DEPLOY_MULTISEND", multiSend);
    }
}
