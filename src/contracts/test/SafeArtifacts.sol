// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
// Unmodified official v1.4.1 artifacts, compiled without IR; deployed via deployCode in ModuleBase.
import {Safe} from "safe/Safe.sol";
import {SafeProxyFactory} from "safe/proxies/SafeProxyFactory.sol";

// A compilation root ensures dependency artifacts are emitted even though tests use ABI interfaces.
contract SafeArtifactImports {
    function deploy() external returns (address singleton, address factory) {
        return (address(new Safe()), address(new SafeProxyFactory()));
    }
}
