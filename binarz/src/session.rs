use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use macros::{query, Entity};

#[Entity("sessions")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SessionEntity {
    #[id]
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub created_at: i64,
}

pub struct SessionRepository;

impl SessionRepository {
    #[query("SELECT * FROM sessions WHERE token = ? LIMIT 1")]
    pub async fn find_by_token(pool: &MySqlPool, token: String) -> anyhow::Result<Option<SessionEntity>> {}

    #[query("DELETE FROM sessions WHERE token = ?")]
    pub async fn delete_by_token(pool: &MySqlPool, token: String) -> anyhow::Result<()>  {}
}