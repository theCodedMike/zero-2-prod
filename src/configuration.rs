use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

const LOCAL_ENVIRONMENT: &'static str = "local";
const PRODUCTION_ENVIRONMENT: &'static str = "prod";

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("configuration");
    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| LOCAL_ENVIRONMENT.into())
        .try_into()
        .expect("Failed to parse APP_ENV");
    let environment_filename = format!("{}.yaml", environment.as_str());
    // Initialise our configuration reader
    let settings = Config::builder()
        .add_source(File::from(config_dir.join("base.yaml")))
        .add_source(File::from(config_dir.join(environment_filename)))
        .build()?;
    // Try to convert the configuration values it read into our Settings type
    settings.try_deserialize()
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
