use actix_web::http::StatusCode;
use actix_web::ResponseError;
use std::fmt::{Debug, Formatter};

#[derive(thiserror::Error)]
pub enum BizErrorEnum {
    #[error("Subscriber's name is empty.")]
    SubscriberNameIsEmpty,

    #[error("Subscriber's name is too long.")]
    SubscriberNameIsTooLong,

    #[error("Subscriber's name contains illegal character.")]
    SubscriberNameContainsIllegalCharacter,

    #[error("Subscriber's email is empty.")]
    SubscriberEmailIsEmpty,

    #[error("Subscriber's email is missing @ symbol.")]
    SubscriberEmailMissAtSymbol,

    #[error("Subscriber's email is missing subject.")]
    SubscriberEmailMissSubject,

    #[error("Subscriber's email is missing domain.")]
    SubscriberEmailMissDomain,

    #[error("Subscriber's email format is incorrect.")]
    SubscriberEmailFormatIsIncorrect,

    #[error("Url is incorrect.")]
    ParseUrlError,

    #[error("Url join path error.")]
    JoinUrlError,

    #[error("Subscription_token is invalid.")]
    SubscriptionTokenInvalidError,

    #[error("Failed to acquire a Postgres connection from the pool.")]
    PgPoolError(#[source] sqlx::Error),

    #[error("Failed to commit SQL transaction to store a new subscriber.")]
    TransactionCommitError(#[source] sqlx::Error),

    #[error("Failed to insert subscriptions.")]
    InsertSubscriptionsError(#[source] sqlx::Error),

    #[error("Failed to update subscriptions.")]
    UpdateSubscriptionsError(#[source] sqlx::Error),

    #[error("Failed to query subscriptions.")]
    QuerySubscriptionsError(#[source] sqlx::Error),

    #[error("Failed to insert subscription_tokens.")]
    InsertSubscriptionTokensError(#[source] sqlx::Error),

    #[error("Failed to query subscription_tokens")]
    QuerySubscriptionTokensError(#[source] sqlx::Error),

    #[error("Failed to send a confirmation email.")]
    SendEmailError(#[from] reqwest::Error),

    #[error("Failed to bind TcpListener.")]
    BindTcpListenerError(#[source] std::io::Error),

    #[error("Failed to listen TcpListener.")]
    ListenTcpListenerError(#[source] std::io::Error),

    #[error("Failed to run server.")]
    RunServerError(#[source] std::io::Error),

    #[error("Failed to determine the current directory.")]
    GetCurrentDirError(#[source] std::io::Error),

    #[error("Failed to parse environment variable.")]
    ParseEnvironmentVariableError(String),

    #[error("Failed to build config sources.")]
    BuildConfigSourcesError(#[source] config::ConfigError),

    #[error("Failed to deserialize config file.")]
    DeserializeConfigurationFileError(#[source] config::ConfigError),

    #[error("Failed to set logger.")]
    SetLoggerError(#[source] tracing_log::log::SetLoggerError),

    #[error("Failed to set subscriber.")]
    SetSubscriberError(#[source] tracing::dispatcher::SetGlobalDefaultError),
}

impl Debug for BizErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::error::error_chain_fmt(self, f)
    }
}

impl ResponseError for BizErrorEnum {
    fn status_code(&self) -> StatusCode {
        match self {
            BizErrorEnum::SubscriberNameIsEmpty
            | BizErrorEnum::SubscriberNameIsTooLong
            | BizErrorEnum::SubscriberNameContainsIllegalCharacter
            | BizErrorEnum::SubscriberEmailIsEmpty
            | BizErrorEnum::SubscriberEmailMissAtSymbol
            | BizErrorEnum::SubscriberEmailMissSubject
            | BizErrorEnum::SubscriberEmailMissDomain
            | BizErrorEnum::SubscriberEmailFormatIsIncorrect => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
