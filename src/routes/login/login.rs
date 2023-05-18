use crate::auth;
use crate::auth::Credentials;
use crate::constant::LOGIN_ERROR_MSG;
use crate::error::BizErrorEnum;
use crate::request::LoginData;
use crate::telemetry;
use actix_web::cookie::Cookie;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse};
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
                Ok(HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/login"))
                    //.insert_header(("Set-Cookie", format!("login_error_msg={}", error)))
                    // 等价于
                    .cookie(Cookie::new(LOGIN_ERROR_MSG, error.to_string()))
                    .finish())
            }
            _ => Err(error),
        },
    }
}
