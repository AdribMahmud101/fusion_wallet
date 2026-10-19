// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {IKavachiModule as I, ISanctionsList, ISafeMinimal} from "./IKavachiModule.sol";

/// @notice §7.14 second split. Immutable entries; Safe owns additions through its module.
/// All writes originate in the module so cancellation atomically invalidates the module queue.
contract KavachiPayeeRegistry {
    address public immutable owner;
    address public immutable module;
    address public immutable sanctionsList;
    mapping(bytes32 => I.Payee) private payees;

    constructor(address safe_, address sanctionsList_) {
        owner = safe_;
        module = msg.sender;
        sanctionsList = sanctionsList_;
    }
    modifier onlyModule() {
        if (msg.sender != module) revert I.Unauthorized();
        _;
    }

    function getPayee(bytes32 id) external view returns (I.Payee memory) {
        return payees[id];
    }

    function account(bytes32 id) external view returns (address) {
        return payees[id].account;
    }

    function guardian() external view returns (address) {
        return I(module).getPolicy().guardian;
    }

    function add(bytes32 id, address account, bytes32 label, uint32 delay)
        external
        onlyModule
        returns (uint64 activeAt)
    {
        if (payees[id].account != address(0)) revert I.PayeeExists();
        _screen(account);
        activeAt = uint64(block.timestamp + delay);
        payees[id] = I.Payee(account, label, activeAt, false);
    }

    function cancel(bytes32 id) external onlyModule {
        if (payees[id].account == address(0)) revert I.PayeeNotActive();
        payees[id].cancelled = true;
    }

    function validate(bytes32 id, address recipient) external view {
        I.Payee memory p = payees[id];
        if (p.account == address(0) || p.cancelled || block.timestamp < p.activeAt) revert I.PayeeNotActive();
        if (p.account != recipient) revert I.RecipientMismatch();
        _screen(recipient);
    }

    function _screen(address account) private view {
        (bool ok, bytes memory result) =
            sanctionsList.staticcall(abi.encodeCall(ISanctionsList.isSanctioned, (account)));
        if (!ok || result.length != 32) revert I.SanctionsCheckFailed();
        uint256 value;
        assembly ("memory-safe") { value := mload(add(result, 32)) }
        if (value > 1) revert I.SanctionsCheckFailed();
        if (value == 1) revert I.PayeeSanctioned();
    }
}
