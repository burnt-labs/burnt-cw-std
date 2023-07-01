use cosmwasm_std::{CustomMsg, Deps, Env, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use crate::{msg::QueryResp, Sales};

impl<'a, T, C, E, Q> Sales<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
    C: CustomMsg,
{
    pub fn primary_sales(&self, deps: &Deps) -> StdResult<QueryResp> {
        let primary_sales = self.primary_sales.load(deps.storage)?;
        Ok(QueryResp::PrimarySales(primary_sales))
    }

    pub fn active_primary_sales(&self, deps: &Deps, env: Env) -> StdResult<QueryResp> {
        let primary_sales = self.primary_sales.load(deps.storage)?;
        for sale in primary_sales {
            if sale.disabled {
                continue
            }
            if (sale.start_time.le(&env.block.time)) && sale.end_time.gt(&env.block.time)
            {
                return Ok(QueryResp::ActivePrimarySale(Some(sale)));
            }
        }
        Ok(QueryResp::ActivePrimarySale(None))
    }
}
