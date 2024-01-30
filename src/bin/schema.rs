use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use nft_minter::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
