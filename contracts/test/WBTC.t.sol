// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/WBTC.sol";

/**
 * @title WBTC Test Contract
 * @dev Test suite for the WBTC ERC20 token
 */
contract WBTCTest is Test {
    WBTC public wbtc;
    address public owner;
    address public user1;
    address public user2;

    function setUp() public {
        owner = address(this);
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");
        
        wbtc = new WBTC(owner);
    }

    function testInitialSupply() public view {
        assertEq(wbtc.totalSupply(), 21_000_000 * 10**8);
        assertEq(wbtc.balanceOf(owner), 21_000_000 * 10**8);
    }

    function testDecimals() public view {
        assertEq(wbtc.decimals(), 8);
    }

    function testMaxSupply() public view{
        assertEq(wbtc.maxSupply(), 21_000_000 * 10**8);
    }

    function testTransfer() public {
        uint256 transferAmount = 1000 * 10**8; // 1000 WBTC
        
        wbtc.transfer(user1, transferAmount);
        
        assertEq(wbtc.balanceOf(user1), transferAmount);
        assertEq(wbtc.balanceOf(owner), 21_000_000 * 10**8 - transferAmount);
    }

    function testBurn() public {
        uint256 burnAmount = 1000 * 10**8; // 1000 WBTC
        uint256 initialBalance = wbtc.balanceOf(owner);
        
        wbtc.burn(burnAmount);
        
        assertEq(wbtc.balanceOf(owner), initialBalance - burnAmount);
        assertEq(wbtc.totalSupply(), 21_000_000 * 10**8 - burnAmount);
    }

    function testBurnFrom() public {
        uint256 burnAmount = 1000 * 10**8; // 1000 WBTC
        
        // First transfer some tokens to user1
        wbtc.transfer(user1, burnAmount);
        
        // User1 approves owner to burn their tokens
        vm.prank(user1);
        wbtc.approve(owner, burnAmount);
        
        // Owner burns tokens from user1
        wbtc.burnFrom(user1, burnAmount);
        
        assertEq(wbtc.balanceOf(user1), 0);
        assertEq(wbtc.totalSupply(), 21_000_000 * 10**8 - burnAmount);
    }

    function testCannotExceedMaxSupply() public {
        uint256 excessAmount = 1;
        
        vm.expectRevert("WBTC: Cannot exceed maximum supply");
        wbtc.mint(user1, excessAmount);
    }

    function testMint() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 WBTC
        uint256 initialSupply = wbtc.totalSupply();
        
        // First burn some tokens to make room for minting
        wbtc.burn(mintAmount);
        
        // Now mint new tokens
        wbtc.mint(user1, mintAmount);
        
        assertEq(wbtc.balanceOf(user1), mintAmount);
        assertEq(wbtc.totalSupply(), initialSupply);
    }

    function testNameAndSymbol() public view {
        assertEq(wbtc.name(), "Wrapped Bitcoin");
        assertEq(wbtc.symbol(), "WBTC");
    }
}
