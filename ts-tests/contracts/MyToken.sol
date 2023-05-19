pragma solidity ^0.8.0;

import '@openzeppelin/contracts/token/ERC20/ERC20.sol';

// This ERC-20 contract mints the maximum amount of tokens to the contract creator.
contract MyToken is ERC20 {

    uint256 private _totalSupply;
    constructor(string memory name_,string memory symbol_, uint totalSupply_ )  ERC20(name_, symbol_) {
        _totalSupply = totalSupply_;
         _mint(msg.sender, 2**256 - 1);
    }
}
