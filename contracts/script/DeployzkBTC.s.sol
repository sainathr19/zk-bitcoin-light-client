// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/zkBTC.sol";

/**
 * @title zkBTC Deployment Script
 * @dev Script to deploy the zkBTC contract
 */
contract DeployZkBTC is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy zkBTC contract with deployer as initial owner
        zkBTC zkbtc = new zkBTC(deployer);
        
        vm.stopBroadcast();
        
        console.log("zkBTC deployed to:", address(zkbtc));
        console.log("Deployer:", deployer);
        console.log("Initial supply:", zkbtc.totalSupply());
        console.log("Max supply:", zkbtc.maxSupply());
        console.log("Decimals:", zkbtc.decimals());
    }
}

