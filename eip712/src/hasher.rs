use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::From;
use std::hash::{Hash as _, Hasher as _};
use std::str::FromStr;

use anyhow::{anyhow, Context as _, Result};
use hex::FromHex as _;
use num::bigint::Sign;
use num::{BigInt, BigUint, Signed as _};
use web3::types::{H256, U256};

use crate::types::*;
use crate::*;

const EIP_712_DOMAIN: &str = "EIP712Domain";

#[derive(Debug)]
pub(crate) struct Hasher<'a> {
    struct_types: HashMap<&'a str, StructType<'a>>,
    domain_separator: H256,
}

impl<'a> Hasher<'a> {
    /// Creates an `Hasher` with default primitive types.
    fn new() -> Self {
        Self {
            struct_types: Default::default(),
            domain_separator: Default::default(),
        }
    }

    pub(crate) fn hash(&self, typed_data: &TypedData) -> Result<H256> {
        let hash_struct = self.hash_struct(&typed_data.primary_type, &typed_data.message)?;
        let mut keccak = Keccak::v256();
        keccak.write(&[0x19, 0x01]);
        keccak.write(self.domain_separator.as_bytes());
        hash_struct.hash(&mut keccak);
        Ok(keccak.finish())
    }

    /// Returns the type hash of the struct.
    ///
    /// > `typeHash = keccak256(encodeType(typeOf(s)))`
    /// >
    /// > If the struct type references other struct struct_types (and these in
    /// > turn reference even more struct struct_types), then the set of
    /// > referenced struct struct_types is collected, sorted by name and
    /// > appended to the encoding.
    fn type_hash(&self, struct_type: &StructType<'a>) -> Result<H256> {
        if let Some(type_hash) = struct_type.type_hash.get() {
            Ok(type_hash)
        } else {
            let mut keccak = Keccak::v256();
            self.type_hash_struct(struct_type, &mut keccak)?;
            let type_hash = keccak.finish();
            struct_type.type_hash.set(Some(type_hash));
            Ok(type_hash)
        }
    }

    fn type_hash_struct<H: std::hash::Hasher>(
        &self,
        struct_type: &'a StructType<'a>,
        hasher: &mut H,
    ) -> Result<()> {
        for type_ in self.get_referenced_structs(struct_type.type_.name())? {
            type_.hash(hasher);
        }
        Ok(())
    }

    fn get_referenced_structs(&'a self, primary_type: &'a str) -> Result<Vec<&'a StructType<'a>>> {
        let mut visited = HashSet::new();
        let mut stack = vec![primary_type];

        while let Some(dep_name) = stack.pop() {
            if visited.insert(dep_name) {
                stack.extend(
                    self.struct_types
                        .get(dep_name)
                        .with_context(|| format!("invalid struct name {}", dep_name))?
                        .members
                        .iter()
                        .map(|m| &m.type_)
                        .filter(|m| m.is_struct_ref() && !visited.contains(m.name()))
                        .map(|m| m.name()),
                );
            }
        }

        visited.remove(primary_type);

        let mut deps: Vec<&'a str> = vec![primary_type];
        deps.extend(visited.iter().copied());
        deps[1..].sort();

        deps.iter()
            .map(|name| {
                self.struct_types
                    .get(name)
                    .with_context(|| format!("invalid struct name {}", name))
            })
            .collect()
    }

