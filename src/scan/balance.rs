use crate::err_custom_create;
use crate::error::WebPortalError;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;
use web3::types::{Address, BlockNumber, U256};

pub async fn cached_get_balance(
    web3: web3::Web3<web3::transports::Http>,
    address: Address,
    block_num: u64,
) -> Result<U256, WebPortalError> {
    lazy_static! {
        static ref CACHE: Mutex<HashMap<String, U256>> = Mutex::new(HashMap::new());
    };

    let key = format!("{:#x}_{}", address, &block_num);
    let balance_from_cache = {
        let cache = CACHE.lock().unwrap();
        cache.get(&key).cloned()
    };

    if let Some(balance) = balance_from_cache {
        Ok(balance)
    } else {
        let balance = web3
            .eth()
            .balance(address, Some(BlockNumber::Number((block_num).into())))
            .await
            .map_err(|e| err_custom_create!("Error getting balance prev block: {}", e))?;
        let mut cache = CACHE.lock().unwrap();
        cache.insert(key, balance);
        Ok(balance)
    }
}
