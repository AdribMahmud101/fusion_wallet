// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {IKavachiModule, ISafeMinimal, ISanctionsList} from "./IKavachiModule.sol";
import {KavachiPayeeRegistry} from "./KavachiPayeeRegistry.sol";
import {KavachiPolicyLib} from "./KavachiPolicyLib.sol";
import {HashV41} from "./HashV41.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Metadata} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import {IERC4626} from "@openzeppelin/contracts/interfaces/IERC4626.sol";

/// Safe 1.4.1's wrapper calls execTransactionFromModule and preserves the target's return data.
interface ISafeReturnData {
    function execTransactionFromModuleReturnData(
        address to,
        uint256 value,
        bytes calldata data,
        ISafeMinimal.Operation operation
    ) external returns (bool success, bytes memory returnData);
}

/// @notice Bounded, Call-only Safe module. Amounts are integer USDC base units.
contract KavachiModule is IKavachiModule {
    address public immutable safe;
    address public immutable asset;
    uint256 public immutable deploymentChainId;
    address public immutable sanctionsList;
    address private immutable vaultA;
    address private immutable vaultB;
    bytes32 private immutable codeHashA;
    bytes32 private immutable codeHashB;
    Policy private _policy;
    ModuleState private state;
    Queue private queued;
    KavachiPayeeRegistry private immutable registry;
    mapping(bytes32 => bool) private paid;
    mapping(bytes32 => uint64) private _approvals;
    bool private entered;

    modifier locked() {
        if (entered) revert Unauthorized();
        entered = true;
        _;
        entered = false;
    }
    modifier onlySafe() {
        if (msg.sender != safe) revert Unauthorized();
        _;
    }
    modifier stopper() {
        if (msg.sender != safe && msg.sender != _policy.guardian && !ISafeMinimal(safe).isOwner(msg.sender)) {
            revert Unauthorized();
        }
        _;
    }

    constructor(address safe_, address asset_, address vaultA_, address vaultB_, address sanctionsList_) {
        if (
            safe_.code.length == 0 || asset_.code.length == 0 || sanctionsList_.code.length == 0
                || vaultA_.code.length == 0 || vaultB_.code.length == 0 || vaultA_ == vaultB_ || vaultA_ == asset_
                || vaultB_ == asset_
        ) revert InvalidPolicy();
        if (
            IERC20Metadata(asset_).decimals() != 6 || IERC4626(vaultA_).asset() != asset_
                || IERC4626(vaultB_).asset() != asset_
        ) revert InvalidPolicy();
        safe = safe_;
        asset = asset_;
        vaultA = vaultA_;
        vaultB = vaultB_;
        sanctionsList = sanctionsList_;
        deploymentChainId = block.chainid;
        codeHashA = vaultA_.codehash;
        codeHashB = vaultB_.codehash;
        registry = new KavachiPayeeRegistry(safe_, sanctionsList_);
        state.paused = true;
        state.obligationsRoot = HashV41.EMPTY_OBLIGATIONS_ROOT;
        _mandate();
    }

    function trustedVaults() external view returns (address[2] memory) {
        return [vaultA, vaultB];
    }

    function getPolicy() external view returns (Policy memory) {
        return _policy;
    }

    function getState() external view returns (ModuleState memory) {
        return state;
    }

    function getQueue() external view returns (Queue memory) {
        return queued;
    }

    function getPayee(bytes32 payeeIdHash) external view returns (Payee memory) {
        return registry.getPayee(payeeIdHash);
    }

    function isPaid(bytes32 obligationIdHash) external view returns (bool) {
        return paid[obligationIdHash];
    }

    function approvalValidUntil(bytes32 digest) external view returns (uint64) {
        return _approvals[digest];
    }

    function hashAction(Action calldata action) external view returns (bytes32) {
        return _digest(action);
    }

    function _mandate() private {
        state.mandateHash = HashV41.computeMandateHash(
            deploymentChainId,
            safe,
            address(this),
            state.policyEpoch,
            state.policyHash,
            state.obligationsRoot,
            state.reservedRaw,
            state.sessionId,
            state.sessionKey,
            state.sessionExpiresAt,
            state.sessionLimitRaw,
            state.paused
        );
    }

    function _digest(Action memory a) private view returns (bytes32) {
        return
            HashV41.computeActionDigest(deploymentChainId, safe, address(this), state.mandateHash, state.sessionId, a);
    }

    function _invalidate(bytes32 reason) private {
        if (queued.exists) {
            bytes32 digest = queued.actionDigest;
            delete queued;
            uint256 consumed = state.actionNonce++;
            emit QueueInvalidated(digest, reason, consumed);
        }
    }

    function _clearSession() private {
        state.sessionId = 0;
        state.sessionKey = address(0);
        state.sessionExpiresAt = 0;
        state.sessionLimitRaw = 0;
        state.sessionUsedRaw = 0;
        state.sessionStartEpoch = 0;
    }

    function _reset(bytes32 reason) private {
        _invalidate(reason);
        _clearSession();
        state.paused = true;
        state.policyEpoch++;
    }

    function configurePolicy(Policy calldata policy) external onlySafe locked {
        bytes32 hash = KavachiPolicyLib.validateAndHash(policy, safe, state.sessionKey, vaultA, vaultB);
        _policy = policy;
        state.configured = true;
        state.policyHash = hash;
        _reset(keccak256("POLICY"));
        _mandate();
        emit PolicyConfigured(state.policyEpoch, state.policyHash, state.mandateHash);
    }

    function commitObligations(bytes32 root, uint256 reservedRaw, uint256 count) external onlySafe locked {
        if (!state.configured) revert WrongLifecycle();
        if (count == 0
                ? (root != HashV41.EMPTY_OBLIGATIONS_ROOT || reservedRaw != 0)
                : (root == 0 || root == HashV41.EMPTY_OBLIGATIONS_ROOT || reservedRaw == 0)) revert InvalidObligationCommitment();
        state.obligationsRoot = root;
        state.reservedRaw = reservedRaw;
        state.obligationCount = count;
        _reset(keccak256("OBLIGATIONS"));
        _mandate();
        emit ObligationsCommitted(state.policyEpoch, root, reservedRaw, count, state.mandateHash);
    }

    function addPayee(bytes32 payeeIdHash, address account, bytes32 labelHash) external onlySafe locked {
        if (!state.configured) revert WrongLifecycle();
        if (
            payeeIdHash == 0 || account == address(0) || account == safe || account == vaultA || account == vaultB
                || account == asset
        ) revert RecipientMismatch();
        uint64 active = registry.add(payeeIdHash, account, labelHash, _policy.payeeDelaySeconds);
        emit PayeeAdded(payeeIdHash, account, labelHash, active);
    }

    function cancelPayee(bytes32 payeeIdHash) external stopper locked {
        registry.cancel(payeeIdHash);
        if (queued.exists && queued.action.payeeIdHash == payeeIdHash) _invalidate(keccak256("PAYEE_CANCELLED"));
        emit PayeeCancelled(payeeIdHash, msg.sender);
    }

    function startSession(address key, uint64 expiresAt, uint256 sessionLimitRaw) external onlySafe locked {
        if (!state.configured || !state.paused || state.sessionKey != address(0)) revert WrongLifecycle();
        if (
            key == address(0) || key == safe || key == _policy.guardian || ISafeMinimal(safe).isOwner(key)
                || sessionLimitRaw == 0 || expiresAt <= block.timestamp || expiresAt > block.timestamp + 1 days
        ) revert InvalidSession();
        state.sessionKey = key;
        state.sessionExpiresAt = expiresAt;
        state.sessionLimitRaw = sessionLimitRaw;
        state.sessionStartEpoch = state.policyEpoch;
        state.sessionUsedRaw = 0;
        state.sessionId = HashV41.computeSessionId(
            deploymentChainId, safe, address(this), state.sessionStartEpoch, key, expiresAt, sessionLimitRaw
        );
        _mandate();
        emit SessionStarted(state.sessionId, key, state.policyEpoch, expiresAt, sessionLimitRaw, state.mandateHash);
    }

    function resume(uint256 expectedEpoch) external onlySafe locked {
        if (!state.paused || !state.configured) revert WrongLifecycle();
        if (expectedEpoch != state.policyEpoch) revert StalePolicy();
        if (state.sessionKey == address(0)) revert InvalidSession();
        if (block.timestamp >= state.sessionExpiresAt) revert SessionExpired();
        state.paused = false;
        state.policyEpoch++;
        _mandate();
        emit Resumed(state.policyEpoch, state.sessionId, state.mandateHash);
    }

    function _pause(bytes32 reason) private {
        if (state.paused) return;
        state.paused = true;
        _invalidate(reason);
        state.policyEpoch++;
        _mandate();
        emit Paused(msg.sender, state.policyEpoch, reason, state.mandateHash);
    }

    function pause(bytes32 reasonCode) external stopper locked {
        _pause(reasonCode);
    }

    function revokeSession() external onlySafe locked {
        bytes32 previous = state.sessionId;
        address key = state.sessionKey;
        _reset(keccak256("SESSION_REVOKED"));
        _mandate();
        emit SessionRevoked(previous, key, state.policyEpoch, state.mandateHash);
    }

    function approveAction(Action calldata action, uint64 validUntil)
        external
        onlySafe
        locked
        returns (bytes32 digest)
    {
        if (state.paused || state.sessionKey == address(0)) revert WrongLifecycle();
        if (action.policyEpoch != state.policyEpoch) revert StalePolicy();
        if (action.expectedNonce != state.actionNonce) revert BadNonce();
        if (validUntil <= block.timestamp || validUntil > block.timestamp + 3600) revert InvalidDeadline();
        digest = _digest(action);
        _approvals[digest] = validUntil;
        emit ActionApproved(digest, state.policyEpoch, action.expectedNonce, validUntil);
    }

    function revokeActionApproval(bytes32 digest) external onlySafe locked {
        delete _approvals[digest];
        emit ActionApprovalRevoked(digest);
    }

    function _agent() private view {
        if (block.chainid != deploymentChainId) revert WrongChain();
        if (!ISafeMinimal(safe).isModuleEnabled(address(this))) revert ModuleNotEnabled();
        if (state.paused) revert ModulePaused();
        if (msg.sender != state.sessionKey || state.sessionKey == address(0)) revert Unauthorized();
        if (block.timestamp >= state.sessionExpiresAt) revert SessionExpired();
        if (_policy.permissionTier == PermissionTier.READ_ONLY) revert ReadOnly();
    }

    function _vault(address v) private view returns (uint256 cap) {
        if (v != vaultA && v != vaultB) revert ForbiddenVault();
        for (uint256 i; i < _policy.vaults.length; ++i) {
            if (_policy.vaults[i].vault == v) cap = _policy.vaults[i].maxExposureRaw;
        }
        if (cap == 0) revert ForbiddenVault();
        if (v.codehash != (v == vaultA ? codeHashA : codeHashB)) revert VaultCodeChanged();
    }

    function _payment(
        bytes32 id,
        bytes32 payee,
        uint256 amount,
        uint64 due,
        address recipient,
        bytes32[] calldata proof,
        bool byOwner
    ) private view {
        bytes32 leaf = HashV41.computeObligationLeaf(id, payee, amount, due);
        if (!HashV41.verifyObligationProof(proof, state.obligationsRoot, leaf)) revert BadObligationProof();
        if (paid[id]) revert ObligationAlreadyPaid();
        if (!byOwner && block.timestamp + _policy.payEarlyWindowSeconds < due) revert PayWindowNotOpen();
        registry.validate(payee, recipient);
        if (IERC20(asset).balanceOf(safe) < amount) revert Underfunded();
        if (amount > state.reservedRaw) revert ReserveViolation();
    }

    function _checks(Action memory a, bytes32[] calldata proof, bool queueing) private view returns (bytes32 digest) {
        _agent();
        digest =
            KavachiPolicyLib.validateAction(a, _policy, state, queued, _approvals, queueing, deploymentChainId, safe);
        uint256 cash = IERC20(asset).balanceOf(safe);
        if (a.kind == ActionKind.DEPOSIT) {
            if (cash < state.reservedRaw) revert Underfunded();
            if (
                a.assets > cash || cash - a.assets < state.reservedRaw
                    || cash - a.assets - state.reservedRaw < _policy.operatingBufferRaw
            ) revert ReserveViolation();
            uint256 cap = _vault(a.destinationVault);
            IERC4626 v = IERC4626(a.destinationVault);
            uint256 exposure = v.convertToAssets(v.balanceOf(safe));
            if (a.assets > cap || exposure > cap - a.assets) revert ExposureExceeded();
            uint256 preview = v.previewDeposit(a.assets);
            uint256 floor = KavachiPolicyLib.shareFloor(v.convertToShares(a.assets), _policy.maxShareSlippageBps);
            if (preview == 0 || preview < a.minSharesOut || a.minSharesOut < floor) revert ShareBoundViolation();
        } else if (a.kind == ActionKind.WITHDRAW) {
            _vault(a.sourceVault);
            IERC4626 v = IERC4626(a.sourceVault);
            if (v.maxWithdraw(safe) < a.assets) revert InsufficientLiquidity();
            uint256 preview = v.previewWithdraw(a.assets);
            uint256 ceiling = KavachiPolicyLib.shareCeiling(v.convertToShares(a.assets), _policy.maxShareSlippageBps);
            if (preview > a.maxSharesBurn || a.maxSharesBurn > ceiling) revert ShareBoundViolation();
        } else {
            _payment(a.obligationIdHash, a.payeeIdHash, a.assets, a.dueAt, a.recipient, proof, false);
        }
    }

    function queueAction(Action calldata action, bytes32[] calldata proof) external locked returns (bytes32 digest) {
        _agent();
        if (queued.exists) revert QueueAlreadyExists();
        digest = _checks(action, proof, true);
        uint64 ready = uint64(block.timestamp + _policy.delaySeconds);
        queued = Queue(true, digest, state.sessionId, ready, action.deadline, action);
        emit ActionQueued(digest, state.sessionId, action.expectedNonce, ready, action.deadline, action.decisionHash);
    }

    function cancelQueuedAction(bytes32 digest) external stopper locked {
        if (!queued.exists) revert NoQueue();
        if (digest != queued.actionDigest) revert QueueMismatch();
        delete queued;
        uint256 consumed = state.actionNonce++;
        emit QueueCancelled(digest, msg.sender, consumed);
    }

    function expireQueuedAction() external locked {
        if (!queued.exists) revert NoQueue();
        if (block.timestamp <= queued.expiresAt) revert QueueNotReady();
        bytes32 digest = queued.actionDigest;
        delete queued;
        uint256 consumed = state.actionNonce++;
        emit QueueExpiredEvent(digest, consumed);
    }

    function _call(address to, bytes memory data, bool token) private returns (bytes memory result) {
        (bool success, bytes memory returned) =
            ISafeReturnData(safe).execTransactionFromModuleReturnData(to, 0, data, ISafeMinimal.Operation.Call);
        if (!success) revert SafeCallFailed();
        if (token && returned.length != 0) {
            uint256 value;
            if (returned.length != 32) revert SafeCallFailed();
            assembly ("memory-safe") { value := mload(add(returned, 32)) }
            if (value != 1) revert SafeCallFailed();
        }
        return returned;
    }

    function _transfer(address recipient, uint256 amount) private {
        IERC20 t = IERC20(asset);
        uint256 beforeSafe = t.balanceOf(safe);
        uint256 beforeRecipient = t.balanceOf(recipient);
        _call(asset, abi.encodeCall(IERC20.transfer, (recipient, amount)), true);
        if (t.balanceOf(safe) != beforeSafe - amount || t.balanceOf(recipient) != beforeRecipient + amount) {
            revert AccountingMismatch();
        }
    }

    function execute(Action calldata action, bytes32[] calldata proof) external locked returns (bytes32 digest) {
        digest = _checks(action, proof, false);
        IERC20 t = IERC20(asset);
        uint256 cashBefore = t.balanceOf(safe);
        uint256 burned;
        uint256 minted;
        if (action.kind == ActionKind.DEPOSIT) {
            IERC4626 v = IERC4626(action.destinationVault);
            uint256 sharesBefore = v.balanceOf(safe);
            _call(asset, abi.encodeCall(IERC20.approve, (action.destinationVault, action.assets)), true);
            _call(action.destinationVault, abi.encodeCall(IERC4626.deposit, (action.assets, safe)), false);
            _call(asset, abi.encodeCall(IERC20.approve, (action.destinationVault, 0)), true);
            if (t.allowance(safe, action.destinationVault) != 0) revert ResidualAllowance();
            uint256 sharesAfter = v.balanceOf(safe);
            if (sharesAfter < sharesBefore || t.balanceOf(safe) != cashBefore - action.assets) {
                revert AccountingMismatch();
            }
            minted = sharesAfter - sharesBefore;
            if (minted < action.minSharesOut) revert ShareBoundViolation();
            if (v.convertToAssets(sharesAfter) > _vault(action.destinationVault)) revert ExposureExceeded();
        } else if (action.kind == ActionKind.WITHDRAW) {
            IERC4626 v = IERC4626(action.sourceVault);
            uint256 sharesBefore = v.balanceOf(safe);
            _call(action.sourceVault, abi.encodeCall(IERC4626.withdraw, (action.assets, safe, safe)), false);
            uint256 sharesAfter = v.balanceOf(safe);
            if (sharesAfter > sharesBefore || t.balanceOf(safe) != cashBefore + action.assets) {
                revert AccountingMismatch();
            }
            burned = sharesBefore - sharesAfter;
            if (burned == 0 || burned > action.maxSharesBurn) revert ShareBoundViolation();
        } else {
            _transfer(action.recipient, action.assets);
            paid[action.obligationIdHash] = true;
            state.reservedRaw -= action.assets;
            emit ObligationPaid(
                action.obligationIdHash, action.payeeIdHash, action.recipient, action.assets, state.reservedRaw, false
            );
        }
        if (queued.exists) delete queued;
        delete _approvals[digest];
        state.actionNonce++;
        if (state.dailyDayIndex != block.timestamp / 1 days) {
            state.dailyDayIndex = block.timestamp / 1 days;
            state.dailyUsedRaw = 0;
        }
        state.dailyUsedRaw += action.assets;
        state.sessionUsedRaw += action.assets;
        _mandate();
        emit ActionExecuted(
            digest,
            action.expectedNonce,
            action.decisionHash,
            action.kind,
            action.assets,
            burned,
            minted,
            t.balanceOf(safe),
            state.reservedRaw,
            state.policyEpoch,
            state.sessionId
        );
    }

    function ownerPayObligation(
        bytes32 obligationIdHash,
        bytes32 payeeIdHash,
        uint256 assets,
        uint64 dueAt,
        bytes32[] calldata proof
    ) external onlySafe locked {
        if (block.chainid != deploymentChainId) revert WrongChain();
        if (!ISafeMinimal(safe).isModuleEnabled(address(this))) revert ModuleNotEnabled();
        if (!state.configured) revert WrongLifecycle();
        if (assets == 0) revert InvalidAmount();
        address recipient = registry.account(payeeIdHash);
        _payment(obligationIdHash, payeeIdHash, assets, dueAt, recipient, proof, true);
        _transfer(recipient, assets);
        paid[obligationIdHash] = true;
        state.reservedRaw -= assets;
        _invalidate(keccak256("OWNER_PAYMENT"));
        _mandate();
        emit ObligationPaid(obligationIdHash, payeeIdHash, recipient, assets, state.reservedRaw, true);
    }

    function emergencyUnwind(bytes32 reasonCode) external stopper locked {
        _reset(reasonCode);
        _mandate();
        emit Paused(msg.sender, state.policyEpoch, reasonCode, state.mandateHash);
        _unwind(vaultA, codeHashA);
        _unwind(vaultB, codeHashB);
        emit EmergencyUnwind(msg.sender, reasonCode, IERC20(asset).balanceOf(safe));
    }

    function _unwind(address v, bytes32 expectedCodeHash) private {
        if (v.codehash != expectedCodeHash) {
            emit UnwindLegFailed(v, abi.encodeWithSelector(VaultCodeChanged.selector));
            return;
        }
        // Low-level reads isolate even a vault whose balanceOf now reverts.
        (bool readOk, bytes memory bal) = v.staticcall{gas: 30000}(abi.encodeCall(IERC20.balanceOf, (safe)));
        if (!readOk || bal.length != 32) {
            emit UnwindLegFailed(v, bal);
            return;
        }
        uint256 shares = abi.decode(bal, (uint256));
        if (shares == 0) return;
        uint256 cashBefore = IERC20(asset).balanceOf(safe);
        (bool ok, bytes memory result) = safe.call{gas: 350000}(
            abi.encodeCall(
                ISafeReturnData.execTransactionFromModuleReturnData,
                (v, 0, abi.encodeCall(IERC4626.redeem, (shares, safe, safe)), ISafeMinimal.Operation.Call)
            )
        );
        if (!ok) {
            emit UnwindLegFailed(v, result);
            return;
        }
        (bool success, bytes memory reason) = abi.decode(result, (bool, bytes));
        if (!success) {
            emit UnwindLegFailed(v, reason);
        } else if (!KavachiPolicyLib.redeemMatches(asset, safe, v, cashBefore, reason)) {
            emit UnwindLegFailed(v, abi.encodeWithSelector(AccountingMismatch.selector));
        }
    }
}
