// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTCVault.sol";
import "../lib/sp1-contracts/contracts/src/v1.1.0/SP1Verifier.sol";

/**
 * @title zkBTCVault Deployment Test
 * @dev Test to verify the vault deployment works correctly
 */
contract ZkBTCVaultDeploymentTest is Test {
    zkBTCVault public vault;
    SP1Verifier public verifier;
    address public deployer;
    
    function setUp() public {
        deployer = address(this);
        
        // Deploy SP1Verifier
        verifier = new SP1Verifier();
        
        // Bitcoin program verification key (placeholder)
        bytes32 bitcoinProgramVKey = 0x0000000000000000000000000000000000000000000000000000000000000001;
        
        // Deploy zkBTCVault
        vault = new zkBTCVault(
            address(verifier),
            bitcoinProgramVKey,
            deployer
        );
    }
    
    function testMintWithProof() public {
        // Test minting with proof (currently disabled verification)
        address recipient = address(0x123);
        bytes32 bitcoinTxHash = keccak256("test-tx-hash");
        
        // Create mock public values (8-byte length + block_hash + 8-byte amount)
        string memory blockHash = "0000000000000000000000000000000000000000000000000000000000000000";
        uint256 amount = 100000000; // 1 BTC in satoshis
        
        bytes memory publicValues = abi.encodePacked(
            uint64(bytes(blockHash).length), // 8-byte length
            bytes(blockHash),                // block hash
            uint64(amount)                  // 8-byte amount
        );
        
        bytes memory proofBytes = "mock-proof";
        
        // Should succeed since verification is disabled
        vault.mintWithProof(recipient, bitcoinTxHash, publicValues, proofBytes);
        
        // Check that tokens were minted
        address zkbtcAddress = vault.getZkBTCToken();
        zkBTC zkbtc = zkBTC(zkbtcAddress);
        
        assertEq(zkbtc.balanceOf(recipient), amount);
        assertEq(zkbtc.totalSupply(), amount);
    }
    
    function testProofReplayProtection() public {
        address recipient = address(0x123);
        bytes32 bitcoinTxHash = keccak256("test-tx-hash");
        
        // Create mock public values (8-byte length + block_hash + 8-byte amount)
        string memory blockHash = "0000000000000000000000000000000000000000000000000000000000000000";
        uint256 amount = 100000000; // 1 BTC in satoshis
        
        bytes memory publicValues = abi.encodePacked(
            uint64(bytes(blockHash).length), // 8-byte length
            bytes(blockHash),                // block hash
            uint64(amount)                  // 8-byte amount
        );
        
        bytes memory proofBytes = "mock-proof";
        
        // First mint should succeed
        vault.mintWithProof(recipient, bitcoinTxHash, publicValues, proofBytes);
        
        // Second mint with same proof should fail
        vm.expectRevert(zkBTCVault.ProofAlreadyUsed.selector);
        vault.mintWithProof(recipient, bitcoinTxHash, publicValues, proofBytes);
    }
}
