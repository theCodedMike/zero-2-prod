use crate::helpers::TestApp;

/// `tokio::test` is the testing equivalent of `tokio::api`.
/// It also spares you from having to specify the `#[test]` attribute.
///
/// You can inspect what code gets generated using
/// `cargo +nightly expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange. No .await, no .expect
    let app = TestApp::spawn_app().await;

    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    // Act
    let response = app.get_health_check().await;

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
