use crate::domain::InvalidReason;
use crate::error::StoreTokenError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use std::fmt::{Debug, Display, Formatter};

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(InvalidReason),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
/*
#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(InvalidReason),

    #[error("Failed to acquire a Postgres connection from the pool.")]
    PoolError(#[source] sqlx::Error),

    #[error("Failed to insert new subscriber in the database.")]
    InsertSubscriberError(#[source] sqlx::Error),

    #[error("Failed to commit SQL transaction to store a new subscriber.")]
    TransactionCommitError(#[source] sqlx::Error),

    #[error("Failed to store the confirmation token for a new subscriber.")]
    StoreTokenError(#[from] StoreTokenError),

    #[error("Failed to send a confirmation email.")]
    SendEmailError(#[from] reqwest::Error),
}
impl From<InvalidReason> for SubscribeError {
    fn from(value: InvalidReason) -> Self {
        Self::ValidationError(value)
    }
}

impl From<StoreTokenError> for SubscribeError {
    fn from(value: StoreTokenError) -> Self {
        Self::StoreTokenError(value)
    }
}

impl From<reqwest::Error> for SubscribeError {
    fn from(value: reqwest::Error) -> Self {
        Self::SendEmailError(value)
    }
}

impl Display for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscribeError::ValidationError(e) => write!(f, "{}", e),
            SubscribeError::PoolError(_) => {
                write!(f, "Failed to acquire a Postgres connection from the pool.")
            }
            SubscribeError::InsertSubscriberError(_) => {
                write!(f, "Failed to insert new subscriber in the database.")
            }
            SubscribeError::TransactionCommitError(_) => write!(
                f,
                "Failed to commit SQL transaction to store a new subscriber."
            ),
            SubscribeError::StoreTokenError(_) => write!(
                f,
                "Failed to store the confirmation token for a new subscriber."
            ),
            SubscribeError::SendEmailError(_) => write!(f, "Failed to send a confirmation email."),
        }
    }
}

impl std::error::Error for SubscribeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // &str does not implement `Error` - we consider it the root cause
            SubscribeError::ValidationError(_) => None,
            SubscribeError::PoolError(e) => Some(e),
            SubscribeError::InsertSubscriberError(e) => Some(e),
            SubscribeError::TransactionCommitError(e) => Some(e),
            SubscribeError::StoreTokenError(e) => Some(e),
            SubscribeError::SendEmailError(e) => Some(e),
        }
    }
}
*/

// We are still using a bespoke implementation of `Debug`
// to get a nice report using the error source chain
impl Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::error::error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
