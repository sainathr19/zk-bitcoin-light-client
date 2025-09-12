// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/WBTC.sol";

/**
 * @title WBTC Deployment Script
 * @dev Script to deploy the WBTC contract
 */
contract DeployWBTC is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy WBTC contract with deployer as initial owner
        WBTC wbtc = new WBTC(deployer);
        
        vm.stopBroadcast();
        
        console.log("WBTC deployed to:", address(wbtc));
        console.log("Deployer:", deployer);
        console.log("Initial supply:", wbtc.totalSupply());
        console.log("Max supply:", wbtc.maxSupply());
        console.log("Decimals:", wbtc.decimals());
    }
}

