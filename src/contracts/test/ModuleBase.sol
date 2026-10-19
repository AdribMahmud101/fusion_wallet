// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {Test} from "forge-std/Test.sol";
import {Vm} from "forge-std/Vm.sol";
import {Enum} from "safe/common/Enum.sol";
import {KavachiModule} from "../src/KavachiModule.sol";
import {IKavachiModule as I} from "../src/IKavachiModule.sol";
import {HashV41} from "../src/HashV41.sol";
import {MockUSDC} from "../src/MockUSDC.sol";
import {KavachiDemoVault} from "../src/KavachiDemoVault.sol";
import {KavachiTestSanctionsList} from "../src/KavachiTestSanctionsList.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface Safe {
    function setup(
        address[] calldata owners,
        uint256 threshold,
        address to,
        bytes calldata data,
        address fallbackHandler,
        address paymentToken,
        uint256 payment,
        address payable paymentReceiver
    ) external;
    function enableModule(address module) external;
    function disableModule(address previous, address module) external;
    function isModuleEnabled(address module) external view returns (bool);
    function VERSION() external view returns (string memory);
    function getThreshold() external view returns (uint256);
    function nonce() external view returns (uint256);
    function getTransactionHash(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address refundReceiver,
        uint256 nonce_
    ) external view returns (bytes32);
    function execTransaction(
        address to,
        uint256 value,
        bytes calldata data,
        Enum.Operation operation,
        uint256 safeTxGas,
        uint256 baseGas,
        uint256 gasPrice,
        address gasToken,
        address payable refundReceiver,
        bytes calldata signatures
    ) external returns (bool);
}

interface SafeProxyFactory {
    function createProxyWithNonce(address singleton, bytes calldata initializer, uint256 saltNonce)
        external
        returns (address);
}

