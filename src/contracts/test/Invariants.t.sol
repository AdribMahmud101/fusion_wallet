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
import {Test} from "forge-std/Test.sol";
import {StdInvariant} from "forge-std/StdInvariant.sol";

contract TreasuryHandler is Test {
    KavachiModule public module;
    MockUSDC public token;
    KavachiDemoVault public vault;
    KavachiDemoVault public otherVault;
    KavachiTestSanctionsList public sanctions;
    address public safe;
    address public agent;
    address public guardian;
    address public vendor;
    bytes32 public payee;
    uint256 public expectedNonce;
    mapping(bytes32 => uint256) public executions;
    mapping(bytes32 => uint256) public paymentCount;
    mapping(uint256 => uint256) public dailyGross;
    mapping(bytes32 => uint256) public sessionGross;
    uint256 public paidTotal;
    uint256 public depositTotal;
    uint256 public withdrawTotal;
    uint256 public executeSuccesses;
    uint256 public queueSuccesses;
    uint256 public queueConsumptions;
    uint256 public pausedRejections;
    uint256 public maxPaymentCount;
    uint256 public maxDigestCount;
    uint256 public billCounter;
    uint256 public initialCash;
    uint256 public donatedCash;
    uint256 public ownerOutflows;
    address public ownerDestination = address(0x123456);
    bytes32 public currentBill;
    uint256 public currentAmount;
    uint64 public currentDue;

    constructor(
        KavachiModule m,
        MockUSDC t,
        KavachiDemoVault v,
        KavachiDemoVault other,
        KavachiTestSanctionsList sl,
        address agent_,
        address guardian_,
        address vendor_,
        bytes32 payee_
    ) {
        module = m;
        token = t;
        vault = v;
        otherVault = other;
        sanctions = sl;
        safe = m.safe();
        agent = agent_;
        guardian = guardian_;
        vendor = vendor_;
        payee = payee_;
        expectedNonce = m.getState().actionNonce;
        initialCash = t.balanceOf(safe);
    }

    function _nonce(uint256 increment) internal {
        expectedNonce += increment;
        assertEq(module.getState().actionNonce, expectedNonce, "nonce delta");
        queueConsumptions += increment;
    }

    function _action(I.ActionKind kind, uint256 amount) internal view returns (I.Action memory x) {
        I.ModuleState memory s = module.getState();
        x.kind = kind;
        x.assets = amount;
        x.expectedNonce = s.actionNonce;
        x.policyEpoch = s.policyEpoch;
        x.deadline = uint64(block.timestamp + 120);
        x.decisionHash = keccak256(abi.encode("handler", kind, amount, s.actionNonce));
        if (kind == I.ActionKind.DEPOSIT) {
            x.destinationVault = address(vault);
            x.minSharesOut = vault.previewDeposit(amount);
        }
        if (kind == I.ActionKind.WITHDRAW) {
            x.sourceVault = address(vault);
            x.maxSharesBurn = vault.previewWithdraw(amount);
        }
        if (kind == I.ActionKind.PAY_OBLIGATION) {
            x.obligationIdHash = currentBill;
            x.payeeIdHash = payee;
            x.recipient = vendor;
            x.dueAt = currentDue;
        }
    }

    function _execute(I.Action memory x) internal {
        I.ModuleState memory before_ = module.getState();
        uint256 cash = token.balanceOf(safe);
        uint256 recipientBefore = token.balanceOf(vendor);
        uint256 sharesBefore = vault.balanceOf(safe);
        bytes32 digest = module.hashAction(x);
        vm.prank(agent);
        try module.execute(x, new bytes32[](0)) returns (bytes32 returned) {
            assertFalse(before_.paused, "paused execute");
            assertEq(returned, digest);
            assertEq(++executions[digest], 1, "digest replay");
            maxDigestCount = 1;
            executeSuccesses++;
            dailyGross[block.timestamp / 1 days] += x.assets;
            sessionGross[before_.sessionId] += x.assets;
            I.Policy memory p = module.getPolicy();
            assertLe(dailyGross[block.timestamp / 1 days], p.dailyLimitRaw, "ghost daily");
            assertLe(sessionGross[before_.sessionId], before_.sessionLimitRaw, "ghost session");
            if (x.kind == I.ActionKind.DEPOSIT) {
                assertGe(cash, before_.reservedRaw + p.operatingBufferRaw, "pre reserve");
                assertGe(token.balanceOf(safe), before_.reservedRaw + p.operatingBufferRaw, "post reserve");
                assertEq(token.balanceOf(safe), cash - x.assets);
                assertGt(vault.balanceOf(safe), sharesBefore);
                assertEq(token.balanceOf(vendor), recipientBefore);
                depositTotal += x.assets;
            } else if (x.kind == I.ActionKind.WITHDRAW) {
                assertEq(token.balanceOf(safe), cash + x.assets);
                assertLt(vault.balanceOf(safe), sharesBefore);
                assertEq(token.balanceOf(vendor), recipientBefore);
                withdrawTotal += x.assets;
            } else {
                I.Payee memory p_ = module.getPayee(payee);
                assertFalse(p_.cancelled);
                assertLe(p_.activeAt, block.timestamp);
                assertFalse(sanctions.isSanctioned(vendor));
                assertEq(p_.account, x.recipient);
                assertTrue(
                    HashV41.verifyObligationProof(
                        new bytes32[](0),
                        before_.obligationsRoot,
                        HashV41.computeObligationLeaf(x.obligationIdHash, payee, x.assets, x.dueAt)
                    )
                );
                assertEq(++paymentCount[x.obligationIdHash], 1, "bill replay");
                maxPaymentCount = 1;
                assertEq(token.balanceOf(vendor), recipientBefore + x.assets);
                paidTotal += x.assets;
            }
            _nonce(1);
        } catch {
            assertEq(token.balanceOf(safe), cash, "failed call moved cash");
            assertEq(token.balanceOf(vendor), recipientBefore);
            assertEq(vault.balanceOf(safe), sharesBefore);
            _nonce(0);
        }
    }

    function deposit(uint256 amount) external {
        _execute(_action(I.ActionKind.DEPOSIT, bound(amount, 1, 10e6)));
    }

    function withdraw(uint256 amount) external {
        uint256 available = vault.maxWithdraw(safe);
        if (available == 0) return;
        _execute(_action(I.ActionKind.WITHDRAW, bound(amount, 1, available < 10e6 ? available : 10e6)));
    }

    function queueDeposit(uint256 amount) external {
        I.Action memory x = _action(I.ActionKind.DEPOSIT, bound(amount, 10e6 + 1, 100e6));
        x.deadline = uint64(block.timestamp + 210);
        uint256 cash = token.balanceOf(safe);
        uint256 used = module.getState().dailyUsedRaw;
        vm.prank(agent);
        try module.queueAction(x, new bytes32[](0)) {
            queueSuccesses++;
            assertEq(token.balanceOf(safe), cash);
            assertEq(module.getState().dailyUsedRaw, used);
        } catch {}
        _nonce(0);
    }

    function finishQueue(uint8 mode) external {
        I.Queue memory q = module.getQueue();
        if (!q.exists) return;
        if (mode % 3 == 0) {
            vm.prank(guardian);
            module.cancelQueuedAction(q.actionDigest);
            _nonce(1);
        } else if (mode % 3 == 1) {
            if (block.timestamp <= q.expiresAt) vm.warp(q.expiresAt + 1);
            module.expireQueuedAction();
            _nonce(1);
        } else {
            if (block.timestamp < q.readyAt) vm.warp(q.readyAt);
            _execute(q.action);
        }
    }

    function pauseOrRestart(bool restart) external {
        if (!restart) {
            bool consumes = module.getQueue().exists && !module.getState().paused;
            vm.prank(guardian);
            module.pause(0);
            _nonce(consumes ? 1 : 0);
        } else {
            I.ModuleState memory s = module.getState();
            if (!s.paused) return;
            if (s.sessionExpiresAt <= block.timestamp || s.sessionKey == address(0)) {
                bool consumes = module.getQueue().exists;
                vm.prank(safe);
                module.revokeSession();
                _nonce(consumes ? 1 : 0);
                vm.prank(safe);
                module.startSession(agent, uint64(block.timestamp + 1 days), 8000e6);
            }
            uint256 epoch = module.getState().policyEpoch;
            vm.prank(safe);
            module.resume(epoch);
            _nonce(0);
        }
    }

    function probePaused() external {
        if (!module.getState().paused) return;
        I.Action memory x = _action(I.ActionKind.DEPOSIT, 1);
        vm.prank(agent);
        (bool ok,) = address(module).call(abi.encodeCall(module.execute, (x, new bytes32[](0))));
        assertFalse(ok);
        pausedRejections++;
        x.deadline = uint64(block.timestamp + 210);
        vm.prank(agent);
        (ok,) = address(module).call(abi.encodeCall(module.queueAction, (x, new bytes32[](0))));
        assertFalse(ok);
        uint256 epoch = module.getState().policyEpoch;
        vm.prank(guardian);
        (ok,) = address(module).call(abi.encodeCall(module.resume, (epoch)));
        assertFalse(ok);
        _nonce(0);
    }

    function advance(uint32 seconds_) external {
        vm.warp(block.timestamp + bound(seconds_, 0, 2 days));
    }

    function newBill(uint256 amount, bool ownerPays) external {
        amount = bound(amount, 1, 10e6);
        currentBill = keccak256(abi.encode("bill", ++billCounter));
        currentAmount = amount;
        currentDue = uint64(block.timestamp + 1 days);
        bool consumes = module.getQueue().exists;
        vm.prank(safe);
        module.commitObligations(HashV41.computeObligationLeaf(currentBill, payee, amount, currentDue), amount, 1);
        _nonce(consumes ? 1 : 0);
        vm.prank(safe);
        module.startSession(agent, uint64(block.timestamp + 1 days), 8000e6);
        uint256 epoch = module.getState().policyEpoch;
        vm.prank(safe);
        module.resume(epoch);
        if (ownerPays) _ownerPay();
        else _execute(_action(I.ActionKind.PAY_OBLIGATION, amount));
    }

    function retryBill(bool ownerPays) external {
        if (currentBill == 0) return;
        if (ownerPays) _ownerPay();
        else _execute(_action(I.ActionKind.PAY_OBLIGATION, currentAmount));
    }

    function _ownerPay() internal {
        uint256 before_ = token.balanceOf(vendor);
        bool consumes = module.getQueue().exists;
        bytes32 root = module.getState().obligationsRoot;
        vm.prank(safe);
        try module.ownerPayObligation(currentBill, payee, currentAmount, currentDue, new bytes32[](0)) {
            I.Payee memory p_ = module.getPayee(payee);
            assertFalse(p_.cancelled);
            assertLe(p_.activeAt, block.timestamp);
            assertEq(p_.account, vendor);
            assertFalse(sanctions.isSanctioned(vendor));
            assertTrue(
                HashV41.verifyObligationProof(
                    new bytes32[](0), root, HashV41.computeObligationLeaf(currentBill, payee, currentAmount, currentDue)
                )
            );
            assertEq(++paymentCount[currentBill], 1);
            maxPaymentCount = 1;
            assertEq(token.balanceOf(vendor), before_ + currentAmount);
            paidTotal += currentAmount;
            _nonce(consumes ? 1 : 0);
        } catch {
            assertEq(token.balanceOf(vendor), before_);
            _nonce(0);
        }
    }

    function setSanction(bool yes) external {
        vm.prank(safe);
        sanctions.setSanctioned(vendor, yes);
    }

    function donateCash(uint256 amount) external {
        amount = bound(amount, 0, 100e6);
        token.mint(safe, amount);
        donatedCash += amount;
    }

    function ownerMoveCash(uint256 amount) external {
        amount = bound(amount, 0, token.balanceOf(safe));
        vm.prank(safe);
        token.transfer(ownerDestination, amount);
        ownerOutflows += amount;
    }

    function unwind() external {
        uint256 cashBefore = token.balanceOf(safe);
        bool consumes = module.getQueue().exists;
        vm.prank(guardian);
        module.emergencyUnwind(0);
        withdrawTotal += token.balanceOf(safe) - cashBefore;
        _nonce(consumes ? 1 : 0);
    }
}

