use once_cell::sync::Lazy;
use reqwest::Response;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero_2_prod::configuration;
use zero_2_prod::configuration::DatabaseSettings;
use zero_2_prod::telemetry;
use zero_2_prod::{startup, startup::Application};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    let mut writer = std::io::sink;
    // 这里先注释掉，会报类型不匹配
    /*if std::env::var("TEST_LOG").is_ok() {
        writer = std::io::stdout;
    }*/
    let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level, writer);
    telemetry::init_subscriber(subscriber);
});

pub struct TestApp {
    pub address: String,
    pub connect_pool: PgPool,
}

impl TestApp {
    /// Spin up an instance of our application
    /// and returns its address (i.e. http://localhost:XXXX)
    /// Launch our application in the background ~somehow~
    /// No .await call, therefore no need for `spawn_app` to be async now.
    /// We are also running tests, so it is not worth it to propagate errors:
    /// if we fail to perform the required setup we can just panic and crash
    /// all the things.
    pub async fn spawn_app() -> Self {
        Lazy::force(&TRACING);

        // Randomise configuration to ensure test isolation
        let configuration = {
            let mut config =
                configuration::get_configuration().expect("Failed to read configuration");
            // Use a different database for each test case
            config.database.database_name = Uuid::new_v4().to_string();
            // Use a random OS port
            config.application.port = 0;
            config
        };

        // Create and migrate the database
        configure_database(&configuration.database).await;

        // Notice the .clone!
        let application = Application::build(configuration.clone())
            .await
            .expect("Failed to build application.");

        // Get the port before spawning the application
        let address = format!("http://127.0.0.1:{}", application.port());

        // Launch the server as a background task
        // tokio::spawn returns a handle to the spawned future,
        // but we have no use for it here, hence the non-binding let
        let _ = tokio::spawn(application.run_until_stopped());

        // We return the application address to the caller!
        TestApp {
            address,
            connect_pool: startup::get_connection_pool(&configuration.database),
        }
    }

    pub async fn post_subscriptions(&self, body: String) -> Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_health_check(&self) -> Response {
        reqwest::Client::new()
            .get(format!("{}/health_check", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// create a database then migrate a table
async fn configure_database(config: &DatabaseSettings) {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let pg_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to migrate the database");
}
