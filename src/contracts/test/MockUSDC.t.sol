// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {Test} from "forge-std/Test.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract MockUSDCTest is Test {
    MockUSDC internal token;
    address internal payer;
    uint256 internal payerKey;
    address internal recipient;
    bytes32 internal constant NONCE = keccak256("authorization");

    function setUp() public {
        vm.chainId(31337);
        vm.warp(1000);
        token = new MockUSDC();
        (payer, payerKey) = makeAddrAndKey("payer");
        recipient = makeAddr("recipient");
        token.mint(payer, 10e6);
    }

    function _sign(bytes32 tag, uint256 validAfter, uint256 validBefore)
        internal
        view
        returns (uint8, bytes32, bytes32)
    {
        bytes32 h = keccak256(abi.encode(tag, payer, recipient, 1e6, validAfter, validBefore, NONCE));
        return vm.sign(payerKey, keccak256(abi.encodePacked(hex"1901", token.DOMAIN_SEPARATOR(), h)));
    }

    function test_transferAuthorizationAndReplay() public {
        (uint8 v, bytes32 r, bytes32 s) = _sign(token.TRANSFER_WITH_AUTHORIZATION_TYPEHASH(), 999, 2000);
        token.transferWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
        assertEq(token.balanceOf(recipient), 1e6);
        assertTrue(token.authorizationState(payer, NONCE));
        assertEq(token.decimals(), 6);
        vm.expectRevert("authorization used");
        token.transferWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
    }

    function test_authorizationBoundToExactValue() public {
        (uint8 v, bytes32 r, bytes32 s) = _sign(token.TRANSFER_WITH_AUTHORIZATION_TYPEHASH(), 999, 2000);
        vm.expectRevert("bad signature");
        token.transferWithAuthorization(payer, recipient, 2e6, 999, 2000, NONCE, v, r, s);
    }

    function test_authorizationTimeBoundaries() public {
        (uint8 v, bytes32 r, bytes32 s) = _sign(token.TRANSFER_WITH_AUTHORIZATION_TYPEHASH(), 1000, 2000);
        vm.expectRevert("authorization time");
        token.transferWithAuthorization(payer, recipient, 1e6, 1000, 2000, NONCE, v, r, s);
        vm.warp(2000);
        vm.expectRevert("authorization time");
        token.transferWithAuthorization(payer, recipient, 1e6, 1000, 2000, NONCE, v, r, s);
    }

    function test_receiveAuthorizationOnlyPayee() public {
        (uint8 v, bytes32 r, bytes32 s) = _sign(token.RECEIVE_WITH_AUTHORIZATION_TYPEHASH(), 999, 2000);
        vm.expectRevert("payee only");
        token.receiveWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
        vm.prank(recipient);
        token.receiveWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
        assertEq(token.balanceOf(recipient), 1e6);
    }

    function test_cancelAuthorization() public {
        bytes32 h = keccak256(abi.encode(token.CANCEL_AUTHORIZATION_TYPEHASH(), payer, NONCE));
        (uint8 v, bytes32 r, bytes32 s) =
            vm.sign(payerKey, keccak256(abi.encodePacked(hex"1901", token.DOMAIN_SEPARATOR(), h)));
        token.cancelAuthorization(payer, NONCE, v, r, s);
        assertTrue(token.authorizationState(payer, NONCE));
        (v, r, s) = _sign(token.TRANSFER_WITH_AUTHORIZATION_TYPEHASH(), 999, 2000);
        vm.expectRevert("authorization used");
        token.transferWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
    }

    function test_authorizationDomainBoundToChain() public {
        (uint8 v, bytes32 r, bytes32 s) = _sign(token.TRANSFER_WITH_AUTHORIZATION_TYPEHASH(), 999, 2000);
        vm.chainId(31338);
        vm.expectRevert("bad signature");
        token.transferWithAuthorization(payer, recipient, 1e6, 999, 2000, NONCE, v, r, s);
    }
}
