use crate::db::model::transaction::{BlockDbObj, TxDbObj, TxTraceDbObj};
use crate::db::ops::transaction::{delete_block_tx, insert_block, insert_tx, insert_tx_trace};
use crate::err_custom_create;
use crate::error::WebPortalError;
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::scan::balance::cached_get_balance;
use std::str::FromStr;
use web3::types::{Action, Address, BlockId, BlockNumber, U256};

pub async fn inspect_block(
    web3: web3::Web3<web3::transports::Http>,
    db: SqlitePool,
    address: Address,
    block_num: u64,
) -> Result<(), WebPortalError> {
    log::info!("This block: {}", block_num);
    let balance_prev = cached_get_balance(web3.clone(), address, block_num - 1)
        .await
        .map_err(|e| err_custom_create!("Error getting balance prev block: {}", e))?;
    let balance_curr = cached_get_balance(web3.clone(), address, block_num)
        .await
        .map_err(|e| err_custom_create!("Error getting balance prev block: {}", e))?;

    let balance_diff = balance_curr.as_u128() as i128 - balance_prev.as_u128() as i128;
    if balance_diff == 0 {
        log::info!("Balance Diff is 0 for block {}", block_num);
        return Ok(());
    }
    log::info!("Balance Diff: {}", balance_diff);
    let block = web3
        .eth()
        .block_with_txs(BlockId::Number(BlockNumber::Number(block_num.into())))
        .await
        .map_err(|e| err_custom_create!("Error getting block: {} {}", block_num, e))?
        .ok_or(err_custom_create!("Block info not found {}", block_num))?;

    // "address": String("0x03e543052f41799de45d97f801f61688240ae7c1"),
    // "amount": String("0x12475fe"),
    // "index": String("0x39d661b"),
    // "validatorIndex": String("0x150cda")

    let mut amount_withdrawn = 0_i128;
    let withdrawals = block.withdrawals.unwrap().as_array().unwrap().clone();

    for withdrawal in withdrawals {
        let withdrawal_obj = withdrawal.as_object().unwrap();
        let amount = withdrawal_obj.get("amount").unwrap().as_str().unwrap();
        let withdrawal_address = withdrawal_obj.get("address").unwrap().as_str().unwrap();
        let _index = withdrawal_obj.get("index").unwrap().as_str().unwrap();
        let _validator_index = withdrawal_obj
            .get("validatorIndex")
            .unwrap()
            .as_str()
            .unwrap();
        if Address::from_str(withdrawal_address).unwrap() == address {
            log::info!("Found withdrawal: {}", withdrawal);
            // amount is given in gwei, so normalize it
            amount_withdrawn += U256::from_str(amount).unwrap().as_u128() as i128 * 1000000000i128;
        }
    }

    let mut from_txs: HashMap<Address, U256> = HashMap::new();
    let mut to_txs: HashMap<Address, U256> = HashMap::new();

    delete_block_tx(&db, format!("{:#x}", address), block_num as i64)
        .await
        .map_err(|e| err_custom_create!("Error deleting block tx: {}", e))?;

    let mut interesting_txs = Vec::new();
    let mut interesting_traces = Vec::new();
    let mut miner_reward = 0i128;

    for (block_index, tx) in block.transactions.into_iter().enumerate() {
        let traces = web3
            .trace()
            .transaction(tx.hash)
            .await
            .map_err(|e| err_custom_create!("Error getting traces: {}", e))?;

        let tx_obj = TxDbObj {
            address: format!("{:#x}", address),
            tx_hash: format!("{:#x}", tx.hash),
            block_number: block_num as i64,
            block_index: block_index as i64,
            gas_used: tx.gas.to_string(),
        };

        let mut tx_interesting = false;
        let mut traces2 = Vec::new();
        for (trace_idx, trace) in traces.into_iter().enumerate() {
            match trace.action {
                Action::Call(call) => {
                    let new_db_part = TxTraceDbObj {
                        address: format!("{:#x}", address),
                        tx_hash: format!("{:#x}", tx.hash),
                        block_number: block_num as i64,
                        block_index: block_index as i64,
                        trace_index: trace_idx as i64,
                        from_addr: format!("{:#x}", call.from),
                        to_addr: format!("{:#x}", call.to),
                        value: call.value.to_string(),
                        gas_used: call.gas.to_string(),
                    };
                    traces2.push(new_db_part);

                    if call.from == address || call.to == address {
                        log::info!("Found transaction: {:?}", tx);
                        tx_interesting = true;
                    }

                    if call.from == block.author && call.to == address {
                        log::info!("Found mev reward: {:?}", tx);
                        miner_reward += call.value.as_u128() as i128;
                    } else if call.to == address {
                        let value = to_txs.entry(call.from).or_insert(U256::from(0));
                        *value += call.value;
                    } else if call.from == address {
                        let value = from_txs.entry(call.to).or_insert(U256::from(0));
                        *value += call.value;
                    }
                }
                Action::Reward(_) => {
                    return Err(err_custom_create!("Reward action found"));
                }
                _ => {
                    log::debug!("Unknown action: {:?}", trace.action);
                    continue;
                }
            }
        }
        if tx_interesting {
            interesting_txs.push(tx_obj.clone());

            for trace in traces2 {
                interesting_traces.push(trace.clone());
            }
        }
    }
    log::debug!("To txs: {:?}", to_txs);
    log::debug!("From txs: {:?}", from_txs);

    let mut sum_to_txs = U256::from(0);
    let mut sum_from_txs = U256::from(0);
    for value in to_txs.values() {
        sum_to_txs += *value;
    }
    for value in from_txs.values() {
        sum_from_txs += *value;
    }
    log::debug!("Sum to tx {}", sum_to_txs);
    log::debug!("Sum from tx {}", sum_from_txs);

    log::info!(
        "Sum diff {}",
        sum_to_txs.as_u128() as i128 - sum_from_txs.as_u128() as i128
    );

    let sum_diff = sum_to_txs.as_u128() as i128 - sum_from_txs.as_u128() as i128;

    if sum_diff < balance_diff {
        log::info!(
            "Sum diff less than balance diff: reward: {}",
            balance_diff - sum_diff
        );
    }

    let mev_reward = miner_reward;
    insert_block(
        &db,
        &BlockDbObj {
            address: format!("{:#x}", address),
            block_number: block_num as i64,
            timestamp: chrono::DateTime::from_timestamp(block.timestamp.as_u64() as i64, 0)
                .unwrap(),
            balance: balance_curr.to_string(),
            balance_diff: balance_diff.to_string(),
            updated: chrono::Utc::now(),
            block_miner: format!("{:#x}", block.author),
            consensus_reward: amount_withdrawn.to_string(),
            mev_reward: mev_reward.to_string(),
            block_reward: U256::zero().to_string(),
            amount_incoming: sum_to_txs.to_string(),
            amount_outgoing: sum_from_txs.to_string(),
        },
    )
    .await
    .map_err(|e| err_custom_create!("Error inserting block: {}", e))?;

    for tx in interesting_txs {
        insert_tx(&db, &tx)
            .await
            .map_err(|e| err_custom_create!("Error inserting tx: {}", e))?;
    }
    for trace in interesting_traces {
        insert_tx_trace(&db, &trace)
            .await
            .map_err(|e| err_custom_create!("Error inserting tx trace: {}", e))?;
    }

    if sum_diff + amount_withdrawn + mev_reward != balance_diff {
        log::error!("Sum diff does not match {} != {}", sum_diff, balance_diff);
        return Err(err_custom_create!("Sum diff does not match"));
    }

    Ok(())
}
