// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ModuleBase, I, MockUSDC, KavachiDemoVault, KavachiModule, Vm} from "./ModuleBase.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract AdversarialToken is MockUSDC {
    uint8 public mode;
    address public callback;
    bytes public callbackData;
    bool public callbackSucceeded;

    function setMode(uint8 m) external {
        mode = m;
    }

    function setCallback(address target, bytes calldata data) external {
        callback = target;
        callbackData = data;
    }

    function approve(address spender, uint256 amount) public override returns (bool) {
        if (mode == 1) return false;
        if (mode == 7) {
            super.approve(spender, amount);
            assembly ("memory-safe") { return(0, 0) }
        }
        if (mode == 8) {
            super.approve(spender, amount);
            assembly ("memory-safe") {
                mstore(0, 2)
                return(0, 32)
            }
        }
        if (mode == 2) {
            if (amount != 0) super.approve(spender, type(uint256).max);
            return true;
        }
        return super.approve(spender, amount);
    }

    function transfer(address to, uint256 amount) public override returns (bool) {
        if (callback != address(0)) (callbackSucceeded,) = callback.call(callbackData);
        if (mode == 3) {
            super.transfer(to, amount);
            return false;
        }
        if (mode == 7) {
            super.transfer(to, amount);
            assembly ("memory-safe") { return(0, 0) }
        }
        if (mode == 8) {
            super.transfer(to, amount);
            assembly ("memory-safe") {
                mstore(0, 2)
                return(0, 32)
            }
        }
        if (mode == 9) revert("token failed");
        if (mode == 4) return true;
        if (mode == 5) {
            super.transfer(to, amount - 1);
            return true;
        }
        return super.transfer(to, amount);
    }
}

contract AdversarialVault is KavachiDemoVault {
    uint8 public mode;
    constructor(IERC20 t) KavachiDemoVault(t, "Hostile fixture", "HF") {}

    function setMode(uint8 m) external {
        mode = m;
    }

    function deposit(uint256 assets, address receiver) public override returns (uint256 shares) {
        if (mode == 1) revert("vault failed");
        shares = super.deposit(assets, receiver);
        if (mode == 2) _burn(receiver, 1);
        if (mode == 4) IERC20(asset()).transfer(receiver, 1);
    }

    function withdraw(uint256 assets, address receiver, address owner_) public override returns (uint256 shares) {
        if (mode == 1) revert("vault failed");
        shares = super.withdraw(assets, receiver, owner_);
        if (mode == 3) _burn(owner_, 1);
        if (mode == 5) IERC20(asset()).transfer(receiver, 1);
    }

    function redeem(uint256 shares, address receiver, address owner_) public override returns (uint256 assets) {
        if (mode == 1) revert("redeem failed");
        if (mode == 6) {
            assembly ("memory-safe") { for {} 1 {} {} }
        }
        assets = super.redeem(shares, receiver, owner_);
        if (mode == 7) return assets + 1;
        return assets;
    }
}

