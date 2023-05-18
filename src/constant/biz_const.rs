/// environment variable
pub const LOCAL_ENVIRONMENT: &str = "local";
pub const PRODUCTION_ENVIRONMENT: &str = "production";

/// validate subscriber's name
pub const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

/// http request header's key
pub const HEADER_KEY: &str = "X-Postmark-Server-Token";

/// login error msg
pub const LOGIN_ERROR_MSG: &str = "login_error_msg";

/// session
pub const SESSION_USER_ID: &str = "user_id";
