// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ModuleBase, I, HashV41, Vm, KavachiModule} from "./ModuleBase.sol";

contract QueueLifecycleTest is ModuleBase {
    function _delayed() internal view returns (I.Action memory x) {
        x = _action(I.ActionKind.DEPOSIT, 11e6);
        x.deadline = uint64(block.timestamp + 210);
    }

    function test_queueReadyExecutionNoChargeUntilExecute() public {
        I.Action memory x = _delayed();
        uint256 cash = token.balanceOf(address(treasury));
        bytes32 digest = _queue(x);
        I.Queue memory q = module.getQueue();
        assertEq(q.actionDigest, digest);
        assertEq(q.sessionId, module.getState().sessionId);
        assertEq(q.readyAt, block.timestamp + 180);
        assertEq(q.expiresAt, x.deadline);
        assertEq(token.balanceOf(address(treasury)), cash);
        assertEq(module.getState().dailyUsedRaw, 0);
        assertEq(module.getState().actionNonce, x.expectedNonce);
        _expect(I.QueueNotReady.selector, x);
        vm.warp(q.readyAt);
        _execute(x);
        assertFalse(module.getQueue().exists);
        assertEq(module.getState().dailyUsedRaw, 11e6);
        assertEq(module.getState().actionNonce, x.expectedNonce + 1);
    }

    function test_queueBusyBeforeNonceAndPolicy() public {
        I.Action memory x = _delayed();
        _queue(x);
        I.Action memory y = _action(I.ActionKind.DEPOSIT, 1e6);
        y.expectedNonce = 100;
        y.policyEpoch = 0;
        _expect(I.QueueBusy.selector, y);
    }

    function test_queueAlreadyExists() public {
        I.Action memory x = _delayed();
        _queue(x);
        vm.expectRevert(I.QueueAlreadyExists.selector);
        _queue(x);
    }

    function test_delayedRequiresQueueEvenWithApproval() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 11e6);
        _approve(x);
        _expect(I.QueueRequired.selector, x);
    }

    function test_queueOnlyDelayed() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.deadline = uint64(block.timestamp + 210);
        vm.expectRevert(I.QueueRequired.selector);
        _queue(x);
    }

    function test_queueDeadlineLowerAndUpper() public {
        I.Action memory x = _delayed();
        x.deadline--;
        vm.expectRevert(I.InvalidDeadline.selector);
        _queue(x);
        x.deadline = uint64(block.timestamp + 1081);
        vm.expectRevert(I.InvalidDeadline.selector);
        _queue(x);
    }

    function test_queueExpiresAndConsumesNonce() public {
        I.Action memory x = _delayed();
        _queue(x);
        vm.warp(x.deadline + 1);
        _expect(I.QueueExpired.selector, x);
        module.expireQueuedAction();
        assertFalse(module.getQueue().exists);
        assertEq(module.getState().actionNonce, 1);
        _expect(I.BadNonce.selector, x);
    }

    function test_queueExpiryBoundary() public {
        I.Action memory x = _delayed();
        _queue(x);
        vm.warp(x.deadline);
        _execute(x);
    }

    function test_queueExpireTooEarly() public {
        I.Action memory x = _delayed();
        _queue(x);
        vm.warp(x.deadline);
        vm.expectRevert(I.QueueNotReady.selector);
        module.expireQueuedAction();
    }

    function test_queueCancelAndReplay() public {
        I.Action memory x = _delayed();
        bytes32 digest = _queue(x);
        vm.prank(guardian);
        module.cancelQueuedAction(digest);
        assertEq(module.getState().actionNonce, 1);
        _expect(I.BadNonce.selector, x);
    }

    function test_queueMismatchCancel() public {
        _queue(_delayed());
        vm.expectRevert(I.QueueMismatch.selector);
        vm.prank(owner);
        module.cancelQueuedAction(bytes32(uint256(1)));
    }

    function test_noQueue() public {
        vm.expectRevert(I.NoQueue.selector);
        module.expireQueuedAction();
        vm.expectRevert(I.NoQueue.selector);
        vm.prank(owner);
        module.cancelQueuedAction(0);
    }

    function test_queueRerunsReserve() public {
        I.Action memory x = _delayed();
        _queue(x);
        vm.prank(address(treasury));
        token.transfer(vendor, 9490e6);
        vm.warp(module.getQueue().readyAt);
        _expect(I.ReserveViolation.selector, x);
    }

    function test_queuePaySanctionsAtQueueAndExecution() public {
        _commit(11e6);
        _session(8000e6);
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 11e6);
        x.deadline = uint64(block.timestamp + 210);
        _sanction(true);
        vm.expectRevert(I.PayeeSanctioned.selector);
        _queue(x);
        _sanction(false);
        _queue(x);
        _sanction(true);
        vm.warp(module.getQueue().readyAt);
        _expect(I.PayeeSanctioned.selector, x);
    }

    function test_cancelPayeeInvalidatesQueue() public {
        _commit(11e6);
        _session(8000e6);
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 11e6);
        x.deadline = uint64(block.timestamp + 210);
        _queue(x);
        vm.prank(guardian);
        module.cancelPayee(PAYEE);
        assertFalse(module.getQueue().exists);
        assertEq(module.getState().actionNonce, 1);
    }

    function test_ownerApprovedDelayedNeedsApprovalAndQueue() public {
        p.permissionTier = I.PermissionTier.OWNER_APPROVED;
        _reconfigure();
        I.Action memory x = _delayed();
        vm.expectRevert(I.ApprovalRequired.selector);
        _queue(x);
        _approve(x);
        _queue(x);
        vm.warp(module.getQueue().readyAt);
        _execute(x);
    }

    function test_queueDecisionHashEvent() public {
        I.Action memory x = _delayed();
        vm.recordLogs();
        bytes32 digest = _queue(x);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        assertEq(logs[0].topics[0], keccak256("ActionQueued(bytes32,bytes32,uint256,uint64,uint64,bytes32)"));
        assertEq(logs[0].topics[1], digest);
        (uint64 ready, uint64 expiry, bytes32 decision) = abi.decode(logs[0].data, (uint64, uint64, bytes32));
        assertEq(ready, block.timestamp + 180);
        assertEq(expiry, x.deadline);
        assertEq(decision, x.decisionHash);
    }

    function test_pauseIdempotentAndSafeOnlyResume() public {
        _queue(_delayed());
        uint256 epoch = module.getState().policyEpoch;
        vm.prank(owner);
        module.pause(keccak256("stop"));
        assertTrue(module.getState().paused);
        assertEq(module.getState().policyEpoch, epoch + 1);
        assertEq(module.getState().actionNonce, 1);
        vm.prank(guardian);
        module.pause(0);
        assertEq(module.getState().policyEpoch, epoch + 1);
        vm.expectRevert(I.Unauthorized.selector);
        vm.prank(guardian);
        module.resume(epoch + 1);
        vm.expectRevert(I.StalePolicy.selector);
        vm.prank(address(treasury));
        module.resume(epoch);
        vm.prank(address(treasury));
        module.resume(epoch + 1);
        assertFalse(module.getState().paused);
    }

    function test_pausedQueueForbidden() public {
        I.Action memory x = _delayed();
        vm.prank(guardian);
        module.pause(0);
        vm.expectRevert(I.ModulePaused.selector);
        _queue(x);
    }

    function test_configureInvalidatesQueueAndSessionButKeepsDailyUsage() public {
        _execute(_action(I.ActionKind.DEPOSIT, 1e6));
        _queue(_delayed());
        vm.prank(address(treasury));
        module.configurePolicy(p);
        I.ModuleState memory s = module.getState();
        assertTrue(s.paused);
        assertEq(s.sessionKey, address(0));
        assertEq(s.sessionId, 0);
        assertEq(s.actionNonce, 2);
        assertEq(s.dailyUsedRaw, 1e6);
        assertFalse(module.getQueue().exists);
    }

    function test_commitInvalidatesQueueAndSession() public {
        _queue(_delayed());
        _commit(1e6);
        assertTrue(module.getState().paused);
        assertEq(module.getState().sessionKey, address(0));
        assertEq(module.getState().actionNonce, 1);
    }

    function test_revokeSessionInvalidatesQueue() public {
        _queue(_delayed());
        vm.prank(address(treasury));
        module.revokeSession();
        assertTrue(module.getState().paused);
        assertEq(module.getState().sessionId, 0);
        assertEq(module.getState().actionNonce, 1);
    }

    function test_ownerPaymentInvalidatesQueue() public {
        _queue(_delayed());
        vm.prank(address(treasury));
        module.ownerPayObligation(BILL, PAYEE, 10e6, due, new bytes32[](0));
        assertFalse(module.getQueue().exists);
        assertEq(module.getState().actionNonce, 1);
    }

    function test_emergencyUnwindBothVaultsUnderMillionGas() public {
        _seedShares(100e6);
        token.mint(address(this), 100e6);
        token.approve(address(b), 100e6);
        b.deposit(100e6, address(treasury));
        _queue(_delayed());
        uint256 cash = token.balanceOf(address(treasury));
        vm.prank(guardian);
        module.emergencyUnwind{gas: 1000000}(keccak256("risk"));
        assertTrue(module.getState().paused);
        assertEq(module.getState().sessionId, 0);
        assertEq(a.balanceOf(address(treasury)), 0);
        assertEq(b.balanceOf(address(treasury)), 0);
        assertEq(token.balanceOf(address(treasury)), cash + 200e6);
        assertEq(module.getState().actionNonce, 1);
    }

    function test_emergencyUnwindFailedLegContinues() public {
        _seedShares(100e6);
        token.mint(address(this), 100e6);
        token.approve(address(b), 100e6);
        b.deposit(100e6, address(treasury));
        vm.mockCallRevert(
            address(a),
            abi.encodeWithSignature("redeem(uint256,address,address)", 100e6, address(treasury), address(treasury)),
            hex"deadbeef"
        );
        vm.recordLogs();
        vm.prank(owner);
        module.emergencyUnwind(0);
        assertEq(a.balanceOf(address(treasury)), 100e6);
        assertEq(b.balanceOf(address(treasury)), 0);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bool found;
        for (uint256 i; i < logs.length; i++) {
            if (logs[i].emitter == address(module) && logs[i].topics[0] == keccak256("UnwindLegFailed(address,bytes)"))
            {
                found = true;
            }
        }
        assertTrue(found);
    }

    function test_ownerSessionKeyGuardianSeparation() public {
        vm.prank(address(treasury));
        module.revokeSession();
        vm.expectRevert(I.InvalidSession.selector);
        vm.prank(address(treasury));
        module.startSession(owner, uint64(block.timestamp + 1 days), 1);
        vm.expectRevert(I.InvalidSession.selector);
        vm.prank(address(treasury));
        module.startSession(guardian, uint64(block.timestamp + 1 days), 1);
        vm.expectRevert(I.InvalidSession.selector);
        vm.prank(address(treasury));
        module.startSession(agent, uint64(block.timestamp + 1 days + 1), 1);
        vm.expectRevert(I.InvalidSession.selector);
        vm.prank(address(treasury));
        module.startSession(agent, uint64(block.timestamp), 1);
    }

    function test_invalidPolicy() public {
        p.delaySeconds = 119;
        vm.expectRevert(I.InvalidPolicy.selector);
        vm.prank(address(treasury));
        module.configurePolicy(p);
        p.delaySeconds = 180;
        p.guardian = owner;
        vm.expectRevert(I.InvalidPolicy.selector);
        vm.prank(address(treasury));
        module.configurePolicy(p);
        p.guardian = guardian;
        p.vaults[1] = p.vaults[0];
        vm.expectRevert(I.InvalidPolicy.selector);
        vm.prank(address(treasury));
        module.configurePolicy(p);
    }

    function test_invalidCommitmentAndEmptyCommitment() public {
        vm.expectRevert(I.InvalidObligationCommitment.selector);
        vm.prank(address(treasury));
        module.commitObligations(0, 0, 0);
        vm.expectRevert(I.InvalidObligationCommitment.selector);
        vm.prank(address(treasury));
        module.commitObligations(HashV41.EMPTY_OBLIGATIONS_ROOT, 1, 1);
        vm.prank(address(treasury));
        module.commitObligations(HashV41.EMPTY_OBLIGATIONS_ROOT, 0, 0);
        assertEq(module.getState().reservedRaw, 0);
    }

    function test_dailyRolloverAndSessionRestartCannotResetDay() public {
        _execute(_action(I.ActionKind.DEPOSIT, 1e6));
        vm.prank(address(treasury));
        module.revokeSession();
        _session(8000e6);
        assertEq(module.getState().dailyUsedRaw, 1e6);
        vm.warp((block.timestamp / 1 days + 1) * 1 days);
        _execute(_action(I.ActionKind.DEPOSIT, 2e6));
        assertEq(module.getState().dailyUsedRaw, 2e6);
        assertEq(module.getState().sessionUsedRaw, 2e6);
    }
}
