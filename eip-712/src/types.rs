use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;
use web3::types::H256;

use crate::MemberType;

lazy_static! {
    static ref PRIMITIVE_TYPES: HashMap<&'static str, Type<'static>> = HashMap::from([
        ("address", Type::Address),
        ("bool", Type::Bool),
        ("string", Type::String),
        ("uint8", Type::Uint(8)),
        ("uint16", Type::Uint(16)),
        ("uint24", Type::Uint(24)),
        ("uint32", Type::Uint(32)),
        ("uint64", Type::Uint(64)),
        ("uint128", Type::Uint(128)),
        ("uint256", Type::Uint(256)),
        ("int8", Type::Int(8)),
        ("int16", Type::Int(16)),
        ("int24", Type::Int(24)),
        ("int32", Type::Int(32)),
        ("int64", Type::Int(64)),
        ("int128", Type::Int(128)),
        ("int256", Type::Int(256)),
        ("bytes", Type::Bytes),
        ("bytes1", Type::FixedBytes(1)),
        ("bytes2", Type::FixedBytes(2)),
        ("bytes3", Type::FixedBytes(3)),
        ("bytes4", Type::FixedBytes(4)),
        ("bytes6", Type::FixedBytes(6)),
        ("bytes5", Type::FixedBytes(5)),
        ("bytes7", Type::FixedBytes(7)),
        ("bytes8", Type::FixedBytes(8)),
        ("bytes9", Type::FixedBytes(9)),
        ("bytes10", Type::FixedBytes(10)),
        ("bytes11", Type::FixedBytes(11)),
        ("bytes12", Type::FixedBytes(12)),
        ("bytes13", Type::FixedBytes(13)),
        ("bytes14", Type::FixedBytes(14)),
        ("bytes15", Type::FixedBytes(15)),
        ("bytes16", Type::FixedBytes(16)),
        ("bytes17", Type::FixedBytes(17)),
        ("bytes18", Type::FixedBytes(18)),
        ("bytes19", Type::FixedBytes(19)),
        ("bytes20", Type::FixedBytes(20)),
        ("bytes21", Type::FixedBytes(21)),
        ("bytes22", Type::FixedBytes(22)),
        ("bytes23", Type::FixedBytes(23)),
        ("bytes24", Type::FixedBytes(24)),
        ("bytes25", Type::FixedBytes(25)),
        ("bytes26", Type::FixedBytes(26)),
        ("bytes27", Type::FixedBytes(27)),
        ("bytes28", Type::FixedBytes(28)),
        ("bytes29", Type::FixedBytes(29)),
        ("bytes30", Type::FixedBytes(30)),
        ("bytes31", Type::FixedBytes(31)),
        ("bytes32", Type::FixedBytes(32)),
    ]);
}

/// Type definitions without struct members.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Type<'a> {
    /// Address.
    Address,
    /// Boolean.
    Bool,
    /// Bytes.
    Bytes,
    /// String.
    String,

    /// Unsigned integer.
    Uint(usize),
    /// Signed integer.
    Int(usize),

    /// Array of bytes with fixed size.
    FixedBytes(usize),
    /// Array type, with fixed size.
    FixedArray(
        usize,   // Array size.
        &'a str, // Type name.
        &'a str, // Full type name.
    ),

    /// Array type, with dynamic size.
    Array(
        &'a str, // Type name.
        &'a str, // Full type name.
    ),
    /// Struct type reference.
    Struct(&'a str),
}

impl<'a> Type<'a> {
    /// Convert from the type name to a `Type`.
    pub(crate) fn try_from_name(name: &'a str) -> Result<Self> {
        if let Some(type_) = PRIMITIVE_TYPES.get(name) {
            return Ok(type_.clone());
        } else if is_ident(name) {
            return Ok(Type::Struct(name));
        } else if let Some(begin) = name.find("[") {
            if let Some(end) = name[begin..].rfind("]") {
                let ident = &name[..begin];
                let size = &name[begin + 1..begin + end];
                if size.is_empty() {
                    return Ok(Type::Array(ident, name));
                } else if let Ok(size) = usize::from_str_radix(size, 10) {
                    return Ok(Type::FixedArray(size, ident, name));
                }
            }
        }
        Err(anyhow!("invalid type name `{}`", name))
    }

