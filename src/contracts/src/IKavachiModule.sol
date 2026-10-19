// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

interface ISanctionsList {
    function isSanctioned(address account) external view returns (bool);
}

/// Minimal subset of Safe v1.4.1 used by the module.
interface ISafeMinimal {
    enum Operation {
        Call,
        DelegateCall
    }
    function execTransactionFromModule(address to, uint256 value, bytes calldata data, Operation operation)
        external
        returns (bool success);
    function isOwner(address owner) external view returns (bool);
    function isModuleEnabled(address module) external view returns (bool);
}

/// Kavachi V4.1 module specification interface. NOT an implementation.
interface IKavachiModule {
    enum ActionKind {
        DEPOSIT,
        WITHDRAW,
        REBALANCE,
        PAY_OBLIGATION
    }
    enum PermissionTier {
        READ_ONLY,
        OWNER_APPROVED,
        AUTONOMOUS
    }

    struct VaultLimit {
        address vault;
        uint256 maxExposureRaw;
    }

    struct Policy {
        PermissionTier permissionTier;
        uint256 operatingBufferRaw;
        uint256 microMaxRaw;
        uint256 delayedMaxRaw;
        uint256 hardMaxRaw;
        uint256 dailyLimitRaw;
        uint32 delaySeconds;
        uint16 maxShareSlippageBps;
        address guardian;
        VaultLimit[] vaults;
        uint32 payeeDelaySeconds;
        uint32 payEarlyWindowSeconds;
    }

    struct Action {
        ActionKind kind;
        address sourceVault;
        address destinationVault;
        uint256 assets;
        uint256 minSharesOut;
        uint256 maxSharesBurn;
        bytes32 obligationIdHash;
        bytes32 payeeIdHash;
        address recipient;
        uint64 dueAt;
        uint256 expectedNonce;
        uint256 policyEpoch;
        uint64 deadline;
        bytes32 decisionHash;
    }

    struct Payee {
        address account;
        bytes32 labelHash;
        uint64 activeAt;
        bool cancelled;
    }

    struct Queue {
        bool exists;
        bytes32 actionDigest;
        bytes32 sessionId;
        uint64 readyAt;
        uint64 expiresAt;
        Action action;
    }

    struct ModuleState {
        bool configured;
        bool paused;
        uint256 policyEpoch;
        bytes32 policyHash;
        bytes32 mandateHash;
        bytes32 obligationsRoot;
        uint256 reservedRaw;
        uint256 obligationCount;
        uint256 actionNonce;
        uint256 dailyDayIndex;
        uint256 dailyUsedRaw;
        bytes32 sessionId;
        uint256 sessionStartEpoch;
        address sessionKey;
        uint64 sessionExpiresAt;
        uint256 sessionLimitRaw;
        uint256 sessionUsedRaw;
    }

    // Errors
    error Unauthorized();
    error WrongChain();
    error ModuleNotEnabled();
    error WrongLifecycle();
    error ModulePaused();
    error InvalidPolicy();
    error InvalidSession();
    error SessionExpired();
    error StalePolicy();
    error BadNonce();
    error ActionExpired();
    error InvalidDeadline();
    error ReadOnly();
    error UnsupportedAction();
    error MissingDecisionHash();
    error ForbiddenVault();
    error VaultCodeChanged();
    error InvalidAmount();
    error HardLimitExceeded();
    error DailyLimitExceeded();
    error SessionLimitExceeded();
    error ApprovalRequired();
    error ApprovalExpired();
    error ReserveViolation();
    error Underfunded();
    error ExposureExceeded();
    error InsufficientLiquidity();
    error ShareBoundViolation();
    error AccountingMismatch();
    error ResidualAllowance();
    error SafeCallFailed();
    error QueueRequired();
    error QueueAlreadyExists();
    error QueueBusy();
    error QueueMismatch();
    error QueueNotReady();
    error QueueExpired();
    error NoQueue();
    error InvalidObligationCommitment();
    error BadObligationProof();
    error ObligationAlreadyPaid();
    error PayWindowNotOpen();
    error PayeeExists();
    error PayeeNotActive();
    error PayeeSanctioned();
    error RecipientMismatch();
    error SanctionsCheckFailed();

