use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::MySqlPool;
use crate::session::{SessionEntity, SessionRepository};
use crate::users::{UserEntity, UserRepository};

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    verify(password, hashed).unwrap_or(false)
}

pub fn is_logged_in(req: actix_web::HttpRequest) -> bool {
    if let Some(cookie) = req.cookie("user_cookies") {
        return true
    }
    return false
}

pub fn get_logged_in_user_token(req: actix_web::HttpRequest) -> Option<String> {
    if let Some(cookie) = req.cookie("user_cookies") {
        return Some(cookie.value().to_string())
    }
    return None
}

pub async fn get_logged_user(pool: &MySqlPool, req: actix_web::HttpRequest) -> Option<UserEntity> {
    let token = get_logged_in_user_token(req);
    if token.is_none() {
        return None
    }

    let session: SessionEntity = SessionRepository::find_by_token(pool, token?).await.unwrap()?;
    let user: UserEntity = UserRepository::find_by_id(pool, session.user_id).await.unwrap()?;
    Some(user)
}