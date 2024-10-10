pub mod transaction;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserDbObj {
    pub uid: String,
    pub email: String,
    #[serde(skip)]
    pub pass_hash: String,
    pub created_date: DateTime<Utc>,
    pub last_pass_change: DateTime<Utc>,

    #[serde(skip)]
    pub set_pass_token: Option<String>,
    #[serde(skip)]
    pub set_pass_token_date: Option<DateTime<Utc>>,
}
