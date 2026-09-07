// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.26;
import {Test} from "forge-std/Test.sol";
import {DiscoveryEvidenceRegistry} from "../src/DiscoveryEvidenceRegistry.sol";

contract DiscoveryEvidenceRegistryTest is Test {
    DiscoveryEvidenceRegistry registry;
    address a = address(10);
    address b = address(11);

    function setUp() public {
        registry = new DiscoveryEvidenceRegistry(address(this), 1 hours);
        registry.grantRole(registry.RELAY_ROLE(), a);
        registry.grantRole(registry.RELAY_ROLE(), b);
        registry.grantRole(registry.WATCHER_ROLE(), address(this));
        registry.setOperatorCluster(a, bytes32(uint256(10)));
        registry.setOperatorCluster(b, bytes32(uint256(11)));
    }

    function _attest(address relay) internal {
        bytes32[] memory producers = new bytes32[](1);
        producers[0] = bytes32(uint256(20));
        vm.prank(relay);
        registry.attest(
            bytes32(uint256(1)),
            bytes32(uint256(2)),
            bytes32(uint256(3)),
            bytes32(uint256(4)),
            bytes32(uint256(5)),
            producers,
            uint64(block.timestamp)
        );
    }

    function _final() internal view returns (bool) {
        return registry.isFinalFor(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)), bytes32(uint256(4)));
    }

    function testIndependentPublicationAndDelayBindExactObjects() public {
        _attest(a);
        assertFalse(_final());
        _attest(b);
        assertFalse(_final());
        vm.warp(block.timestamp + 1 hours);
        assertTrue(_final());
        assertFalse(
            registry.isFinalFor(bytes32(uint256(1)), bytes32(uint256(9)), bytes32(uint256(3)), bytes32(uint256(4)))
        );
        registry.revokeRole(registry.RELAY_ROLE(), a);
        assertFalse(_final());
    }

    function testQuarantineCannotBeVotedAway() public {
        _attest(a);
        _attest(b);
        vm.warp(block.timestamp + 1 hours);
        assertTrue(_final());
        registry.quarantine(bytes32(uint256(1)), bytes32(uint256(99)));
        assertFalse(_final());
        vm.expectRevert();
        this.republish();
    }

    function republish() external {
        _attest(a);
    }

    function testProducerAndDuplicateControlDoNotCount() public {
        registry.setOperatorCluster(a, bytes32(uint256(20)));
        vm.expectRevert();
        this.republish();
        registry.setOperatorCluster(a, bytes32(uint256(10)));
        registry.setOperatorCluster(b, bytes32(uint256(10)));
        _attest(a);
        vm.expectRevert();
        this.publishB();
    }

    function publishB() external {
        _attest(b);
    }

    function testQuarantineBeforePublicationIsPermanent() public {
        registry.quarantine(bytes32(uint256(1)), bytes32(uint256(99)));
        vm.expectRevert();
        this.republish();
    }
}