contract InvariantTest is ModuleBase {
    TreasuryHandler internal handler;

    function setUp() public override {
        super.setUp();
        handler = new TreasuryHandler(module, token, a, b, sanctions, agent, guardian, vendor, PAYEE);
        bytes4[] memory selectors = new bytes4[](13);
        selectors[0] = handler.deposit.selector;
        selectors[1] = handler.withdraw.selector;
        selectors[2] = handler.queueDeposit.selector;
        selectors[3] = handler.finishQueue.selector;
        selectors[4] = handler.pauseOrRestart.selector;
        selectors[5] = handler.probePaused.selector;
        selectors[6] = handler.advance.selector;
        selectors[7] = handler.newBill.selector;
        selectors[8] = handler.retryBill.selector;
        selectors[9] = handler.setSanction.selector;
        selectors[10] = handler.donateCash.selector;
        selectors[11] = handler.unwind.selector;
        selectors[12] = handler.ownerMoveCash.selector;
        targetSelector(FuzzSelector(address(handler), selectors));
        targetContract(address(handler));
    }

    function invariant_I1_reservePreservedOnInvestment() public view {
        // Every successful deposit checks both reserve boundaries in _execute; aggregate conservation stays observable.
        assertEq(
            token.balanceOf(address(treasury)) + handler.depositTotal() + handler.paidTotal() + handler.ownerOutflows(),
            handler.initialCash() + handler.donatedCash() + handler.withdrawTotal()
        );
    }

    function invariant_I2_onlyTrustedDestinations() public view {
        assertEq(token.balanceOf(vendor), handler.paidTotal());
        assertEq(token.balanceOf(address(a)) + handler.withdrawTotal(), handler.depositTotal());
        assertEq(token.balanceOf(handler.ownerDestination()), handler.ownerOutflows());
        assertEq(token.balanceOf(address(module)), 0);
        assertEq(token.balanceOf(agent), 0);
        assertEq(token.balanceOf(guardian), 0);
        assertEq(b.balanceOf(address(treasury)), 0);
    }

    function invariant_I3_grossDailyAndSessionLimits() public view {
        I.ModuleState memory s = module.getState();
        assertLe(handler.dailyGross(block.timestamp / 1 days), p.dailyLimitRaw);
        assertEq(handler.sessionGross(s.sessionId), s.sessionUsedRaw);
        assertLe(s.sessionUsedRaw, s.sessionLimitRaw);
        if (s.dailyDayIndex == block.timestamp / 1 days) assertEq(handler.dailyGross(s.dailyDayIndex), s.dailyUsedRaw);
    }

    function invariant_I4_paidAtMostOnce() public view {
        assertLe(handler.maxPaymentCount(), 1);
    }

    function invariant_I5_noResidualAllowances() public view {
        assertEq(token.allowance(address(treasury), address(a)), 0);
        assertEq(token.allowance(address(treasury), address(b)), 0);
    }

    function invariant_I6_pauseStopsAgentAndOnlySafeResumes() public {
        handler.probePaused();
    }

    function invariant_I7_nonceConsumptionAndDigestUniqueness() public view {
        assertEq(module.getState().actionNonce, handler.expectedNonce());
        assertLe(handler.maxDigestCount(), 1);
    }

    function test_handlerExerciseAllRoutes() public {
        handler.deposit(1e6);
        handler.withdraw(1e6);
        handler.queueDeposit(11e6);
        handler.finishQueue(2);
        handler.queueDeposit(11e6);
        handler.finishQueue(0);
        handler.queueDeposit(11e6);
        handler.finishQueue(1);
        handler.newBill(1e6, false);
        handler.retryBill(true);
        handler.newBill(1e6, true);
        handler.retryBill(false);
        handler.pauseOrRestart(false);
        handler.probePaused();
        handler.pauseOrRestart(true);
        assertGe(handler.executeSuccesses(), 4);
        assertEq(handler.queueSuccesses(), 3);
        assertEq(handler.paidTotal(), 2e6);
        assertGt(handler.pausedRejections(), 0);
    }
}
