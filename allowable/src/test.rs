#[cfg(test)]
mod tests {
    use burnt_glue::module::Module;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, DepsMut, Env, MessageInfo,
    };

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryResp};
    use crate::Allowable;

    const CREATOR: &str = "cosmos188rjfzzrdxlus60zgnrvs4rg0l73hct3azv93z";

    fn setup_allowable(deps: &mut DepsMut, _env: &Env, _info: &MessageInfo) -> Allowable<'static> {
        let allowable = Allowable::default();
        allowable
            .ownable
            .borrow_mut()
            .owner
            .save(deps.storage, &Addr::unchecked(CREATOR))
            .unwrap();

        allowable
    }

    #[test]
    fn enabled_flag() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(CREATOR, &[]);

        let mut allowable = setup_allowable(&mut deps.as_mut(), &env, &info);

        let instantiate_msg = InstantiateMsg {
            enabled: true,
            allowed_addrs: vec![],
        };
        // Instantiate the contract
        let result = allowable.instantiate(&mut deps.as_mut(), &env, &info, instantiate_msg);
        assert!(result.is_ok());

        // Check we are enabled by default
        let msg = QueryMsg::IsEnabled {};
        let enabled = allowable.query(&deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(enabled, QueryResp::IsEnabled { is_enabled: true });

        let msg = ExecuteMsg::SetEnabled { enabled: false };
        allowable
            .execute(&mut deps.as_mut(), env.clone(), info.clone(), msg)
            .unwrap();

        let msg = QueryMsg::IsEnabled {};
        let enabled = allowable.query(&deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(enabled, QueryResp::IsEnabled { is_enabled: false });
    }

    #[test]
    fn is_allowed() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let addrs = vec![
            Addr::unchecked("SomeAddress"),
            Addr::unchecked("SomeOtherAddress"),
        ];

        let info = mock_info("owner", &[]);
        let instantiate_msg = InstantiateMsg {
            enabled: true,
            allowed_addrs: vec![],
        };

        let mut allowable = Allowable::default();
        allowable
            .ownable
            .borrow_mut()
            .owner
            .save(deps.as_mut().storage, &Addr::unchecked(CREATOR))
            .unwrap();

        // Instantiate the contract
        let result = allowable.instantiate(&mut deps.as_mut(), &env, &info, instantiate_msg);
        assert!(result.is_ok());

        let msg = QueryMsg::IsAllowed {
            address: addrs[0].clone(),
        };
        let _info = mock_info(CREATOR, &[]);
        let allowed = allowable.query(&deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(allowed, QueryResp::IsAllowed { is_allowed: false });

        let info = mock_info(CREATOR, &[]);
        let msg = ExecuteMsg::AddAllowedAddrs {
            addresses: addrs.clone(),
        };
        allowable
            .execute(&mut deps.as_mut(), env.clone(), info, msg)
            .unwrap();

        let msg = QueryMsg::IsAllowed {
            address: addrs[0].clone(),
        };
        let _info = mock_info(CREATOR, &[]);
        let allowed = allowable.query(&deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(allowed, QueryResp::IsAllowed { is_allowed: true });

        let msg = QueryMsg::IsAllowed {
            address: addrs[1].clone(),
        };
        let info = mock_info(CREATOR, &[]);
        let allowed = allowable.query(&deps.as_ref(), env.clone(), msg).unwrap();
        assert_eq!(allowed, QueryResp::IsAllowed { is_allowed: true });

        let msg = ExecuteMsg::RemoveAllowedAddrs {
            addresses: vec![addrs[0].clone()],
        };
        allowable
            .execute(&mut deps.as_mut(), env.clone(), info, msg)
            .unwrap();

        let allowed = allowable
            .is_allowed(&deps.as_ref(), addrs[0].clone())
            .unwrap();
        assert_eq!(allowed, false);

        // Clear all addresses
        allowable.clear_addrs(&mut deps.as_mut()).unwrap();
        let allowed = allowable
            .is_allowed(&deps.as_ref(), addrs[1].clone())
            .unwrap();
        assert_eq!(allowed, false);

        // If disabled, everyone is allowed.
        allowable.set_enabled(&mut deps.as_mut(), false).unwrap();
        let allowed = allowable
            .is_allowed(&deps.as_ref(), addrs[0].clone())
            .unwrap();
        assert_eq!(allowed, true);
    }

    // Test execute method of allowable contract
    #[test]
    fn execute_() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let addrs = vec![
            Addr::unchecked("SomeAddress"),
            Addr::unchecked("SomeOtherAddress"),
        ];

        let info = mock_info("owner", &[]);
        let instantiate_msg = InstantiateMsg {
            enabled: true,
            allowed_addrs: vec![],
        };

        let mut allowable = Allowable::default();
        allowable
            .ownable
            .borrow_mut()
            .owner
            .save(deps.as_mut().storage, &Addr::unchecked(CREATOR))
            .unwrap();

        // Instantiate the contract
        let result = allowable.instantiate(&mut deps.as_mut(), &env, &info, instantiate_msg);
        assert!(result.is_ok());

        let msg = ExecuteMsg::AddAllowedAddrs {
            addresses: Vec::new(),
        };

        // Execute a message that is not allowed
        // AddAllowedAddrs
        let info = mock_info(CREATOR, &[]);
        allowable
            .execute(&mut deps.as_mut(), env.clone(), info, msg)
            .unwrap();

        let result = allowable
            .query(
                &deps.as_ref(),
                env,
                QueryMsg::IsAllowed {
                    address: addrs[0].clone(),
                },
            )
            .unwrap();

        assert_eq!(result, QueryResp::IsAllowed { is_allowed: false });
    }
}
