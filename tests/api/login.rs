use crate::helpers;
use crate::helpers::TestApp;
// use zero_2_prod::constant::LOGIN_ERROR_MSG;

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = TestApp::spawn_app().await;

    // Act - Part 1 - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert
    //let login_cookie = response
    //    .cookies()
    //    .find(|c| c.name() == LOGIN_ERROR_MSG)
    //    .expect("Failed to find login_error_msg cookie");
    //assert_eq!(login_cookie.value(), "Invalid username.");
    helpers::assert_is_redirect_to(&response, "/login");

    // Act - Part 2 - Follow the redirect
    // Request Headers:
    // Cookie: login_error_msg=Invalid username.
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Invalid username.</i></p>"#));

    // Act - Part 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Invalid username.</i></p>"#));
}