abstract contract ModuleBase is Test {
    Safe internal treasury;
    KavachiModule internal module;
    MockUSDC internal token;
    KavachiDemoVault internal a;
    KavachiDemoVault internal b;
    KavachiTestSanctionsList internal sanctions;
    address internal agent;
    address internal guardian;
    address internal owner;
    address internal approver;
    uint256 internal ownerKey;
    uint256 internal approverKey;
    address internal vendor;
    bytes32 internal constant PAYEE = keccak256("vendor");
    bytes32 internal constant BILL = keccak256("bill");
    uint64 internal due;
    I.Policy internal p;

    function setUp() public virtual {
        vm.chainId(31337);
        vm.warp(1792300000);
        agent = makeAddr("agent");
        guardian = makeAddr("guardian");
        vendor = makeAddr("vendor");
        (owner, ownerKey) = makeAddrAndKey("owner");
        (approver, approverKey) = makeAddrAndKey("approver");
        Safe singleton = Safe(deployCode("Safe.sol:Safe"));
        SafeProxyFactory factory = SafeProxyFactory(deployCode("SafeProxyFactory.sol:SafeProxyFactory"));
        address[] memory owners = new address[](2);
        owners[0] = owner;
        owners[1] = approver;
        treasury = Safe(
            payable(address(
                    factory.createProxyWithNonce(
                        address(singleton),
                        abi.encodeCall(
                            Safe.setup,
                            (owners, 2, address(0), bytes(""), address(0), address(0), 0, payable(address(0)))
                        ),
                        1
                    )
                ))
        );
        token = new MockUSDC();
        a = new KavachiDemoVault(token, "Simulated A", "sA");
        b = new KavachiDemoVault(token, "Simulated B", "sB");
        sanctions = new KavachiTestSanctionsList(address(treasury));
        module = new KavachiModule(address(treasury), address(token), address(a), address(b), address(sanctions));
        _safeTx(address(treasury), abi.encodeCall(treasury.enableModule, (address(module))));
        token.mint(address(treasury), 10000e6);
        p.permissionTier = I.PermissionTier.AUTONOMOUS;
        p.operatingBufferRaw = 500e6;
        p.microMaxRaw = 10e6;
        p.delayedMaxRaw = 100e6;
        p.hardMaxRaw = 2500e6;
        p.dailyLimitRaw = 4000e6;
        p.delaySeconds = 180;
        p.maxShareSlippageBps = 50;
        p.guardian = guardian;
        p.vaults.push(I.VaultLimit(address(a) < address(b) ? address(a) : address(b), 6000e6));
        p.vaults.push(I.VaultLimit(address(a) < address(b) ? address(b) : address(a), 6000e6));
        p.payeeDelaySeconds = 3600;
        p.payEarlyWindowSeconds = 3 days;
        vm.prank(address(treasury));
        module.configurePolicy(p);
        vm.prank(address(treasury));
        module.addPayee(PAYEE, vendor, keccak256("Vendor"));
        vm.warp(block.timestamp + 3600);
        due = uint64(block.timestamp + 1 days);
        _commit(10e6);
        _session(8000e6);
    }

    function _safeTx(address to, bytes memory data) internal {
        bytes32 digest = treasury.getTransactionHash(
            to, 0, data, Enum.Operation.Call, 0, 0, 0, address(0), address(0), treasury.nonce()
        );
        (uint8 v1, bytes32 r1, bytes32 s1) = vm.sign(ownerKey, digest);
        (uint8 v2, bytes32 r2, bytes32 s2) = vm.sign(approverKey, digest);
        bytes memory sig1 = abi.encodePacked(r1, s1, v1);
        bytes memory sig2 = abi.encodePacked(r2, s2, v2);
        bytes memory signatures = owner < approver ? bytes.concat(sig1, sig2) : bytes.concat(sig2, sig1);
        assertTrue(
            treasury.execTransaction(
                to, 0, data, Enum.Operation.Call, 0, 0, 0, address(0), payable(address(0)), signatures
            ),
            "Safe execution"
        );
    }

    function _commit(uint256 amount) internal {
        vm.prank(address(treasury));
        module.commitObligations(HashV41.computeObligationLeaf(BILL, PAYEE, amount, due), amount, 1);
    }

    function _session(uint256 limit) internal {
        vm.prank(address(treasury));
        module.startSession(agent, uint64(block.timestamp + 1 days), limit);
        uint256 epoch = module.getState().policyEpoch;
        vm.prank(address(treasury));
        module.resume(epoch);
    }

    function _reconfigure() internal {
        vm.prank(address(treasury));
        module.configurePolicy(p);
        _session(8000e6);
    }

    function _action(I.ActionKind kind, uint256 amount) internal view returns (I.Action memory x) {
        I.ModuleState memory s = module.getState();
        x.kind = kind;
        x.assets = amount;
        x.expectedNonce = s.actionNonce;
        x.policyEpoch = s.policyEpoch;
        x.deadline = uint64(block.timestamp + 120);
        x.decisionHash = keccak256("passport");
        if (kind == I.ActionKind.DEPOSIT) {
            x.destinationVault = address(a);
            x.minSharesOut = a.previewDeposit(amount);
        } else if (kind == I.ActionKind.WITHDRAW) {
            x.sourceVault = address(a);
            x.maxSharesBurn = a.previewWithdraw(amount);
        } else if (kind == I.ActionKind.PAY_OBLIGATION) {
            x.obligationIdHash = BILL;
            x.payeeIdHash = PAYEE;
            x.recipient = vendor;
            x.dueAt = due;
        }
    }

    function _execute(I.Action memory x) internal returns (bytes32) {
        vm.prank(agent);
        return module.execute(x, new bytes32[](0));
    }

    function _queue(I.Action memory x) internal returns (bytes32) {
        vm.prank(agent);
        return module.queueAction(x, new bytes32[](0));
    }

    function _approve(I.Action memory x) internal {
        vm.prank(address(treasury));
        module.approveAction(x, uint64(block.timestamp + 3600));
    }

    function _seedShares(uint256 amount) internal {
        token.mint(address(this), amount);
        token.approve(address(a), amount);
        a.deposit(amount, address(treasury));
    }

    function _expect(bytes4 error_, I.Action memory x) internal {
        vm.expectRevert(error_);
        _execute(x);
    }

    function _sanction(bool yes) internal {
        vm.prank(address(treasury));
        sanctions.setSanctioned(vendor, yes);
    }
}
