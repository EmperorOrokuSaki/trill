// SPDX-License-Identifier: MIT 
pragma solidity >=0.7.0;

contract NFT {
    function mint(uint128 amount) pure external {
        uint128[] memory map = new uint128[](amount); // Define an array in memory

        for (uint256 index = 0; index < amount; index++) {
            map[index] = uint128(index); // Populate the array with values
        }
    }
}