pragma solidity ^0.4.24;

contract Example13 {
    
    struct EIP712Domain {
        string  name;
        string  version;
        address verifyingContract;
        uint256 chainId;
    }

    struct Test {
        bytes1 v_bytes1;
        bytes2 v_bytes2;
        bytes3 v_bytes3;
        bytes4 v_bytes4;
        bytes5 v_bytes5;
        bytes6 v_bytes6;
        bytes7 v_bytes7;
        bytes8 v_bytes8;
        bytes9 v_bytes9;
        bytes10 v_bytes10;
        bytes11 v_bytes11;
        bytes12 v_bytes12;
        bytes13 v_bytes13;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(bytes1 v_bytes1,bytes2 v_bytes2,bytes3 v_bytes3,bytes4 v_bytes4,bytes5 v_bytes5,bytes6 v_bytes6,bytes7 v_bytes7,bytes8 v_bytes8,bytes9 v_bytes9,bytes10 v_bytes10,bytes11 v_bytes11,bytes12 v_bytes12,bytes13 v_bytes13)"
    );
    bytes32 DOMAIN_SEPARATOR;

    constructor () public {
        DOMAIN_SEPARATOR = hash(EIP712Domain({
            name: "Test",
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

    function hash(Test test_value) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            TEST_TYPEHASH,
            test_value.v_bytes1,
            test_value.v_bytes2,
            test_value.v_bytes3,
            test_value.v_bytes4,
            test_value.v_bytes5,
            test_value.v_bytes6,
            test_value.v_bytes7,
            test_value.v_bytes8,
            test_value.v_bytes9,
            test_value.v_bytes10,
            test_value.v_bytes11,
            test_value.v_bytes12,
            test_value.v_bytes13
        ));
    }

    function hash_typed_data(Test test_value) internal view returns (bytes32) {
        // Note: we need to use `encodePacked` here instead of `encode`.
        return keccak256(abi.encodePacked(
            "\x19\x01",
            DOMAIN_SEPARATOR,
            hash(test_value)
        ));
    }

    function test() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_bytes1: 0x01,
            v_bytes2: 0x0102,
            v_bytes3: 0x010203,
            v_bytes4: 0x01020304,
            v_bytes5: 0x0102030405,
            v_bytes6: 0x010203040506,
            v_bytes7: 0x01020304050607,
            v_bytes8: 0x0102030405060708,
            v_bytes9: 0x010203040506070809,
            v_bytes10: 0x0102030405060708090a,
            v_bytes11: 0x0102030405060708090a0b,
            v_bytes12: 0x0102030405060708090a0b0c,
            v_bytes13: 0x0102030405060708090a0b0c0d
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}

contract Example26 {
    
    struct EIP712Domain {
        string  name;
        string  version;
        address verifyingContract;
        uint256 chainId;
    }

    struct Test {
        bytes14 v_bytes14;
        bytes15 v_bytes15;
        bytes16 v_bytes16;
        bytes17 v_bytes17;
        bytes18 v_bytes18;
        bytes19 v_bytes19;
        bytes20 v_bytes20;
        bytes21 v_bytes21;
        bytes22 v_bytes22;
        bytes23 v_bytes23;
        bytes24 v_bytes24;
        bytes25 v_bytes25;
        bytes26 v_bytes26;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(bytes14 v_bytes14,bytes15 v_bytes15,bytes16 v_bytes16,bytes17 v_bytes17,bytes18 v_bytes18,bytes19 v_bytes19,bytes20 v_bytes20,bytes21 v_bytes21,bytes22 v_bytes22,bytes23 v_bytes23,bytes24 v_bytes24,bytes25 v_bytes25,bytes26 v_bytes26)"
    );

    bytes32 DOMAIN_SEPARATOR;

    constructor () public {
        DOMAIN_SEPARATOR = hash(EIP712Domain({
            name: "Test",
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

    function hash(Test test_value) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            TEST_TYPEHASH,
            test_value.v_bytes14,
            test_value.v_bytes15,
            test_value.v_bytes16,
            test_value.v_bytes17,
            test_value.v_bytes18,
            test_value.v_bytes19,
            test_value.v_bytes20,
            test_value.v_bytes21,
            test_value.v_bytes22,
            test_value.v_bytes23,
            test_value.v_bytes24,
            test_value.v_bytes25,
            test_value.v_bytes26
        ));
    }

    function hash_typed_data(Test test_value) internal view returns (bytes32) {
        // Note: we need to use `encodePacked` here instead of `encode`.
        return keccak256(abi.encodePacked(
            "\x19\x01",
            DOMAIN_SEPARATOR,
            hash(test_value)
        ));
    }

    function test() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_bytes14: 0x0102030405060708090a0b0c0d0e,
            v_bytes15: 0x0102030405060708090a0b0c0d0e0f,
            v_bytes16: 0x0102030405060708090a0b0c0d0e0f10,
            v_bytes17: 0x0102030405060708090a0b0c0d0e0f1011,
            v_bytes18: 0x0102030405060708090a0b0c0d0e0f101112,
            v_bytes19: 0x0102030405060708090a0b0c0d0e0f10111213,
            v_bytes20: 0x000102030405060708090a0b0c0d0e0f1011121314,
            v_bytes21: 0x0102030405060708090a0b0c0d0e0f101112131415,
            v_bytes22: 0x0102030405060708090a0b0c0d0e0f10111213141516,
            v_bytes23: 0x0102030405060708090a0b0c0d0e0f1011121314151617,
            v_bytes24: 0x0102030405060708090a0b0c0d0e0f101112131415161718,
            v_bytes25: 0x0102030405060708090a0b0c0d0e0f10111213141516171819,
            v_bytes26: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}

contract Example32 {
    
    struct EIP712Domain {
        string  name;
        string  version;
        address verifyingContract;
        uint256 chainId;
    }

    struct Test {
        bytes27 v_bytes27;
        bytes28 v_bytes28;
        bytes29 v_bytes29;
        bytes30 v_bytes30;
        bytes31 v_bytes31;
        bytes32 v_bytes32;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(bytes27 v_bytes27,bytes28 v_bytes28,bytes29 v_bytes29,bytes30 v_bytes30,bytes31 v_bytes31,bytes32 v_bytes32)"
    );

    bytes32 DOMAIN_SEPARATOR;

    constructor () public {
        DOMAIN_SEPARATOR = hash(EIP712Domain({
            name: "Test",
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

    function hash(Test test_value) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            TEST_TYPEHASH,
            test_value.v_bytes27,
            test_value.v_bytes28,
            test_value.v_bytes29,
            test_value.v_bytes30,
            test_value.v_bytes31,
            test_value.v_bytes32
        ));
    }

    function hash_typed_data(Test test_value) internal view returns (bytes32) {
        // Note: we need to use `encodePacked` here instead of `encode`.
        return keccak256(abi.encodePacked(
            "\x19\x01",
            DOMAIN_SEPARATOR,
            hash(test_value)
        ));
    }

    function test() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_bytes27: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b,
            v_bytes28: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c,
            v_bytes29: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d,
            v_bytes30: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e,
            v_bytes31: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f,
            v_bytes32: 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}
