use sqlx::types::chrono::Utc;
use crate::blocked::BlockedRepository;
pub fn get_current_timestamp() -> i64 {
    let now = Utc::now();
    let timestamp_secs = now.timestamp();
    timestamp_secs
}

pub async fn is_user_blocked(pool: &sqlx::MySqlPool, my_id: i64, target_id: i64) -> bool {
    match BlockedRepository::check_existence(&pool, my_id, target_id).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(_) => false,
    }
}