    /// Encodes the data into EIP-712 format.
    ///
    /// EIP-712:
    ///
    /// > Each encoded member value is exactly 32-byte long.
    /// >
    /// > The atomic values are encoded as follows: Boolean `false` and `true`
    /// > are encoded as `uint256` values `0` and `1` respectively. Addresses
    /// > are encoded as `uint160`. Integer values are sign-extended to 256-bit
    /// > and encoded in big endian order. `bytes1` to `bytes31` are arrays
    /// > with a beginning (index `0`) and an end (index `length - 1`), they
    /// > are zero-padded at the end to bytes32 and encoded in beginning to end
    /// > order. This corresponds to their encoding in ABI v1 and v2.
    /// >
    /// > The dynamic values `bytes` and `string` are encoded as a `keccak256`
    /// > hash of their contents.
    /// >
    /// > The array values are encoded as the `keccak256` hash of the
    /// > concatenated encodeData of their contents (i.e. the encoding of
    /// > `SomeType[5]` is identical to that of a struct containing five
    /// > members of type `SomeType`).
    /// >
    /// > The struct values are encoded recursively as `hashStruct(value)`.
    /// > This is undefined for cyclical data.
    fn hash_value(&self, type_: &Type, value: &serde_json::Value) -> Result<H256> {
        match type_ {
            Type::Address => {
                if let serde_json::Value::String(hex) = value {
                    let mut buf = H256::zero();
                    let enc = U256::from_str(hex).context("invalid address")?;
                    enc.to_big_endian(buf.as_fixed_bytes_mut());
                    Ok(buf)
                } else {
                    Err(anyhow!("expected address got {}", value))
                }
            }
            Type::Bool => {
                if let serde_json::Value::Bool(yes) = value {
                    let mut buf = H256::zero();
                    if *yes {
                        buf.as_mut()[31] = 1u8;
                    }
                    Ok(buf)
                } else {
                    Err(anyhow!("expected boolean got {}", value))
                }
            }
            Type::Bytes => {
                if let serde_json::Value::String(hex) = value {
                    let val = hex.strip_prefix("0x").unwrap_or(hex);
                    let buf = Vec::from_hex(val).context("invalid bytes")?;
                    let mut keccak = Keccak::v256();
                    keccak.write(&buf);
                    Ok(keccak.finish())
                } else {
                    Err(anyhow!("expected bytes got {}", value))
                }
            }
            Type::String => {
                if let serde_json::Value::String(val) = value {
                    let mut keccak = Keccak::v256();
                    keccak.write(val.as_bytes());
                    Ok(keccak.finish())
                } else {
                    Err(anyhow!("expected string got {}", value))
                }
            }
            Type::Int(size) if type_.is_valid() => {
                let int_value = match value {
                    serde_json::Value::Number(number) => {
                        if let Some(val) = number.as_u64() {
                            BigInt::from(val)
                        } else if let Some(val) = number.as_i64() {
                            BigInt::from(val)
                        } else {
                            return Err(anyhow!("invalid int{} {}", size, number));
                        }
                    }
                    serde_json::Value::String(value) => {
                        let (sign, uhex) = if let Some(uhex) = value.strip_prefix("-") {
                            (Sign::Minus, uhex)
                        } else {
                            (Sign::Plus, value.as_str())
                        };
                        let hex = uhex.strip_prefix("0x").unwrap_or(uhex);
                        let int_value = BigUint::parse_bytes(hex.as_bytes(), 16)
                            .with_context(|| format!("invalid int{} {}", size, value))?;
                        BigInt::from_biguint(sign, int_value)
                    }
                    _ => return Err(anyhow!("expected int{} got {}", size, value)),
                };
                let bytes = int_value.to_signed_bytes_be();
                if !bytes.is_empty() && bytes.len() <= size / 8 {
                    let mut buf = if int_value.is_negative() {
                        H256::repeat_byte(0xffu8)
                    } else {
                        H256::zero()
                    };
                    buf[32 - bytes.len()..].copy_from_slice(&bytes);
                    Ok(buf)
                } else {
                    Err(anyhow!("invalid int{} {}", size, value))
                }
            }
            Type::Uint(size) if type_.is_valid() => {
                let int_value = match value {
                    serde_json::Value::Number(number) => BigUint::from(
                        number
                            .as_u64()
                            .with_context(|| format!("invalid uint{} {}", size, value))?,
                    ),
                    serde_json::Value::String(value) => {
                        let hex = value.strip_prefix("0x").unwrap_or(value);
                        BigUint::parse_bytes(hex.as_bytes(), 16)
                            .with_context(|| format!("invalid uint{} {}", size, value))?
                    }
                    _ => return Err(anyhow!("expected uint{} got {}", size, value)),
                };
                let bytes = int_value.to_bytes_be();
                if !bytes.is_empty() && bytes.len() <= size / 8 {
                    let mut buf = H256::zero();
                    buf[32 - bytes.len()..].copy_from_slice(&bytes);
                    Ok(buf)
                } else {
                    Err(anyhow!("invalid uint{} {}", size, value))
                }
            }
            Type::FixedBytes(size) if type_.is_valid() => {
                if let serde_json::Value::String(hex) = value {
                    let hex = hex.strip_prefix("0x").unwrap_or(hex);
                    if hex.len() != size * 2 {
                        Err(anyhow!("invalid bytes{} {}", size, value))
                    } else {
                        let buf = Vec::from_hex(hex).context("invalid bytes")?;
                        let mut padded = H256::zero();
                        padded[..*size].copy_from_slice(&buf);
                        Ok(padded)
                    }
                } else {
                    Err(anyhow!("expected bytes{} got {}", size, value))
                }
            }
            Type::FixedArray(size, name, reference_name) => {
                if let serde_json::Value::Array(values) = value {
                    if values.len() != *size {
                        return Err(anyhow!(
                            "expected {} got {}[{}]",
                            reference_name,
                            name,
                            size
                        ));
                    }
                }
                self.hash_array(type_, value)
            }
            Type::Array(_, _) => self.hash_array(type_, value),
            Type::Struct(name) => self.hash_struct(name, value),
            _ => Err(anyhow!("invalid type {:?}", type_)),
        }
    }

