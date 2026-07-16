// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {TempoForgeToken} from "../src/TempoForgeToken.sol";

contract TempoForgeTokenTest is Test {
    TempoForgeToken internal token;
    address internal owner = address(0xA11CE);

    function setUp() public {
        vm.prank(owner);
        token = new TempoForgeToken(owner);
    }

    function testInitialSupply() public view {
        assertEq(token.balanceOf(owner), 1_000_000 ether);
    }

    function testMintOnlyOwner(address caller, address to, uint256 amount) public {
        vm.assume(caller != owner);
        vm.assume(to != address(0));
        amount = bound(amount, 1, 1e24);
        vm.prank(caller);
        vm.expectRevert();
        token.mint(to, amount);
    }
}
