// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.7.0;

import "./lib/forge-std/src/Script.sol";
import "./src/test.sol";

contract MyScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("ANVIL_PK");
        vm.startBroadcast(deployerPrivateKey);

        NFT nft = new NFT();

        vm.stopBroadcast();
    }
}