use crate::auth::credentials::Credentials;
use crate::error::BizErrorEnum;
use crate::telemetry;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

/// PHC string format:  
/// ${algorithm}${algorithm version}${,-separated algorithm parameters}${hash}${salt}
///
/// For example:
/// $argon2id$v=19$m=19456,t=2,p=1$OzLfJ+WIZzODQlNBT20mbw$8DU86CFOWvlJu5D+75BV6DidbTJLM92egH4+ZJxXZU4
///
///
#[tracing::instrument(name = "Validate credentials", skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &PgPool,
) -> Result<uuid::Uuid, BizErrorEnum> {
    // query user_id, password_hash from table
    let (user_id, password_hash_from_db) = get_stored_credentials(&credentials.username, pool)
        .await?
        .ok_or(BizErrorEnum::InvalidUsername)?;

    // PHC string format takes care of salt for us, implicitly
    // Offload CPU-intensive task to a separate thread-pool using tokio::task::spawn_blocking.
    telemetry::spawn_blocking_with_tracing(move || {
        verify_password_hash(password_hash_from_db, credentials.password)
    })
    .await
    // spawn_blocking is fallible - we have a nested Result here!
    .map_err(|e| BizErrorEnum::SpawnBlockingTaskError(e))??;

    Ok(user_id)
}

#[tracing::instrument(name = "Get stored credentials", skip(pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(uuid::Uuid, Secret<String>)>, BizErrorEnum> {
    let row = sqlx::query!(
        r#"
        SELECT user_id, password_hash FROM users WHERE username = $1
    "#,
        username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query users table: {:?}", e);
        BizErrorEnum::QueryUsersError(e)
    })?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}

/// Argon2 can automatically infer what load parameters and
/// salt should be used to verify if the password candidate is a match
#[tracing::instrument(
    name = "Verify password hash",
    skip(password_hash_from_db, password_from_user)
)]
fn verify_password_hash(
    password_hash_from_db: Secret<String>,
    password_from_user: Secret<String>,
) -> Result<(), BizErrorEnum> {
    // parse hash in PHC string format
    let expected_password_hash = PasswordHash::new(&password_hash_from_db.expose_secret())
        .map_err(|e| BizErrorEnum::Argon2HashParseError(e))?;
    Argon2::default()
        .verify_password(
            password_from_user.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .map_err(|e| BizErrorEnum::InvalidPassword(e))?;
    Ok(())
}
