pragma solidity ^0.4.24;

contract Example {
    
    struct EIP712Domain {
        string  name;
        string  version;
        uint256 chainId;
        address verifyingContract;
    }

    struct Test {
        uint8[] v_array;
        Message v_struct;
    }

    struct Message {
        string message;
    }

    bytes32 constant EIP712DOMAIN_TYPEHASH = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    );

    bytes32 constant TEST_TYPEHASH = keccak256(
        "Test(uint8[] v_array,Message v_struct)Message(string message)"
    );

    bytes32 constant MESSAGE_TYPEHASH = keccak256(
        "Message(string message)"
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
            keccak256(abi.encode(test_value.v_array[0])),
            hash(test_value.v_struct)
        ));
    }

    function hash(Message message) internal pure returns (bytes32) {
        return keccak256(abi.encode(
            MESSAGE_TYPEHASH,
            keccak256(bytes(message.message))
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
        uint8[] memory v_array = new uint8[](1);
        v_array[0] = 17;

        Message memory message = Message({message: "Hello World!"});

        Test memory test_value = Test({
            v_array: v_array,
            v_struct: message
        });
        return (hash(test_value), hash_typed_data(test_value));
    }
}