    /// Validates the type.
    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Type::Address | Type::Bool | Type::Bytes | Type::String => true,
            Type::Uint(size) | Type::Int(size) => match size {
                8 | 16 | 32 | 64 | 128 | 256 => true,
                _ => false,
            },
            Type::FixedBytes(size) if 0 < *size && *size <= 32 => true,
            Type::FixedArray(_, _, _) | Type::Array(_, _) | Type::Struct(_) => true,
            _ => false,
        }
    }

    /// Gets the type name.
    ///
    /// Returns `<invalid>` if the type name is invalid.
    pub(crate) fn name(&self) -> &'a str {
        match self {
            Type::Address => "address",
            Type::Bool => "bool",
            Type::Bytes => "bytes",
            Type::String => "string",
            Type::Uint(8) => "uint8",
            Type::Uint(16) => "uint16",
            Type::Uint(24) => "uint24",
            Type::Uint(32) => "uint32",
            Type::Uint(64) => "uint64",
            Type::Uint(128) => "uint128",
            Type::Uint(256) => "uint256",
            Type::Int(8) => "int8",
            Type::Int(16) => "int16",
            Type::Int(24) => "int24",
            Type::Int(32) => "int32",
            Type::Int(64) => "int64",
            Type::Int(128) => "int128",
            Type::Int(256) => "int256",
            Type::FixedBytes(1) => "bytes1",
            Type::FixedBytes(2) => "bytes2",
            Type::FixedBytes(3) => "bytes3",
            Type::FixedBytes(4) => "bytes4",
            Type::FixedBytes(5) => "bytes5",
            Type::FixedBytes(6) => "bytes6",
            Type::FixedBytes(7) => "bytes7",
            Type::FixedBytes(8) => "bytes8",
            Type::FixedBytes(9) => "bytes9",
            Type::FixedBytes(10) => "bytes10",
            Type::FixedBytes(11) => "bytes11",
            Type::FixedBytes(12) => "bytes12",
            Type::FixedBytes(13) => "bytes13",
            Type::FixedBytes(14) => "bytes14",
            Type::FixedBytes(15) => "bytes15",
            Type::FixedBytes(16) => "bytes16",
            Type::FixedBytes(17) => "bytes17",
            Type::FixedBytes(18) => "bytes18",
            Type::FixedBytes(19) => "bytes19",
            Type::FixedBytes(20) => "bytes20",
            Type::FixedBytes(21) => "bytes21",
            Type::FixedBytes(22) => "bytes22",
            Type::FixedBytes(23) => "bytes23",
            Type::FixedBytes(24) => "bytes24",
            Type::FixedBytes(25) => "bytes25",
            Type::FixedBytes(26) => "bytes26",
            Type::FixedBytes(27) => "bytes27",
            Type::FixedBytes(28) => "bytes28",
            Type::FixedBytes(29) => "bytes29",
            Type::FixedBytes(30) => "bytes30",
            Type::FixedBytes(31) => "bytes31",
            Type::FixedBytes(32) => "bytes32",
            Type::FixedArray(_, name, _) | Type::Array(name, _) | Type::Struct(name) => name,
            _ => "<invalid>",
        }
    }

    /// Gets the type reference name (i.e. Type[n], Type[], or Type).
    pub(crate) fn reference_name(&self) -> &'a str {
        match self {
            Type::FixedArray(_, _, name) | Type::Array(_, name) => name,
            _ => self.name(),
        }
    }

    /// Returns the underlying `Type`.
    pub(crate) fn remove_reference(&self) -> Result<Type<'a>> {
        if self.is_array() {
            Type::try_from_name(self.name())
        } else {
            Ok(self.clone())
        }
    }

    /// Returns `true` if this type is atomic.
    pub(crate) fn is_atomic(&self) -> bool {
        match self {
            Type::Address | Type::Bool | Type::Uint(_) | Type::Int(_) | Type::FixedBytes(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if this type is dynamic.
    pub(crate) fn is_dynamic(&self) -> bool {
        match self {
            Type::Bytes | Type::String => true,
            _ => false,
        }
    }

    /// Returns `true` if this is an array type.
    pub(crate) fn is_array(&self) -> bool {
        match self {
            Type::FixedArray(_, _, _) | Type::Array(_, _) => true,
            _ => false,
        }
    }

    /// Returns `true` if this is a struct type.
    pub(crate) fn is_struct(&self) -> bool {
        if let Type::Struct(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if this type is a reference to a struct type.
    pub(crate) fn is_struct_ref(&self) -> bool {
        self.is_struct() || (self.is_array() && !is_primitive_type(self.name()))
    }
}

impl<'a> Hash for Type<'a> {
    /// Feeds the type reference name into the given `Hasher`.
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.reference_name().as_bytes());
    }
}

/// Struct type definition, with members.
#[derive(Debug)]
pub(crate) struct StructType<'a> {
    pub(crate) type_: Type<'a>,
    pub(crate) type_hash: Cell<Option<H256>>,
    pub(crate) members: Vec<StructMemberType<'a>>,
}

impl<'a> StructType<'a> {
    pub(crate) fn try_from_named_struct(
        name: &'a str,
        members: &'a Vec<MemberType>,
    ) -> Result<Self> {
        let type_ = Type::try_from_name(name)?;
        let mut visited = HashSet::new();
        let mut member_types = Vec::new();
        for member in members {
            if !visited.insert(&member.name) {
                return Err(anyhow!("duplicate member {}", member.name));
            }
            member_types.push(StructMemberType::try_from(member)?);
        }
        Ok(StructType {
            type_,
            type_hash: Cell::new(None),
            members: member_types,
        })
    }
}

impl<'a> Hash for StructType<'a> {
    /// Feeds the encoding of this struct into the given `Hasher`.
    ///
    /// Format: `name ‖ "(" ‖ member₁ ‖ "," ‖ member₂ ‖ "," ‖ … ‖ memberₙ ")"`
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.type_.name().as_bytes());
        state.write("(".as_bytes());
        if !self.members.is_empty() {
            self.members[0].hash(state);
            for member in &self.members[1..] {
                state.write(",".as_bytes());
                member.hash(state);
            }
        }
        state.write(")".as_bytes());
    }
}

