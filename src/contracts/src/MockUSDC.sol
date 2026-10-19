// SPDX-License-Identifier: MIT
pragma solidity 0.8.24;
import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/// @notice ANVIL ONLY. Permissionless minting; EIP-3009 transfers for local arena / x402 tests.
contract MockUSDC is ERC20, EIP712 {
    bytes32 public constant TRANSFER_WITH_AUTHORIZATION_TYPEHASH = keccak256(
        "TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)"
    );
    bytes32 public constant RECEIVE_WITH_AUTHORIZATION_TYPEHASH = keccak256(
        "ReceiveWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)"
    );
    bytes32 public constant CANCEL_AUTHORIZATION_TYPEHASH =
        keccak256("CancelAuthorization(address authorizer,bytes32 nonce)");
    mapping(address => mapping(bytes32 => bool)) public authorizationState;
    event AuthorizationUsed(address indexed authorizer, bytes32 indexed nonce);
    event AuthorizationCanceled(address indexed authorizer, bytes32 indexed nonce);

    constructor() ERC20("Anvil Mock USDC", "aUSDC") EIP712("Anvil Mock USDC", "2") {
        require(block.chainid == 31337, "ANVIL ONLY");
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function DOMAIN_SEPARATOR() external view returns (bytes32) {
        return _domainSeparatorV4();
    }

    function transferWithAuthorization(
        address from,
        address to,
        uint256 value,
        uint256 validAfter,
        uint256 validBefore,
        bytes32 nonce,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        _authorized(TRANSFER_WITH_AUTHORIZATION_TYPEHASH, from, to, value, validAfter, validBefore, nonce, v, r, s);
    }

    function receiveWithAuthorization(
        address from,
        address to,
        uint256 value,
        uint256 validAfter,
        uint256 validBefore,
        bytes32 nonce,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        require(msg.sender == to, "payee only");
        _authorized(RECEIVE_WITH_AUTHORIZATION_TYPEHASH, from, to, value, validAfter, validBefore, nonce, v, r, s);
    }

    function cancelAuthorization(address authorizer, bytes32 nonce, uint8 v, bytes32 r, bytes32 s) external {
        require(!authorizationState[authorizer][nonce], "authorization used");
        require(
            ECDSA.recover(
                _hashTypedDataV4(keccak256(abi.encode(CANCEL_AUTHORIZATION_TYPEHASH, authorizer, nonce))), v, r, s
            ) == authorizer,
            "bad signature"
        );
        authorizationState[authorizer][nonce] = true;
        emit AuthorizationCanceled(authorizer, nonce);
    }

    function _authorized(
        bytes32 tag,
        address from,
        address to,
        uint256 value,
        uint256 validAfter,
        uint256 validBefore,
        bytes32 nonce,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) private {
        require(block.timestamp > validAfter && block.timestamp < validBefore, "authorization time");
        require(!authorizationState[from][nonce], "authorization used");
        require(
            ECDSA.recover(
                _hashTypedDataV4(keccak256(abi.encode(tag, from, to, value, validAfter, validBefore, nonce))), v, r, s
            ) == from,
            "bad signature"
        );
        authorizationState[from][nonce] = true;
        _transfer(from, to, value);
        emit AuthorizationUsed(from, nonce);
    }
}
