// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTCVault.sol";
import "../src/zkBTC.sol";
import "../lib/sp1-contracts/contracts/src/v1.1.0/SP1Verifier.sol";

contract zkBTCVaultTest is Test {
    zkBTCVault public vault;
    zkBTC public zkbtc;
    SP1Verifier public verifier;
    
    address public owner = address(0x1);
    address public user = address(0x2);
    address public anotherUser = address(0x3);
    
    bytes32 public constant BITCOIN_PROGRAM_VKEY = keccak256("bitcoin-program-vkey");
    bytes32 public constant BITCOIN_TX_HASH = keccak256("bitcoin-tx-hash");
    
    // Mock public values for testing (format: [8-byte length][block_hash string][8-byte total_amount])
    bytes public constant MOCK_PUBLIC_VALUES = abi.encodePacked(
        uint64(64), // Length of block hash string (64 characters)
        "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Block hash
        uint64(1240000000) // Amount in satoshis (12.4 BTC)
    );
    
    bytes public constant MOCK_PROOF_BYTES = hex"1234567890abcdef";
    
    event TokensMinted(address indexed to, uint256 amount, bytes32 indexed bitcoinTxHash, bytes32 proofHash);
    event ProofVerified(bytes32 indexed bitcoinTxHash, bool isValid);

    function setUp() public {
        // Deploy SP1 verifier (mock)
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
        
        // Give user some ETH for gas
        vm.deal(user, 1 ether);
        vm.deal(anotherUser, 1 ether);
    }

    function testConstructor() public view {
        assertEq(address(vault.zkbtcToken()), address(zkbtc));
        assertEq(address(vault.verifier()), address(verifier));
        assertEq(vault.bitcoinProgramVKey(), BITCOIN_PROGRAM_VKEY);
        assertEq(vault.owner(), owner);
        
        // Check zkBTC initial state
        assertEq(zkbtc.totalSupply(), 0);
        assertEq(zkbtc.maxSupply(), 21_000_000 * 10**8); // 21M BTC with 8 decimals
    }

    function testMintWithProof() public {
        uint256 initialSupply = zkbtc.totalSupply();
        uint256 expectedAmount = 1240000000;
        
        vm.prank(user);
        vm.expectEmit(true, true, true, true);
        emit TokensMinted(user, expectedAmount, BITCOIN_TX_HASH, keccak256(abi.encodePacked(BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES)));
        
        vm.expectEmit(true, false, false, false);
        emit ProofVerified(BITCOIN_TX_HASH, true);
        
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        // Check token balances
        assertEq(zkbtc.totalSupply(), initialSupply + expectedAmount);
        assertEq(zkbtc.balanceOf(user), expectedAmount);
        
        // Check vault state
        assertTrue(vault.isProofUsed(keccak256(abi.encodePacked(BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES))));
        assertEq(vault.getMintedAmount(BITCOIN_TX_HASH), expectedAmount);
    }

    function testMintWithProofInvalidRecipient() public {
        vm.prank(user);
        vm.expectRevert(zkBTCVault.InvalidRecipient.selector);
        vault.mintWithProof(address(0), BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
    }

    function testMintWithProofInvalidBitcoinTxHash() public {
        vm.prank(user);
        vm.expectRevert(zkBTCVault.InvalidBitcoinTxHash.selector);
        vault.mintWithProof(user, bytes32(0), MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
    }

    function testMintWithProofZeroAmount() public {
        // Create public values with zero amount
        bytes memory zeroAmountValues = abi.encodePacked(
            uint64(64), // Length of block hash string
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Block hash
            uint64(0) // Zero amount
        );
        
        vm.prank(user);
        vm.expectRevert(zkBTCVault.ZeroMintAmount.selector);
        vault.mintWithProof(user, BITCOIN_TX_HASH, zeroAmountValues, MOCK_PROOF_BYTES);
    }

    function testMintWithProofReplayAttack() public {
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        // Try to use the same proof again
        vm.prank(anotherUser);
        vm.expectRevert(zkBTCVault.ProofAlreadyUsed.selector);
        vault.mintWithProof(anotherUser, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
    }

    function testMintWithProofMaxSupplyExceeded() public {
        // Create public values with very large amount
        bytes memory largeAmountValues = abi.encodePacked(
            uint64(64), // Length of block hash string
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Block hash
            uint64(21_000_000 * 10**8 + 1) // Amount exceeding max supply
        );
        
        vm.prank(user);
        vm.expectRevert(zkBTCVault.MintAmountExceedsMaxSupply.selector);
        vault.mintWithProof(user, BITCOIN_TX_HASH, largeAmountValues, MOCK_PROOF_BYTES);
    }

    function testMultipleMintsSameTransaction() public {
        uint256 expectedAmount = 1240000000;
        
        // First mint
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        // Create different proof for same transaction (different proof bytes)
        bytes memory differentProof = hex"abcdef1234567890";
        
        // Second mint with different proof but same transaction
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, differentProof);
        
        // Check balances
        assertEq(zkbtc.balanceOf(user), expectedAmount);
        assertEq(zkbtc.balanceOf(anotherUser), expectedAmount);
        assertEq(zkbtc.totalSupply(), expectedAmount * 2);
        
        // Check minted amount for this transaction
        assertEq(vault.getMintedAmount(BITCOIN_TX_HASH), expectedAmount * 2);
    }

    function testGetVaultInfo() public view {
        (address zkbtcAddress, address verifierAddress, bytes32 programVKey, uint256 totalSupply, uint256 maxSupply) = vault.getVaultInfo();
        
        assertEq(zkbtcAddress, address(zkbtc));
        assertEq(verifierAddress, address(verifier));
        assertEq(programVKey, BITCOIN_PROGRAM_VKEY);
        assertEq(totalSupply, 0);
        assertEq(maxSupply, 21_000_000 * 10**8);
    }

    function testIsProofUsed() public {
        bytes32 proofHash = keccak256(abi.encodePacked(BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES));
        
        // Initially not used
        assertFalse(vault.isProofUsed(proofHash));
        
        // After minting, should be used
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        assertTrue(vault.isProofUsed(proofHash));
    }

    function testGetMintedAmount() public {
        uint256 expectedAmount = 1240000000;
        
        // Initially zero
        assertEq(vault.getMintedAmount(BITCOIN_TX_HASH), 0);
        
        // After minting
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        assertEq(vault.getMintedAmount(BITCOIN_TX_HASH), expectedAmount);
    }

    function testDifferentPublicValuesFormats() public {
        // Test with different block hash lengths
        bytes memory shortBlockHashValues = abi.encodePacked(
            uint64(8), // Length of block hash string (8 characters)
            "12345678", // Short block hash
            uint64(1000000) // Amount
        );
        
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, shortBlockHashValues, MOCK_PROOF_BYTES);
        
        assertEq(zkbtc.balanceOf(user), 1000000);
        
        // Test with longer block hash
        bytes memory longBlockHashValues = abi.encodePacked(
            uint64(128), // Length of block hash string (128 characters)
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Long block hash
            uint64(5000000) // Amount
        );
        
        bytes32 differentTxHash = keccak256("different-tx-hash");
        vm.prank(anotherUser);
        vault.mintWithProof(anotherUser, differentTxHash, longBlockHashValues, MOCK_PROOF_BYTES);
        
        assertEq(zkbtc.balanceOf(anotherUser), 5000000);
    }

    function testEdgeCasePublicValues() public {
        // Test with minimum valid length
        bytes memory minValidValues = abi.encodePacked(
            uint64(1), // Length of block hash string (1 character)
            "a", // Single character block hash
            uint64(1) // Minimum amount
        );
        
        bytes32 edgeTxHash = keccak256("edge-tx-hash");
        vm.prank(user);
        vault.mintWithProof(user, edgeTxHash, minValidValues, MOCK_PROOF_BYTES);
        
        assertEq(zkbtc.balanceOf(user), 1);
    }

    function testFuzzMintWithProof(address to, bytes32 bitcoinTxHash, uint256 amount) public {
        vm.assume(to != address(0));
        vm.assume(bitcoinTxHash != bytes32(0));
        vm.assume(amount > 0);
        vm.assume(amount <= 21_000_000 * 10**8); // Within max supply
        
        // Create public values with fuzzed amount
        bytes memory fuzzValues = abi.encodePacked(
            uint64(64), // Length of block hash string
            "ef0c2fa8517414b742094a020da7eba891b47d660ef66f126ad01e5be99a2fd0", // Block hash
            uint64(amount) // Fuzzed amount
        );
        
        uint256 initialSupply = zkbtc.totalSupply();
        
        vm.prank(user);
        vault.mintWithProof(to, bitcoinTxHash, fuzzValues, MOCK_PROOF_BYTES);
        
        assertEq(zkbtc.totalSupply(), initialSupply + amount);
        assertEq(zkbtc.balanceOf(to), amount);
    }

    function testGasUsage() public {
        uint256 gasStart = gasleft();
        
        vm.prank(user);
        vault.mintWithProof(user, BITCOIN_TX_HASH, MOCK_PUBLIC_VALUES, MOCK_PROOF_BYTES);
        
        uint256 gasUsed = gasStart - gasleft();
        
        // Log gas usage for optimization reference
        console.log("Gas used for mintWithProof:", gasUsed);
        
        // Ensure gas usage is reasonable (adjust threshold as needed)
        assertLt(gasUsed, 500_000);
    }
}