    // Events
    event PolicyConfigured(uint256 indexed policyEpoch, bytes32 policyHash, bytes32 mandateHash);
    event ObligationsCommitted(
        uint256 indexed policyEpoch, bytes32 indexed root, uint256 reservedRaw, uint256 count, bytes32 mandateHash
    );
    event PayeeAdded(bytes32 indexed payeeIdHash, address indexed account, bytes32 labelHash, uint64 activeAt);
    event PayeeCancelled(bytes32 indexed payeeIdHash, address indexed actor);
    event SessionStarted(
        bytes32 indexed sessionId,
        address indexed key,
        uint256 indexed policyEpoch,
        uint64 expiresAt,
        uint256 limitRaw,
        bytes32 mandateHash
    );
    event SessionRevoked(
        bytes32 indexed previousSessionId, address indexed previousKey, uint256 indexed policyEpoch, bytes32 mandateHash
    );
    event ActionApproved(bytes32 indexed digest, uint256 indexed policyEpoch, uint256 expectedNonce, uint64 validUntil);
    event ActionApprovalRevoked(bytes32 indexed digest);
    event ActionQueued(
        bytes32 indexed digest,
        bytes32 indexed sessionId,
        uint256 indexed nonce,
        uint64 readyAt,
        uint64 expiresAt,
        bytes32 decisionHash
    );
    event QueueCancelled(bytes32 indexed digest, address indexed actor, uint256 consumedNonce);
    event QueueExpiredEvent(bytes32 indexed digest, uint256 consumedNonce);
    event QueueInvalidated(bytes32 indexed digest, bytes32 reasonCode, uint256 consumedNonce);
    event ActionExecuted(
        bytes32 indexed actionDigest,
        uint256 indexed nonce,
        bytes32 indexed decisionHash,
        ActionKind kind,
        uint256 assets,
        uint256 sharesBurned,
        uint256 sharesMinted,
        uint256 cashAfter,
        uint256 reservedAfter,
        uint256 policyEpoch,
        bytes32 sessionId
    );
    event ObligationPaid(
        bytes32 indexed obligationIdHash,
        bytes32 indexed payeeIdHash,
        address indexed recipient,
        uint256 amountRaw,
        uint256 reservedAfter,
        bool byOwner
    );
    event Paused(address indexed actor, uint256 indexed policyEpoch, bytes32 reasonCode, bytes32 mandateHash);
    event Resumed(uint256 indexed policyEpoch, bytes32 indexed sessionId, bytes32 mandateHash);
    event EmergencyUnwind(address indexed actor, bytes32 reasonCode, uint256 cashAfter);
    event UnwindLegFailed(address indexed vault, bytes reason);

    // Views
    function safe() external view returns (address);
    function asset() external view returns (address);
    function deploymentChainId() external view returns (uint256);
    function trustedVaults() external view returns (address[2] memory);
    function sanctionsList() external view returns (address);
    function getPolicy() external view returns (Policy memory);
    function getState() external view returns (ModuleState memory);
    function getQueue() external view returns (Queue memory);
    function getPayee(bytes32 payeeIdHash) external view returns (Payee memory);
    function isPaid(bytes32 obligationIdHash) external view returns (bool);
    function approvalValidUntil(bytes32 digest) external view returns (uint64);
    function hashAction(Action calldata action) external view returns (bytes32);

    // Safe-only (threshold 2)
    function configurePolicy(Policy calldata policy) external;
    function commitObligations(bytes32 root, uint256 reservedRaw, uint256 count) external;
    function addPayee(bytes32 payeeIdHash, address account, bytes32 labelHash) external;
    function startSession(address key, uint64 expiresAt, uint256 sessionLimitRaw) external;
    function resume(uint256 expectedEpoch) external;
    function revokeSession() external;
    function approveAction(Action calldata action, uint64 validUntil) external returns (bytes32 digest);
    function revokeActionApproval(bytes32 digest) external;
    function ownerPayObligation(
        bytes32 obligationIdHash,
        bytes32 payeeIdHash,
        uint256 assets,
        uint64 dueAt,
        bytes32[] calldata proof
    ) external;

    // Stop / pull-back actions (Safe, any single Safe owner, or guardian)
    function pause(bytes32 reasonCode) external;
    function cancelQueuedAction(bytes32 digest) external;
    function cancelPayee(bytes32 payeeIdHash) external;
    function emergencyUnwind(bytes32 reasonCode) external;

    // Agent (session key)
    function queueAction(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);
    function execute(Action calldata action, bytes32[] calldata proof) external returns (bytes32 digest);

    // Anyone, after expiry
    function expireQueuedAction() external;
}
