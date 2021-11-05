use crate::models::transaction_info::TransactionInfo;
use crate::tests::helpers::signer::TestSigner;
use web3::types::Address;

#[test]
fn test_ethereum_signing() {
    let chain_id = 0;
    let signer = TestSigner::new();
    let sender_address = signer.ethereum_address();

    let transaction_json = serde_json::json!({
        "from": sender_address,
        "to": Address::random(),
        "value": "0x1"
    });

    let transaction =
        crate::known_transaction_request_type_from_json(transaction_json, 60, Some(chain_id))
            .expect("Could not identify transaction")
            .signable_transaction_request();
    let original_message = transaction.message_hash(chain_id);

    let (signature_bytes, recovery_id) = transaction
        .sign_transaction(chain_id, |message| {
            signer.sign_recoverable(&message, Some(chain_id))
        })
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

    let request_type =
        crate::known_transaction_request_type_from_json(transaction_json, 60, Some(chain_id))
            .expect("Could not identify transaction");
    let info = request_type.transaction_request().transaction_info();

    assert!(
        matches!(info, TransactionInfo::TokenTransfer { .. }),
        "Transaction should be a token transfer"
    );
}
