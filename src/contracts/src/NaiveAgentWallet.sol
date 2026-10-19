// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

/// @notice ANVIL ONLY — DELIBERATELY UNPROTECTED comparator. Agent has arbitrary call authority.
contract NaiveAgentWallet {
    address public immutable agent;

    constructor(address agent_) {
        require(block.chainid == 31337, "ANVIL ONLY");
        agent = agent_;
    }

    function execute(address to, bytes calldata data) external returns (bytes memory) {
        require(msg.sender == agent, "agent only");
        (bool success, bytes memory result) = to.call(data);
        require(success, "call failed");
        return result;
    }
}
