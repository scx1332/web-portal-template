use crate::db::model::transaction::{BlockDbObj, ScanDbObj, TxDbObj, TxTraceDbObj};
use sqlx::SqlitePool;

pub async fn delete_block_tx(
    conn: &SqlitePool,
    address: String,
    block_number: i64,
) -> Result<(), sqlx::Error> {
    let _res = sqlx::query(r"DELETE FROM block WHERE block_number = $1 and address = $2;")
        .bind(block_number)
        .bind(address)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn get_all_scans(conn: &SqlitePool) -> Result<Vec<ScanDbObj>, sqlx::Error> {
    let res = sqlx::query_as::<_, ScanDbObj>(r"SELECT * FROM scan;")
        .fetch_all(conn)
        .await?;
    Ok(res)
}

pub async fn get_scan(conn: &SqlitePool, address: &str) -> Result<Option<ScanDbObj>, sqlx::Error> {
    let res = sqlx::query_as::<_, ScanDbObj>(r"SELECT * FROM scan WHERE address = $1;")
        .bind(address)
        .fetch_optional(conn)
        .await?;
    Ok(res)
}

pub async fn insert_scan(conn: &SqlitePool, scan: &ScanDbObj) -> Result<ScanDbObj, sqlx::Error> {
    let res = sqlx::query_as::<_, ScanDbObj>(
        r"INSERT INTO scan
    (address, first_block_number, first_block_timestamp, next_block_number, next_block_timestamp)
    VALUES ($1, $2, $3, $4, $5) RETURNING *;
    ",
    )
    .bind(&scan.address)
    .bind(scan.first_block_number)
    .bind(scan.first_block_timestamp)
    .bind(scan.next_block_number)
    .bind(scan.next_block_timestamp)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn delete_scan(conn: &SqlitePool, address: &str) -> Result<(), sqlx::Error> {
    let _res = sqlx::query(r"DELETE FROM scan WHERE address = $1;")
        .bind(address)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn update_scan(conn: &SqlitePool, scan: &ScanDbObj) -> Result<ScanDbObj, sqlx::Error> {
    let res = sqlx::query_as::<_, ScanDbObj>(
        r"UPDATE scan
    SET
    first_block_number = $1,
    first_block_timestamp = $2,
    next_block_number = $3,
    next_block_timestamp = $4
    WHERE address = $5 RETURNING *;
    ",
    )
    .bind(scan.first_block_number)
    .bind(scan.first_block_timestamp)
    .bind(scan.next_block_number)
    .bind(scan.next_block_timestamp)
    .bind(&scan.address)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn get_blocks(conn: &SqlitePool, address: &str) -> Result<Vec<BlockDbObj>, sqlx::Error> {
    let res = sqlx::query_as::<_, BlockDbObj>(r"SELECT * FROM block WHERE address = $1;")
        .bind(address)
        .fetch_all(conn)
        .await?;
    Ok(res)
}

pub async fn insert_block(
    conn: &SqlitePool,
    block: &BlockDbObj,
) -> Result<BlockDbObj, sqlx::Error> {
    let res = sqlx::query_as::<_, BlockDbObj>(
        r"INSERT INTO block
    (address, block_number, timestamp, balance, balance_diff, updated, block_miner, consensus_reward, mev_reward, block_reward, amount_incoming, amount_outgoing)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *;
    ",
    )
        .bind(&block.address)
    .bind(block.block_number)
    .bind(block.timestamp)
    .bind(&block.balance)
    .bind(&block.balance_diff)
    .bind(block.updated)
    .bind(&block.block_miner)
    .bind(&block.consensus_reward)
    .bind(&block.mev_reward)
    .bind(&block.block_reward)
    .bind(&block.amount_incoming)
    .bind(&block.amount_outgoing)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn insert_tx(conn: &SqlitePool, tx: &TxDbObj) -> Result<TxDbObj, sqlx::Error> {
    let res = sqlx::query_as::<_, TxDbObj>(
        r"INSERT INTO tx
(address, tx_hash, block_number, block_index, gas_used)
VALUES ($1, $2, $3, $4, $5) RETURNING *;
",
    )
    .bind(&tx.address)
    .bind(&tx.tx_hash)
    .bind(tx.block_number)
    .bind(tx.block_index)
    .bind(&tx.gas_used)
    .fetch_one(conn)
    .await?;
    Ok(res)
}

pub async fn insert_tx_trace(
    conn: &SqlitePool,
    trace: &TxTraceDbObj,
) -> Result<TxTraceDbObj, sqlx::Error> {
    let res = sqlx::query_as::<_, TxTraceDbObj>(
        r"INSERT INTO tx_trace
(address, tx_hash, block_number, block_index, trace_index, from_addr, to_addr, value, gas_used)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *;
",
    )
    .bind(&trace.address)
    .bind(&trace.tx_hash)
    .bind(trace.block_number)
    .bind(trace.block_index)
    .bind(trace.trace_index)
    .bind(&trace.from_addr)
    .bind(&trace.to_addr)
    .bind(&trace.value)
    .bind(&trace.gas_used)
    .fetch_one(conn)
    .await?;
    Ok(res)
}
