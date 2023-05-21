use crate::auth;
use crate::auth::Credentials;
use crate::error::BizErrorEnum;
use crate::request::LoginData;
use crate::session_state::TypedSession;
use crate::telemetry;
use crate::utils;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

/// HMAC: hash-based message authentication code
/// role: verify that the query parameters have been set by our API and that they have not
///       been altered by a third party
#[tracing::instrument(
    name = "Login",
    skip(form, pool, session),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, BizErrorEnum> {
    let credentials: Credentials = form.into_inner().into();
    telemetry::record_field("username", &credentials.username);

    match auth::validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            telemetry::record_field("user_id", &user_id);
            // Avoid session fixation attack
            session.renew();
            // If failed, redirect to login page
            if let Err(error) = session.insert_user_id(user_id) {
                return Ok(redirect_to_login_when_error(error));
            };
            // if login is successfully, redirect to the dashboard
            Ok(utils::redirect_to("/admin/dashboard"))
        }
        Err(error) => match error {
            // if username or password is wrong, login again.
            // At the same time, there will be a tip on the login page.
            BizErrorEnum::InvalidUsername | BizErrorEnum::InvalidPassword(_) => {
                // Use cookie instead of hmac to store error msg
                /*let encoded_error = urlencoding::Encoded::new(error.to_string());
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
                };*/
                // There are two types of cookies: session cookies and persistent cookies
                // Session cookies are stored in memory, they are deleted when the session ends (i.e. the browser is closed)
                // Persistent cookies, instead, are saved to disk and will still be there when you re-open the browser
                // A vanilla Set-Cookie header creates a session cookie
                // To set a persistent cookie you must specify an expiration policy using a cookie attribute - either Max-Age or Expires
                //
                // Response Headers:
                // set-cookie: login_error_msg=Invalid username.

                Ok(redirect_to_login_when_error(error))
            }
            _ => Err(error),
        },
    }
}

/// Redirect to the login page with an error message.
pub fn redirect_to_login_when_error(error: BizErrorEnum) -> HttpResponse {
    FlashMessage::error(error.to_string()).send();
    utils::redirect_to("/login")
}
