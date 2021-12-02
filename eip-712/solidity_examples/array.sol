pragma solidity ^0.4.24;

contract Example {
    
    struct EIP712Domain {
        string  name;
        string  version;
        uint256 chainId;
        address verifyingContract;
    }

    struct Array {
        uint8[] values;
        uint8[1] values1;
        uint8[3] values3;
    }

    struct Message {
        string message;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant ARRAY_TYPEHASH = keccak256(
        "Array(uint8[] values,uint8[1] values1,uint8[3] values3)"
    );

    bytes32 DOMAIN_SEPARATOR;

    constructor () public {
        DOMAIN_SEPARATOR = hash(EIP712Domain({
            name: "Array Test",
            version: '1',
            chainId: 1,
            verifyingContract: 0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC
        }));
    }

    function hash(EIP712Domain eip712Domain) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            EIP712DOMAIN_TYPEHASH,
            keccak256(bytes(eip712Domain.name)),
            keccak256(bytes(eip712Domain.version)),
            eip712Domain.chainId,
            eip712Domain.verifyingContract
        ));
    }

    function hash(Array array) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            ARRAY_TYPEHASH,
            keccak256(abi.encode(
                array.values[0],
                array.values[1]
            )),
            keccak256(abi.encode(
                array.values1[0]
            )),
            keccak256(abi.encode(
                array.values3[0],
                array.values3[1],
                array.values3[2]
            ))
        ));
    }

    function hash_typed_data(Array array) internal view returns (bytes32) {
        // Note: we need to use `encodePacked` here instead of `encode`.
        return keccak256(abi.encodePacked(
            "\x19\x01",
            DOMAIN_SEPARATOR,
            hash(array)
        ));
    }

    function test() public view returns (bytes32, bytes32, bytes32, bytes32) {
        uint8[] memory values = new uint8[](2);
        values[0] = 1;
        values[1] = 2;

        Array memory array = Array({
            values: values,
            values1: [17],
            values3: [9, 8, 7]
        });

        return (
            keccak256(abi.encode(
                array.values[0],
                array.values[1]
            )),
            keccak256(abi.encode(
                array.values1[0]
            )),
            keccak256(abi.encode(
                array.values3[0],
                array.values3[1],
                array.values3[2]
            )),
            hash_typed_data(array));
    }
}
