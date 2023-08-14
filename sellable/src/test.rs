#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Uint128,
    };
    use cw721_base::{msg::InstantiateMsg as cw721_baseInstantiateMsg, MintMsg};
    use cw_storage_plus::Map;
    use ownable::Ownable;
    use std::{cell::RefCell, rc::Rc};
    use token::Tokens;

    use crate::{errors::ContractError, Sellable};

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";
    const BUYER: &str = "burnt1e2fuwe3uhq8zd9nkkk876nawrwdulgv47mkgww";

    fn setup_sellable_module(
        deps: &mut DepsMut,
        env: &Env,
        info: &MessageInfo,
    ) -> Sellable<'static, Empty, Empty, Empty, Empty> {
        let sellable = Sellable::<Empty, Empty, Empty, Empty>::new(
            Rc::new(RefCell::new(Tokens::default())),
            Rc::new(RefCell::new(Ownable::default())),
            Map::new("listed_tokens"),
        );
        // Instantiate the ownable module
        sellable
            .ownable
            .borrow_mut()
            .owner
            .save(deps.storage, &Addr::unchecked(CREATOR))
            .unwrap();

        // Instantiate the token contract
        sellable
            .tokens
            .borrow()
            .contract
            .instantiate(
                deps.branch(),
                env.clone(),
                info.clone(),
                cw721_baseInstantiateMsg {
                    name: "burnt".to_string(),
                    symbol: "BRNT".to_string(),
                    minter: CREATOR.to_string(),
                },
            )
            .unwrap();

        sellable
    }

    #[test]
    fn sellable_token_list() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(CREATOR, &[]);

        let mut sellable = setup_sellable_module(&mut deps.as_mut(), &env, &info);

        // Mint a token
        sellable
            .tokens
            .borrow_mut()
            .contract
            .mint(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                MintMsg::<Empty> {
                    token_id: "1".to_string(),
                    owner: CREATOR.to_string(),
                    token_uri: Some("uri".to_string()),
                    extension: Empty {},
                },
            )
            .unwrap();

        // List a token
        let listings = schemars::Map::from([(
            "1".to_string(),
            Coin {
                amount: Uint128::new(10),
                denom: "uturnt".to_string(),
            },
        )]);
        sellable
            .try_list(&mut deps.as_mut(), env.clone(), info.clone(), listings)
            .unwrap();

        // List a non-minted token
        let non_minted_listings = schemars::Map::from([(
            "hello".to_string(),
            Coin {
                amount: Uint128::new(10),
                denom: "uturnt".to_string(),
            },
        )]);
        let list_result = sellable.try_list(
            &mut deps.as_mut(),
            env.clone(),
            info.clone(),
            non_minted_listings,
        );
        match list_result {
            Ok(_) => panic!(),
            Err(err) => match err {
                ContractError::TokenIDNotFoundError => {}
                _ => panic!(),
            },
        }

        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 1);
    }

    #[test]
    fn test_buy_listed_tokens() {
        // List a token
        // Send a request to buy a token with insufficient funds
        // Send a request to buy with enough fund but wrong fund denom
        // Send a request to buy with enough funds and denom
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(CREATOR, &[]);

        let mut sellable = setup_sellable_module(&mut deps.as_mut(), &env, &info);
        assert_eq!(
            sellable
                .tokens
                .borrow()
                .contract
                .token_count(&deps.storage)
                .unwrap(),
            0
        );

        // Mint a token
        sellable
            .tokens
            .borrow_mut()
            .contract
            .mint(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                MintMsg::<Empty> {
                    token_id: "1".to_string(),
                    owner: CREATOR.to_string(),
                    token_uri: Some("uri".to_string()),
                    extension: Empty {},
                },
            )
            .unwrap();
        assert_eq!(
            sellable
                .tokens
                .borrow()
                .contract
                .token_count(&deps.storage)
                .unwrap(),
            1
        );

        // List newly minted token
        let listings = schemars::Map::from([(
            "1".to_string(),
            Coin {
                amount: Uint128::new(10),
                denom: "uturnt".to_string(),
            },
        )]);
        sellable
            .try_list(&mut deps.as_mut(), env.clone(), info.clone(), listings)
            .unwrap();
        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 1);

        // Buy token with no funds
        sellable
            .try_buy_token(&mut deps.as_mut(), info, "1".to_string())
            .expect_err("Expect error");

        // Buy token with insufficient funds
        let i_funds = mock_info(
            BUYER,
            &[Coin {
                amount: Uint128::new(5),
                denom: "uturnt".to_string(),
            }],
        );
        sellable
            .try_buy_token(&mut deps.as_mut(), i_funds, "1".to_string())
            .expect_err("Expect error");

        // Buy token with sufficient funds and wrong denom
        let i_funds = mock_info(
            BUYER,
            &[Coin {
                amount: Uint128::new(5),
                denom: "turnt".to_string(),
            }],
        );
        sellable
            .try_buy_token(&mut deps.as_mut(), i_funds, "1".to_string())
            .expect_err("Expect error");

        // Buy token with sufficient funds and enough denom
        let new_funds = mock_info(
            BUYER,
            &[Coin {
                amount: Uint128::new(12),
                denom: "uturnt".to_string(),
            }],
        );
        let buy_resp = sellable
            .try_buy_token(&mut deps.as_mut(), new_funds, "1".to_string())
            .expect("purchased ticket");

        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 0);

        assert_eq!(buy_resp.response.messages.len(), 2);
        assert_eq!(
            buy_resp.response.messages[0].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: CREATOR.to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(10),
                    denom: "uturnt".to_string()
                }],
            })
        );
        assert_eq!(
            buy_resp.response.messages[1].msg,
            CosmosMsg::Bank(BankMsg::Send {
                to_address: BUYER.to_string(),
                amount: vec![Coin {
                    amount: Uint128::new(2),
                    denom: "uturnt".to_string()
                }],
            })
        );
    }

    #[test]
    fn try_delist_token() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(CREATOR, &[]);

        let mut sellable = setup_sellable_module(&mut deps.as_mut(), &env, &info);
        assert_eq!(
            sellable
                .tokens
                .borrow()
                .contract
                .token_count(&deps.storage)
                .unwrap(),
            0
        );

        // Mint a token
        sellable
            .tokens
            .borrow_mut()
            .contract
            .mint(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                MintMsg::<Empty> {
                    token_id: "1".to_string(),
                    owner: CREATOR.to_string(),
                    token_uri: Some("uri".to_string()),
                    extension: Empty {},
                },
            )
            .unwrap();
        assert_eq!(
            sellable
                .tokens
                .borrow()
                .contract
                .token_count(&deps.storage)
                .unwrap(),
            1
        );

        // List newly minted token
        let listings = schemars::Map::from([(
            "1".to_string(),
            Coin {
                amount: Uint128::new(10),
                denom: "uturnt".to_string(),
            },
        )]);
        sellable
            .try_list(&mut deps.as_mut(), env.clone(), info.clone(), listings)
            .unwrap();
        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 1);

        // De-list the ticket
        sellable
            .try_delist(&mut deps.as_mut(), info, "1".to_string())
            .unwrap();
        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 0);
    }
}
