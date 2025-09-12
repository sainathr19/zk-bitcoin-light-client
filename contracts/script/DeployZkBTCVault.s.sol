// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/zkBTCVault.sol";
import "../lib/sp1-contracts/contracts/src/v1.1.0/SP1Verifier.sol";

/**
 * @title zkBTCVault Deployment Script
 * @dev Script to deploy the zkBTCVault contract
 */
contract DeployZkBTCVault is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy SP1Verifier
        SP1Verifier verifier = new SP1Verifier();
        console.log("SP1Verifier deployed to:", address(verifier));
        
        // Bitcoin program verification key (placeholder)
        bytes32 bitcoinProgramVKey = 0x0000000000000000000000000000000000000000000000000000000000000001;
        
        // Deploy zkBTCVault
        zkBTCVault vault = new zkBTCVault(
            address(verifier),
            bitcoinProgramVKey,
            deployer
        );
        
        vm.stopBroadcast();
        
        console.log("zkBTCVault deployed to:", address(vault));
        console.log("Deployer:", deployer);
        console.log("SP1Verifier:", address(verifier));
        console.log("Bitcoin Program VKey:", vm.toString(bitcoinProgramVKey));
        console.log("zkBTC Token:", vault.getZkBTCToken());
    }
}
