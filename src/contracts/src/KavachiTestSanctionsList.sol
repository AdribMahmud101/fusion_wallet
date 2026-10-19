// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {ISanctionsList} from "./IKavachiModule.sol";

/// @notice TEST LIST for demonstrations; not production sanctions coverage. Owner MUST be the Safe.
contract KavachiTestSanctionsList is ISanctionsList, Ownable {
    mapping(address => bool) public isSanctioned;
    event TestSanctionSet(address indexed account, bool sanctioned);
    constructor(address safe_) Ownable(safe_) {}

    function setSanctioned(address account, bool sanctioned) external onlyOwner {
        isSanctioned[account] = sanctioned;
        emit TestSanctionSet(account, sanctioned);
    }
}
