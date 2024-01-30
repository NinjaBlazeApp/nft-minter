use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Creator {
    pub address: String,
    pub share: u8,
}

#[cw_serde]
pub struct RoyaltyInfo {
    pub creators: Vec<Creator>,
    pub seller_fee_basis_points: u16,
    pub primary_sell_happened: bool,
}

#[cw_serde]
pub enum TalisExecMsg {
    Mint {
        max_supply: Option<u64>,
        metadata_uri: String,
        owner: String,
        royalty: RoyaltyInfo,
    },
}
