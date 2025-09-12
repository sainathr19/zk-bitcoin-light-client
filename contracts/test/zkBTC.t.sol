// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/zkBTC.sol";

contract zkBTCTest is Test {
    zkBTC public zkbtc;
    
    address public owner = address(0x1);
    address public user = address(0x2);
    address public anotherUser = address(0x3);
    
    uint256 public constant MAX_SUPPLY = 21_000_000 * 10**8; // 21M BTC with 8 decimals
    uint256 public constant INITIAL_SUPPLY = 0;

    event Transfer(address indexed from, address indexed to, uint256 value);

    function setUp() public {
        vm.prank(owner);
        zkbtc = new zkBTC(owner);
        
        // Give users some ETH for gas
        vm.deal(user, 1 ether);
        vm.deal(anotherUser, 1 ether);
    }

    function testConstructor() public view {
        assertEq(zkbtc.name(), "zk BTC");
        assertEq(zkbtc.symbol(), "zkBTC");
        assertEq(zkbtc.decimals(), 8);
        assertEq(zkbtc.totalSupply(), INITIAL_SUPPLY);
        assertEq(zkbtc.maxSupply(), MAX_SUPPLY);
        assertEq(zkbtc.owner(), owner);
    }

    function testMint() public {
        uint256 mintAmount = 1000 * 10**8; // 1000 BTC
        
        vm.prank(owner);
        vm.expectEmit(true, true, false, false);
        emit Transfer(address(0), user, mintAmount);
        
        zkbtc.mint(user, mintAmount);
        
        assertEq(zkbtc.totalSupply(), mintAmount);
        assertEq(zkbtc.balanceOf(user), mintAmount);
    }

    function testMintOnlyOwner() public {
        uint256 mintAmount = 1000 * 10**8;
        
        vm.prank(user);
        vm.expectRevert();
        zkbtc.mint(user, mintAmount);
    }

    function testMintZeroAmount() public {
        uint256 initialSupply = zkbtc.totalSupply();
        
        vm.prank(owner);
        zkbtc.mint(user, 0); // OpenZeppelin ERC20 allows zero mint
        
        // Verify no tokens were actually minted
        assertEq(zkbtc.totalSupply(), initialSupply);
        assertEq(zkbtc.balanceOf(user), 0);
    }

    function testMintExceedsMaxSupply() public {
        uint256 excessAmount = MAX_SUPPLY + 1;
        
        vm.prank(owner);
        vm.expectRevert("zkBTC: Cannot exceed maximum supply");
        zkbtc.mint(user, excessAmount);
    }

    function testMintToZeroAddress() public {
        uint256 mintAmount = 1000 * 10**8;
        
        vm.prank(owner);
        vm.expectRevert(); // OpenZeppelin ERC20InvalidReceiver error
        zkbtc.mint(address(0), mintAmount);
    }

    function testMultipleMints() public {
        uint256 mintAmount1 = 1000 * 10**8;
        uint256 mintAmount2 = 500 * 10**8;
        
        vm.startPrank(owner);
        zkbtc.mint(user, mintAmount1);
        zkbtc.mint(anotherUser, mintAmount2);
        vm.stopPrank();
        
        assertEq(zkbtc.totalSupply(), mintAmount1 + mintAmount2);
        assertEq(zkbtc.balanceOf(user), mintAmount1);
        assertEq(zkbtc.balanceOf(anotherUser), mintAmount2);
    }

    function testTransfer() public {
        uint256 mintAmount = 1000 * 10**8;
        uint256 transferAmount = 100 * 10**8;
        
        // Mint tokens to user
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        // Transfer tokens
        vm.prank(user);
        vm.expectEmit(true, true, false, false);
        emit Transfer(user, anotherUser, transferAmount);
        
        zkbtc.transfer(anotherUser, transferAmount);
        
        assertEq(zkbtc.balanceOf(user), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
    }

    function testTransferInsufficientBalance() public {
        uint256 mintAmount = 100 * 10**8;
        uint256 transferAmount = 200 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        vm.expectRevert(); // OpenZeppelin ERC20InsufficientBalance error
        zkbtc.transfer(anotherUser, transferAmount);
    }

    function testTransferToZeroAddress() public {
        uint256 mintAmount = 100 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        vm.expectRevert(); // OpenZeppelin ERC20InvalidReceiver error
        zkbtc.transfer(address(0), mintAmount);
    }

    function testApprove() public {
        uint256 mintAmount = 1000 * 10**8;
        uint256 approveAmount = 100 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        bool success = zkbtc.approve(anotherUser, approveAmount);
        
        assertTrue(success);
        assertEq(zkbtc.allowance(user, anotherUser), approveAmount);
    }

    function testTransferFrom() public {
        uint256 mintAmount = 1000 * 10**8;
        uint256 approveAmount = 100 * 10**8;
        uint256 transferAmount = 50 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        zkbtc.approve(anotherUser, approveAmount);
        
        vm.prank(anotherUser);
        vm.expectEmit(true, true, false, false);
        emit Transfer(user, anotherUser, transferAmount);
        
        zkbtc.transferFrom(user, anotherUser, transferAmount);
        
        assertEq(zkbtc.balanceOf(user), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
        assertEq(zkbtc.allowance(user, anotherUser), approveAmount - transferAmount);
    }

    function testTransferFromInsufficientAllowance() public {
        uint256 mintAmount = 1000 * 10**8;
        uint256 approveAmount = 50 * 10**8;
        uint256 transferAmount = 100 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        zkbtc.approve(anotherUser, approveAmount);
        
        vm.prank(anotherUser);
        vm.expectRevert(); // OpenZeppelin ERC20InsufficientAllowance error
        zkbtc.transferFrom(user, anotherUser, transferAmount);
    }

    function testTransferFromInsufficientBalance() public {
        uint256 mintAmount = 50 * 10**8;
        uint256 approveAmount = 100 * 10**8;
        uint256 transferAmount = 100 * 10**8;
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        zkbtc.approve(anotherUser, approveAmount);
        
        vm.prank(anotherUser);
        vm.expectRevert(); // OpenZeppelin ERC20InsufficientBalance error
        zkbtc.transferFrom(user, anotherUser, transferAmount);
    }

    function testMaxSupplyEdgeCase() public {
        // Mint exactly the max supply
        vm.prank(owner);
        zkbtc.mint(user, MAX_SUPPLY);
        
        assertEq(zkbtc.totalSupply(), MAX_SUPPLY);
        assertEq(zkbtc.balanceOf(user), MAX_SUPPLY);
        
        // Try to mint one more satoshi
        vm.prank(owner);
        vm.expectRevert("zkBTC: Cannot exceed maximum supply");
        zkbtc.mint(user, 1);
    }

    function testFuzzMint(uint256 amount) public {
        vm.assume(amount > 0);
        vm.assume(amount <= MAX_SUPPLY);
        
        vm.prank(owner);
        zkbtc.mint(user, amount);
        
        assertEq(zkbtc.totalSupply(), amount);
        assertEq(zkbtc.balanceOf(user), amount);
    }

    function testFuzzTransfer(uint256 mintAmount, uint256 transferAmount) public {
        vm.assume(mintAmount > 0);
        vm.assume(mintAmount <= MAX_SUPPLY);
        vm.assume(transferAmount <= mintAmount);
        vm.assume(transferAmount > 0);
        
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        
        vm.prank(user);
        zkbtc.transfer(anotherUser, transferAmount);
        
        assertEq(zkbtc.balanceOf(user), mintAmount - transferAmount);
        assertEq(zkbtc.balanceOf(anotherUser), transferAmount);
    }

    function testGasUsage() public {
        uint256 mintAmount = 1000 * 10**8;
        
        uint256 gasStart = gasleft();
        vm.prank(owner);
        zkbtc.mint(user, mintAmount);
        uint256 mintGas = gasStart - gasleft();
        
        gasStart = gasleft();
        vm.prank(user);
        zkbtc.transfer(anotherUser, mintAmount / 2);
        uint256 transferGas = gasStart - gasleft();
        
        console.log("Gas used for mint:", mintGas);
        console.log("Gas used for transfer:", transferGas);
        
        // Ensure gas usage is reasonable
        assertLt(mintGas, 100_000);
        assertLt(transferGas, 100_000);
    }
}