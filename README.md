# Blockchain Transaction Types

A generic abstraction on a variety of basic Blockchain transactions.

Allows signing and identifying transactions.

## Examples

### Signing

```rust
let transaction_json = serde_json::json!({
    "from": sender_address,
    "to": Address::random(),
    "value": "0x1"
});

let transaction =
    known_transaction_request_type_from_json(transaction_json, 60, Some(chain_id))
        .expect("Could not identify transaction")
        .signable_transaction_request();

let (signature_bytes, recovery_id) = transaction
    .sign_transaction(chain_id, |message| {
        signer.sign_recoverable(&message, Some(chain_id))
    })?;
```


### Identifying

```rust
let transaction_json = serde_json::json!({
    "from": sender_address,
    "to": Address::random(),
    "value": "0x1"
});

let request_type = known_transaction_request_type_from_json(transaction_json, 60, Some(chain_id))?;
let info = request_type.transaction_request().transaction_info();
```