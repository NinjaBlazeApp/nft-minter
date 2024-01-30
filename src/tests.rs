#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{
        api::{Creator, RoyaltyInfo},
        contract::{ContractConfig, NftMintContract, NFT},
        error::ContractError,
    };
    use cosmwasm_std::{
        coin,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        Addr, Decimal, Empty, OwnedDeps, Uint128,
    };
    use sylvia::types::{ExecCtx, InstantiateCtx};

    #[test]
    fn test_contract() {
        let admin = Addr::unchecked("admin");

        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let contract_config = ContractConfig {
            cw_collection: Addr::unchecked("cw-collection"),
            withdrawal_address: admin,
            start: env.block.time.plus_minutes(1),
            end: env.block.time.plus_minutes(10),

            fee_partner_fee: Decimal::from_str("0.1").unwrap(),
            nfts: vec![NFT {
                metadata: "ipfs://".to_string(),
                price: coin(100, "inj"),
            }],
            royalty: RoyaltyInfo {
                creators: vec![Creator {
                    address: "bob".to_string(),
                    share: 100,
                }],
                primary_sell_happened: true,
                seller_fee_basis_points: 1000,
            },
        };

        let contract = NftMintContract::new();

        contract
            .instantiate(InstantiateCtx::from((
                deps.as_mut(),
                env.clone(),
                mock_info("admin", &[]),
            )))
            .unwrap();

        let (mut deps, contract) = test_start_message(deps, contract, contract_config);

        assert_eq!(
            contract
                .mint(
                    ExecCtx::from((deps.as_mut(), env.clone(), mock_info("alice", &[]))),
                    0,
                    Uint128::from(1u128),
                    None,
                )
                .unwrap_err(),
            ContractError::SaleNoActive
        );

        env.block.time = env.block.time.plus_minutes(2);

        assert_eq!(
            contract
                .mint(
                    ExecCtx::from((deps.as_mut(), env.clone(), mock_info("alice", &[]))),
                    0,
                    Uint128::from(1u128),
                    None,
                )
                .unwrap_err(),
            ContractError::PaymentError(cw_utils::PaymentError::NoFunds {})
        );

        contract
            .mint(
                ExecCtx::from((
                    deps.as_mut(),
                    env.clone(),
                    mock_info("alice", &[coin(100, "inj")]),
                )),
                0,
                Uint128::from(1u128),
                None,
            )
            .unwrap();

        assert_eq!(
            contract
                .mint(
                    ExecCtx::from((
                        deps.as_mut(),
                        env.clone(),
                        mock_info("alice", &[coin(100, "inj")]),
                    )),
                    0,
                    Uint128::from(5u128),
                    None,
                )
                .unwrap_err(),
            ContractError::PaymentError(cw_utils::PaymentError::NoFunds {})
        );

        contract
            .mint(
                ExecCtx::from((
                    deps.as_mut(),
                    env.clone(),
                    mock_info("alice", &[coin(500, "inj")]),
                )),
                0,
                Uint128::from(5u128),
                None,
            )
            .unwrap();

        env.block.time = env.block.time.plus_minutes(11);

        assert_eq!(
            contract
                .mint(
                    ExecCtx::from((deps.as_mut(), env.clone(), mock_info("alice", &[]))),
                    0,
                    Uint128::from(1u128),
                    None,
                )
                .unwrap_err(),
            ContractError::SaleNoActive
        );
    }

    fn test_start_message(
        mut deps: OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        contract: NftMintContract<'_>,
        contract_config: ContractConfig,
    ) -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        NftMintContract<'_>,
    ) {
        assert_eq!(
            contract
                .start(
                    ExecCtx::from((deps.as_mut(), mock_env(), mock_info("alice", &[]))),
                    contract_config.clone(),
                )
                .unwrap_err(),
            ContractError::Unauthorized
        );

        contract
            .start(
                ExecCtx::from((deps.as_mut(), mock_env(), mock_info("admin", &[]))),
                contract_config.clone(),
            )
            .unwrap();

        assert_eq!(
            contract
                .start(
                    ExecCtx::from((deps.as_mut(), mock_env(), mock_info("admin", &[]))),
                    contract_config,
                )
                .unwrap_err(),
            ContractError::ConfigAlreadyExists
        );

        return (deps, contract);
    }
}
