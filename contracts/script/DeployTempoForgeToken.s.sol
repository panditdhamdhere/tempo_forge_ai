// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script, console2} from "forge-std/Script.sol";
import {TempoForgeToken} from "../src/TempoForgeToken.sol";

contract DeployTempoForgeToken is Script {
    function run() external {
        uint256 pk = vm.envUint("PRIVATE_KEY");
        address owner = vm.envOr("TOKEN_OWNER", vm.addr(pk));

        vm.startBroadcast(pk);
        TempoForgeToken token = new TempoForgeToken(owner);
        console2.log("TempoForgeToken deployed", address(token));
        console2.log("owner", owner);
        vm.stopBroadcast();
    }
}
