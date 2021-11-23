pragma solidity ^0.4.24;

contract Example {
    
    struct EIP712Domain {
        string  name;
        string  version;
        address verifyingContract;
        uint256 chainId;
    }

    struct Test {
        int8   v_int8;
        int16  v_int16;
        int32  v_int32;
        int64  v_int64;
        int128 v_int128;
        int256 v_int256;
        int8   v_int8_num;
        int16  v_int16_num;
        int32  v_int32_num;
        int64  v_int64_num;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(int8 v_int8,int16 v_int16,int32 v_int32,int64 v_int64,int128 v_int128,int256 v_int256,int8 v_int8_num,int16 v_int16_num,int32 v_int32_num,int64 v_int64_num)"
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
            test_value.v_int8,
            test_value.v_int16,
            test_value.v_int32,
            test_value.v_int64,
            test_value.v_int128,
            test_value.v_int256,
            test_value.v_int8_num,
            test_value.v_int16_num,
            test_value.v_int32_num,
            test_value.v_int64_num
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
            v_int8: 0x0,
            v_int16: 0x1,
            v_int32: 0x2,
            v_int64: 0x4,
            v_int128: 0x8,
            v_int256: 0x10,
            v_int8_num: 0,
            v_int16_num: 1,
            v_int32_num: 8,
            v_int64_num: 16
        });
        return (hash(test_value), hash_typed_data(test_value));
    }

    function test_max() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_int8: 0x7F,
            v_int16: 0x7FFF,
            v_int32: 0x7FFFFFFF,
            v_int64: 0x7FFFFFFFFFFFFFFF,
            v_int128: 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_int256: 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_int8_num: 127,
            v_int16_num: 32767,
            v_int32_num: 2147483647,
            v_int64_num: 9223372036854775807
        });
        return (hash(test_value), hash_typed_data(test_value));
    }

    function test_min() public view returns (bytes32, bytes32) {
        Test memory test_value = Test({
            v_int8: -0x7F,
            v_int16: -0x7FFF,
            v_int32: -0x7FFFFFFF,
            v_int64: -0x7FFFFFFFFFFFFFFF,
            v_int128: -0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_int256: -0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
            v_int8_num: -128,
            v_int16_num: -32768,
            v_int32_num: -2147483648,
            v_int64_num: -9223372036854775808
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}
