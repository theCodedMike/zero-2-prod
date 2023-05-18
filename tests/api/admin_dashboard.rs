use crate::helpers;
use crate::helpers::TestApp;

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let app = TestApp::spawn_app().await;

    // Act
    let response = app.get_admin_dashboard().await;

    // Assert
    helpers::assert_is_redirect_to(&response, "/login");
}
