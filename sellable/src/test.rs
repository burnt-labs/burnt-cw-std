#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Attribute, BankMsg, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Uint128,
    };
    use cw721_base::{msg::InstantiateMsg as cw721_baseInstantiateMsg, MintMsg};
    use cw_storage_plus::Map;
    use ownable::Ownable;
    use std::{cell::RefCell, rc::Rc};
    use token::Tokens;

    use crate::{errors::ContractError, Sellable};

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";
    const BUYER: &str = "burnt1e2fuwe3uhq8zd9nkkk876nawrwdulgv47mkgww";
    const VENDOR: &str = "burnt1e2fuwe3uhq8zd9nkkk876nawrwsulgv47mkgww";

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
        // List a minted token with wrong owner
        let list_result = sellable
            .try_list(
                &mut deps.as_mut(),
                env.clone(),
                mock_info(VENDOR, &[]),
                listings.clone(),
            )
            .expect_err("Should not be able to list token with wrong owner");
        match list_result {
            ContractError::Unauthorized => {}
            _ => panic!(),
        }
        let res = sellable
            .try_list(
                &mut deps.as_mut(),
                env.clone(),
                info.clone(),
                listings.clone(),
            )
            .unwrap();
        // make sure listed token events are emitted
        let events = res.response.events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "sellable-list_items".to_string(),);
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("by".to_string(), info.sender.to_string()),
                Attribute::new(
                    "contract_address".to_string(),
                    env.contract.address.to_string()
                ),
                Attribute::new(
                    "listings".to_string(),
                    serde_json::to_string(&listings).unwrap().as_str()
                ),
            ]
        );
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
            .try_buy(&mut deps.as_mut(), &env, info, Some("1".to_string()))
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
            .try_buy(&mut deps.as_mut(), &env, i_funds, Some("1".to_string()))
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
            .try_buy(&mut deps.as_mut(), &env, i_funds, Some("1".to_string()))
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
            .try_buy(&mut deps.as_mut(), &env, new_funds, Some("1".to_string()))
            .expect("purchased ticket");

        // make sure listed token events are emitted
        let events = buy_resp.response.events;
        // There should be 3 events - sellable-buy_item, sellable-funds_sent, sellable-refund_sent
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].ty, "sellable-buy_item".to_string(),);
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("buyer".to_string(), BUYER.to_string()),
                Attribute::new(
                    "contract_address".to_string(),
                    env.contract.address.to_string()
                ),
                Attribute::new("seller".to_string(), CREATOR.to_string()),
                Attribute::new("purchased_token_id".to_string(), "1".to_string()),
                Attribute::new(
                    "price".to_string(),
                    serde_json::to_string(&Coin {
                        amount: Uint128::new(10),
                        denom: "uturnt".to_string(),
                    })
                    .unwrap()
                ),
            ]
        );
        assert_eq!(events[1].ty, "sellable-funds_sent".to_string(),);
        assert_eq!(
            events[1].attributes,
            vec![
                Attribute::new("to".to_string(), CREATOR.to_string()),
                Attribute::new(
                    "contract_address".to_string(),
                    env.contract.address.to_string()
                ),
                Attribute::new(
                    "amount".to_string(),
                    serde_json::to_string(&Coin {
                        amount: Uint128::new(10),
                        denom: "uturnt".to_string(),
                    })
                    .unwrap()
                ),
            ]
        );
        assert_eq!(events[2].ty, "sellable-refund_sent".to_string(),);
        assert_eq!(
            events[2].attributes,
            vec![
                Attribute::new("to".to_string(), BUYER.to_string()),
                Attribute::new(
                    "contract_address".to_string(),
                    env.contract.address.to_string()
                ),
                Attribute::new(
                    "amount".to_string(),
                    serde_json::to_string(&Coin {
                        amount: Uint128::new(2),
                        denom: "uturnt".to_string(),
                    })
                    .unwrap()
                ),
            ]
        );

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
        let res = sellable
            .try_delist(
                &mut deps.as_mut(),
                env.clone(),
                info.clone(),
                "1".to_string(),
            )
            .unwrap();
        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 0);
        // make sure listed token events are emitted
        let events = res.response.events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "sellable-delist_item".to_string(),);
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("by".to_string(), info.sender.to_string()),
                Attribute::new(
                    "contract_address".to_string(),
                    env.contract.address.to_string()
                ),
                Attribute::new("token_id".to_string(), "1".to_string()),
            ]
        );
    }
}
