// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTCVault.sol";
import "../src/zkBTC.sol";
import "../lib/sp1-contracts/contracts/src/v1.1.0/SP1Verifier.sol";

contract IntegrationTest is Test {
    zkBTCVault public vault;
    zkBTC public zkbtc;
    SP1Verifier public verifier;
    
    address public owner = address(0x1);
    address public user = address(0x2);
    address public anotherUser = address(0x3);
    
    bytes32 public constant BITCOIN_PROGRAM_VKEY = keccak256("bitcoin-program-vkey");
    
    // Real mainnet transaction data for testing
    bytes32 public constant MAINNET_TX_HASH = 0x15e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8521;
    bytes32 public constant MAINNET_TX_HASH_2 = 0x25e10745f15593a899cef391191bdd3d7c12412cc4696b7bcb669d0feadc8522;
    
    // Mock public values representing the real transaction
    bytes public constant MAINNET_PUBLIC_VALUES = abi.encodePacked(
        uint64(64), // Length of block hash string
        "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Block hash
        uint64(1240000000) // Amount: 12.4 BTC in satoshis
    );
    
    bytes public constant MAINNET_PROOF_BYTES = hex"1234567890abcdef1234567890abcdef";
    bytes public constant MAINNET_PROOF_BYTES_2 = hex"abcdef1234567890abcdef1234567890";

    event TokensMinted(address indexed to, uint256 amount, bytes32 indexed bitcoinTxHash, bytes32 proofHash);

    function setUp() public {
        // Deploy SP1 verifier
        verifier = new SP1Verifier();
        
        // Deploy vault
        vm.prank(owner);
        vault = new zkBTCVault(
            address(verifier),
            BITCOIN_PROGRAM_VKEY,
            owner
        );
        
        // Get the deployed zkBTC token
        zkbtc = vault.zkbtcToken();
        
        // Give users some ETH for gas
        vm.deal(user, 1 ether);
        vm.deal(anotherUser, 1 ether);
    }

    function testFullIntegrationFlow() public {
        uint256 expectedAmount = 1240000000; // 12.4 BTC
        
        // Step 1: User calls mintWithProof
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // Step 2: Verify token was minted correctly
        assertEq(zkbtc.balanceOf(user), expectedAmount);
        assertEq(zkbtc.totalSupply(), expectedAmount);
        
        // Step 3: User transfers some tokens
        uint256 transferAmount = expectedAmount / 2;
        vm.prank(user);
        zkbtc.transfer(anotherUser, transferAmount);
        
        assertEq(zkbtc.balanceOf(user), expectedAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
        
        // Step 4: Another user mints with different transaction
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, MAINNET_TX_HASH_2, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES_2);
        
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount + expectedAmount);
        assertEq(zkbtc.totalSupply(), expectedAmount * 2);
    }

    function testMultipleUsersMinting() public {
        uint256 expectedAmount = 1240000000;
        
        // User 1 mints
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // User 2 mints with different transaction
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, MAINNET_TX_HASH_2, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES_2);
        
        // Verify both users have tokens
        assertEq(zkbtc.balanceOf(user), expectedAmount);
        assertEq(zkbtc.balanceOf(anotherUser), expectedAmount);
        assertEq(zkbtc.totalSupply(), expectedAmount * 2);
        
        // Verify vault state
        assertEq(vault.getMintedAmount(MAINNET_TX_HASH), expectedAmount);
        assertEq(vault.getMintedAmount(MAINNET_TX_HASH_2), expectedAmount);
    }

    function testVaultAndTokenConsistency() public {
        uint256 expectedAmount = 1240000000;
        
        // Mint through vault
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // Verify vault and token are consistent
        assertEq(vault.getMintedAmount(MAINNET_TX_HASH), expectedAmount);
        assertEq(zkbtc.totalSupply(), expectedAmount);
        assertEq(zkbtc.balanceOf(user), expectedAmount);
        
        // Verify vault info
        (address zkbtcAddress, address verifierAddress, bytes32 programVKey, uint256 totalSupply, uint256 maxSupply) = vault.getVaultInfo();
        
        assertEq(zkbtcAddress, address(zkbtc));
        assertEq(verifierAddress, address(verifier));
        assertEq(programVKey, BITCOIN_PROGRAM_VKEY);
        assertEq(totalSupply, expectedAmount);
        assertEq(maxSupply, 21_000_000 * 10**8);
    }

    function testReplayProtectionAcrossUsers() public {
        // User 1 mints
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // User 2 tries to use same proof (should fail)
        vm.prank(anotherUser);
        vm.expectRevert(zkBTCVault.ProofAlreadyUsed.selector);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // Verify only user 1 has tokens
        assertEq(zkbtc.balanceOf(user), 1240000000);
        assertEq(zkbtc.balanceOf(anotherUser), 0);
    }

    function testTokenTransfersAfterMinting() public {
        uint256 mintAmount = 1240000000;
        uint256 transferAmount = 100000000; // 1 BTC
        
        // Mint tokens
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // Transfer tokens
        vm.prank(user);
        zkbtc.transfer(anotherUser, transferAmount);
        
        // Verify balances
        assertEq(zkbtc.balanceOf(user), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
        
        // Verify total supply unchanged
        assertEq(zkbtc.totalSupply(), mintAmount);
    }

    function testApprovalAndTransferFrom() public {
        uint256 mintAmount = 1240000000;
        uint256 approveAmount = 100000000;
        uint256 transferAmount = 50000000;
        
        // Mint tokens
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        // Approve tokens
        vm.prank(user);
        zkbtc.approve(anotherUser, approveAmount);
        
        // Transfer from
        vm.prank(anotherUser);
        zkbtc.transferFrom(user, anotherUser, transferAmount);
        
        // Verify balances and allowance
        assertEq(zkbtc.balanceOf(user), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
        assertEq(zkbtc.allowance(user, anotherUser), approveAmount - transferAmount);
    }

    function testMaxSupplyScenario() public {
        // Create public values with amount close to max supply
        uint256 largeAmount = 20_000_000 * 10**8; // 20M BTC
        bytes memory largeAmountValues = abi.encodePacked(
            uint64(64),
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0",
            uint64(largeAmount)
        );
        
        // First mint should succeed
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, largeAmountValues, MAINNET_PROOF_BYTES);
        
        assertEq(zkbtc.totalSupply(), largeAmount);
        
        // Second mint with remaining supply should succeed
        uint256 remainingAmount = 1_000_000 * 10**8; // 1M BTC
        bytes memory remainingAmountValues = abi.encodePacked(
            uint64(64),
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0",
            uint64(remainingAmount)
        );
        
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, MAINNET_TX_HASH_2, remainingAmountValues, MAINNET_PROOF_BYTES_2);
        
        assertEq(zkbtc.totalSupply(), largeAmount + remainingAmount);
        assertEq(zkbtc.totalSupply(), 21_000_000 * 10**8); // Exactly max supply
    }

    function testGasOptimization() public {
        uint256 gasStart = gasleft();
        
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
        
        uint256 vaultGas = gasStart - gasleft();
        
        // Transfer gas
        gasStart = gasleft();
        vm.prank(user);
        zkbtc.transfer(anotherUser, 100000000);
        uint256 transferGas = gasStart - gasleft();
        
        console.log("Vault mint gas:", vaultGas);
        console.log("Token transfer gas:", transferGas);
        
        // Ensure reasonable gas usage
        assertLt(vaultGas, 500_000);
        assertLt(transferGas, 100_000);
    }

    function testEventEmission() public {
        bytes32 expectedProofHash = keccak256(abi.encodePacked(MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES));
        
        vm.prank(user);
        vm.expectEmit(true, true, true, true);
        emit TokensMinted(user, 1240000000, MAINNET_TX_HASH, expectedProofHash);
        
        vault.mintWithProof(user, MAINNET_TX_HASH, MAINNET_PUBLIC_VALUES, MAINNET_PROOF_BYTES);
    }

    function testFuzzIntegration(uint256 amount1, uint256 amount2) public {
        vm.assume(amount1 > 0);
        vm.assume(amount2 > 0);
        vm.assume(amount1 <= 10_000_000 * 10**8); // Reasonable amounts
        vm.assume(amount2 <= 10_000_000 * 10**8);
        vm.assume(amount1 + amount2 <= 21_000_000 * 10**8);
        
        // Create public values for amounts
        bytes memory values1 = abi.encodePacked(uint64(64), "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", uint64(amount1));
        bytes memory values2 = abi.encodePacked(uint64(64), "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", uint64(amount2));
        
        // Mint for both users
        vm.prank(user);
        vault.mintWithProof(user, MAINNET_TX_HASH, values1, MAINNET_PROOF_BYTES);
        
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, MAINNET_TX_HASH_2, values2, MAINNET_PROOF_BYTES_2);
        
        // Verify balances
        assertEq(zkbtc.balanceOf(user), amount1);
        assertEq(zkbtc.balanceOf(anotherUser), amount2);
        assertEq(zkbtc.totalSupply(), amount1 + amount2);
    }
}
