//! tests/health_check.rs

use std::net::TcpListener;
use zero_2_prod;

/// `tokio::test` is the testing equivalent of `tokio::main`.
/// It also spares you from having to specify the `#[test]` attribute.
///
/// You can inspect what code gets generated using
/// `cargo +nightly expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange. No .await, no .expect
    let address = spawn_app();
    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(address + "/health_check")
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
/// Launch our application in the background ~somehow~
/// No .await call, therefore no need for `spawn_app` to be async now.
/// We are also running tests, so it is not worth it to propagate errors:
/// if we fail to perform the required setup we can just panic and crash
/// all the things.
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().expect("Failed to get address").port();
    println!("PORT: {}", port);

    let server = zero_2_prod::run(listener).expect("Failed to bind address");
    // Launch the server as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it here, hence the non-binding let
    let _ = tokio::spawn(server);

    // We return the application address to the caller!
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(address + "/subscriptions")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_msg) in test_cases {
        // Act
        let response = client
            .post(address.clone() + "/subscriptions")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was '{}'",
            error_msg
        );
    }
}
