use crate::helpers;
use crate::helpers::{ConfirmationLinks, TestApp};
use uuid::Uuid;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = TestApp::spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // We assert that no request is fired at Postmark!
        .expect(0)
        .mount(&app.email_server)
        .await;
    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter(&newsletter_request_body).await;
    helpers::assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("No confirmed subscribers!!!"));
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = TestApp::spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter(&newsletter_request_body).await;
    helpers::assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published"));
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = TestApp::spawn_app().await;
    app.test_user.login(&app).await;

    let test_cases = vec![
        (
            serde_json::json!({
                "text_content": "Newsletter body as plain text",
                "html_content": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter!"
            }),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletter(&invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletters_requests_missing_authorization_are_rejected() {
    // Arrange
    let app = TestApp::spawn_app().await;

    let body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>"
    });
    let response = app.post_newsletter(&body).await;

    // Assert
    helpers::assert_is_redirect_to(&response, "/login");
}

/// Use the public API of the application under test to create
/// an unconfirmed subscriber.
///
/// When using mount, your Mocks will be active until the MockServer is shut down.
/// When using mount_as_scoped, your Mocks will be active as long as the returned MockGuard is not dropped.
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        // We are not using `mount`!
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .expect("Failed to post subscription");

    // We now inspect the requests received by the mock Postmark server
    // to retrieve the confirmation link and return it
    let request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // We can then reuse the same helper and just add
    // an extra step to actually call the confirmation link!
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[deprecated(since = "1.0", note = "old style")]
async fn newsletters_non_existing_user_is_rejected() {
    // Arrange
    let app = TestApp::spawn_app().await;
    // Random credentials
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", &app.address))
        .basic_auth(username, Some(password))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[deprecated(since = "1.0", note = "old style")]
async fn newsletters_invalid_password_is_rejected() {
    // Arrange
    let app = TestApp::spawn_app().await;
    let username = &app.test_user.username;
    // Random credentials
    let password = Uuid::new_v4().to_string();
    assert_ne!(app.test_user.password, password);

    let body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletter", &app.address))
        .basic_auth(username, Some(password))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    // Arrange
    let app = TestApp::spawn_app().await;

    // Act
    let response = app.get_newsletter().await;

    // Assert
    helpers::assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    // Arrange
    let app = TestApp::spawn_app().await;

    // Act
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": Uuid::new_v4().to_string()
    });
    let response = app.post_newsletter(&newsletter_request_body).await;

    // Assert
    helpers::assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = TestApp::spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let key = Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": key
    });
    let response = app.post_newsletter(&body).await;
    helpers::assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published!"));

    // Act - Part 3 - Submit newsletter form **again**
    let response = app.post_newsletter(&body).await;
    helpers::assert_is_redirect_to(&response, "/admin/newsletter");

    // Act - Part 4 - Follow the redirect
    let html_page = app.get_newsletter_html().await;
    assert!(html_page.contains("The newsletter issue has been published!"));
}
