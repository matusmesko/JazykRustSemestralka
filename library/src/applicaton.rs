use std::sync::Once;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Condition};
use actix_web::http::{header, Method};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use crate::config::{CorsSettings, Settings};
use crate::logger;
use crate::logger::LogLevel;
use crate::entity_registry;

pub struct ServerRun;
static CORS_LOG_ONCE: Once = Once::new();
impl ServerRun {
    pub async fn start() -> anyhow::Result<()> {

        let settings = Settings::load().map_err(|e| {
            let msg = format!("Failed to load config: {}", e);
            anyhow::anyhow!(msg)
        })?;

        let pool: MySqlPool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&settings.database_url)
            .await
            .map_err(|e| {
                let msg = format!("Failed to connect to DB: {}", e);
                anyhow::anyhow!(msg)
            })?;

        entity_registry::run_all(&pool).await.map_err(|e| anyhow::anyhow!(e));

        let port = settings.port;
        let app_settings = settings.clone();

        logger!(LogLevel::Info, "Server starting on 127.0.0.1:{}", port);
        HttpServer::new(move || {

            let worker_settings = app_settings.clone();
            let cors_config = worker_settings.cors.unwrap_or_default();
            let is_enabled = cors_config.enabled;
            App::new()
                .app_data(web::Data::new(pool.clone()))

                .wrap(Condition::new(is_enabled, configure_cors(&cors_config)))
                .route("/health", web::get().to(|| async { "OK" }))
                .configure(|cfg| crate::controller_registry::configure_all(cfg))
        })
            .bind(("127.0.0.1", port))?
            .run()
            .await?;
        logger!(LogLevel::Info, "Server is running!");


        Ok(())
    }
}



pub fn configure_cors(settings: &CorsSettings) -> Cors {
    let mut cors = Cors::default();

    if !settings.enabled {
        CORS_LOG_ONCE.call_once(|| {
            logger!(LogLevel::Warn, "Disabling CORS - this is not recommended for production environments!");
        });
        return cors;
    }


    if let Some(origins) = &settings.allowed_origins {
        for origin in origins {
            cors = cors.allowed_origin(origin);
        }
    } else {
        cors = cors.allow_any_origin();
    }


    if let Some(methods) = &settings.allowed_methods {
        let parsed_methods: Vec<Method> = methods
            .iter()
            .map(|m| Method::from_bytes(m.as_bytes()).unwrap())
            .collect();
        cors = cors.allowed_methods(parsed_methods);
    }


    if let Some(headers) = &settings.allowed_headers {
        for h in headers {
            cors = cors.allowed_header(header::HeaderName::from_bytes(h.as_bytes()).unwrap());
        }
    }

    if let Some(true) = settings.allow_credentials {
        cors = cors.supports_credentials();
    }

    if let Some(max_age) = settings.max_age {
        cors = cors.max_age(max_age as usize);
    }
    CORS_LOG_ONCE.call_once(|| {
        logger!(LogLevel::Info, "CORS is enabled and running!");
    });
    cors
}
