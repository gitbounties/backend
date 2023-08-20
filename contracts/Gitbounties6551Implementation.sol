// SPDX-License-Identifier: MIT 
pragma solidity ^0.8.19;

import "./node_modules/@openzeppelin/contracts/utils/introspection/IERC165.sol";
import "./node_modules/@openzeppelin/contracts/token/ERC721/IERC721.sol";
import "./node_modules/@openzeppelin/contracts/token/ERC721/IERC721Receiver.sol";
import "./node_modules/@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";
import "./node_modules/@openzeppelin/contracts/token/ERC1155/IERC1155Receiver.sol";
//6551 references
import "../lib/reference/src/interfaces/IERC6551Account.sol";
import "../lib/reference/src/interfaces/IERC6551Executable.sol";
import "../lib/reference/src/lib/ERC6551AccountLib.sol";

contract Gitbounties6551Implementation is IERC165, IERC6551Account, IERC721Receiver, IERC6551Executable {

    //// ERRORS ////
    error GitbountiesDoesNotAccept721s();
    error GitbountiesDoesNotAccept1155s();
    ////////////////

    event BountiesAdded(address indexed sender, uint256 amount, unit256 newBalance);

    event BountiesReduced(address indexed recipient, uint256 amount, unit256 newBalance);

    event IssueResolved(address indexed recipient, uint256 amount);

    function nonce() external pure returns (uint256) {
        return (0);
    }

    receive() external payable {
        emit BountiesAdded(msg.sender, msg.value, address(this).balance);
    }

    function executeCall(
        address to,
        uint256 value,
        bytes calldata data
    ) external payable returns (bytes memory) {
        revert GitbountiesCannotExecuteCalls();
    }

    function token()
        public
        view
        returns (uint256 chainId, address tokenContract, uint256 tokenId)
    {
        return ERC6551AccountLib.token();
    }

    function owner() public view returns (address) {
        (uint256 chainId, address tokenContract, uint256 tokenId) = this
            .token();
        if (chainId != block.chainid) return address(0);

        return IERC721(tokenContract).ownerOf(tokenId);
    }

    /*
    function supportsInterface(bytes4 interfaceId) public pure returns (bool) {
        return (interfaceId == type(IERC165).interfaceId ||
            interfaceId == type(IERC6551Account).interfaceId);
    }
    */

    function executable(
        address to,
        uint256 amount,
        bytes calldata data,
        uint256 operation
    ) external payable returns (bytes memory) {
        require(_isValidSigner(msg.sender), "Invalid signer");
        require(operation == 0, "Only call operations are supported");

        address payable owner = payable(to);
        (bool success, bytes memory result) = owner.call{ value: amount }(data);
        
        require(sent, "Failed to reduce Bounties");
        uint newBalance = address(this).balance;
        emit BountiesReduced(to, amount, newBalance);
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external override returns (bytes4) {
        (uint256 nftChainId, address nftAddress, uint256 nftId) = token();
        if (
            nftChainId != block.chainid ||
            tokenId != nftId ||
            msg.sender != nftAddress
        ) {
            revert GitbountiesDoesNotAccept721s();
        } else {
            address payable formerOwner = payable(from);
            uint finalBalance = address(this).balance;
            (bool sent, bytes memory data) = formerOwner.call{ value: finalBalance }("");
            require(sent, "Failed to send Bounties");
            emit IssueResolved(formerOwner, finalBalance);
        }
        return IERC721Receiver.onERC721Received.selector;
    }

    function onERC1155Received(
        address operator,
        address from,
        uint256 id,
        uint256 value,
        bytes calldata data
    ) external pure returns (bytes4) {
        revert GitbountiesDoesNotAccept1155s();
    }
}
