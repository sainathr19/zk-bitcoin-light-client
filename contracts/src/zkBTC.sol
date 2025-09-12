// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../lib/sp1-contracts/contracts/lib/openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "../lib/sp1-contracts/contracts/lib/openzeppelin-contracts/contracts/access/Ownable.sol";
import "../lib/sp1-contracts/contracts/lib/openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Permit.sol";

/**
 * @title zkBTC (Zero-Knowledge Bitcoin)
 * @dev ERC20 token representing zero-knowledge verified Bitcoin with 8 decimals and 21 million total supply
 * @notice This contract implements zkBTC with the same supply cap as Bitcoin (21 million)
 */
contract zkBTC is ERC20, Ownable, ERC20Permit {
    /// @notice Maximum supply of zkBTC tokens (21 million with 8 decimals)
    uint256 public constant MAX_SUPPLY = 21_000_000 * 10**8; // 21,000,000 zkBTC with 8 decimals
    
    /// @notice Number of decimals for zkBTC (8 decimals like Bitcoin)
    uint8 public constant DECIMALS = 8;

    /**
     * @dev Constructor that initializes the zkBTC token
     * @param initialOwner The address that will own the contract and receive initial supply
     */
    constructor(address initialOwner) 
        ERC20("zk BTC", "zkBTC") 
        Ownable(initialOwner)
        ERC20Permit("zk BTC")
    {
        // No initial minting - supply starts at zero
        // Tokens will be minted only through verified Bitcoin transaction proofs
    }

    /**
     * @dev Override decimals to return 8 (Bitcoin's decimal precision)
     * @return The number of decimals used to get its user representation
     */
    function decimals() public pure override returns (uint8) {
        return DECIMALS;
    }

    /**
     * @dev Returns the maximum supply of zkBTC tokens
     * @return The maximum number of tokens that can ever exist
     */
    function maxSupply() public pure returns (uint256) {
        return MAX_SUPPLY;
    }

    /**
     * @dev Function to mint additional tokens (only owner)
     * @param to The address to mint tokens to
     * @param amount The amount of tokens to mint
     * @notice This function allows the owner to mint tokens up to the maximum supply
     */
    function mint(address to, uint256 amount) public onlyOwner {
        require(totalSupply() + amount <= MAX_SUPPLY, "zkBTC: Cannot exceed maximum supply");
        _mint(to, amount);
    }

    /**
     * @dev Function to burn tokens from caller's balance
     * @param amount The amount of tokens to burn
     */
    function burn(uint256 amount) public {
        _burn(_msgSender(), amount);
    }

    /**
     * @dev Function to burn tokens from a specific account (requires allowance)
     * @param from The account to burn tokens from
     * @param amount The amount of tokens to burn
     */
    function burnFrom(address from, uint256 amount) public {
        _spendAllowance(from, _msgSender(), amount);
        _burn(from, amount);
    }
}
