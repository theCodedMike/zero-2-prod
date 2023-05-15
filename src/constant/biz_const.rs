/// environment variable
pub const LOCAL_ENVIRONMENT: &str = "local";
pub const PRODUCTION_ENVIRONMENT: &str = "production";

/// validate subscriber's name
pub const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

/// http request header's key
pub const HEADER_KEY: &str = "X-Postmark-Server-Token";
