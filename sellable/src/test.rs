#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, Uint64,
    };
    use cw721_base::{msg::InstantiateMsg as cw721_baseInstantiateMsg, MintMsg};
    use cw_storage_plus::Map;
    use ownable::Ownable;
    use token::Tokens;

    use crate::Sellable;

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";

    #[test]
    fn sellable_token_list() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(CREATOR, &[]);

        let mut sellable = Sellable::<Empty, Empty, Empty, Empty>::new(
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
        let listings = schemars::Map::from([("1".to_string(), Uint64::new(10))]);
        sellable
            .try_list(&mut deps.as_mut(), env.clone(), info.clone(), listings)
            .unwrap();

        let result = sellable.listed_tokens(&deps.as_ref(), None, None).unwrap();
        assert_eq!(result.tokens.len(), 1);
    }
}
