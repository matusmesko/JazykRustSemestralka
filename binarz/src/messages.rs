use serde::{Deserialize, Serialize};
use macros::{query, Entity};


#[Entity("messages")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageEntity {
    #[id]
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    #[value("TEXT")]
    pub content: String,
    pub sent_time: i64,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct sendMessageDTO {
    pub content: String,
}

pub struct MessageRepository;

impl MessageRepository {
    #[query("SELECT * FROM messages WHERE (sender_id = ? AND receiver_id = ?) OR (sender_id = ? AND receiver_id = ?) ORDER BY sent_time ASC")]
    pub async fn get_chat_history(pool: &::sqlx::MySqlPool, my_id: i64, target_id: i64, target_id_copy: i64, my_id_copy: i64) -> anyhow::Result<Vec<MessageEntity>> {}

    #[query("SELECT * FROM messages WHERE id = ? LIMIT 1")]
    pub async fn find_by_id(pool: &::sqlx::MySqlPool, id: i64) -> anyhow::Result<Option<MessageEntity>> {}

    #[query("DELETE FROM messages WHERE id = ?")]
    pub async fn delete_by_id(pool: &::sqlx::MySqlPool, id: i64) -> anyhow::Result<()> {}
}