use serde::{Deserialize, Serialize};
use macros::{query, Entity};

#[Entity("blocked_users")]
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct BlockedUsers {
    #[id]
    pub id: i64,
    pub user_id: i64,
    pub blocked_user_id: i64,
}

#[derive(Deserialize, Serialize)]
pub struct blogUserDTO {
    pub user_id: i64,
}


pub struct BlockedRepository;

impl BlockedRepository {

    #[query("DELETE FROM blocked_users WHERE user_id = ? AND blocked_user_id = ?")]
    pub async fn unblock_user(pool: &::sqlx::MySqlPool, user_id: i64, blocked_id: i64) -> anyhow::Result<()> {}

    #[query("SELECT * FROM blocked_users WHERE user_id = ? AND blocked_user_id = ?")]
    pub async fn check_existence(pool: &::sqlx::MySqlPool, my_id: i64, target_id: i64) -> anyhow::Result<Option<BlockedUsers>> {}
}