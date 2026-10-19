// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ModuleBase, I, KavachiModule, KavachiDemoVault} from "../ModuleBase.sol";

/// Anvil fixture, deliberately upgradeable. Implementation slot avoids ERC20 storage.
contract ArenaVaultProxy {
    bytes32 constant SLOT = keccak256("arena.implementation");

    constructor(address implementation) {
        upgrade(implementation);
    }

    function upgrade(address implementation) public {
        bytes32 slot = SLOT;
        assembly { sstore(slot, implementation) }
    }

    fallback() external payable {
        bytes32 slot = SLOT;
        assembly {
            let implementation := sload(slot)
            calldatacopy(0, 0, calldatasize())
            let ok := delegatecall(gas(), implementation, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch ok
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}

contract ArenaProxySwapTest is ModuleBase {
    /// Regression observation, not a claim that an undetected upgrade is safe.
    function test_A17_proxyImplementationSwapEscapesCodeHashCheck() public {
        ArenaVaultProxy proxy = new ArenaVaultProxy(address(a));
        module = new KavachiModule(address(treasury), address(token), address(proxy), address(b), address(sanctions));
        vm.prank(address(treasury));
        treasury.enableModule(address(module));
        a = KavachiDemoVault(address(proxy));
        p.vaults[0] = I.VaultLimit(address(a) < address(b) ? address(a) : address(b), 6000e6);
        p.vaults[1] = I.VaultLimit(address(a) < address(b) ? address(b) : address(a), 6000e6);
        vm.prank(address(treasury));
        module.configurePolicy(p);
        _commit(10e6);
        _session(8000e6);
        bytes32 beforeHash = address(proxy).codehash;
        KavachiDemoVault replacement = new KavachiDemoVault(token, "Swapped", "SW");
        proxy.upgrade(address(replacement));
        assertEq(address(proxy).codehash, beforeHash);
        _execute(_action(I.ActionKind.DEPOSIT, 1e6));
        assertEq(a.balanceOf(address(treasury)), 1e6, "Upgrade was not detected");
        assertFalse(module.getState().paused, "No VAULT_IMPL_CHANGED latch");
    }
}
