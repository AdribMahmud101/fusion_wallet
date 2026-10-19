// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

// Separate compilation root: official Safe sources need the existing safe-legacy profile.
import {SafeL2} from "safe/SafeL2.sol";
import {CompatibilityFallbackHandler} from "safe/handler/CompatibilityFallbackHandler.sol";
import {MultiSendCallOnly} from "safe/libraries/MultiSendCallOnly.sol";
import {SafeProxyFactory} from "safe/proxies/SafeProxyFactory.sol";

contract OfficialArtifacts {
    function artifacts() external pure returns (bytes32, bytes32, bytes32, bytes32) {
        return (
            keccak256(type(SafeL2).creationCode),
            keccak256(type(CompatibilityFallbackHandler).creationCode),
            keccak256(type(MultiSendCallOnly).creationCode),
            keccak256(type(SafeProxyFactory).creationCode)
        );
    }
}
