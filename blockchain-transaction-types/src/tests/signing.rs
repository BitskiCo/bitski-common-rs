use crate::models::coin_type::CoinType;
use crate::models::transaction_info::TransactionInfo;
use crate::tests::helpers::signer::TestSigner;
use web3::types::Address;

#[tokio::test]
async fn test_ethereum_signing() {
    let chain_id = 0;
    let signer = TestSigner::new();
    let sender_address = signer.ethereum_address();

    let transaction_json = serde_json::json!({
        "from": sender_address,
        "to": Address::random(),
        "value": "0x1"
    });

    let transaction = crate::known_transaction_request_type_from_json(
        transaction_json,
        CoinType::Ethereum,
        Some(chain_id),
    )
    .expect("Could not identify transaction")
    .signable_transaction_request();
    let original_message = transaction.message_hash(chain_id);

    let (signature_bytes, recovery_id) = transaction
        .sign_transaction(chain_id, move |message| {
            signer.sign_recoverable(message, Some(chain_id))
        })
        .await
        .expect("Could not sign transaction");

    let recovered_address =
        web3::signing::recover(&original_message, &signature_bytes, recovery_id as i32)
            .expect("Could not recover signature");

    assert_eq!(recovered_address, sender_address, "Address should match");
}

#[test]
fn test_ethereum_transfer_token_info() {
    let chain_id = 0;
    let signer = TestSigner::new();
    let sender_address = signer.ethereum_address();

    let transaction_json = serde_json::json!({
        "from": sender_address,
        "to": Address::random(),
        "value": "0x1"
    });

    let request_type = crate::known_transaction_request_type_from_json(
        transaction_json,
        CoinType::Ethereum,
        Some(chain_id),
    )
    .expect("Could not identify transaction");
    let info = request_type.transaction_request().transaction_info();

    assert!(
        matches!(info, TransactionInfo::TokenTransfer { .. }),
        "Transaction should be a token transfer"
    );
}

#[test]
fn test_1155_transfer_token_info() {
    let chain_id = 0;
    let signer = TestSigner::new();
    let sender_address = signer.ethereum_address();
    let contract_address = Address::random();
    let to = "0x0d4a03B23Ae95409A4ecfE9396A9D39ca4f0fed1".to_owned();
    let amount = "0x0000000000000000000000000000000000000000000000000000000000000001".to_owned();
    let token_id =
        Some("0x000000000000000000000000000000000000000000000000000000000003df5a".to_owned());

    let transaction_json = serde_json::json!({
        "from": sender_address,
        "to": contract_address,
        "data": format!("0xf242432a000000000000000000000000{}0000000000000000000000000d4a03b23ae95409a4ecfe9396a9d39ca4f0fed1000000000000000000000000000000000000000000000000000000000003df5a000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d4a03b23ae95409a4ecfe9396a9d39ca4f0fed1000000000000000000000000000000000000000000000000000000000003df5a000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000", format!("{:02x}", sender_address))
    });

    let request_type = crate::known_transaction_request_type_from_json(
        transaction_json,
        CoinType::Ethereum,
        Some(chain_id),
    )
    .expect("Could not identify transaction");
    let info = request_type.transaction_request().transaction_info();

    let from = format!("0x{:02x}", sender_address);
    assert!(
        matches!(
            info,
            TransactionInfo::TokenTransfer {
                from,
                to,
                amount,
                token_id,
                token_info: None
            }
        ),
        "Transaction should be a token transfer"
    );
}

#[test]
fn test_ethereum_address_token_info() {
    use crate::models::account::Account;
    let public_key =
        hex::decode("032fa5b4bfb4cddf97122f3a4b87be49fa43d9cd70d93bbb48ea8bc25be620cdf3").unwrap();
    let address = web3::types::Address::from_public_key(public_key.as_slice()).unwrap();
    assert_eq!(
        address.address(),
        "0xccbad6e6bc69d6f15d02a68f78b7869bd7ea7eed"
    );
}

#[tokio::test]
async fn test_721_signature() {
    let chain_id = 0;
    let signer = TestSigner::new();
    let sender_address = signer.ethereum_address();
    let json = serde_json::json!({
      "types": {
        "EIP712Domain": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "version",
            "type": "string"
          },
          {
            "name": "chainId",
            "type": "uint256"
          },
          {
            "name": "verifyingContract",
            "type": "address"
          }
        ],
        "Person": [
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "wallet",
            "type": "address"
          }
        ],
        "Mail": [
          {
            "name": "from",
            "type": "Person"
          },
          {
            "name": "to",
            "type": "Person"
          },
          {
            "name": "contents",
            "type": "string"
          }
        ]
      },
      "primaryType": "Mail",
      "domain": {
        "name": "Ether Mail",
        "version": "1",
        "chainId": 1,
        "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
      },
      "message": {
        "from": {
          "name": "Cow",
          "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
        },
        "to": {
          "name": "Bob",
          "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
        },
        "contents": "Hello, Bob!"
      }
    });

    let message = crate::known_message_type_from_json(json, CoinType::Ethereum, Some(chain_id))
        .expect("Could not identify message")
        .signable_message();
    let (signature_bytes, recovery_id) = message
        .sign_message(chain_id, move |message| {
            signer.sign_recoverable(message, Some(chain_id))
        })
        .await
        .expect("Could not sign transaction");
}
