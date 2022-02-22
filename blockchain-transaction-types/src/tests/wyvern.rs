use crate::models::ethereum_message::Message as EthereumMessage;
use crate::models::message::Message;
use crate::models::transaction_info::TransactionInfo;

#[test]
fn test_wyvern_sell_order() {
    let wyvern_meta_transaction = serde_json::json!({
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
        "Order": [
          {
            "name": "exchange",
            "type": "address"
          },
          {
            "name": "maker",
            "type": "address"
          },
          {
            "name": "taker",
            "type": "address"
          },
          {
            "name": "makerRelayerFee",
            "type": "uint256"
          },
          {
            "name": "takerRelayerFee",
            "type": "uint256"
          },
          {
            "name": "makerProtocolFee",
            "type": "uint256"
          },
          {
            "name": "takerProtocolFee",
            "type": "uint256"
          },
          {
            "name": "feeRecipient",
            "type": "address"
          },
          {
            "name": "feeMethod",
            "type": "uint8"
          },
          {
            "name": "side",
            "type": "uint8"
          },
          {
            "name": "saleKind",
            "type": "uint8"
          },
          {
            "name": "target",
            "type": "address"
          },
          {
            "name": "howToCall",
            "type": "uint8"
          },
          {
            "name": "calldata",
            "type": "bytes"
          },
          {
            "name": "replacementPattern",
            "type": "bytes"
          },
          {
            "name": "staticTarget",
            "type": "address"
          },
          {
            "name": "staticExtradata",
            "type": "bytes"
          },
          {
            "name": "paymentToken",
            "type": "address"
          },
          {
            "name": "basePrice",
            "type": "uint256"
          },
          {
            "name": "extra",
            "type": "uint256"
          },
          {
            "name": "listingTime",
            "type": "uint256"
          },
          {
            "name": "expirationTime",
            "type": "uint256"
          },
          {
            "name": "salt",
            "type": "uint256"
          },
          {
            "name": "nonce",
            "type": "uint256"
          }
        ]
      },
      "domain": {
        "name": "Wyvern Exchange Contract",
        "version": "2.3",
        "chainId": 1,
        "verifyingContract": "0x7f268357a8c2552623316e2562d90e642bb538e5"
      },
      "primaryType": "Order",
      "message": {
        "maker": "0xf020b2ae0995acedff07f9fc8298681f5461278a",
        "exchange": "0x7f268357a8c2552623316e2562d90e642bb538e5",
        "taker": "0x0000000000000000000000000000000000000000",
        "makerRelayerFee": "250",
        "takerRelayerFee": "0",
        "makerProtocolFee": "0",
        "takerProtocolFee": "0",
        "feeRecipient": "0x5b3256965e7c3cf26e11fcaf296dfc8807c01073",
        "feeMethod": 1,
        "side": 1,
        "saleKind": 0,
        "target": "0xbaf2127b49fc93cbca6269fade0f7f31df4c88a7",
        "howToCall": 1,
        "calldata": "0xfb16a595000000000000000000000000f020b2ae0995acedff07f9fc8298681f5461278a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000008c225a147c9be7c010961cc92c4e20f3ee93ecca0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000000",
        "replacementPattern": "0x000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "staticTarget": "0x0000000000000000000000000000000000000000",
        "staticExtradata": "0x",
        "paymentToken": "0x6b175474e89094c44da98b954eedeac495271d0f",
        "basePrice": "999000000000000000000",
        "extra": "0",
        "listingTime": "1645396001",
        "expirationTime": "0",
        "salt": "41323573726568630309062759941600676279092443246404216056651428130338573442731",
        "nonce": 0
      }
    }
    );

    let wyvern_message = EthereumMessage::from_json(wyvern_meta_transaction)
        .expect("Could not decode example message");

    let info = wyvern_message
        .meta_transaction_info()
        .expect("Should have been able to decode the Wyvern meta transaction");

  let expected_seller = "0xf020b2ae0995acedff07f9fc8298681f5461278a".to_owned();
  let expected_token_id = "0000000000000000000000000000000000000000000000000000000000000001".to_owned();



    assert!(
        matches!(
            info,
            TransactionInfo::TokenSale {
                seller: expected_seller,
                token_id: expected_token_id,
                ..
            }
        ),
        "Should have decoded the Wyvern meta transaction as a token sale"
    );
}
