/// environment variable
pub const LOCAL_ENVIRONMENT: &str = "local";
pub const PRODUCTION_ENVIRONMENT: &str = "production";

/// encrypt password
pub const DELIMITER: &str = "     ";
pub const SALT: &str = "I_STILL_LOVE_YOU";

/// validate subscriber's name
pub const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

/// http request header's key
pub const HEADER_KEY: &str = "X-Postmark-Server-Token";
