use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use linkify::{LinkFinder, LinkKind};
use once_cell::sync::Lazy;
use reqwest::{Response, Url};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
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
    telemetry::init_subscriber(subscriber).expect("Failed to set subscriber.");
});

pub struct TestApp {
    pub address: String,
    pub connect_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
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

        // Launch a mock server to stand in for Postmark's API
        let email_server = MockServer::start().await;

        // Randomise configuration to ensure test isolation
        let configuration = {
            let mut config =
                configuration::get_configuration().expect("Failed to read configuration");
            // Use a different database for each test case
            config.database.database_name = Uuid::new_v4().to_string();
            // Use a random OS port
            config.application.port = 0;
            // Use the mock server as email API
            config.email_client.base_url = email_server.uri();
            config
        };

        // Create and migrate the database
        configure_database(&configuration.database).await;

        // Notice the .clone!
        let application = Application::build(configuration.clone())
            .await
            .expect("Failed to build application.");

        // Get the port before spawning the application
        let app_port = application.port();

        // Launch the server as a background task
        // tokio::spawn returns a handle to the spawned future,
        // but we have no use for it here, hence the non-binding let
        let _ = tokio::spawn(application.run_until_stopped());

        let client = reqwest::Client::builder()
            // By default, a Client will automatically handle HTTP redirects
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .build()
            .expect("Failed to build client instance");
        // We return the application address to the caller!
        let test_app = TestApp {
            address: format!("http://127.0.0.1:{}", app_port),
            connect_pool: startup::get_connection_pool(&configuration.database),
            email_server,
            port: app_port,
            test_user: TestUser::new(),
            api_client: client,
        };
        test_app.test_user.store(&test_app.connect_pool).await;
        test_app
    }

    pub async fn post_subscriptions(&self, body: String) -> Response {
        self.api_client
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to post subscriptions.")
    }

    pub async fn get_health_check(&self) -> Response {
        self.api_client
            .get(format!("{}/health_check", &self.address))
            .send()
            .await
            .expect("Failed to get health_check.")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> Response {
        self.api_client
            .post(&format!("{}/newsletter", &self.address))
            // No longer randomly generated on the spot!
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to post newsletter.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to post login.")
    }
    // Our tests will only look at the HTML page, therefore
    // we do not expose the underlying reqwest::Response
    pub async fn get_login_html(&self) -> String {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
            .text()
            .await
            .expect("Failed to get login html.")
    }

    pub async fn get_admin_dashboard_html(&self) -> String {
        self.get_admin_dashboard()
            .await
            .text()
            .await
            .expect("Failed to get admin dashboard.")
    }

    pub async fn get_admin_dashboard(&self) -> Response {
        self.api_client
            .get(&format!("{}/admin/dashboard", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_change_password(&self) -> Response {
        self.api_client
            .get(&format!("{}/admin/password", &self.address))
            .send()
            .await
            .expect("Failed to get change_password.")
    }

    pub async fn get_change_password_html(&self) -> String {
        self.get_change_password()
            .await
            .text()
            .await
            .expect("Failed to get change password html.")
    }

    pub async fn post_change_password<Body>(&self, body: &Body) -> Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(&format!("{}/admin/password", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to post change_password.")
    }

    pub async fn post_logout(&self) -> Response {
        self.api_client
            .post(&format!("{}/admin/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Extract the confirmation links embedded in the request to the email API.
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        // Parse the body as JSON, starting from raw bytes
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links = LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == LinkKind::Url)
                .collect::<Vec<_>>();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_url =
                Url::parse(&raw_link).expect("Failed to parse confirmation link");
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_url.host_str().unwrap(), "127.0.0.1");
            confirmation_url.set_port(Some(self.port)).unwrap();

            confirmation_url
        };

        // looks like this:
        // http://127.0.0.1:54922/subscriptions/confirm?subscription_token=kMEVpO6hQfQsFylxUE3R9ouAZ
        let html_link = get_link(body["HtmlBody"].as_str().unwrap());
        let text_link = get_link(body["TextBody"].as_str().unwrap());

        ConfirmationLinks {
            html: html_link,
            plain_text: text_link,
        }
    }
}

pub fn assert_is_redirect_to(response: &Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
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

/// Confirmation links embedded in the request to the email API.
#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: Url,
    pub plain_text: Url,
}

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());

        let password_hash = Argon2::default()
            .hash_password(self.password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();

        sqlx::query!(
            r#"
            INSERT INTO users(user_id, username, password_hash) VALUES ($1, $2, $3)
        "#,
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to create test user.");
    }
}