/// Struct member definition.
#[derive(Debug)]
pub(crate) struct StructMemberType<'a> {
    pub(crate) name: &'a str,
    pub(crate) type_: Type<'a>,
}

impl<'a> TryFrom<&'a MemberType> for StructMemberType<'a> {
    type Error = anyhow::Error;

    fn try_from(that: &'a MemberType) -> Result<Self> {
        Ok(StructMemberType {
            name: &that.name,
            type_: Type::try_from_name(&that.r#type)?,
        })
    }
}

impl<'a> Hash for StructMemberType<'a> {
    /// Feeds the encoding of this member into the given `Hasher`.
    ///
    /// Format: `type ‖ " " ‖ name`
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_.hash(state);
        state.write(" ".as_bytes());
        state.write(self.name.as_bytes());
    }
}

/// Returns `true` if the string is an atomic type.
pub(crate) fn is_primitive_type(ident: &str) -> bool {
    PRIMITIVE_TYPES.contains_key(ident)
}

/// Returns `true` if the string is a valid [Solidity identifier][ident].
/// [ident]: https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityLexer.Identifier
pub(crate) fn is_ident(ident: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-zA-Z$_][a-zA-Z0-9$_]+$").unwrap();
    }
    RE.is_match(ident)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct BufHasher(Vec<u8>);

    impl Hasher for BufHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0.extend(bytes);
        }
        fn finish(&self) -> u64 {
            panic!("unexpected");
        }
    }

    #[test]
    fn is_indent_true() {
        assert!(is_ident("foo"));
        assert!(is_ident("$bar9"));
        assert!(is_ident("_bar$"));
        assert!(is_ident("_4bar_"));
    }

    #[test]
    fn is_ident_false() {
        assert!(!is_ident("!foo"));
        assert!(!is_ident("0bar"));
        assert!(!is_ident(" baz "));
    }

    #[test]
    fn type_try_from_name_address() {
        let type_ = Type::try_from_name("address").unwrap();
        match type_ {
            Type::Address => (),
            _ => panic!("expected address"),
        }

        assert_eq!(type_.name(), "address");
        assert_eq!(type_.reference_name(), "address");
        assert!(type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(!type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, type_);
    }

    #[test]
    fn type_try_from_name_bool() {
        let type_ = Type::try_from_name("bool").unwrap();
        match type_ {
            Type::Bool => (),
            _ => panic!("expected bool"),
        }

        assert_eq!(type_.name(), "bool");
        assert_eq!(type_.reference_name(), "bool");
        assert!(type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(!type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, type_);
    }

    #[test]
    fn type_try_from_name_uint() {
        for (type_name, expected_size) in [
            ("uint8", 8),
            ("uint16", 16),
            ("uint24", 24),
            ("uint32", 32),
            ("uint64", 64),
            ("uint128", 128),
            ("uint256", 256),
        ] {
            let type_ =
                Type::try_from_name(type_name).expect(format!("`{:?}` parses", type_name).as_str());
            match type_ {
                Type::Uint(size) if size == expected_size => (),
                _ => panic!("expected {}", type_name),
            }

            assert_eq!(type_.name(), type_name);
            assert_eq!(type_.reference_name(), type_name);
            assert!(type_.is_atomic(), "`{}` is atomic", type_name);
            assert!(!type_.is_dynamic(), "`{}` is not dynamic", type_name);
            assert!(!type_.is_array(), "`{}` is not an array", type_name);
            assert!(!type_.is_struct(), "`{}` is not a struct", type_name);
            assert!(
                !type_.is_struct_ref(),
                "`{}` is not a struct reference",
                type_name
            );

            let remove_reference = type_
                .remove_reference()
                .expect(format!("`{}` remove reference", type_name).as_str());

            assert_eq!(remove_reference, type_);
        }
    }

    #[test]
    fn type_try_from_name_int() {
        for (type_name, expected_size) in [
            ("int8", 8),
            ("int16", 16),
            ("int24", 24),
            ("int32", 32),
            ("int64", 64),
            ("int128", 128),
            ("int256", 256),
        ] {
            let type_ =
                Type::try_from_name(type_name).expect(format!("`{:?}` parses", type_name).as_str());
            match type_ {
                Type::Int(size) if size == expected_size => (),
                _ => panic!("expected {}", type_name),
            }

            assert_eq!(type_.name(), type_name);
            assert_eq!(type_.reference_name(), type_name);
            assert!(type_.is_atomic(), "`{}` is atomic", type_name);
            assert!(!type_.is_dynamic(), "`{}` is not dynamic", type_name);
            assert!(!type_.is_array(), "`{}` is not an array", type_name);
            assert!(!type_.is_struct(), "`{}` is not a struct", type_name);
            assert!(
                !type_.is_struct_ref(),
                "`{}` is not a struct reference",
                type_name
            );

            let remove_reference = type_
                .remove_reference()
                .expect(format!("`{}` remove reference", type_name).as_str());

            assert_eq!(remove_reference, type_);
        }
    }

    #[test]
    fn type_try_from_name_fixed_bytes() {
        for (type_name, expected_size) in [
            ("bytes1", 1),
            ("bytes2", 2),
            ("bytes3", 3),
            ("bytes4", 4),
            ("bytes5", 5),
            ("bytes6", 6),
            ("bytes7", 7),
            ("bytes8", 8),
            ("bytes9", 9),
            ("bytes10", 10),
            ("bytes11", 11),
            ("bytes12", 12),
            ("bytes13", 13),
            ("bytes14", 14),
            ("bytes15", 15),
            ("bytes16", 16),
            ("bytes17", 17),
            ("bytes18", 18),
            ("bytes19", 19),
            ("bytes20", 20),
            ("bytes21", 21),
            ("bytes22", 22),
            ("bytes23", 23),
            ("bytes24", 24),
            ("bytes25", 25),
            ("bytes26", 26),
            ("bytes27", 27),
            ("bytes28", 28),
            ("bytes29", 29),
            ("bytes30", 30),
            ("bytes31", 31),
            ("bytes32", 32),
        ] {
            let type_ =
                Type::try_from_name(type_name).expect(format!("`{:?}` parses", type_name).as_str());
            match type_ {
                Type::FixedBytes(size) if size == expected_size => (),
                _ => panic!("expected {}", type_name),
            }

            assert_eq!(type_.name(), type_name);
            assert_eq!(type_.reference_name(), type_name);
            assert!(type_.is_atomic(), "`{}` is atomic", type_name);
            assert!(!type_.is_dynamic(), "`{}` is not dynamic", type_name);
            assert!(!type_.is_array(), "`{}` is not an array", type_name);
            assert!(!type_.is_struct(), "`{}` is not a struct", type_name);
            assert!(
                !type_.is_struct_ref(),
                "`{}` is not a struct reference",
                type_name
            );

            let remove_reference = type_
                .remove_reference()
                .expect(format!("`{}` remove reference", type_name).as_str());

            assert_eq!(remove_reference, type_);
        }
    }

    #[test]
    fn type_try_from_name_bytes() {
        let type_ = Type::try_from_name("bytes").unwrap();
        match type_ {
            Type::Bytes => (),
            _ => panic!("expected bytes"),
        }

        assert_eq!(type_.name(), "bytes");
        assert_eq!(type_.reference_name(), "bytes");
        assert!(!type_.is_atomic());
        assert!(type_.is_dynamic());
        assert!(!type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, type_);
    }

    #[test]
    fn type_try_from_name_string() {
        let type_ = Type::try_from_name("string").unwrap();
        match type_ {
            Type::String => (),
            _ => panic!("expected string"),
        }

        assert_eq!(type_.name(), "string");
        assert_eq!(type_.reference_name(), "string");
        assert!(!type_.is_atomic());
        assert!(type_.is_dynamic());
        assert!(!type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, type_);
    }

    #[test]
    fn type_try_from_name_fixed_array() {
        let type_ = Type::try_from_name("uint8[3]").expect("`uint8[3]` parses");
        match type_ {
            Type::FixedArray(3, "uint8", "uint8[3]") => (),
            _ => panic!("expected uint8[3]"),
        }

        assert_eq!(type_.name(), "uint8");
        assert_eq!(type_.reference_name(), "uint8[3]");
        assert!(!type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, Type::Uint(8));
    }

    #[test]
    fn type_try_from_name_array() {
        let type_ = Type::try_from_name("uint8[]").expect("`uint8[]` parses");
        match type_ {
            Type::Array("uint8", "uint8[]") => (),
            _ => panic!("expected uint8[]"),
        }

        assert_eq!(type_.name(), "uint8");
        assert_eq!(type_.reference_name(), "uint8[]");
        assert!(!type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(type_.is_array());
        assert!(!type_.is_struct());
        assert!(!type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, Type::Uint(8));
    }

    #[test]
    fn type_try_from_name_array_struct_ref() {
        let type_ = Type::try_from_name("Type[]").expect("`Type[]` parses");
        match type_ {
            Type::Array("Type", "Type[]") => (),
            _ => panic!("expected Type[]"),
        }

        assert_eq!(type_.name(), "Type");
        assert_eq!(type_.reference_name(), "Type[]");
        assert!(!type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(type_.is_array());
        assert!(!type_.is_struct());
        assert!(type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, Type::Struct("Type"));
    }

    #[test]
    fn type_try_from_struct() {
        let type_ = Type::try_from_name("Type").expect("`Type` parses");
        match type_ {
            Type::Struct("Type") => (),
            _ => panic!("expected Type"),
        }

        assert_eq!(type_.name(), "Type");
        assert_eq!(type_.reference_name(), "Type");
        assert!(!type_.is_atomic());
        assert!(!type_.is_dynamic());
        assert!(!type_.is_array());
        assert!(type_.is_struct());
        assert!(type_.is_struct_ref());

        let remove_reference = type_.remove_reference().unwrap();
        assert_eq!(remove_reference, Type::Struct("Type"));
    }

    #[test]
    fn type_try_from_ok() {
        assert!(Type::try_from_name("uint7").is_ok());
        assert!(Type::try_from_name("$foo_").is_ok());
        assert!(Type::try_from_name("_bar$").is_ok());
        assert!(Type::try_from_name("baz").is_ok());
    }

    #[test]
    fn type_try_from_err() {
        assert!(Type::try_from_name("8").is_err());
        assert!(Type::try_from_name(" hello").is_err());
        assert!(Type::try_from_name("rrr[1]]").is_err());
        assert!(Type::try_from_name("Hello World").is_err());
    }

    #[test]
    fn type_hash_address() {
        let mut buf = BufHasher::default();
        Type::Address.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "address");
    }

    #[test]
    fn type_hash_bool() {
        let mut buf = BufHasher::default();
        Type::Bool.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "bool");
    }

    #[test]
    fn type_hash_bytes() {
        let mut buf = BufHasher::default();
        Type::Bytes.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "bytes");
    }

    #[test]
    fn type_hash_string() {
        let mut buf = BufHasher::default();
        Type::String.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "string");
    }

    #[test]
    fn type_hash_uint() {
        for size in [8, 16, 32, 64, 128, 256] {
            let mut buf = BufHasher::default();
            Type::Uint(size).hash(&mut buf);

            let type_name = String::from_utf8(buf.0).unwrap();
            assert_eq!(type_name, format!("uint{}", size));
        }
    }

    #[test]
    fn type_hash_int() {
        for size in [8, 16, 32, 64, 128, 256] {
            let mut buf = BufHasher::default();
            Type::Int(size).hash(&mut buf);

            let type_name = String::from_utf8(buf.0).unwrap();
            assert_eq!(type_name, format!("int{}", size));
        }
    }

    #[test]
    fn type_hash_fixed_bytes() {
        for size in 1usize..=32usize {
            let mut buf = BufHasher::default();
            Type::FixedBytes(size).hash(&mut buf);

            let type_name = String::from_utf8(buf.0).unwrap();
            assert_eq!(type_name, format!("bytes{}", size));
        }
    }

    #[test]
    fn type_hash_fixed_array() {
        let mut buf = BufHasher::default();
        Type::FixedArray(5, "int", "int[5]").hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "int[5]");
    }

    #[test]
    fn type_hash_array() {
        let mut buf = BufHasher::default();
        Type::Array("int", "int[]").hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "int[]");
    }

    #[test]
    fn type_hash_struct() {
        let mut buf = BufHasher::default();
        Type::Struct("Type").hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "Type");
    }

    #[test]
    fn struct_type_from_named_struct_mail() {
        let members = vec![
            MemberType {
                name: "from".to_string(),
                r#type: "Person".to_string(),
            },
            MemberType {
                name: "to".to_string(),
                r#type: "Person".to_string(),
            },
            MemberType {
                name: "contents".to_string(),
                r#type: "string".to_string(),
            },
        ];
        let type_ = StructType::try_from_named_struct("Mail", &members).unwrap();
        assert_eq!(type_.type_.name(), "Mail");
        assert_eq!(type_.members.len(), 3);
        assert_eq!(type_.members[0].name, "from");
        assert_eq!(type_.members[0].type_, Type::Struct("Person"));
        assert_eq!(type_.members[1].name, "to");
        assert_eq!(type_.members[1].type_, Type::Struct("Person"));
        assert_eq!(type_.members[2].name, "contents");
        assert_eq!(type_.members[2].type_, Type::String);
    }

    #[test]
    fn struct_type_from_named_struct_empty() {
        let members = vec![];
        let type_ = StructType::try_from_named_struct("Empty", &members).unwrap();
        assert_eq!(type_.type_.name(), "Empty");
        assert!(type_.members.is_empty());
    }

    #[test]
    fn struct_type_from_named_struct_err() {
        let members = vec![];
        assert!(StructType::try_from_named_struct("9nope", &members).is_err());

        let members = vec![MemberType {
            name: "foo".to_string(),
            r#type: "9nope".to_string(),
        }];
        assert!(StructType::try_from_named_struct("Type", &members).is_err());

        let members = vec![
            MemberType {
                name: "foo".to_string(),
                r#type: "bool".to_string(),
            },
            MemberType {
                name: "foo".to_string(),
                r#type: "bool".to_string(),
            },
        ];
        assert!(StructType::try_from_named_struct("Type", &members).is_err());
    }

    #[test]
    fn struct_type_hash() {
        let type_ = StructType {
            type_: Type::Struct("Mail"),
            type_hash: Cell::default(),
            members: vec![
                StructMemberType {
                    name: "from",
                    type_: Type::Struct("Person"),
                },
                StructMemberType {
                    name: "to",
                    type_: Type::Struct("Person"),
                },
                StructMemberType {
                    name: "contents",
                    type_: Type::String,
                },
            ],
        };

        let mut buf = BufHasher::default();
        type_.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "Mail(Person from,Person to,string contents)");
    }

    #[test]
    fn struct_member_type_try_from_ok() {
        for name in ["foo", "123", " ", "$"] {
            let member_type = MemberType {
                name: name.to_string(),
                r#type: "string".to_string(),
            };

            let struct_member_type = StructMemberType::try_from(&member_type).unwrap();
            assert_eq!(struct_member_type.name, name);
            assert_eq!(struct_member_type.type_.name(), "string");
        }
    }

    #[test]
    fn struct_member_type_try_from_err() {
        for type_ in ["123", " ", "$"] {
            let member_type = MemberType {
                name: "foo".to_string(),
                r#type: type_.to_string(),
            };
            assert!(StructMemberType::try_from(&member_type).is_err());
        }
    }

    #[test]
    fn struct_member_type_hash() {
        let type_ = StructMemberType {
            name: "value",
            type_: Type::Struct("Type"),
        };

        let mut buf = BufHasher::default();
        type_.hash(&mut buf);

        let type_name = String::from_utf8(buf.0).unwrap();
        assert_eq!(type_name, "Type value");
    }
}
