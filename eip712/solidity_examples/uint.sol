pragma solidity ^0.4.24;

contract Example {
    
    struct EIP712Domain {
        string  name;
        string  version;
        address verifyingContract;
        uint256 chainId;
    }

    struct Test {
        uint8   v_uint8;
        uint16  v_uint16;
        uint32  v_uint32;
        uint64  v_uint64;
        uint128 v_uint128;
        uint256 v_uint256;
        uint8   v_uint8_num;
        uint16  v_uint16_num;
        uint32  v_uint32_num;
        uint64  v_uint64_num;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(uint8 v_uint8,uint16 v_uint16,uint32 v_uint32,uint64 v_uint64,uint128 v_uint128,uint256 v_uint256,uint8 v_uint8_num,uint16 v_uint16_num,uint32 v_uint32_num,uint64 v_uint64_num)"
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
            test_value.v_uint8,
            test_value.v_uint16,
            test_value.v_uint32,
            test_value.v_uint64,
            test_value.v_uint128,
            test_value.v_uint256,
            test_value.v_uint8_num,
            test_value.v_uint16_num,
            test_value.v_uint32_num,
            test_value.v_uint64_num
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
            v_uint8: 0x0,
            v_uint16: 0x1,
            v_uint32: 0x2,
            v_uint64: 0x4,
            v_uint128: 0x8,
            v_uint256: 0x10,
            v_uint8_num: 0,
            v_uint16_num: 1,
            v_uint32_num: 8,
            v_uint64_num: 16
        });
        return (hash(test_value), hash_typed_data(test_value));
    }

    function test_max() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_uint8: 0xFF,
            v_uint16: 0xFFFF,
            v_uint32: 0xFFFFFFFF,
            v_uint64: 0xFFFFFFFFFFFFFFFF,
            v_uint128: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_uint256: 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_uint8_num: 255,
            v_uint16_num: 65535,
            v_uint32_num: 4294967295,
            v_uint64_num: 18446744073709551615
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}