contract AdversarialTest is ModuleBase {
    AdversarialToken internal hostile;
    AdversarialVault internal evilVault;

    function setUp() public override {
        super.setUp();
        hostile = new AdversarialToken();
        evilVault = new AdversarialVault(hostile);
        KavachiDemoVault other = new KavachiDemoVault(hostile, "Other", "O");
        module = new KavachiModule(
            address(treasury), address(hostile), address(evilVault), address(other), address(sanctions)
        );
        vm.prank(address(treasury));
        treasury.enableModule(address(module));
        token = hostile;
        a = evilVault;
        b = other;
        p.vaults[0] = I.VaultLimit(address(a) < address(b) ? address(a) : address(b), 6000e6);
        p.vaults[1] = I.VaultLimit(address(a) < address(b) ? address(b) : address(a), 6000e6);
        token.mint(address(treasury), 10000e6);
        vm.prank(address(treasury));
        module.configurePolicy(p);
        vm.prank(address(treasury));
        module.addPayee(PAYEE, vendor, 0);
        vm.warp(block.timestamp + 3600);
        _commit(10e6);
        _session(8000e6);
    }

    function test_safeCallFailedDeposit() public {
        evilVault.setMode(1);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.DEPOSIT, 1e6));
        assertEq(token.allowance(address(treasury), address(a)), 0);
    }

    function test_safeCallFailedWithdraw() public {
        _seedShares(10e6);
        evilVault.setMode(1);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.WITHDRAW, 1e6));
    }

    function test_falseApproveRejected() public {
        hostile.setMode(1);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_falseTransferWithBalanceChangeRejectedAtomically() public {
        hostile.setMode(3);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
        assertEq(token.balanceOf(vendor), 0);
        assertFalse(module.isPaid(BILL));
        assertEq(module.getState().reservedRaw, 10e6);
    }

    function test_residualAllowanceRejected() public {
        hostile.setMode(2);
        _expect(I.ResidualAllowance.selector, _action(I.ActionKind.DEPOSIT, 1e6));
        assertEq(token.allowance(address(treasury), address(a)), 0);
    }

    function test_accountingMismatchPay() public {
        hostile.setMode(4);
        _expect(I.AccountingMismatch.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_feeOnTransferRejected() public {
        hostile.setMode(5);
        _expect(I.AccountingMismatch.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_depositObservedShareBound() public {
        evilVault.setMode(2);
        _expect(I.ShareBoundViolation.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_withdrawObservedShareBound() public {
        _seedShares(10e6);
        evilVault.setMode(3);
        _expect(I.ShareBoundViolation.selector, _action(I.ActionKind.WITHDRAW, 1e6));
    }

    function test_reentrancyCannotExecuteAgain() public {
        I.Action memory x = _action(I.ActionKind.PAY_OBLIGATION, 10e6);
        hostile.setCallback(address(module), abi.encodeCall(module.execute, (x, new bytes32[](0))));
        _execute(x);
        assertFalse(hostile.callbackSucceeded());
        assertEq(module.getState().actionNonce, 1);
        assertEq(token.balanceOf(vendor), 10e6);
    }

    function test_failedUnwindOneLegStillRecoversOther() public {
        _seedShares(10e6);
        token.mint(address(this), 10e6);
        token.approve(address(b), 10e6);
        b.deposit(10e6, address(treasury));
        evilVault.setMode(1); // redeem itself still works: use official Safe's result for a genuinely reverting redeem below.
        vm.mockCallRevert(
            address(a),
            abi.encodeWithSignature("redeem(uint256,address,address)", 10e6, address(treasury), address(treasury)),
            hex"01"
        );
        vm.prank(guardian);
        module.emergencyUnwind(0);
        assertEq(b.balanceOf(address(treasury)), 0);
        assertTrue(module.getState().paused);
    }

    function test_accountingMismatchDeposit() public {
        evilVault.setMode(4);
        _expect(I.AccountingMismatch.selector, _action(I.ActionKind.DEPOSIT, 1e6));
    }

    function test_accountingMismatchWithdraw() public {
        _seedShares(10e6);
        evilVault.setMode(5);
        _expect(I.AccountingMismatch.selector, _action(I.ActionKind.WITHDRAW, 1e6));
    }

    function test_safeCallFailedPay() public {
        hostile.setMode(9);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_noReturnTokenSupported() public {
        hostile.setMode(7);
        _execute(_action(I.ActionKind.DEPOSIT, 1e6));
        _execute(_action(I.ActionKind.PAY_OBLIGATION, 10e6));
        assertEq(token.balanceOf(vendor), 10e6);
    }

    function test_malformedTokenReturnsRejected() public {
        hostile.setMode(8);
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.DEPOSIT, 1e6));
        _expect(I.SafeCallFailed.selector, _action(I.ActionKind.PAY_OBLIGATION, 10e6));
    }

    function test_unwindGasConsumingLegCannotBlockOther() public {
        _seedShares(10e6);
        token.mint(address(this), 10e6);
        token.approve(address(b), 10e6);
        b.deposit(10e6, address(treasury));
        evilVault.setMode(6);
        vm.prank(guardian);
        module.emergencyUnwind{gas: 1000000}(0);
        assertEq(a.balanceOf(address(treasury)), 10e6);
        assertEq(b.balanceOf(address(treasury)), 0);
        assertTrue(module.getState().paused);
    }

    function test_unwindChecksObservedBalancesAndReturn() public {
        _seedShares(10e6);
        evilVault.setMode(7);
        vm.recordLogs();
        vm.prank(guardian);
        module.emergencyUnwind(0);
        Vm.Log[] memory logs = vm.getRecordedLogs();
        bool found;
        for (uint256 i; i < logs.length; i++) {
            if (logs[i].emitter == address(module) && logs[i].topics[0] == keccak256("UnwindLegFailed(address,bytes)"))
            {
                bytes memory reason = abi.decode(logs[i].data, (bytes));
                assertEq(bytes4(reason), I.AccountingMismatch.selector);
                found = true;
            }
        }
        assertTrue(found);
        assertEq(a.balanceOf(address(treasury)), 0);
    }
}
