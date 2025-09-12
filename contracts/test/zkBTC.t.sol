// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTC.sol";

/**
 * @title zkBTC Test Contract
 * @dev Test suite for the zkBTC ERC20 token
 */
contract zkBTCTest is Test {
    zkBTC public zkbtc;
    address public owner;
    address public user1;
    address public user2;

    function setUp() public {
        owner = address(this);
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");
        
        zkbtc = new zkBTC(owner);
    }

    function testInitialSupply() public view {
        assertEq(zkbtc.totalSupply(), 0);
        assertEq(zkbtc.balanceOf(owner), 0);
    }

    function testDecimals() public view {
        assertEq(zkbtc.decimals(), 8);
    }

    function testMaxSupply() public view{
        assertEq(zkbtc.maxSupply(), 21_000_000 * 10**8);
    }

    function testTransfer() public {
        uint256 transferAmount = 1000 * 10**8; // 1000 zkBTC
        
        // First mint some tokens to owner
        zkbtc.mint(owner, transferAmount * 2);
        
        zkbtc.transfer(user1, transferAmount);
        
        assertEq(zkbtc.balanceOf(user1), transferAmount);
        assertEq(zkbtc.balanceOf(owner), transferAmount);
    }

    function testBurn() public {
        uint256 burnAmount = 1000 * 10**8; // 1000 zkBTC
        
        // First mint some tokens to owner
        zkbtc.mint(owner, burnAmount * 2);
        uint256 initialBalance = zkbtc.balanceOf(owner);
        
        zkbtc.burn(burnAmount);
        
        assertEq(zkbtc.balanceOf(owner), initialBalance - burnAmount);
        assertEq(zkbtc.totalSupply(), burnAmount);
    }

    function testBurnFrom() public {
        uint256 burnAmount = 1000 * 10**8; // 1000 zkBTC
        
        // First mint some tokens to owner
        zkbtc.mint(owner, burnAmount * 2);
        
        // Transfer some tokens to user1
        zkbtc.transfer(user1, burnAmount);
        
        // User1 approves owner to burn their tokens
        vm.prank(user1);
        zkbtc.approve(owner, burnAmount);
        
        // Owner burns tokens from user1
        zkbtc.burnFrom(user1, burnAmount);
        
        assertEq(zkbtc.balanceOf(user1), 0);
        assertEq(zkbtc.totalSupply(), burnAmount);
    }

    function testCannotExceedMaxSupply() public {
        uint256 excessAmount = 1;
        
        // Try to mint more than max supply
        vm.expectRevert("zkBTC: Cannot exceed maximum supply");
        zkbtc.mint(user1, 21_000_000 * 10**8 + excessAmount);
    }

    function testMint() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 zkBTC
        uint256 initialSupply = zkbtc.totalSupply();
        
        // Mint new tokens
        zkbtc.mint(user1, mintAmount);
        
        assertEq(zkbtc.balanceOf(user1), mintAmount);
        assertEq(zkbtc.totalSupply(), initialSupply + mintAmount);
    }

    function testNameAndSymbol() public view {
        assertEq(zkbtc.name(), "zk BTC");
        assertEq(zkbtc.symbol(), "zkBTC");
    }
}
