use crate::models::transaction_info::TransactionInfo;

use bigdecimal::BigDecimal;
use ethabi::{Param, ParamType};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use web3::types::{Address, Bytes, BytesArray, U256};

const WYVERN_2_3_EXCHANGE_CONTRACT_ADDRESS: &str = "0x7f268357a8c2552623316e2562d90e642bb538e5";

const MERKLE_VALIDATOR_CONTRACT_ADDRESS: &str = "0xbaf2127b49fc93cbca6269fade0f7f31df4c88a7";

lazy_static! {
    static ref MATCH_ERC721_USING_CRITERIA: ethabi::Function = matchERC721UsingCriteria();
}

/* An order on the exchange. */
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WyvernOrder {
    /* Exchange address, intended as a versioning mechanism. */
    exchange: Address,
    /* Order maker address. */
    maker: Address,
    /* Order taker address, if specified. */
    taker: Address,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: BigDecimal,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: BigDecimal,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: BigDecimal,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: BigDecimal,
    /* Order fee recipient or zero address for taker order. */
    feeRecipient: Address,
    /* Fee method (protocol token or split fee). */
    // feeMethod: FeeMethod
    /* Side (buy/sell). */
    // SaleKindInterface.Side side;
    /* Kind of sale. */
    // SaleKindInterface.SaleKind saleKind;
    /* Target. */
    target: Address,
    /* HowToCall. */
    // AuthenticatedProxy.HowToCall howToCall;
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-address for no static call. */
    staticTarget: Address,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-address as a sentinel value for Ether. */
    paymentToken: Address,
    /* Base price of the order (in paymentTokens). */
    basePrice: BigDecimal,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: BigDecimal,
    /* Listing timestamp. */
    listingTime: BigDecimal,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: BigDecimal,
    /* Order salt, used to prevent duplicate hashes. */
    salt: BigDecimal,
    /* NOTE: uint nonce is an additional component of the order but is read from storage */
}

pub fn parse_wyvern_meta_transaction(
    chain_id: u64,
    info: &bitski_eip_712::TypedData,
) -> Option<TransactionInfo> {
    match serde_json::from_value(info.message.clone()) {
        Ok(order) => parse_wyvern_order(chain_id, order),
        Err(error) => {
            println!("Error parsing Wyvern order: {:#?}", error);
            None
        }
    }
}

fn parse_wyvern_order(chain_id: u64, order: WyvernOrder) -> Option<TransactionInfo> {
    match (
        chain_id,
        serde_json::json!(order.target).as_str().unwrap_or_default(),
    ) {
        (1, MERKLE_VALIDATOR_CONTRACT_ADDRESS) => parse_merkle_validator_order(chain_id, order),
        (chain_id, address) => {
            println!(
                "Unknown target contract, chain id {}, address: {}",
                chain_id, address
            );
            None
        }
    }
}

fn parse_merkle_validator_order(chain_id: u64, order: WyvernOrder) -> Option<TransactionInfo> {
    let calldata_string = serde_json::json!(order.calldata)
        .as_str()
        .unwrap_or_default()
        .to_owned();
    match &calldata_string[0..10] {
        "0xfb16a595" => parse_merkle_validator_erc721_order(chain_id, order),
        _ => {
            println!("Unknown calldata: {}", calldata_string);
            None
        }
    }
}

fn parse_merkle_validator_erc721_order(
    chain_id: u64,
    order: WyvernOrder,
) -> Option<TransactionInfo> {
    let mut decoded_input = MATCH_ERC721_USING_CRITERIA
        .decode_input(&order.calldata.0[4..])
        .unwrap_or_default();

    let from = decoded_input.pop().unwrap();
    let to = decoded_input.pop().unwrap();
    let token = decoded_input.pop().unwrap();
    let tokenId = decoded_input.pop().unwrap();

    // TODO: check which end of transaction we are on
    Some(TransactionInfo::TokenSale {
        seller: format!("0x{}", from),
        buyer: format!("0x{}", to),
        amount: order.basePrice,
        currency: order.paymentToken.to_string(),
        token_id: Some(format!("0x{}", tokenId)),
        token_info: None,
    })
}

fn matchERC721UsingCriteria() -> ethabi::Function {
    serde_json::from_value(json!(
    {
        "inputs": [
            {
                "internalType": "address",
                "name": "from",
                "type": "address"
            },
            {
                "internalType": "address",
                "name": "to",
                "type": "address"
            },
            {
                "internalType": "contract IERC721",
                "name": "token",
                "type": "address"
            },
            {
                "internalType": "uint256",
                "name": "tokenId",
                "type": "uint256"
            },
            {
                "internalType": "bytes32",
                "name": "root",
                "type": "bytes32"
            },
            {
                "internalType": "bytes32[]",
                "name": "proof",
                "type": "bytes32[]"
            }
        ],
        "name": "matchERC721UsingCriteria",
        "outputs": [
            {
                "internalType": "bool",
                "name": "",
                "type": "bool"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "function"
    }))
    .unwrap()
}
