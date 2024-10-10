use crate::db::model::transaction::ScanDbObj;
use crate::db::ops::transaction::{get_scan, insert_scan, update_scan};
use crate::err_custom_create;
use crate::error::WebPortalError;
use crate::scan::balance::cached_get_balance;
use crate::scan::block::inspect_block;
use sqlx::SqlitePool;
use std::env;
use web3::types::{Address, BlockId, BlockNumber};

pub async fn scan_address(
    db: SqlitePool,
    address: Address,
    block_start: u64,
    block_end: Option<u64>,
) -> Result<(), WebPortalError> {
    let rpc_endpoint =
        env::var("SCANNER_RPC_FULL_NODE").unwrap_or_else(|_| "http://localhost:8545".to_string());

    let web3 = web3::Web3::new(web3::transports::Http::new(&rpc_endpoint).unwrap());

    let current_block_number = web3
        .eth()
        .block_number()
        .await
        .map_err(|e| err_custom_create!("Error getting current block number: {}", e))?;

    let block_end = if let Some(block_end) = block_end {
        if block_end > current_block_number.as_u64() - 100 {
            return Err(err_custom_create!(
                "Block end is too close to the current block number"
            ));
        }
        block_end
    } else {
        (current_block_number - 100).as_u64()
    };

    let existing_scan = get_scan(&db, format!("{:#x}", address).as_str())
        .await
        .map_err(|e| err_custom_create!("Error getting scan: {}", e))?;

    let existing_scan = if let Some(existing_scan) = existing_scan {
        existing_scan
    } else {
        let block_info = web3
            .eth()
            .block(BlockId::Number(BlockNumber::Number(block_start.into())))
            .await
            .map_err(|e| err_custom_create!("Error getting block: {} {}", block_start, e))?
            .ok_or(err_custom_create!("Block info not found {}", block_start))?;

        let new_scan = ScanDbObj {
            address: format!("{:#x}", address),
            first_block_number: block_info.number.unwrap().as_u64() as i64,
            first_block_timestamp: chrono::DateTime::from_timestamp(
                block_info.timestamp.as_u64() as i64,
                0,
            )
            .unwrap(),
            next_block_number: block_info.number.unwrap().as_u64() as i64,
            next_block_timestamp: chrono::DateTime::from_timestamp(
                block_info.timestamp.as_u64() as i64,
                0,
            )
            .unwrap(),
        };
        insert_scan(&db, &new_scan)
            .await
            .map_err(|e| err_custom_create!("Error inserting scan: {}", e))?;
        get_scan(&db, format!("{:#x}", address).as_str())
            .await
            .map_err(|e| err_custom_create!("Error getting scan: {}", e))?
            .ok_or(err_custom_create!("Scan should be found now"))?
    };

    let block_start = existing_scan.next_block_number as u64;

    let number_of_blocks = block_end - block_start;

    if number_of_blocks < 1 {
        log::info!("No blocks to scan");
        return Ok(());
    }

    let mut prev_checked_block = None;
    let mut block_num = block_start;
    let mut current_scan = existing_scan;
    loop {
        if block_num >= block_end {
            break;
        }
        if prev_checked_block.is_some() {
            let prev_balance =
                cached_get_balance(web3.clone(), address, prev_checked_block.unwrap())
                    .await
                    .map_err(|e| err_custom_create!("Error getting balance: {}", e))?;
            let future_50_blocks_balance =
                cached_get_balance(web3.clone(), address, block_num + 50)
                    .await
                    .map_err(|e| err_custom_create!("Error getting balance: {}", e))?;
            if future_50_blocks_balance == prev_balance {
                log::info!("Balance did not change in 50 blocks");
                block_num += 49;
                prev_checked_block = Some(block_num);
                continue;
            }
        }
        match inspect_block(web3.clone(), db.clone(), address, block_num).await {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Error inspecting block: {}", e);
            }
        }
        prev_checked_block = Some(block_num);
        block_num += 1;
        current_scan.next_block_number = block_num as i64;
        let block_info = web3
            .eth()
            .block(BlockId::Number(BlockNumber::Number(block_num.into())))
            .await
            .map_err(|e| err_custom_create!("Error getting block: {} {}", block_start, e))?
            .ok_or(err_custom_create!("Block info not found {}", block_start))?;

        current_scan.next_block_timestamp =
            chrono::DateTime::from_timestamp(block_info.timestamp.as_u64() as i64, 0).unwrap();
        update_scan(&db, &current_scan)
            .await
            .map_err(|e| err_custom_create!("Error updating scan: {}", e))?;
    }

    log::info!("Finished");

    Ok(())
}