    fn hash_struct(&self, name: &str, value: &serde_json::Value) -> Result<H256> {
        let type_ = self
            .struct_types
            .get(name)
            .with_context(|| format!("invalid struct name {}", name))?;
        if let serde_json::Value::Object(obj) = value {
            let type_hash = self.type_hash(type_)?;
            let mut keccak = Keccak::v256();
            type_hash.hash(&mut keccak);

            let mut visited: HashSet<&str> = HashSet::new();
            for member in &type_.members {
                if !visited.insert(member.name) {
                    return Err(anyhow!("duplicate member {}", member.name));
                }
                if let Some(val) = obj.get(member.name) {
                    if !val.is_null() {
                        let buf = self.hash_value(&member.type_, val)?;
                        keccak.write(buf.as_fixed_bytes());
                        continue;
                    }
                }
                keccak.write(&[0u8; 32]);
            }

            if obj.keys().all(|key| visited.contains(key.as_str())) {
                return Ok(keccak.finish());
            }
        }
        Err(anyhow!("not an object {}", value))
    }

    fn hash_array(&self, type_: &Type, value: &serde_json::Value) -> Result<H256> {
        let reference_type = type_.remove_reference()?;
        if let serde_json::Value::Array(arr) = value {
            let mut keccak = Keccak::v256();
            for val in arr {
                let buf = self.hash_value(&reference_type, val)?;
                keccak.write(buf.as_fixed_bytes());
            }
            Ok(keccak.finish())
        } else {
            Err(anyhow!("expected {} got {}", type_.reference_name(), value))
        }
    }
}

impl<'a> TryFrom<&'a TypedData> for Hasher<'a> {
    type Error = anyhow::Error;

    /// Builds an `Hasher` from `TypedData`.
    fn try_from(typed_data: &'a TypedData) -> Result<Self> {
        if !typed_data.types.contains_key(EIP_712_DOMAIN) {
            return Err(anyhow!("missing struct type {}", EIP_712_DOMAIN));
        }

        let mut hasher = Self::new();

        // Define struct types
        for (name, members) in &typed_data.types {
            // Can't define a struct type twice or redefine a built-in type
            if hasher.struct_types.contains_key(name.as_str()) || is_primitive_type(name) {
                return Err(anyhow!("type {} is already defined", name));
            }
            let def = StructType::try_from_named_struct(name, members)?;
            hasher.struct_types.insert(name, def);
        }

        // Set domain separator
        hasher.domain_separator = hasher.hash_struct(EIP_712_DOMAIN, &typed_data.domain)?;

        Ok(hasher)
    }
}

struct Keccak(tiny_keccak::Keccak);

impl Keccak {
    fn v256() -> Keccak {
        Keccak(tiny_keccak::Keccak::v256())
    }

    fn finish(self) -> H256 {
        use tiny_keccak::Hasher;
        let mut buf = H256::zero();
        self.0.finalize(buf.as_fixed_bytes_mut());
        buf
    }
}

