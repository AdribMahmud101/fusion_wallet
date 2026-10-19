// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ModuleBase, I, HashV41, Vm, KavachiModule, KavachiDemoVault} from "./ModuleBase.sol";

contract CoreTest is ModuleBase {
    function test_realSafe_payDepositWithdraw() public {
        assertEq(treasury.VERSION(), "1.4.1");
        assertEq(treasury.getThreshold(), 2);
        uint256 cash = token.balanceOf(address(treasury));
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
        assertEq(token.balanceOf(vendor), 10e6);
        assertTrue(module.isPaid(BILL));
        assertEq(module.getState().reservedRaw, 0);
        _execute(_action(I.ActionKind.DEPOSIT, 10e6));
        assertEq(a.balanceOf(address(treasury)), 10e6);
        _execute(_action(I.ActionKind.WITHDRAW, 10e6));
        assertEq(token.balanceOf(address(treasury)), cash - 10e6);
        assertEq(module.getState().dailyUsedRaw, 30e6);
    }

    function test_realSafe_signedPolicyAndOwnerPayment() public {
        _safeTx(address(module), abi.encodeCall(module.configurePolicy, (p)));
        _safeTx(address(module), abi.encodeCall(module.ownerPayObligation, (BILL, PAYEE, 10e6, due, new bytes32[](0))));
        assertTrue(module.getState().paused);
        assertTrue(module.isPaid(BILL));
        assertEq(token.balanceOf(vendor), 10e6);
    }

    function test_wrongChain() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        vm.chainId(84532);
        _expect(I.WrongChain.selector, x);
    }

    function test_moduleDisabled() public {
        vm.prank(address(treasury));
        treasury.disableModule(address(1), address(module));
        _expect(I.ModuleNotEnabled.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_paused() public {
        vm.prank(guardian);
        module.pause(0);
        _expect(I.ModulePaused.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_wrongCaller() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        vm.expectRevert(I.Unauthorized.selector);
        module.execute(x, new bytes32[](0));
    }

    function test_sessionExpired() public {
        vm.warp(module.getState().sessionExpiresAt);
        _expect(I.SessionExpired.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_readOnly() public {
        p.permissionTier = I.PermissionTier.READ_ONLY;
        _reconfigure();
        _expect(I.ReadOnly.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_badNonce() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.expectedNonce++;
        _expect(I.BadNonce.selector, x);
    }

    function test_stalePolicy() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.policyEpoch--;
        _expect(I.StalePolicy.selector, x);
    }

    function test_actionExpired() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.deadline = uint64(block.timestamp - 1);
        _expect(I.ActionExpired.selector, x);
    }

    function test_invalidImmediateDeadline() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.deadline++;
        _expect(I.InvalidDeadline.selector, x);
    }

    function test_deadlineAfterSession() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.deadline = module.getState().sessionExpiresAt + 1;
        _expect(I.InvalidDeadline.selector, x);
    }

    function test_missingDecisionHash() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.decisionHash = 0;
        _expect(I.MissingDecisionHash.selector, x);
    }

    function test_rebalanceUnsupported() public {
        _expect(I.UnsupportedAction.selector, _action(I.ActionKind.REBALANCE, 1e6));
    }

    function test_badDepositShape() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.sourceVault = address(b);
        _expect(I.UnsupportedAction.selector, x);
    }

    function test_badWithdrawShape() public {
        I.Action memory x = _action(I.ActionKind.WITHDRAW, 1e6);
        x.destinationVault = address(b);
        _expect(I.UnsupportedAction.selector, x);
    }

    function test_badPayShape() public {
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 10e6);
        x.minSharesOut = 1;
        _expect(I.UnsupportedAction.selector, x);
    }

    function test_zeroAmount() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.assets = 0;
        _expect(I.InvalidAmount.selector, x);
    }

    function test_hardLimit() public {
        _expect(I.HardLimitExceeded.selector, _action(I.ActionKind.DEPOSIT, 2500e6 + 1));
    }

    function test_dailyLimit() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 2500e6);
        _approve(x);
        _execute(x);
        x = _action(I.ActionKind.DEPOSIT, 1500e6);
        _approve(x);
        _execute(x);
        _expect(I.DailyLimitExceeded.selector, _action(I.ActionKind.DEPOSIT, 1));
    }

    function test_sessionLimit() public {
        vm.prank(address(treasury));
        module.revokeSession();
        _session(1e6);
        _execute(_action(I.ActionKind.DEPOSIT, 1e6));
        _expect(I.SessionLimitExceeded.selector, _action(I.ActionKind.DEPOSIT, 1));
    }

    function test_approvalRequired() public {
        _expect(I.ApprovalRequired.selector, _action(I.ActionKind.DEPOSIT, 101e6));
    }

    function test_approvalExpired() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 101e6);
        vm.prank(address(treasury));
        module.approveAction(x, uint64(block.timestamp + 1));
        vm.warp(block.timestamp + 2);
        _expect(I.ApprovalExpired.selector, x);
    }

    function test_approvedDeadlineBound() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 101e6);
        x.deadline = uint64(block.timestamp + 901);
        _approve(x);
        _expect(I.InvalidDeadline.selector, x);
    }

    function test_ownerApprovedInstant() public {
        p.permissionTier = I.PermissionTier.OWNER_APPROVED;
        _reconfigure();
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        _expect(I.ApprovalRequired.selector, x);
        _approve(x);
        _execute(x);
        assertEq(module.approvalValidUntil(module.hashAction(x)), 0);
    }

    function test_revokeApproval() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 101e6);
        _approve(x);
        bytes32 digest = module.hashAction(x);
        vm.prank(address(treasury));
        module.revokeActionApproval(digest);
        _expect(I.ApprovalRequired.selector, x);
    }

    function test_depositReserve() public {
        vm.prank(address(treasury));
        token.transfer(vendor, 9490e6);
        _expect(I.ReserveViolation.selector, _action(I.ActionKind.DEPOSIT, 1));
    }

    function test_depositUnderfunded() public {
        vm.prank(address(treasury));
        token.transfer(vendor, 9995e6);
        _expect(I.Underfunded.selector, _action(I.ActionKind.DEPOSIT, 1));
    }

    function test_exposureExceeded() public {
        _seedShares(6000e6);
        _expect(I.ExposureExceeded.selector, _action(I.ActionKind.DEPOSIT, 1));
    }

    function test_forbiddenVault() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.destinationVault = vendor;
        _expect(I.ForbiddenVault.selector, x);
    }

    function test_vaultNotInPolicy() public {
        p.vaults.pop();
        _reconfigure();
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.destinationVault = address(a) > address(b) ? address(a) : address(b);
        _expect(I.ForbiddenVault.selector, x);
    }

    function test_vaultCodeChanged() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        vm.etch(address(a), hex"00");
        _expect(I.VaultCodeChanged.selector, x);
    }

    function test_depositMinShares() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.minSharesOut++;
        _expect(I.ShareBoundViolation.selector, x);
    }

    function test_depositLooseSlippage() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        x.minSharesOut = 1;
        _expect(I.ShareBoundViolation.selector, x);
    }

    function test_withdrawLiquidity() public {
        _expect(I.InsufficientLiquidity.selector, _action(I.ActionKind.WITHDRAW, 1e6));
    }

    function test_withdrawShareBound() public {
        _seedShares(1e6);
        I.Action memory x = _action(I.ActionKind.WITHDRAW, 1e6);
        x.maxSharesBurn--;
        _expect(I.ShareBoundViolation.selector, x);
    }

    function test_withdrawLooseSlippage() public {
        _seedShares(1e6);
        I.Action memory x = _action(I.ActionKind.WITHDRAW, 1e6);
        x.maxSharesBurn *= 2;
        _expect(I.ShareBoundViolation.selector, x);
    }

    function test_withdrawAllowedUnderfunded() public {
        _seedShares(1e6);
        vm.prank(address(treasury));
        token.transfer(vendor, 10000e6);
        _execute(_action(I.ActionKind.WITHDRAW, 1e6));
        assertEq(token.balanceOf(address(treasury)), 1e6);
    }

    function test_badProof() public {
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 10e6);
        x.assets--;
        _expect(I.BadObligationProof.selector, x);
    }

    function test_duplicatePayAgentAndOwner() public {
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
        _expect(I.ObligationAlreadyPaid.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
        vm.expectRevert(I.ObligationAlreadyPaid.selector);
        vm.prank(address(treasury));
        module.ownerPayObligation(BILL, PAYEE, 10e6, due, new bytes32[](0));
        _commit(10e6);
        _session(8000e6);
        _expect(I.ObligationAlreadyPaid.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_ownerThenAgentDuplicate() public {
        vm.prank(address(treasury));
        module.ownerPayObligation(BILL, PAYEE, 10e6, due, new bytes32[](0));
        _expect(I.ObligationAlreadyPaid.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payWindow() public {
        due = uint64(block.timestamp + 4 days);
        _commit(10e6);
        _session(8000e6);
        _expect(I.PayWindowNotOpen.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payeeCooling() public {
        bytes32 id = keccak256("cooling");
        vm.prank(address(treasury));
        module.addPayee(id, vendor, 0);
        bytes32 leaf = HashV41.computeObligationLeaf(BILL, id, 10e6, due);
        vm.prank(address(treasury));
        module.commitObligations(leaf, 10e6, 1);
        _session(8000e6);
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 10e6);
        x.payeeIdHash = id;
        _expect(I.PayeeNotActive.selector, x);
        vm.warp(module.getPayee(id).activeAt);
        x.deadline = uint64(block.timestamp + 120);
        _execute(x);
    }

    function test_payeeCancelled() public {
        vm.prank(owner);
        module.cancelPayee(PAYEE);
        _expect(I.PayeeNotActive.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payeeImmutable() public {
        vm.prank(guardian);
        module.cancelPayee(PAYEE);
        vm.expectRevert(I.PayeeExists.selector);
        vm.prank(address(treasury));
        module.addPayee(PAYEE, makeAddr("replacement"), 0);
    }

    function test_recipientMismatch() public {
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 10e6);
        x.recipient = guardian;
        _expect(I.RecipientMismatch.selector, x);
    }

    function test_sanctionedAtPay() public {
        _sanction(true);
        _expect(I.PayeeSanctioned.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_sanctionedAtAdd() public {
        _sanction(true);
        vm.expectRevert(I.PayeeSanctioned.selector);
        vm.prank(address(treasury));
        module.addPayee(keccak256("blocked"), vendor, 0);
    }

    function test_sanctionsFailClosed() public {
        vm.etch(address(sanctions), hex"60006000fd");
        _expect(I.SanctionsCheckFailed.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_sanctionsMalformedClosed() public {
        vm.etch(address(sanctions), hex"600260005260206000f3");
        _expect(I.SanctionsCheckFailed.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payInsufficientCash() public {
        vm.prank(address(treasury));
        token.transfer(vendor, 9995e6);
        _expect(I.Underfunded.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payExceedsReserve() public {
        bytes32 root = HashV41.computeObligationLeaf(BILL, PAYEE, 10e6, due);
        vm.prank(address(treasury));
        module.commitObligations(root, 1e6, 1);
        _session(8000e6);
        _expect(I.ReserveViolation.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_payAllowedUnderfunded() public {
        bytes32 root = HashV41.computeObligationLeaf(BILL, PAYEE, 10e6, due);
        vm.prank(address(treasury));
        module.commitObligations(root, 10001e6, 1);
        _session(8000e6);
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
        assertEq(module.getState().reservedRaw, 9991e6);
    }

    function test_ownerPayBypassesWindowTierSessionWhilePaused() public {
        due = uint64(block.timestamp + 10 days);
        _commit(10e6);
        vm.prank(address(treasury));
        module.ownerPayObligation(BILL, PAYEE, 10e6, due, new bytes32[](0));
        assertTrue(module.isPaid(BILL));
        assertEq(module.getState().dailyUsedRaw, 0);
    }

    function test_decisionHashEventAndBinding() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        bytes32 digest = module.hashAction(x);
        vm.recordLogs();
        _execute(x);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bool found;
        bytes32 signature = keccak256(
            "ActionExecuted(bytes32,uint256,bytes32,uint8,uint256,uint256,uint256,uint256,uint256,uint256,bytes32)"
        );
        for (uint256 i; i < logs.length; i++) {
            if (logs[i].emitter == address(module) && logs[i].topics[0] == signature) {
                assertEq(logs[i].topics[1], digest);
                assertEq(logs[i].topics[2], bytes32(x.expectedNonce));
                assertEq(logs[i].topics[3], x.decisionHash);
                found = true;
            }
        }
        assertTrue(found);
    }

    function test_hashActionUsesProductionDomainAndState() public view {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1e6);
        I.ModuleState memory s = module.getState();
        assertEq(
            module.hashAction(x),
            keccak256(
                abi.encode(
                    keccak256("KAVACHI_ACTION_V4_1"),
                    block.chainid,
                    address(treasury),
                    address(module),
                    s.mandateHash,
                    s.sessionId,
                    x
                )
            )
        );
    }

    function testFuzz_depositPreservesReserve(uint256 amount) public {
        amount = bound(amount, 1, 10e6);
        _execute(_action(I.ActionKind.DEPOSIT, amount));
        assertGe(token.balanceOf(address(treasury)), module.getState().reservedRaw + p.operatingBufferRaw);
        assertEq(token.allowance(address(treasury), address(a)), 0);
    }
}
