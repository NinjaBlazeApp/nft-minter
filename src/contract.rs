use crate::api::{RoyaltyInfo, TalisExecMsg};
use crate::attributes::ATTR_ACTION;
use crate::error::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, ensure, ensure_eq, to_json_binary, Addr, BankMsg, Coin, Decimal, Timestamp, Uint128,
    WasmMsg,
};
use cw_storage_plus::Item;

use cw_utils::{must_pay, PaymentError};
use sylvia::contract;
use sylvia::cw_std::Response;
use sylvia::types::{ExecCtx, InstantiateCtx};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

#[cw_serde]
pub struct NFT {
    pub metadata: String,
    pub price: Coin,
}

#[cw_serde]
pub struct ContractConfig {
    pub withdrawal_address: Addr,
    pub fee_partner_fee: Decimal,

    pub nfts: Vec<NFT>,

    pub cw_collection: Addr,

    pub start: Timestamp,
    pub end: Timestamp,

    pub royalty: RoyaltyInfo,
}

pub struct NftMintContract<'a> {
    pub(crate) manager: Item<'a, Addr>,
    pub(crate) config: Item<'a, ContractConfig>,
    pub(crate) count: Item<'a, Uint128>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(ContractError)]
impl NftMintContract<'_> {
    pub fn new() -> Self {
        Self {
            manager: Item::new("manager"),
            config: Item::new("config"),
            count: Item::new("count"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, ContractError> {
        self.manager.save(ctx.deps.storage, &ctx.info.sender)?;
        Ok(Response::new().add_attribute(ATTR_ACTION, "instantiate"))
    }

    #[msg(exec)]
    pub fn start(&self, ctx: ExecCtx, config: ContractConfig) -> Result<Response, ContractError> {
        let manager = self.manager.load(ctx.deps.storage)?;
        ensure_eq!(ctx.info.sender, manager, ContractError::Unauthorized);

        ensure!(
            !self.config.exists(ctx.deps.storage),
            ContractError::ConfigAlreadyExists
        );
        self.count.save(ctx.deps.storage, &Uint128::from(0u128))?;
        self.config.save(ctx.deps.storage, &config)?;
        Ok(Response::new().add_attribute(ATTR_ACTION, "start"))
    }

    #[msg(exec)]
    pub fn mint(
        &self,
        ctx: ExecCtx,
        nft_type: usize,
        amount: Uint128,
        partner: Option<Addr>,
    ) -> Result<Response, ContractError> {
        let config = self.config.load(ctx.deps.storage)?;

        ensure!(
            config.start <= ctx.env.block.time,
            ContractError::SaleNoActive
        );

        ensure!(ctx.env.block.time < config.end, ContractError::SaleNoActive);

        let nft = &config.nfts[nft_type];

        let pay_amount = must_pay(&ctx.info, nft.price.denom.as_str())?;

        ensure_eq!(
            pay_amount,
            nft.price.amount.checked_mul(amount).unwrap(),
            ContractError::PaymentError(PaymentError::NoFunds {})
        );

        let partner_addr = partner.unwrap_or(config.withdrawal_address.clone());
        let partner_fee: Uint128 = pay_amount.mul_floor(config.fee_partner_fee);

        let fee_msg = BankMsg::Send {
            to_address: partner_addr.to_string(),
            amount: coins(partner_fee.u128(), nft.price.denom.as_str()),
        };

        let send_msg = BankMsg::Send {
            to_address: config.withdrawal_address.to_string(),
            amount: coins((pay_amount - partner_fee).u128(), nft.price.denom.as_str()),
        };

        let count = self.count.load(ctx.deps.storage)?;

        let mint_msgs: Vec<WasmMsg> = (count.into()..(count + amount).into())
            .map(|index: u128| WasmMsg::Execute {
                contract_addr: config.cw_collection.to_string(),
                msg: to_json_binary(&TalisExecMsg::Mint {
                    owner: ctx.info.sender.to_string(),
                    max_supply: None,
                    metadata_uri: format!("{}#{}", nft.metadata.clone(), index),
                    royalty: config.royalty.clone(),
                })
                .unwrap(),
                funds: vec![],
            })
            .collect();

        self.count.save(ctx.deps.storage, &(count + amount))?;

        Ok(Response::default()
            .add_attribute(ATTR_ACTION, "mint")
            .add_attribute("address", ctx.info.sender.to_string())
            .add_attribute("nft_type", nft_type.to_string())
            .add_attribute("amount", amount)
            .add_attribute("partner", partner_addr)
            .add_attribute("partner_fee", partner_fee.to_string())
            .add_messages(mint_msgs)
            .add_message(fee_msg)
            .add_message(send_msg))
    }

    #[msg(exec)]
    pub fn stop(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let manager = self.manager.load(ctx.deps.storage)?;
        ensure_eq!(ctx.info.sender, manager, ContractError::Unauthorized);

        let mut config = self.config.load(ctx.deps.storage)?;
        if config.end < ctx.env.block.time {
            return Err(ContractError::SaleNoActive);
        }

        config.end = ctx.env.block.time;
        self.config.save(ctx.deps.storage, &config)?;

        Ok(Response::new().add_attribute(ATTR_ACTION, "stop"))
    }
}
