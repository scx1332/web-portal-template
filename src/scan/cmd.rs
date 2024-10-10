use crate::db::ops::transaction::delete_scan;
use crate::err_custom_create;
use crate::error::WebPortalError;
use crate::scan::run::scan_address;
use clap::Parser;
use sqlx::SqlitePool;
use web3::types::Address;

#[derive(Debug, Clone, Parser)]
pub struct ScanCommand {
    #[arg(long)]
    address: Address,
    #[arg(long)]
    block_start: u64,
    #[arg(long)]
    block_end: Option<u64>,
    #[arg(long)]
    remove_prev_scan: bool,
}

pub async fn scan_command(
    conn: SqlitePool,
    scan_command: ScanCommand,
) -> Result<(), WebPortalError> {
    let ScanCommand {
        address,
        block_start,
        block_end,
        remove_prev_scan,
    } = scan_command;

    if remove_prev_scan {
        log::warn!("Deleting scan for address: {}", address);

        delete_scan(&conn, &format!("{address:#x}"))
            .await
            .map_err(|e| {
                log::error!("Error deleting previous scan: {e}");
                err_custom_create!("Error: {e}")
            })?;
    }

    scan_address(conn.clone(), address, block_start, block_end).await?;

    Ok(())
}
