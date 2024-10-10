use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanDbObj {
    pub address: String,
    pub first_block_number: i64,
    pub first_block_timestamp: chrono::DateTime<chrono::Utc>,
    pub next_block_number: i64,
    pub next_block_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockDbObj {
    pub address: String,
    pub block_number: i64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: String,
    pub balance_diff: String,
    pub updated: chrono::DateTime<chrono::Utc>,
    pub block_miner: String,
    pub consensus_reward: String,
    pub mev_reward: String,
    pub block_reward: String,
    pub amount_incoming: String,
    pub amount_outgoing: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxDbObj {
    pub address: String,
    pub tx_hash: String,
    pub block_number: i64,
    pub block_index: i64,
    pub gas_used: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TxTraceDbObj {
    pub address: String,
    pub tx_hash: String,
    pub block_number: i64,
    pub block_index: i64,
    pub trace_index: i64,
    pub from_addr: String,
    pub to_addr: String,
    pub value: String,
    pub gas_used: String,
}
