#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use burnt_glue::module::Module;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, Timestamp,
    };
    use cw_storage_plus::{Item, Map};
    use ownable::Ownable;
    use sellable::Sellable;
    use token::Tokens;

    use crate::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
        Sales,
    };
    use cw721_base::msg::InstantiateMsg as cw721_baseInstantiateMsg;
    use serde_json::{from_str, json};

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";

    #[test]
    fn add_primary_sales() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info(CREATOR, &[]);

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
        sales
            .instantiate(&mut deps.as_mut(), &env, &info, InstantiateMsg {})
            .expect("sale module instantiated");
        // get all primary sales
        let query_msg = QueryMsg::PrimarySales {};
        let primary_sales = sales
            .query(&deps.as_ref(), env.clone(), query_msg.clone())
            .unwrap();
        match primary_sales {
            crate::msg::QueryResp::PrimarySales(primary_sales) => {
                assert_eq!(primary_sales.len(), 0)
            }
            _ => assert!(false),
        }
        // create a primary sale
        let json_exec_msg = json!({
            "primary_sale": {
                "total_supply": "1",
                "start_time": "1674567586",
                "end_time": "1675567587",
                "price": {
                    "denom": "USDC",
                    "amount": "10"
                }
            }
        })
        .to_string();
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        // test creating multiple active primary sales
        let fake_info = mock_info("hacker", &[]);
        sales
            .execute(
                &mut deps.as_mut(),
                env.clone(),
                fake_info.clone(),
                execute_msg.clone(),
            )
            .expect_err("primary sales should not be added");
        // set block time
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("primary sales added");
        let primary_sales = sales
            .query(&deps.as_ref(), env.clone(), query_msg.clone())
            .unwrap();
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match primary_sales {
            crate::msg::QueryResp::PrimarySales(primary_sales) => {
                assert_eq!(primary_sales.len(), 1)
            }
            _ => assert!(false),
        }
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => {
                assert_eq!(sale.start_time.seconds().to_string(), "1674567586")
            }
            _ => assert!(false),
        }
        // buy an item
        let json_exec_msg = json!({
            "buy_item": {
                    "token_id": "1",
                    "owner": CREATOR,
                    "token_uri": "url",
                    "extension": {}
                }
        })
        .to_string();
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("item bought");
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => assert_eq!(sale.disabled, true),
            _ => assert!(false),
        }

        // create a new primary sale
        let json_exec_msg = json!({
            "primary_sale": {
                "total_supply": "1",
                "start_time": "1676567587",
                "end_time": "1677567587",
                "price": {
                    "denom": "USDC",
                    "amount": "10"
                }
            }
        })
        .to_string();
        env.block.time = Timestamp::from_seconds(1676567587);
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("primary sales added");
        // halt ongoing primary sale
        let json_exec_msg = json!({
            "halt_sale": { }
        })
        .to_string();
        let execute_msg: ExecuteMsg<Empty> = from_str(&json_exec_msg).unwrap();
        sales
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), execute_msg)
            .expect("any ongoing sale halted");
        let active_primary_sale = sales
            .query(&deps.as_ref(), env.clone(), QueryMsg::ActivePrimarySale {})
            .unwrap();
        match active_primary_sale {
            crate::msg::QueryResp::ActivePrimarySale(Some(sale)) => assert_eq!(sale.disabled, true),
            _ => assert!(false),
        }
    }
}
