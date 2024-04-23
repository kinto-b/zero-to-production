use config::Config;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(log::LevelFilter::Trace)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment: String = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());

    let settings = Config::builder()
        .add_source(config::File::with_name("configuration"))
        .add_source(config::File::with_name(&format!(
            "configuration-{}",
            environment
        )))
        .add_source(
            config::Environment::with_prefix("app")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize()
}