impl std::hash::Hasher for Keccak {
    fn write(&mut self, bytes: &[u8]) {
        use tiny_keccak::Hasher;
        self.0.update(bytes);
    }

    fn finish(&self) -> u64 {
        log::warn!("not implemented");
        0
    }
}

#[cfg(test)]
mod tests {
    use hex::ToHex as _;
    use serde_json::json;

    use super::*;

    #[derive(Default)]
    struct BufHasher(Vec<u8>);

    impl std::hash::Hasher for BufHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0.extend(bytes);
        }
        fn finish(&self) -> u64 {
            panic!("unexpected call");
        }
    }

    const EMAIL_JSON: &'static str = r#"{
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Person": [
                {"name": "name", "type": "string"},
                {"name": "wallet", "type": "address"}
            ],
            "Mail": [
                {"name": "from", "type": "Person"},
                {"name": "to", "type": "Person"},
                {"name": "contents", "type": "string"}
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
    }"#;

    #[test]
    fn hasher_try_from_typed_data_ok() {
        let typed_data = serde_json::from_str::<TypedData>(EMAIL_JSON).unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();

        assert_eq!(
            format!("{}", hasher.domain_separator.encode_hex::<String>()),
            "f2cee375fa42b42143804025fc449deafd50cc031ca257e0b194a650a912090f"
        );
    }

    #[test]
    fn hasher_try_from_typed_data_err_missing_domain() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "Person": [
                    {"name": "name", "type": "string"},
                    {"name": "wallet", "type": "address"}
                ],
                "Mail": [
                    {"name": "from", "type": "Person"},
                    {"name": "to", "type": "Person"},
                    {"name": "contents", "type": "string"}
                ]
            },
            "primaryType": "Mail",
            "domain": {},
            "message": {}
        }))
        .unwrap();
        assert!(Hasher::try_from(&typed_data).is_err());
    }

    #[test]
    fn hasher_try_from_typed_data_err_duplicate_struct_member() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "message", "type": "string"},
                    {"name": "message", "type": "string"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "message": "Hello World!"
            }
        }))
        .unwrap();
        assert!(Hasher::try_from(&typed_data).is_err());
    }

    #[test]
    fn hasher_hash() {
        let typed_data = serde_json::from_str::<TypedData>(EMAIL_JSON).unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher.hash(&typed_data).unwrap();

        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
        );
    }

    #[test]
    fn hasher_type_hash() {
        let typed_data = serde_json::from_str::<TypedData>(EMAIL_JSON).unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();

        let struct_type = hasher.struct_types.get("Mail").unwrap();
        let type_hash = hasher.type_hash(struct_type).unwrap();

        assert_eq!(
            format!("{}", type_hash.encode_hex::<String>()),
            "a0cedeb2dc280ba39b857546d74f5549c3a1d7bdc2dd96bf881f76108e23dac2"
        );
    }

    #[test]
    fn hasher_type_hash_struct_mail() {
        let typed_data = serde_json::from_str::<TypedData>(EMAIL_JSON).unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();

        let mut buf = BufHasher::default();
        let struct_type = hasher.struct_types.get("Mail").unwrap();
        hasher.type_hash_struct(struct_type, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf.0).unwrap(),
            "Mail(Person from,Person to,string contents)Person(string name,address wallet)"
        );
    }

    #[test]
    fn hasher_type_hash_struct_faucet() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Root": [
                    {"name": "type", "type": "Type"},
                    {"name": "value", "type": "Value"},
                    {"name": "leaf", "type": "Leaf"}
                ],
                "Leaf": [],
                "Type": [
                    {"name": "name", "type": "string"}
                ],
                "Value": [
                    {"name": "value", "type": "string"}
                ]
            },
            "primaryType": "Root",
            "domain": {
                "name": "Tree Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {}
        }))
        .unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();

        let mut buf = BufHasher::default();
        let struct_type = hasher.struct_types.get("Root").unwrap();
        hasher.type_hash_struct(struct_type, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf.0).unwrap(),
            "Root(Type type,Value value,Leaf leaf)Leaf()Type(string name)Value(string value)"
        );
    }

    #[test]
    fn hasher_get_referenced_structs() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Root": [
                    {"name": "left", "type": "Leaf"},
                    {"name": "right", "type": "Leaf"}
                ],
                "Leaf": [
                    {"name": "value", "type": "Value"},
                    {"name": "type", "type": "Type"}
                ],
                "Type": [
                    {"name": "name", "type": "string"}
                ],
                "Value": [
                    {"name": "value", "type": "string"}
                ]
            },
            "primaryType": "Root",
            "domain": {
                "name": "Tree Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {}
        }))
        .unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();

        let structs = hasher.get_referenced_structs("Root").unwrap();
        assert_eq!(structs.len(), 4);
        assert_eq!(structs[0].type_.name(), "Root");
        assert_eq!(structs[1].type_.name(), "Leaf");
        assert_eq!(structs[2].type_.name(), "Type");
        assert_eq!(structs[3].type_.name(), "Value");
    }

    #[test]
    fn hasher_hash_value_address() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_address", "type": "address"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_address": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "64612d1fd3d1a558ba3f51b23feb608ca334e5568202f0ff26a6a95d8e444623"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "1e897ef9aceaa099bc8e0baab1e52127166564502f55948887e0f06984eff5b5"
        );
    }

    #[test]
    fn hasher_hash_value_bool() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_false", "type": "bool"},
                    {"name": "v_true", "type": "bool"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_false": false,
                "v_true": true
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "62276bf7dce5a6cea0053236c6cea05f8298c9956c386352eb554595f43e983f"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "f515bc982973d2cc5ec72919ed1224657b10b9e8b03ec2ce470ee53959ec2a03"
        );
    }

    #[test]
    fn hasher_hash_value_dynamic() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_string", "type": "string"},
                    {"name": "v_bytes", "type": "bytes"},
                    {"name": "v_bytes_empty", "type": "bytes"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_string": "Hello World!",
                "v_bytes": "0x01020304",
                "v_bytes_empty": ""
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "f004df1c12b1783b0887dc6391fc1bd0571b3bdc0d5622d101da13b067c5ec5b"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "7ee2aa2263033c2d19060ed792dfc99655338932d51f48176e43e0fb0522f298"
        );
    }

    #[test]
    fn hasher_hash_value_uint() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_uint8", "type": "uint8"},
                    {"name": "v_uint16", "type": "uint16"},
                    {"name": "v_uint32", "type": "uint32"},
                    {"name": "v_uint64", "type": "uint64"},
                    {"name": "v_uint128", "type": "uint128"},
                    {"name": "v_uint256", "type": "uint256"},
                    {"name": "v_uint8_num", "type": "uint8"},
                    {"name": "v_uint16_num", "type": "uint16"},
                    {"name": "v_uint32_num", "type": "uint32"},
                    {"name": "v_uint64_num", "type": "uint64"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_uint8": "0x0",
                "v_uint16": "0x1",
                "v_uint32": "0x2",
                "v_uint64": "0x4",
                "v_uint128": "0x8",
                "v_uint256": "0x10",
                "v_uint8_num": 0u64,
                "v_uint16_num": 1u64,
                "v_uint32_num": 8u64,
                "v_uint64_num": 16u64
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "66954cda7bb110ca2b58210ae37f32de437c78916a4fd0f5c7735051e34d247e"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "fe33de685fbbfcb8d18ee64b659dab8e751f80ed7f56d12f00e85b2a681435e2"
        );
    }

    #[test]
    fn hasher_hash_value_uint_max() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_uint8", "type": "uint8"},
                    {"name": "v_uint16", "type": "uint16"},
                    {"name": "v_uint32", "type": "uint32"},
                    {"name": "v_uint64", "type": "uint64"},
                    {"name": "v_uint128", "type": "uint128"},
                    {"name": "v_uint256", "type": "uint256"},
                    {"name": "v_uint8_num", "type": "uint8"},
                    {"name": "v_uint16_num", "type": "uint16"},
                    {"name": "v_uint32_num", "type": "uint32"},
                    {"name": "v_uint64_num", "type": "uint64"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_uint8": "0xFF",
                "v_uint16": "0xFFFF",
                "v_uint32": "0xFFFFFFFF",
                "v_uint64": "0xFFFFFFFFFFFFFFFF",
                "v_uint128": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_uint256": "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_uint8_num": 255u64,
                "v_uint16_num": 65535u64,
                "v_uint32_num": 4294967295u64,
                "v_uint64_num": 18446744073709551615u64,
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "500ceda2d78d584a9aa062ef3e00e8e04f34b0ad0c1bcfa9965de87ee354459c"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "491cff300ff40705b3782533654af6b181ba35c59683fbde1d11a3921aacf06d"
        );
    }

    #[test]
    fn hasher_hash_value_int() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_int8", "type": "int8"},
                    {"name": "v_int16", "type": "int16"},
                    {"name": "v_int32", "type": "int32"},
                    {"name": "v_int64", "type": "int64"},
                    {"name": "v_int128", "type": "int128"},
                    {"name": "v_int256", "type": "int256"},
                    {"name": "v_int8_num", "type": "int8"},
                    {"name": "v_int16_num", "type": "int16"},
                    {"name": "v_int32_num", "type": "int32"},
                    {"name": "v_int64_num", "type": "int64"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_int8": "0x0",
                "v_int16": "0x1",
                "v_int32": "0x2",
                "v_int64": "0x4",
                "v_int128": "0x8",
                "v_int256": "0x10",
                "v_int8_num": 0i64,
                "v_int16_num": 1i64,
                "v_int32_num": 8i64,
                "v_int64_num": 16i64
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "0d4458c9a346993386c3e3e9729a5e54678ad9d55665195c9ce140f17451b2e0"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "57a9df3f2202a688108d9446701028f33444cda1370960a588b2208631a16e57"
        );
    }

    #[test]
    fn hasher_hash_value_int_max() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_int8", "type": "int8"},
                    {"name": "v_int16", "type": "int16"},
                    {"name": "v_int32", "type": "int32"},
                    {"name": "v_int64", "type": "int64"},
                    {"name": "v_int128", "type": "int128"},
                    {"name": "v_int256", "type": "int256"},
                    {"name": "v_int8_num", "type": "int8"},
                    {"name": "v_int16_num", "type": "int16"},
                    {"name": "v_int32_num", "type": "int32"},
                    {"name": "v_int64_num", "type": "int64"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_int8": "0x7F",
                "v_int16": "0x7FFF",
                "v_int32": "0x7FFFFFFF",
                "v_int64": "0x7FFFFFFFFFFFFFFF",
                "v_int128": "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_int256": "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_int8_num": 127i64,
                "v_int16_num": 32767i64,
                "v_int32_num": 2147483647i64,
                "v_int64_num": 9223372036854775807i64
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "fc337dca39e22d42195c4b76ac5cd82f280e27b9a74d3be6079f8f02623f6675"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "040543314d107dac1b81fc51d113beed2066cc826db6a1e3d530f33eb624f4cb"
        );
    }

    #[test]
    fn hasher_hash_value_int_min() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_int8", "type": "int8"},
                    {"name": "v_int16", "type": "int16"},
                    {"name": "v_int32", "type": "int32"},
                    {"name": "v_int64", "type": "int64"},
                    {"name": "v_int128", "type": "int128"},
                    {"name": "v_int256", "type": "int256"},
                    {"name": "v_int8_num", "type": "int8"},
                    {"name": "v_int16_num", "type": "int16"},
                    {"name": "v_int32_num", "type": "int32"},
                    {"name": "v_int64_num", "type": "int64"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_int8": "-0x7F",
                "v_int16": "-0x7FFF",
                "v_int32": "-0x7FFFFFFF",
                "v_int64": "-0x7FFFFFFFFFFFFFFF",
                "v_int128": "-0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_int256": "-0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                "v_int8_num": -128i64,
                "v_int16_num": -32768i64,
                "v_int32_num": -2147483648i64,
                "v_int64_num": -9223372036854775808i64
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "4ffef993e5f5076e31f7ad198c7f2aba3c93e59914c64fb4f81f3a16baa28d02"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "cf513b5d5ad9e2c37de26f7683ae9c355d48c424256818681941df9bf0f169af"
        );
    }

    #[test]
    fn hasher_hash_value_bytes13() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_bytes1", "type": "bytes1"},
                    {"name": "v_bytes2", "type": "bytes2"},
                    {"name": "v_bytes3", "type": "bytes3"},
                    {"name": "v_bytes4", "type": "bytes4"},
                    {"name": "v_bytes5", "type": "bytes5"},
                    {"name": "v_bytes6", "type": "bytes6"},
                    {"name": "v_bytes7", "type": "bytes7"},
                    {"name": "v_bytes8", "type": "bytes8"},
                    {"name": "v_bytes9", "type": "bytes9"},
                    {"name": "v_bytes10", "type": "bytes10"},
                    {"name": "v_bytes11", "type": "bytes11"},
                    {"name": "v_bytes12", "type": "bytes12"},
                    {"name": "v_bytes13", "type": "bytes13"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_bytes1": "0x01",
                "v_bytes2": "0x0102",
                "v_bytes3": "0x010203",
                "v_bytes4": "0x01020304",
                "v_bytes5": "0x0102030405",
                "v_bytes6": "0x010203040506",
                "v_bytes7": "0x01020304050607",
                "v_bytes8": "0x0102030405060708",
                "v_bytes9": "0x010203040506070809",
                "v_bytes10": "0x0102030405060708090a",
                "v_bytes11": "0x0102030405060708090a0b",
                "v_bytes12": "0x0102030405060708090a0b0c",
                "v_bytes13": "0x0102030405060708090a0b0c0d"
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "8de403772562040006e06655b7187c0b7cf87274b7d82d957b677ec817a7e59d"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "8c1528e4fb48d913bd8213f1eeeeed9e44fd317c8d5990e0e12449aa49ac856c"
        );
    }

    #[test]
    fn hasher_hash_value_bytes27() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_bytes14", "type": "bytes14"},
                    {"name": "v_bytes15", "type": "bytes15"},
                    {"name": "v_bytes16", "type": "bytes16"},
                    {"name": "v_bytes17", "type": "bytes17"},
                    {"name": "v_bytes18", "type": "bytes18"},
                    {"name": "v_bytes19", "type": "bytes19"},
                    {"name": "v_bytes20", "type": "bytes20"},
                    {"name": "v_bytes21", "type": "bytes21"},
                    {"name": "v_bytes22", "type": "bytes22"},
                    {"name": "v_bytes23", "type": "bytes23"},
                    {"name": "v_bytes24", "type": "bytes24"},
                    {"name": "v_bytes25", "type": "bytes25"},
                    {"name": "v_bytes26", "type": "bytes26"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_bytes14": "0x0102030405060708090a0b0c0d0e",
                "v_bytes15": "0x0102030405060708090a0b0c0d0e0f",
                "v_bytes16": "0x0102030405060708090a0b0c0d0e0f10",
                "v_bytes17": "0x0102030405060708090a0b0c0d0e0f1011",
                "v_bytes18": "0x0102030405060708090a0b0c0d0e0f101112",
                "v_bytes19": "0x0102030405060708090a0b0c0d0e0f10111213",
                "v_bytes20": "0x0102030405060708090a0b0c0d0e0f1011121314",
                "v_bytes21": "0x0102030405060708090a0b0c0d0e0f101112131415",
                "v_bytes22": "0x0102030405060708090a0b0c0d0e0f10111213141516",
                "v_bytes23": "0x0102030405060708090a0b0c0d0e0f1011121314151617",
                "v_bytes24": "0x0102030405060708090a0b0c0d0e0f101112131415161718",
                "v_bytes25": "0x0102030405060708090a0b0c0d0e0f10111213141516171819",
                "v_bytes26": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a"
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "119984790ce56094d974f795efd91d5718d88f4db35784244224a28f8ec694c0"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "cf952d316b91fbe220f36db8b3471d357cd5658faeb7c58836815c368d3ebdb3"
        );
    }

    #[test]
    fn hasher_hash_value_bytes32() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_bytes27", "type": "bytes27"},
                    {"name": "v_bytes28", "type": "bytes28"},
                    {"name": "v_bytes29", "type": "bytes29"},
                    {"name": "v_bytes30", "type": "bytes30"},
                    {"name": "v_bytes31", "type": "bytes31"},
                    {"name": "v_bytes32", "type": "bytes32"}
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_bytes27": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b",
                "v_bytes28": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c",
                "v_bytes29": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d",
                "v_bytes30": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e",
                "v_bytes31": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
                "v_bytes32": "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20"
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "6edee5d43a336628cf20a77394e9b6cb3371674c70029388ed8e384b764ec172"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "d2c7479a3485740f41292ec511d9da85da1f84bad49323e9d4a1d3c5016d0cb6"
        );
    }

    #[test]
    fn hasher_hash_value_reference() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "v_array", "type": "uint8[]"},
                    {"name": "v_struct", "type": "Message"}
                ],
                "Message": [
                    {"name": "message", "type": "string"}
                ]
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "v_array": [17],
                "v_struct": {"message": "Hello World!"}
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "1c15c6c2a84919517e45702009100d9d8f958796f0cc3f138ba7e87b5ecd41e9"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "6e934a84aae4ffe60a91a44b1564bcfbe2a20f74829c259aa5c9949950adfb56"
        );
    }

    #[test]
    fn hasher_hash_struct_ok() {
        let typed_data = serde_json::from_str::<TypedData>(EMAIL_JSON).unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();

        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "c52c0ee5d84264471806290a3f2c4cecfc5490626bf912d01f240d7a274b371e"
        );
    }

    #[test]
    fn hasher_hash_struct_ok_optional_fields() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"},
                    {"name": "metadata", "type": "string"}
                ],
                "Test": [
                    {"name": "message", "type": "string"},
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "metadata": "test"
            },
            "message": {
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "6c356e95b8cbedbda73e904b83431a7161093c2a15830b46f6ad460dd8c93885"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "7b1275efbb90ebd79b5a6719a55fbc14bca636720da305be12b899e1f5dda576"
        );
    }

    #[test]
    fn hasher_hash_struct_err_extra_struct_member() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Test": [
                    {"name": "message", "type": "string"},
                ],
            },
            "primaryType": "Test",
            "domain": {
                "name": "Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "message": "Hello World!",
                "extra": "error"
            }
        }))
        .unwrap();

        let hasher = Hasher::try_from(&typed_data).unwrap();

        assert!(hasher
            .hash_struct(&typed_data.primary_type, &typed_data.message)
            .is_err());

        assert!(hasher.hash(&typed_data).is_err());
    }

    #[test]
    fn hasher_hash_array() {
        let typed_data = serde_json::from_value::<TypedData>(json!({
            "types": {
                "EIP712Domain": [
                    {"name": "name", "type": "string"},
                    {"name": "version", "type": "string"},
                    {"name": "chainId", "type": "uint256"},
                    {"name": "verifyingContract", "type": "address"}
                ],
                "Array": [
                    {"name": "values", "type": "uint8[]"},
                    {"name": "values1", "type": "uint8[1]"},
                    {"name": "values3", "type": "uint8[3]"}
                ]
            },
            "primaryType": "Array",
            "domain": {
                "name": "Array Test",
                "version": "1",
                "chainId": 1,
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "message": {
                "values": [1, 2],
                "values1": [17],
                "values3": [9, 8, 7]
            }
        }))
        .unwrap();
        let hasher = Hasher::try_from(&typed_data).unwrap();
        let result = hasher
            .hash_array(&Type::Array("uint8", "uint8[]"), &json!([1, 2]))
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "e90b7bceb6e7df5418fb78d8ee546e97c83a08bbccc01a0644d599ccd2a7c2e0"
        );

        let result = hasher
            .hash_array(&Type::Array("uint8", "uint8[1]"), &json!([17]))
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "31ecc21a745e3968a04e9570e4425bc18fa8019c68028196b546d1669c200c68"
        );

        let result = hasher
            .hash_array(&Type::Array("uint8", "uint8[3]"), &json!([9, 8, 7]))
            .unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "accf129a6a79fef4f7ce83aec82a72903fc576d12e3ce46716c41ce860282a9e"
        );

        let result = hasher.hash(&typed_data).unwrap();
        assert_eq!(
            format!("{}", result.encode_hex::<String>()),
            "b7aba063c3c6220f0bb7d951ef14fdb0b5829b4c41a86517685131360ecfb7e1"
        );
    }
}
