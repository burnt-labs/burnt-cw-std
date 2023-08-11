#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use burnt_glue::module::Module;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Attribute, Coin, Empty, Timestamp, Uint128, Uint64,
    };
    use cw_storage_plus::{Item, Map};
    use ownable::Ownable;
    use sellable::Sellable;
    use token::Tokens;

    use crate::{
        msg::{CreatePrimarySale, ExecuteMsg, InstantiateMsg, QueryMsg},
        PrimarySale, Sales,
    };
    use cw721_base::{msg::InstantiateMsg as cw721_baseInstantiateMsg, MintMsg};
    use serde_json::{from_str, json};

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";
    const USER: &str = "burnt188rjfzzrdxlus60zgnrvs4rg0l73hct3mlvdpe";

    #[test]
    fn add_primary_sales() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info(CREATOR, &[Coin::new(20, "USDC")]);

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
            .save(&mut deps.storage, &Addr::unchecked(CREATOR))
            .unwrap();
        // Instantiate the token contract
        sellable
            .tokens
            .borrow()
            .contract
            .instantiate(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                cw721_baseInstantiateMsg {
                    name: "burnt".to_string(),
                    symbol: "BRNT".to_string(),
                    minter: CREATOR.to_string(),
                },
            )
            .unwrap();
        let mut sales: Sales<Empty, Empty, Empty, Empty> =
            Sales::new(Rc::new(RefCell::new(sellable)), Item::new("primary_sales"));
        // instantiate sale module
        let sales_instantiate_msg = InstantiateMsg {
            sale: Some(CreatePrimarySale {
                total_supply: Uint64::from(1_u64),
                start_time: Uint64::from(1664567586_u64),
                end_time: Uint64::from(1665567587_u64),
                price: vec![Coin {
                    denom: "uturnt".to_string(),
                    amount: Uint128::from(10_u64),
                }],
            }),
        };
        let res = sales
            .instantiate(
                &mut deps.as_mut(),
                &env,
                &info,
                sales_instantiate_msg.clone(),
            )
            .expect("sale module instantiated");
        // make sure sale instantiation event is emitted
        let events = res.response.events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "sales-add_primary_sale");
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("by", info.sender.to_string()),
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new(
                    "sale_object",
                    serde_json::to_string(&PrimarySale::from(sales_instantiate_msg.sale.unwrap()))
                        .unwrap()
                ),
            ]
        );
        // get all primary sales
        let query_msg = QueryMsg::PrimarySales {};
        let primary_sales = sales
            .query(&deps.as_ref(), env.clone(), query_msg.clone())
            .unwrap();
        match primary_sales {
            crate::msg::QueryResp::PrimarySales(primary_sales) => {
                assert_eq!(primary_sales.len(), 1)
            }
            _ => panic!(),
        }
        // create a primary sale
        let primary_sale = CreatePrimarySale {
            total_supply: Uint64::from(1_u64),
            start_time: Uint64::from(1674567586_u64),
            end_time: Uint64::from(1675567587_u64),
            price: vec![Coin {
                denom: "USDC".to_string(),
                amount: Uint128::from(10_u64),
            }],
        };
        let execute_msg = ExecuteMsg::PrimarySale(primary_sale.clone());
        // test creating multiple active primary sales
        let fake_info = mock_info("hacker", &[]);
        sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                fake_info,
                execute_msg.clone(),
            )
            .expect_err("primary sales should not be added");
        // set block time
        env.block.time = Timestamp::from_seconds(1674567586_u64);
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("primary sales added");
        let primary_sales = sales.query(&deps.as_ref(), env.clone(), query_msg).unwrap();
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match primary_sales {
            crate::msg::QueryResp::PrimarySales(primary_sales) => {
                assert_eq!(primary_sales.len(), 2);
            }
            _ => panic!(),
        }
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => {
                assert_eq!(sale.start_time.seconds().to_string(), "1674567586")
            }
            _ => panic!(),
        }
        // buy an item
        let mint_msg = MintMsg::<Empty> {
            token_id: "1".to_string(),
            owner: CREATOR.to_string(),
            token_uri: Some("url".to_string()),
            extension: Empty {},
        };
        let execute_msg = ExecuteMsg::BuyItem(mint_msg.clone());
        let buyer_info = Rc::new(RefCell::new(mock_info(USER, &[Coin::new(20, "USDC")])));
        let res = sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                buyer_info.borrow_mut().clone(),
                execute_msg,
            )
            .expect("item bought");
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(None) => assert!(true),
            _ => panic!(),
        }
        // make sure sale buy event is emitted
        let events = res.response.events;
        // there should be 4 events: sales-token_minted, sales-funds_sent, sales-refund_sent, sales-sale_ended
        // the last event sales-sale_ended is not emitted because the sale is not ended
        assert_eq!(events.len(), 4);
        assert_eq!(events[0].ty, "sales-token_minted");
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("by", env.contract.address.to_string()),
                Attribute::new("for", buyer_info.borrow().sender.to_string()),
                Attribute::new("token_metadata", serde_json::to_string(&mint_msg).unwrap()),
            ]
        );
        assert_eq!(events[1].ty, "sales-sale_ended");
        let mut primary_sale = PrimarySale::from(primary_sale);
        primary_sale.tokens_minted = Uint64::from(1_u64);
        primary_sale.disabled = true;
        assert_eq!(
            events[1].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new(
                    "sale_object",
                    serde_json::to_string(&PrimarySale::from(primary_sale)).unwrap()
                ),
            ]
        );
        assert_eq!(events[2].ty, "sales-funds_sent");
        assert_eq!(
            events[2].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("to", CREATOR.to_string()),
                Attribute::new("amount", "10"),
                Attribute::new("denom", "USDC"),
            ]
        );
        assert_eq!(events[3].ty, "sales-refund_sent");
        assert_eq!(
            events[3].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("to", buyer_info.borrow().sender.to_string()),
                Attribute::new("amount", "10"),
                Attribute::new("denom", "USDC"),
            ]
        );
        // create a new primary sale
        let execute_msg = CreatePrimarySale {
            total_supply: Uint64::from(1_u64),
            start_time: Uint64::from(1676567587_u64),
            end_time: Uint64::from(1677567587_u64),
            price: vec![Coin {
                denom: "USDC".to_string(),
                amount: Uint128::from(10_u64),
            }],
        };
        env.block.time = Timestamp::from_seconds(1676567587);
        let mut execute_halt_sale_msg = PrimarySale::from(execute_msg.clone());
        sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                info.clone(),
                ExecuteMsg::PrimarySale(execute_msg),
            )
            .expect("primary sales added");
        // halt ongoing primary sale
        let json_exec_msg = json!({
            "halt_sale": { }
        })
        .to_string();
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        let res = sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                info.clone(),
                execute_msg.clone(),
            )
            .expect("any ongoing sale halted");
        // make sure alse halt event is emitted
        let events = res.response.events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, "sales-halt_sale");
        execute_halt_sale_msg.disabled = true;
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("by", info.sender.to_string()),
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new(
                    "sale_object",
                    serde_json::to_string(&execute_halt_sale_msg).unwrap()
                ),
            ]
        );
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(None) => assert!(true),
            _ => panic!(),
        }

        // TEST: unlimited number of item for sale
        // create a new primary sale
        let json_exec_msg = json!({
            "primary_sale": {
                "total_supply": "0",
                "start_time": "1678567587",
                "end_time": "1679567587",
                "price": [{
                    "denom": "USDC",
                    "amount": "10"
                }]
            }
        })
        .to_string();
        env.block.time = Timestamp::from_seconds(1678567587);
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("primary sales added");
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => {
                assert_eq!(sale.total_supply.u64(), 0);
                assert!(!sale.disabled);
            }
            _ => panic!(),
        }
        let mint_msg = MintMsg::<Empty> {
            token_id: "unlimited_buy".to_string(),
            owner: CREATOR.to_string(),
            token_uri: Some("url".to_string()),
            extension: Empty {},
        };

        let execute_msg = ExecuteMsg::BuyItem(mint_msg.clone());
        let buyer_info = Rc::new(RefCell::new(mock_info(USER, &[Coin::new(20, "USDC")])));
        let res = sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                buyer_info.borrow_mut().clone(),
                execute_msg,
            )
            .expect("item bought");
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => {
                assert_eq!(sale.tokens_minted.u64(), 1);
                assert!(!sale.disabled);
            }
            _ => panic!(),
        }
        // make sure sale buy event is emitted
        let events = res.response.events;
        // there should be 3 events: sales-token_minted, sales-funds_sent, sales-refund_sent
        // the last event sales-sale_ended is not emitted because the sale is not ended
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].ty, "sales-token_minted");
        assert_eq!(
            events[0].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("by", env.contract.address.to_string()),
                Attribute::new("for", buyer_info.borrow().sender.to_string()),
                Attribute::new("token_metadata", serde_json::to_string(&mint_msg).unwrap()),
            ]
        );
        assert_eq!(events[1].ty, "sales-funds_sent");
        assert_eq!(
            events[1].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("to", CREATOR.to_string()),
                Attribute::new("amount", "10"),
                Attribute::new("denom", "USDC"),
            ]
        );
        assert_eq!(events[2].ty, "sales-refund_sent");
        assert_eq!(
            events[2].attributes,
            vec![
                Attribute::new("contract_address", env.contract.address.to_string()),
                Attribute::new("to", buyer_info.borrow().sender.to_string()),
                Attribute::new("amount", "10"),
                Attribute::new("denom", "USDC"),
            ]
        );
    }
}
