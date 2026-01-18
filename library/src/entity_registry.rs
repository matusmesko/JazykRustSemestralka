
use std::future::Future;
use std::pin::Pin;

use anyhow::Result;
use sqlx::MySqlPool;
use crate::logger;
use crate::logger::LogLevel;

pub struct EntityRegistry {
    pub name: &'static str,
    pub run: for<'a> fn(&'a MySqlPool) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>,
}

inventory::collect!(EntityRegistry);


pub async fn run_all(pool: &sqlx::MySqlPool) -> anyhow::Result<()> {
    let entities = inventory::iter::<EntityRegistry>;
    let count = entities.into_iter().count();
    logger!(LogLevel::Info, "Found {} entities to register", count);

    for entity in inventory::iter::<EntityRegistry> {
        logger!(LogLevel::Info, "Registering entity: [{}]", entity.name);
        (entity.run)(pool).await?;
    }

    logger!(LogLevel::Info, "All entities registered successfully.");
    Ok(())
}