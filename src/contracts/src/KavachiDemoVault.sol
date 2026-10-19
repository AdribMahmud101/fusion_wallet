// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC4626} from "@openzeppelin/contracts/token/ERC20/extensions/ERC4626.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/// @notice SIMULATED YIELD: sponsor donations, not protocol revenue or a promised rate.
contract KavachiDemoVault is ERC4626 {
    using SafeERC20 for IERC20;
    address public immutable deployer;
    event SimulatedYieldDripped(uint256 assets);

    constructor(IERC20 asset_, string memory name_, string memory symbol_) ERC20(name_, symbol_) ERC4626(asset_) {
        deployer = msg.sender;
    }

    function drip(uint256 assets) external {
        require(msg.sender == deployer, "deployer only");
        IERC20(asset()).safeTransferFrom(msg.sender, address(this), assets);
        emit SimulatedYieldDripped(assets);
    }
}
