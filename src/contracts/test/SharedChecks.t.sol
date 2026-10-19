// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ModuleBase, I} from "./ModuleBase.sol";

/// Every shared §7.7 guard is exercised against each submission action shape.
contract SharedChecksTest is ModuleBase {
    function _kind(uint256 i) internal pure returns (I.ActionKind) {
        return i == 0 ? I.ActionKind.DEPOSIT : i == 1 ? I.ActionKind.WITHDRAW : I.ActionKind.PAY_OBLIGATION;
    }

    function _all(bytes4 error_) internal {
        for (uint256 i; i < 3; i++) {
            _expect(error_, _action(_kind(i), 10e6));
        }
    }

    function test_commonWrongChainAllKinds() public {
        vm.chainId(84532);
        _all(I.WrongChain.selector);
    }

    function test_commonDisabledAllKinds() public {
        vm.prank(address(treasury));
        treasury.disableModule(address(1), address(module));
        _all(I.ModuleNotEnabled.selector);
    }

    function test_commonPausedAllKinds() public {
        vm.prank(guardian);
        module.pause(0);
        _all(I.ModulePaused.selector);
    }

    function test_commonUnauthorizedAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            vm.expectRevert(I.Unauthorized.selector);
            vm.prank(guardian);
            module.execute(x, new bytes32[](0));
        }
    }

    function test_commonSessionExpiredAllKinds() public {
        vm.warp(module.getState().sessionExpiresAt);
        _all(I.SessionExpired.selector);
    }

    function test_commonReadOnlyAllKinds() public {
        p.permissionTier = I.PermissionTier.READ_ONLY;
        _reconfigure();
        _all(I.ReadOnly.selector);
    }

    function test_commonQueueBusyAllKinds() public {
        I.Action memory q = _action(I.ActionKind.DEPOSIT, 11e6);
        q.deadline = uint64(block.timestamp + 210);
        _queue(q);
        _all(I.QueueBusy.selector);
    }

    function test_commonBadNonceAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.expectedNonce++;
            _expect(I.BadNonce.selector, x);
        }
    }

    function test_commonStalePolicyAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.policyEpoch--;
            _expect(I.StalePolicy.selector, x);
        }
    }

    function test_commonDeadlineExpiredAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.deadline = uint64(block.timestamp - 1);
            _expect(I.ActionExpired.selector, x);
        }
    }

    function test_commonDeadlineTooFarAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.deadline++;
            _expect(I.InvalidDeadline.selector, x);
        }
    }

    function test_commonMissingDecisionAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.decisionHash = 0;
            _expect(I.MissingDecisionHash.selector, x);
        }
    }

    function test_commonZeroAmountAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 10e6);
            x.assets = 0;
            _expect(I.InvalidAmount.selector, x);
        }
    }

    function test_commonHardLimitAllKinds() public {
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 2500e6 + 1);
            _expect(I.HardLimitExceeded.selector, x);
        }
    }

    function test_commonSessionLimitAllKinds() public {
        vm.prank(address(treasury));
        module.revokeSession();
        _session(1);
        _all(I.SessionLimitExceeded.selector);
    }

    function test_commonDailyLimitAllKinds() public {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 2500e6);
        _approve(x);
        _execute(x);
        x = _action(I.ActionKind.DEPOSIT, 1500e6);
        _approve(x);
        _execute(x);
        _all(I.DailyLimitExceeded.selector);
    }

    function test_commonApprovalRequiredAllKinds() public {
        p.permissionTier = I.PermissionTier.OWNER_APPROVED;
        _reconfigure();
        _all(I.ApprovalRequired.selector);
    }

    function test_commonApprovalExpiredAllKinds() public {
        p.permissionTier = I.PermissionTier.OWNER_APPROVED;
        _reconfigure();
        I.Action[3] memory actions;
        for (uint256 i; i < 3; i++) {
            actions[i] = _action(_kind(i), 10e6);
            vm.prank(address(treasury));
            module.approveAction(actions[i], uint64(block.timestamp + 1));
        }
        vm.warp(block.timestamp + 2);
        for (uint256 i; i < 3; i++) {
            _expect(I.ApprovalExpired.selector, actions[i]);
        }
    }

    function test_commonDelayedRequiredAllKinds() public {
        for (uint256 i; i < 3; i++) {
            _expect(I.QueueRequired.selector, _action(_kind(i), 11e6));
        }
    }

    function test_commonDelayedNotReadyAllKinds() public {
        // Separate queues share a nonce only until cancellation consumes it.
        _seedShares(100e6);
        _commit(11e6);
        _session(8000e6);
        for (uint256 i; i < 3; i++) {
            I.Action memory x = _action(_kind(i), 11e6);
            x.deadline = uint64(block.timestamp + 210);
            bytes32 digest = _queue(x);
            _expect(I.QueueNotReady.selector, x);
            vm.prank(guardian);
            module.cancelQueuedAction(digest);
        }
    }
}
