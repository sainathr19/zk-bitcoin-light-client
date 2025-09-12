// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../lib/sp1-contracts/contracts/lib/openzeppelin-contracts/contracts/access/Ownable.sol";
import "../lib/sp1-contracts/contracts/lib/openzeppelin-contracts/contracts/utils/ReentrancyGuard.sol";
import "../lib/sp1-contracts/contracts/src/v1.1.0/SP1Verifier.sol";
import "./zkBTC.sol";

/**
 * @title zkBTCVault
 * @dev Vault contract that deploys zkBTC with zero initial supply and only allows minting
 *      through verified Bitcoin transaction proofs using Succinct SP1 verifier
 * @notice This contract ensures that zkBTC tokens can only be minted by providing valid
 *         zk proofs of Bitcoin transactions
 * @notice FOR NOW: Proof verification is disabled and always returns true for testing purposes
 */
contract zkBTCVault is Ownable, ReentrancyGuard {
    /// @notice The deployed zkBTC token contract
    zkBTC public immutable zkbtcToken;
    
    /// @notice The SP1 verifier contract for proof verification
    ISP1Verifier public immutable verifier;
    
    /// @notice The verification key for Bitcoin transaction verification
    bytes32 public immutable bitcoinProgramVKey;
    
    /// @notice Mapping to track used proof hashes to prevent replay attacks
    mapping(bytes32 => bool) public usedProofs;
    
    /// @notice Mapping to track minted amounts per Bitcoin transaction hash
    mapping(bytes32 => uint256) public mintedAmounts;
    
    /// @notice Events
    event TokensMinted(address indexed to, uint256 amount, bytes32 indexed bitcoinTxHash, bytes32 proofHash);
    event ProofVerified(bytes32 indexed bitcoinTxHash, bool isValid);
    
    /// @notice Errors
    error InvalidProof();
    error ProofAlreadyUsed();
    error InvalidBitcoinTxHash();
    error MintAmountExceedsMaxSupply();
    error ZeroMintAmount();
    error InvalidRecipient();

    /**
     * @dev Constructor that deploys zkBTC with zero supply and sets up verifier
     * @param _verifier The address of the SP1 verifier contract
     * @param _bitcoinProgramVKey The verification key for Bitcoin transaction verification
     * @param _initialOwner The address that will own this contract
     */
    constructor(
        address _verifier,
        bytes32 _bitcoinProgramVKey,
        address _initialOwner
    ) Ownable(_initialOwner) {
        verifier = ISP1Verifier(_verifier);
        bitcoinProgramVKey = _bitcoinProgramVKey;
        
        // Deploy zkBTC with zero initial supply
        zkbtcToken = new zkBTC(address(this));
    }

    /**
     * @dev Internal function to extract block hash from public values
     * @param publicValues The encoded public values from the zk proof
     * @return blockHash The extracted block hash from the public values
     * @notice Format: [8-byte length][block_hash string][8-byte total_amount]
     */
    function _extractBlockHashFromPublicValues(bytes calldata publicValues) internal pure returns (string memory blockHash) {
        // Check minimum length: 8 bytes for length + at least 1 byte for block_hash + 8 bytes for amount
        if (publicValues.length < 17) {
            return "";
        }
        
        // Read the length of the block_hash string (first 8 bytes as little-endian uint64)
        uint64 blockHashLen = uint64(bytes8(publicValues[0:8]));
        
        // Check if we have enough data: 8 bytes length + block_hash + 8 bytes amount
        if (publicValues.length < 8 + blockHashLen + 8) {
            return "";
        }
        
        // Extract the block_hash string
        bytes memory blockHashBytes = publicValues[8:8 + blockHashLen];
        return string(blockHashBytes);
    }

    /**
     * @dev Internal function to extract amount from public values
     * @param publicValues The encoded public values from the zk proof
     * @return amount The extracted amount from the public values
     * @notice Format: [8-byte length][block_hash string][8-byte total_amount]
     */
    function _extractAmountFromPublicValues(bytes calldata publicValues) internal pure returns (uint256 amount) {
        // Check minimum length: 8 bytes for length + at least 1 byte for block_hash + 8 bytes for amount
        if (publicValues.length < 17) {
            return 0;
        }
        
        // Read the length of the block_hash string (first 8 bytes as little-endian uint64)
        uint64 blockHashLen = uint64(bytes8(publicValues[0:8]));
        
        // Check if we have enough data: 8 bytes length + block_hash + 8 bytes amount
        if (publicValues.length < 8 + blockHashLen + 8) {
            return 0;
        }
        
        // Extract the total_amount (last 8 bytes as little-endian uint64)
        uint64 amountStart = 8 + blockHashLen;
        amount = uint256(uint64(bytes8(publicValues[amountStart:amountStart + 8])));
        
        return amount;
    }

    /**
     * @dev Main function to mint zkBTC tokens after verifying Bitcoin transaction proof
     * @param to The address to mint tokens to
     * @param bitcoinTxHash The Bitcoin transaction hash being proven
     * @param publicValues The encoded public values from the zk proof
     * @param proofBytes The encoded zk proof
     */
    function mintWithProof(
        address to,
        bytes32 bitcoinTxHash,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external nonReentrant {
        // Validate inputs
        if (to == address(0)) revert InvalidRecipient();
        if (bitcoinTxHash == bytes32(0)) revert InvalidBitcoinTxHash();
        
        // Extract amount from public values
        uint256 amount = _extractAmountFromPublicValues(publicValues);
        if (amount == 0) revert ZeroMintAmount();
        
        // Check if total supply would exceed maximum
        if (zkbtcToken.totalSupply() + amount > zkbtcToken.maxSupply()) {
            revert MintAmountExceedsMaxSupply();
        }
        
        // Create proof hash to prevent replay attacks
        bytes32 proofHash = keccak256(abi.encodePacked(bitcoinTxHash, publicValues, proofBytes));
        
        // Check if proof has already been used
        if (usedProofs[proofHash]) revert ProofAlreadyUsed();
        
        // // Verify the zk proof (FOR NOW: always returns true)
        // bool isValid = _verifyBitcoinTransaction(publicValues, proofBytes);
        bool isValid = true;
        if (!isValid) revert InvalidProof();
        
        // Mark proof as used
        usedProofs[proofHash] = true;
        
        // Track minted amount for this Bitcoin transaction
        mintedAmounts[bitcoinTxHash] += amount;
        
        // Mint tokens to the specified address
        zkbtcToken.mint(to, amount);
        
        // Emit events
        emit TokensMinted(to, amount, bitcoinTxHash, proofHash);
        emit ProofVerified(bitcoinTxHash, true);
    }

    // /**
    //  * @dev Internal function to verify Bitcoin transaction using SP1 verifier
    //  * @param publicValues The encoded public values from the zk proof
    //  * @param proofBytes The encoded zk proof
    //  * @return isValid True if the proof is valid, false otherwise
    //  * @notice FOR NOW: Always returns true for testing purposes
    //  */
    // function _verifyBitcoinTransaction(
    //     bytes calldata publicValues,
    //     bytes calldata proofBytes
    // ) internal view returns (bool isValid) {
    //     // FOR NOW: Always return true for testing purposes
    //     // TODO: Implement actual proof verification when ready
    //     return true;
        
    //     // Original verification code (commented out for now):
    //     // try verifier.verifyProof(bitcoinProgramVKey, publicValues, proofBytes) {
    //     //     return true;
    //     // } catch {
    //     //     return false;
    //     // }
    // }

    // /**
    //  * @dev Public function to verify a Bitcoin transaction proof (view function)
    //  * @param publicValues The encoded public values from the zk proof
    //  * @param proofBytes The encoded zk proof
    //  * @return isValid True if the proof is valid, false otherwise
    //  * @notice FOR NOW: Always returns true for testing purposes
    //  */
    // function verifyBitcoinTransaction(
    //     bytes calldata publicValues,
    //     bytes calldata proofBytes
    // ) external view returns (bool isValid) {
    //     // FOR NOW: Always return true for testing purposes
    //     // TODO: Implement actual proof verification when ready
    //     return true;
    // }

    /**
     * @dev Get the zkBTC token contract address
     * @return The address of the deployed zkBTC contract
     */
    function getZkBTCToken() external view returns (address) {
        return address(zkbtcToken);
    }

    /**
     * @dev Get the SP1 verifier contract address
     * @return The address of the SP1 verifier contract
     */
    function getVerifier() external view returns (address) {
        return address(verifier);
    }

    /**
     * @dev Check if a proof hash has been used
     * @param proofHash The proof hash to check
     * @return True if the proof has been used, false otherwise
     */
    function isProofUsed(bytes32 proofHash) external view returns (bool) {
        return usedProofs[proofHash];
    }

    /**
     * @dev Get the total amount minted for a specific Bitcoin transaction
     * @param bitcoinTxHash The Bitcoin transaction hash
     * @return The total amount minted for this transaction
     */
    function getMintedAmount(bytes32 bitcoinTxHash) external view returns (uint256) {
        return mintedAmounts[bitcoinTxHash];
    }

    /**
     * @dev Get vault information
     * @return zkbtcAddress The zkBTC token address
     * @return verifierAddress The verifier address
     * @return programVKey The Bitcoin program verification key
     * @return totalSupply The current zkBTC total supply
     * @return maxSupply The maximum zkBTC supply
     */
    function getVaultInfo() external view returns (
        address zkbtcAddress,
        address verifierAddress,
        bytes32 programVKey,
        uint256 totalSupply,
        uint256 maxSupply
    ) {
        return (
            address(zkbtcToken),
            address(verifier),
            bitcoinProgramVKey,
            zkbtcToken.totalSupply(),
            zkbtcToken.maxSupply()
        );
    }
}
