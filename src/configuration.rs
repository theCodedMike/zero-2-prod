use crate::constant::{LOCAL_ENVIRONMENT, PRODUCTION_ENVIRONMENT};
use crate::domain::SubscriberEmail;
use crate::error::BizErrorEnum;
use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use std::time::Duration;
use tracing::log::LevelFilter;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
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
            .host(self.host.as_str())
            .username(self.username.as_str())
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(self.database_name.as_str());
        options.log_statements(LevelFilter::Trace);
        options
    }
}

#[derive(Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, BizErrorEnum> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

pub fn get_configuration() -> Result<Settings, BizErrorEnum> {
    let base_path = std::env::current_dir().map_err(|e| {
        tracing::error!("Failed to get current dir.");
        BizErrorEnum::GetCurrentDirError(e)
    })?;
    let config_dir = base_path.join("configuration");
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| LOCAL_ENVIRONMENT.into())
        .try_into()
        .map_err(|e| {
            tracing::error!("Failed to parse APP_ENVIRONMENT: {:?}", e);
            BizErrorEnum::ParseEnvironmentVariableError(e)
        })?;
    let environment_filename = format!("{}.yaml", environment.as_str());
    // Initialise our configuration reader
    let settings = Config::builder()
        .add_source(File::from(config_dir.join("base.yaml")))
        .add_source(File::from(config_dir.join(environment_filename)))
        .build()
        .map_err(|e| {
            tracing::error!("Failed to build config sources.");
            BizErrorEnum::BuildConfigSourcesError(e)
        })?;
    // Try to convert the configuration values it read into our Settings type
    settings.try_deserialize().map_err(|e| {
        tracing::error!("Failed to deserialize config file.");
        BizErrorEnum::DeserializeConfigurationFileError(e)
    })
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => LOCAL_ENVIRONMENT,
            Environment::Production => PRODUCTION_ENVIRONMENT,
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            LOCAL_ENVIRONMENT => Ok(Self::Local),
            PRODUCTION_ENVIRONMENT => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}
