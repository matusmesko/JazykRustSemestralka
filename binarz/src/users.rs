use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use macros::{query, Entity};

#[Entity("users")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, PartialEq, Eq)]
pub struct UserEntity {
    #[id]
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserLoginDTO {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterDTO {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct pubUserDTO {
    pub id: i64,
    pub username: String,
}


pub struct UserRepository;

impl UserRepository {
    #[query("SELECT * FROM users WHERE username = ?")]
    pub async fn find_by_username(pool: &MySqlPool, username: String) -> anyhow::Result<Option<UserEntity>> {}

    #[query("SELECT * FROM users WHERE id = ?")]
    pub async fn find_by_id(pool: &MySqlPool, id: i64) -> anyhow::Result<Option<UserEntity>> {}

    pub async fn update_blocked_users(pool: &MySqlPool, user_id: i64, blocked_users: Vec<UserEntity>) -> anyhow::Result<()> {
        let blocked_ids: Vec<String> = blocked_users.iter().map(|u| u.id.to_string()).collect();
        let blocked_str = blocked_ids.join(",");
        sqlx::query("UPDATE users SET blocked_users = ? WHERE id = ?")
            .bind(blocked_str)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[query("SELECT id, username FROM users WHERE id != ?")]
    pub async fn find_all_except_me(pool: &MySqlPool, my_id: i64) -> anyhow::Result<Vec<pubUserDTO>> {}

}



