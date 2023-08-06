pragma solidity ^0.8.9;

// TODO currently just using simple incrementer placeholder contract
contract Gitbounties {

    uint _value;

    constructor(uint value) {
        _value = value;
    }

    function increment() public {
        _value += 1;
    }

    function get_value() public view returns(uint) {
        return _value;
    }

}
