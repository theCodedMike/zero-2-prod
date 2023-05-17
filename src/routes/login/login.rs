use crate::auth;
use crate::auth::Credentials;
use crate::error::BizErrorEnum;
use crate::request::LoginData;
use crate::startup::HmacSecret;
use crate::telemetry;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use sqlx::PgPool;

/// HMAC: hash-based message authentication code
/// role: verify that the query parameters have been set by our API and that they have not
///       been altered by a third party
#[tracing::instrument(
    name = "Login",
    skip(form, pool),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, BizErrorEnum> {
    let credentials: Credentials = form.into_inner().into();
    telemetry::record_field("username", &credentials.username);

    match auth::validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            telemetry::record_field("user_id", &user_id);
            // if login is successfully, redirect to the homepage; otherwise login again.
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(error) => match error {
            // if username or password is wrong, login again.
            // At the same time, there will be a tip on the login page.
            BizErrorEnum::InvalidUsername | BizErrorEnum::InvalidPassword(_) => {
                let encoded_error = urlencoding::Encoded::new(error.to_string());
                let query_param = format!("error={}", encoded_error);
                let hmac_tag = {
                    let mut hmac =
                        Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
                            .map_err(|e| {
                            tracing::error!("Failed to generate hmac instance.");
                            BizErrorEnum::HmacGenerateError(e)
                        })?;
                    hmac.update(query_param.as_bytes());
                    hmac.finalize().into_bytes()
                };
                Ok(HttpResponse::SeeOther()
                    .insert_header((LOCATION, format!("/login?{query_param}&tag={hmac_tag:x}")))
                    .finish())
            }
            _ => Err(error),
        },
    }
}
