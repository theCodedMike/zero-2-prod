use actix_web::ResponseError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// A new error type, wrapping a sqlx::Error
pub struct StoreTokenError(sqlx::Error);

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while trying to store a subscription token."
        )
    }
}

impl Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::error::error_chain_fmt(self, f)
    }
}

//  Nuke it!
impl ResponseError for StoreTokenError {}

impl Error for StoreTokenError {
    // The compiler transparently casts `&sqlx::Error` into a `&dyn Error`
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<sqlx::Error> for StoreTokenError {
    fn from(value: sqlx::Error) -> Self {
        Self(value)
    }
}
