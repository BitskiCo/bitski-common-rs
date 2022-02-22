use bigdecimal::BigDecimal;

#[derive(Clone, PartialEq, Default, Debug)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: Option<String>,
    pub decimals: u8,
    pub total_supply: Option<u64>,
    pub image: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TransactionInfo {
    TokenTransfer {
        from: String,
        to: String,
        amount: String,
        token_id: Option<String>,
        token_info: Option<TokenInfo>,
    },
    TokenSale {
        seller: String,
        buyer: String,
        amount: BigDecimal,
        currency: String,
        token_id: Option<String>,
        token_info: Option<TokenInfo>,
    },
    Unknown {
        value: Option<String>,
    },
}
