use cosmwasm_std::{Addr, CustomQuery, Empty, QuerierWrapper, Storage};

/// A helper to read an external contract's storage.
/// This is useful if you want to avoid the gas cost of loading the contract and running a smart query
/// for a simple storage read.
///
/// # Caveats
/// This is not a full storage implementation and cannot be used for range queries or mutations.
/// It will panic if you try. It is only for reading a single key at a time from another contract's storage.
///
/// Using this means the contract you are reading from can break your contract by changing the storage layout.
pub struct ExternalStorage<'a, C: CustomQuery = Empty> {
    contract_addr: Addr,
    querier: QuerierWrapper<'a, C>,
}

impl<'a, C: CustomQuery> ExternalStorage<'a, C> {
    pub fn new(contract_addr: Addr, querier: QuerierWrapper<'a, C>) -> Self {
        Self {
            contract_addr,
            querier,
        }
    }
}

impl<C: CustomQuery> Storage for ExternalStorage<'_, C> {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.querier
            .query_wasm_raw(&self.contract_addr, key)
            .expect("external storage query failed")
    }

    // TODO: feature-gate? #[cfg(feature = "iterator")]
    fn range<'a>(
        &'a self,
        _start: Option<&[u8]>,
        _end: Option<&[u8]>,
        _order: cosmwasm_std::Order,
    ) -> Box<dyn Iterator<Item = cosmwasm_std::Record> + 'a> {
        panic!("cannot range over external storage");
    }

    fn set(&mut self, _key: &[u8], _value: &[u8]) {
        panic!("cannot mutate external storage");
    }

    fn remove(&mut self, _key: &[u8]) {
        panic!("cannot mutate external storage");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::{testing::mock_dependencies, StdResult};
    use cw_multi_test::{App, Executor};

    mod storage_contract {
        use super::*;

        use cosmwasm_schema::cw_serde;
        use cosmwasm_std::{
            to_json_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response,
            StdResult,
        };
        use cw_multi_test::{Contract, ContractWrapper};
        use cw_storage_plus::{Item, Map};

        const MAP: Map<&str, bool> = Map::new("map");
        const STORAGE_CONTRACT: Item<Addr> = Item::new("storage_contract");

        #[cw_serde]
        pub struct InstantiateMsg {
            /// The address of the contract that will store the data
            /// This is only set for one of the instances of this contract
            /// The other one will only write to its own storage
            pub storage_contract: Option<String>,
        }

        fn instantiate(
            deps: DepsMut,
            _: Env,
            _: MessageInfo,
            msg: InstantiateMsg,
        ) -> StdResult<Response> {
            if let Some(contract) = msg.storage_contract {
                STORAGE_CONTRACT.save(deps.storage, &deps.api.addr_validate(&contract)?)?;
            }
            Ok(Response::new())
        }

        fn execute(deps: DepsMut, _: Env, _: MessageInfo, _: Empty) -> StdResult<Response> {
            // We'll call this in one instance
            MAP.save(deps.storage, "entry", &true)?;
            Ok(Response::new())
        }

        fn query(deps: Deps, _: Env, _: Empty) -> StdResult<Binary> {
            // We'll call this in another instance, getting the value set in execute of the first instance
            let external_storage =
                ExternalStorage::new(STORAGE_CONTRACT.load(deps.storage)?, deps.querier.clone());

            let value = MAP.load(&external_storage, "entry")?;
            to_json_binary(&value)
        }

        pub fn contract() -> Box<dyn Contract<Empty>> {
            Box::new(ContractWrapper::new(execute, instantiate, query))
        }
    }

    #[test]
    fn external_storage_works() {
        let mut app = App::default();
        let user = app.api().addr_make("address");

        let code_id = app.store_code(storage_contract::contract());
        let contract1 = app
            .instantiate_contract(code_id, user.clone(), &Empty {}, &[], "cntrct", None)
            .unwrap();

        let contract2 = app
            .instantiate_contract(
                code_id,
                user.clone(),
                &storage_contract::InstantiateMsg {
                    storage_contract: Some(contract1.to_string()),
                },
                &[],
                "cntrct",
                None,
            )
            .unwrap();

        // querying should fail as we have not set storage in contract1 yet
        let result: StdResult<bool> = app.wrap().query_wasm_smart(contract2.clone(), &Empty {});
        assert!(result.is_err());

        // set storage in contract1
        app.execute_contract(user, contract1, &Empty {}, &[])
            .unwrap();

        // now querying should work
        let result: bool = app.wrap().query_wasm_smart(contract2, &Empty {}).unwrap();
        assert!(result);
    }
}
