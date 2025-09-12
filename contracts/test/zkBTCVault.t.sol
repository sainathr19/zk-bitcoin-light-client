// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTCVault.sol";
import "../src/zkBTC.sol";

/**
 * @title zkBTCVault Test Contract
 * @dev Test suite for the zkBTCVault contract
 */
contract zkBTCVaultTest is Test {
    zkBTCVault public vault;
    zkBTC public zkbtc;
    address public owner;
    address public user1;
    address public user2;
    address public mockVerifier;
    bytes32 public mockProgramVKey;

    function setUp() public {
        owner = address(this);
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");
        
        // Create a mock verifier address (not actually used since we return true)
        mockVerifier = makeAddr("mockVerifier");
        mockProgramVKey = keccak256("mockProgramVKey");
        
        // Deploy the vault
        vault = new zkBTCVault(mockVerifier, mockProgramVKey, owner);
        
        // Get the deployed zkBTC token
        zkbtc = zkBTC(vault.getZkBTCToken());
    }

    function testVaultDeployment() public view {
        assertEq(address(vault.getZkBTCToken()), address(zkbtc));
        assertEq(vault.getVerifier(), mockVerifier);
        assertEq(vault.owner(), owner);
    }

    function testInitialSupply() public view {
        assertEq(zkbtc.totalSupply(), 0);
        assertEq(zkbtc.maxSupply(), 21_000_000 * 10**8);
    }

    function testMintWithProof() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 zkBTC
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Mint tokens using the vault
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // Check balances
        assertEq(zkbtc.balanceOf(user1), mintAmount);
        assertEq(zkbtc.totalSupply(), mintAmount);
    }

    function testMintMultipleTimes() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 zkBTC
        bytes32 bitcoinTxHash1 = keccak256("testBitcoinTx1");
        bytes32 bitcoinTxHash2 = keccak256("testBitcoinTx2");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Mint tokens for first transaction
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash1, publicValues, proofBytes);
        
        // Mint tokens for second transaction
        vault.mintWithProof(user2, mintAmount, bitcoinTxHash2, publicValues, proofBytes);
        
        // Check balances
        assertEq(zkbtc.balanceOf(user1), mintAmount);
        assertEq(zkbtc.balanceOf(user2), mintAmount);
        assertEq(zkbtc.totalSupply(), mintAmount * 2);
    }

    function testProofReplayProtection() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 zkBTC
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // First mint should succeed
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // Second mint with same proof should fail
        vm.expectRevert(zkBTCVault.ProofAlreadyUsed.selector);
        vault.mintWithProof(user2, mintAmount, bitcoinTxHash, publicValues, proofBytes);
    }

    // function testVerifyBitcoinTransaction() public view {
    //     bytes memory publicValues = abi.encode(true);
    //     bytes memory proofBytes = abi.encode("mockProof");
        
    //     // Should always return true for now
    //     bool isValid = vault.verifyBitcoinTransaction(publicValues, proofBytes);
    //     assertTrue(isValid);
    // }

    function testGetVaultInfo() public view {
        (
            address zkbtcAddress,
            address verifierAddress,
            bytes32 programVKey,
            uint256 totalSupply,
            uint256 maxSupply
        ) = vault.getVaultInfo();
        
        assertEq(zkbtcAddress, address(zkbtc));
        assertEq(verifierAddress, mockVerifier);
        assertEq(programVKey, mockProgramVKey);
        assertEq(totalSupply, 0);
        assertEq(maxSupply, 21_000_000 * 10**8);
    }

    function testMintAmountTracking() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 zkBTC
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Mint tokens
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // Check minted amount tracking
        assertEq(vault.getMintedAmount(bitcoinTxHash), mintAmount);
    }

    function testMintToZeroAddress() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        vm.expectRevert(zkBTCVault.InvalidRecipient.selector);
        vault.mintWithProof(address(0), mintAmount, bitcoinTxHash, publicValues, proofBytes);
    }

    function testMintZeroAmount() public {
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        vm.expectRevert(zkBTCVault.ZeroMintAmount.selector);
        vault.mintWithProof(user1, 0, bitcoinTxHash, publicValues, proofBytes);
    }

    function testMintInvalidBitcoinTxHash() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        vm.expectRevert(zkBTCVault.InvalidBitcoinTxHash.selector);
        vault.mintWithProof(user1, mintAmount, bytes32(0), publicValues, proofBytes);
    }

    function testMintExceedsMaxSupply() public {
        uint256 mintAmount = 21_000_001 * 10**8; // Exceeds max supply
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        vm.expectRevert(zkBTCVault.MintAmountExceedsMaxSupply.selector);
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
    }

    function testMintCloseToMaxSupply() public {
        uint256 mintAmount = 21_000_000 * 10**8; // Exactly max supply
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Should succeed
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        assertEq(zkbtc.totalSupply(), mintAmount);
        assertEq(zkbtc.balanceOf(user1), mintAmount);
    }

    function testIsProofUsed() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Create proof hash
        bytes32 proofHash = keccak256(abi.encodePacked(bitcoinTxHash, publicValues, proofBytes));
        
        // Initially should not be used
        assertFalse(vault.isProofUsed(proofHash));
        
        // Mint tokens
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // Now should be used
        assertTrue(vault.isProofUsed(proofHash));
    }

    function testEventsEmitted() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Expect events to be emitted
        vm.expectEmit(true, true, true, true);
        emit zkBTCVault.TokensMinted(user1, mintAmount, bitcoinTxHash, keccak256(abi.encodePacked(bitcoinTxHash, publicValues, proofBytes)));
        
        vm.expectEmit(true, true, false, true);
        emit zkBTCVault.ProofVerified(bitcoinTxHash, true);
        
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
    }

    function testMultipleMintsSameBitcoinTx() public {
        uint256 mintAmount1 = 1000 * 10**8;
        uint256 mintAmount2 = 500 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues1 = abi.encode(true);
        bytes memory publicValues2 = abi.encode(false);
        bytes memory proofBytes1 = abi.encode("mockProof1");
        bytes memory proofBytes2 = abi.encode("mockProof2");
        
        // First mint
        vault.mintWithProof(user1, mintAmount1, bitcoinTxHash, publicValues1, proofBytes1);
        
        // Second mint with different proof but same Bitcoin tx
        vault.mintWithProof(user2, mintAmount2, bitcoinTxHash, publicValues2, proofBytes2);
        
        // Check balances
        assertEq(zkbtc.balanceOf(user1), mintAmount1);
        assertEq(zkbtc.balanceOf(user2), mintAmount2);
        assertEq(zkbtc.totalSupply(), mintAmount1 + mintAmount2);
        
        // Check total minted for this Bitcoin tx
        assertEq(vault.getMintedAmount(bitcoinTxHash), mintAmount1 + mintAmount2);
    }

    function testZkBTCBurn() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Mint tokens
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // User burns some tokens
        uint256 burnAmount = 200 * 10**8;
        vm.prank(user1);
        zkbtc.burn(burnAmount);
        
        assertEq(zkbtc.balanceOf(user1), mintAmount - burnAmount);
        assertEq(zkbtc.totalSupply(), mintAmount - burnAmount);
    }

    function testZkBTCTransfer() public {
        uint256 mintAmount = 1000 * 10**8;
        bytes32 bitcoinTxHash = keccak256("testBitcoinTx");
        bytes memory publicValues = abi.encode(true);
        bytes memory proofBytes = abi.encode("mockProof");
        
        // Mint tokens
        vault.mintWithProof(user1, mintAmount, bitcoinTxHash, publicValues, proofBytes);
        
        // Transfer tokens
        uint256 transferAmount = 300 * 10**8;
        vm.prank(user1);
        zkbtc.transfer(user2, transferAmount);
        
        assertEq(zkbtc.balanceOf(user1), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(user2), transferAmount);
        assertEq(zkbtc.totalSupply(), mintAmount);
    }
}