use crate::utils;
use actix_web::body::BoxBody;
use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use argon2::password_hash;
use std::fmt::{Debug, Formatter};

#[derive(thiserror::Error)]
pub enum BizErrorEnum {
    // VALIDATE SUBSCRIBER'S NAME AND EMAIL
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

    // VALIDATE NEWSLETTER'S TITLE AND CONTENT
    #[error("Newsletter's title is empty.")]
    NewsletterTitleIsEmpty,

    #[error("Newsletter's content is empty.")]
    NewsletterContentIsEmpty,

    // VALIDATE URL
    #[error("Url is incorrect.")]
    ParseUrlError,

    #[error("Url join path error.")]
    JoinUrlError,

    // VALIDATE AUTH
    #[error("The 'Authorization' header was missing.")]
    AuthorizationHeaderIsMissing,

    #[error("The 'Authorization' header was not a valid UTF8 string.")]
    AuthorizationHeaderIsInvalidUtf8String(#[source] actix_web::http::header::ToStrError),

    #[error("The authorization scheme was not 'Basic'.")]
    AuthorizationSchemeNotBasic,

    #[error("Failed to base64-decode 'Basic' credentials.")]
    Base64DecodeError(#[source] base64::DecodeError),

    #[error("The decoded credential string is not valid UTF8.")]
    CredentialStringIsInvalidUtf8String(#[source] std::string::FromUtf8Error),

    #[error("A username must be provided in 'Basic' auth.")]
    CredentialMissingUsername,

    #[error("A password must be provided in 'Basic' auth.")]
    CredentialMissingPassword,

    #[error("Invalid username.")]
    InvalidUsername,

    #[error("Invalid password.")]
    InvalidPassword(#[source] password_hash::Error),

    #[error("The user has not logged in")]
    UserNotLoggedIn,

    // VALIDATE DATABASE ACCESS
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

    #[error("Failed to query users.")]
    QueryUsersError(#[source] sqlx::Error),

    #[error("Failed to update users.")]
    UpdateUsersError(#[source] sqlx::Error),

    // OTHER
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

    #[error("Failed to spawn blocking task.")]
    SpawnBlockingTaskError(#[source] tokio::task::JoinError),

    #[error("Failed to build a redis session store")]
    RedisSessionStoreBuildError(#[source] anyhow::Error),

    #[error("Failed to insert key to session")]
    ActixSessionInsertError(#[source] actix_session::SessionInsertError),

    #[error("Failed to get key from session")]
    ActixSessionGetError(#[source] actix_session::SessionGetError),

    // Argon
    #[error("Failed to parse hash in PHC string format.")]
    Argon2HashParseError(#[source] password_hash::Error),

    #[error("Failed to generate hmac instance.")]
    HmacGenerateError(#[source] hmac::digest::InvalidLength),

    #[error("Failed to parse hmac tag")]
    HexStringDecodedError(#[source] hex::FromHexError),

    #[error("Failed to verify hmac tag")]
    HmacVerifySliceError(#[source] hmac::digest::MacError),

    #[error("Failed to hash password")]
    Argon2HashPasswordError(#[source] password_hash::Error),
}

impl Debug for BizErrorEnum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        crate::error::error_chain_fmt(self, f)
    }
}

impl ResponseError for BizErrorEnum {
    /// `status_code` is invoked by the default `error_response` implementation.
    /// We are providing a bespoke `error_response` implementation
    /// therefore there is no need to maintain a `status_code` implementation anymore.
    /*fn status_code(&self) -> StatusCode {
        match self {
            BizErrorEnum::SubscriberNameIsEmpty
            | BizErrorEnum::SubscriberNameIsTooLong
            | BizErrorEnum::SubscriberNameContainsIllegalCharacter
            | BizErrorEnum::SubscriberEmailIsEmpty
            | BizErrorEnum::SubscriberEmailMissAtSymbol
            | BizErrorEnum::SubscriberEmailMissSubject
            | BizErrorEnum::SubscriberEmailMissDomain
            | BizErrorEnum::SubscriberEmailFormatIsIncorrect => StatusCode::BAD_REQUEST,

            BizErrorEnum::AuthorizationHeaderIsMissing
            | BizErrorEnum::AuthorizationHeaderIsInvalidUtf8String(_)
            | BizErrorEnum::AuthorizationSchemeNotBasic
            | BizErrorEnum::Base64DecodeError(_)
            | BizErrorEnum::CredentialStringIsInvalidUtf8String(_)
            | BizErrorEnum::CredentialMissingUsername
            | BizErrorEnum::CredentialMissingPassword => StatusCode::UNAUTHORIZED,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }*/

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            BizErrorEnum::UserNotLoggedIn => {
                FlashMessage::error("You are not logged in, please log in before proceeding")
                    .send();
                utils::redirect_to("/login")
            }
            BizErrorEnum::SubscriberNameIsEmpty
            | BizErrorEnum::SubscriberNameIsTooLong
            | BizErrorEnum::SubscriberNameContainsIllegalCharacter
            | BizErrorEnum::SubscriberEmailIsEmpty
            | BizErrorEnum::SubscriberEmailMissAtSymbol
            | BizErrorEnum::SubscriberEmailMissSubject
            | BizErrorEnum::SubscriberEmailMissDomain
            | BizErrorEnum::SubscriberEmailFormatIsIncorrect
            | BizErrorEnum::NewsletterTitleIsEmpty
            | BizErrorEnum::NewsletterContentIsEmpty => HttpResponse::new(StatusCode::BAD_REQUEST),

            BizErrorEnum::AuthorizationHeaderIsMissing
            | BizErrorEnum::AuthorizationHeaderIsInvalidUtf8String(_)
            | BizErrorEnum::AuthorizationSchemeNotBasic
            | BizErrorEnum::Base64DecodeError(_)
            | BizErrorEnum::CredentialStringIsInvalidUtf8String(_)
            | BizErrorEnum::CredentialMissingUsername
            | BizErrorEnum::CredentialMissingPassword
            | BizErrorEnum::InvalidUsername
            | BizErrorEnum::InvalidPassword(_)
            | BizErrorEnum::ActixSessionInsertError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    // actix_web::http::header provides a collection of constants
                    // for the names of several well-known/standard HTTP headers
                    .insert(actix_web::http::header::WWW_AUTHENTICATE, header_value);

                response
            }

            _ => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
