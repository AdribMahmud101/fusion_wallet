// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {
    ModuleBase,
    I,
    HashV41,
    KavachiModule,
    MockUSDC,
    KavachiDemoVault,
    KavachiTestSanctionsList
} from "./ModuleBase.sol";
import {NaiveAgentWallet} from "../src/NaiveAgentWallet.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract BoundariesTest is ModuleBase {
    function test_tierEdgesMicroDelayedAndOwner() public {
        _execute(_action(I.ActionKind.DEPOSIT, 10e6));
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 10e6 + 1);
        x.deadline = uint64(block.timestamp + 210);
        _queue(x);
        vm.warp(module.getQueue().readyAt);
        _execute(x);
        x = _action(I.ActionKind.DEPOSIT, 100e6);
        x.deadline = uint64(block.timestamp + 210);
        _queue(x);
        vm.warp(module.getQueue().readyAt);
        _execute(x);
        x = _action(I.ActionKind.DEPOSIT, 100e6 + 1);
        _expect(I.ApprovalRequired.selector, x);
        _approve(x);
        _execute(x);
        x = _action(I.ActionKind.DEPOSIT, 2500e6);
        _approve(x);
        _execute(x);
        _expect(I.HardLimitExceeded.selector, _action(I.ActionKind.DEPOSIT, 2500e6 + 1));
    }

    function test_payWindowExactBoundary() public {
        due = uint64(block.timestamp + 3 days + 1);
        _commit(10e6);
        _session(8000e6);
        _expect(I.PayWindowNotOpen.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
        vm.warp(block.timestamp + 1);
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_approvalValidAtExpiryAndConsumed() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 101e6);
        x.deadline = uint64(block.timestamp + 900);
        vm.prank(address(treasury));
        module.approveAction(x, uint64(block.timestamp + 900));
        bytes32 digest = module.hashAction(x);
        vm.warp(x.deadline);
        _execute(x);
        assertEq(module.approvalValidUntil(digest), 0);
        _expect(I.BadNonce.selector, x);
    }

    function test_policyHashSessionIdAndMandateFollowProductionHashing() public view {
        I.ModuleState memory s = module.getState();
        assertEq(s.policyHash, HashV41.computePolicyHash(module.getPolicy()));
        assertEq(
            s.sessionId,
            HashV41.computeSessionId(
                block.chainid,
                address(treasury),
                address(module),
                s.sessionStartEpoch,
                s.sessionKey,
                s.sessionExpiresAt,
                s.sessionLimitRaw
            )
        );
        assertEq(s.mandateHash, HashV41.computeMandateHash(block.chainid, address(treasury), address(module), s));
    }

    function test_payPreservesFreeCashAndMandateCommitsRemainingReserve() public {
        uint256 before_ = token.balanceOf(address(treasury)) - module.getState().reservedRaw;
        bytes32 mandate = module.getState().mandateHash;
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
        I.ModuleState memory s = module.getState();
        assertEq(token.balanceOf(address(treasury)) - s.reservedRaw, before_);
        assertNotEq(s.mandateHash, mandate);
        assertEq(s.mandateHash, HashV41.computeMandateHash(block.chainid, address(treasury), address(module), s));
    }

    function test_onlySafeAdministration() public {
        bytes[] memory calls = new bytes[](9);
        calls[0] = abi.encodeCall(module.configurePolicy, (p));
        calls[1] = abi.encodeCall(module.commitObligations, (HashV41.EMPTY_OBLIGATIONS_ROOT, 0, 0));
        calls[2] = abi.encodeCall(module.addPayee, (keccak256("new"), vendor, bytes32(0)));
        calls[3] = abi.encodeCall(module.startSession, (agent, uint64(block.timestamp + 10), 1));
        calls[4] = abi.encodeCall(module.resume, (module.getState().policyEpoch));
        calls[5] = abi.encodeCall(module.revokeSession, ());
        calls[6] = abi.encodeCall(module.approveAction, (_action(I.ActionKind.DEPOSIT, 1), uint64(block.timestamp + 1)));
        calls[7] = abi.encodeCall(module.revokeActionApproval, (bytes32(0)));
        calls[8] = abi.encodeCall(module.ownerPayObligation, (BILL, PAYEE, 10e6, due, new bytes32[](0)));
        for (uint256 i; i < calls.length; i++) {
            vm.prank(owner);
            (bool ok, bytes memory result) = address(module).call(calls[i]);
            assertFalse(ok);
            assertEq(bytes4(result), I.Unauthorized.selector);
        }
    }

    function test_agentCannotStopOrCallArbitrarily() public {
        bytes[] memory calls = new bytes[](5);
        calls[0] = abi.encodeCall(module.pause, (bytes32(0)));
        calls[1] = abi.encodeCall(module.cancelPayee, (PAYEE));
        calls[2] = abi.encodeCall(module.cancelQueuedAction, (bytes32(0)));
        calls[3] = abi.encodeCall(module.emergencyUnwind, (bytes32(0)));
        calls[4] =
            abi.encodeWithSignature("execute(address,bytes)", vendor, abi.encodeCall(IERC20.transfer, (vendor, 1e6)));
        for (uint256 i; i < calls.length; i++) {
            vm.prank(agent);
            (bool ok,) = address(module).call(calls[i]);
            assertFalse(ok);
        }
    }

    function test_liveOwnerRemovalRemovesStopAuthority() public {
        // Official Safe preserves setup owner order: sentinel -> owner -> approver.
        vm.prank(address(treasury));
        (bool ok,) = address(treasury)
            .call(abi.encodeWithSignature("removeOwner(address,address,uint256)", address(1), owner, 1));
        assertTrue(ok);
        vm.expectRevert(I.Unauthorized.selector);
        vm.prank(owner);
        module.pause(0);
        vm.prank(approver);
        module.pause(0);
        assertTrue(module.getState().paused);
    }

    function test_constructorRejectsInvalidDependencies() public {
        vm.expectRevert(I.InvalidPolicy.selector);
        new KavachiModule(address(0), address(token), address(a), address(b), address(sanctions));
        vm.expectRevert(I.InvalidPolicy.selector);
        new KavachiModule(address(treasury), address(token), address(a), address(a), address(sanctions));
        vm.expectRevert(I.InvalidPolicy.selector);
        new KavachiModule(address(treasury), address(token), address(token), address(b), address(sanctions));
        MockUSDC other = new MockUSDC();
        KavachiDemoVault wrongAsset = new KavachiDemoVault(other, "Wrong", "W");
        vm.expectRevert(I.InvalidPolicy.selector);
        new KavachiModule(address(treasury), address(token), address(wrongAsset), address(b), address(sanctions));
    }

    function test_testListOwnedBySafe() public {
        assertEq(sanctions.owner(), address(treasury));
        vm.expectRevert();
        vm.prank(owner);
        sanctions.setSanctioned(vendor, true);
    }

    function test_demoVaultDripIsDeployerOnlySimulatedYield() public {
        _seedShares(10e6);
        token.mint(address(this), 1e6);
        token.approve(address(a), 1e6);
        uint256 before_ = a.convertToAssets(a.balanceOf(address(treasury)));
        a.drip(1e6);
        assertGt(a.convertToAssets(a.balanceOf(address(treasury))), before_);
        vm.expectRevert("deployer only");
        vm.prank(agent);
        a.drip(1);
        // Share rounding after simulated yield must still permit an ordinary bounded withdrawal.
        _execute(_action(I.ActionKind.WITHDRAW, 1e6));
    }

    function test_anvilOnlyComparatorsCannotDeployOnBase() public {
        vm.chainId(84532);
        vm.expectRevert("ANVIL ONLY");
        new MockUSDC();
        vm.expectRevert("ANVIL ONLY");
        new NaiveAgentWallet(agent);
    }
}
