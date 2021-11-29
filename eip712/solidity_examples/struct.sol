pragma solidity ^0.4.24;

contract OptionalFieldsExample {
    
    struct EIP712Domain {
        string  name;
        string  version;
        uint256 chainId;
    }

    struct Test {
        string message;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(string message)"
    );

    bytes32 DOMAIN_SEPARATOR;

    constructor () public {
        DOMAIN_SEPARATOR = hash(EIP712Domain({
            name: "Test",
            version: '1',
            chainId: 1
        }));
    }

    function hash(EIP712Domain eip712Domain) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            EIP712DOMAIN_TYPEHASH,
            keccak256(bytes(eip712Domain.name)),
            keccak256(bytes(eip712Domain.version)),
            eip712Domain.chainId
        ));
    }

    function hash() internal pure returns (bytes32) {
        return keccak256(abi.encode(
            TEST_TYPEHASH
        ));
    }

    function hash_typed_data() internal view returns (bytes32) {
        // Note: we need to use `encodePacked` here instead of `encode`.
        return keccak256(abi.encodePacked(
            "\x19\x01",
            DOMAIN_SEPARATOR,
            hash()
        ));
    }

    function test() public view returns (bytes32, bytes32) {
        return (hash(), hash_typed_data());
    }
}
