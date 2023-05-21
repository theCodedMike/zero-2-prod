use crate::auth::Credentials;
use crate::error::BizErrorEnum;
use crate::request::ChangePasswordData;
use crate::routes;
use crate::session_state::TypedSession;
use crate::{auth, utils};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use validator::HasLen;

#[tracing::instrument(name = "Change password", skip(form, pool, session))]
pub async fn change_password(
    form: web::Form<ChangePasswordData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
) -> Result<HttpResponse, BizErrorEnum> {
    let password_data = form.into_inner();

    // Get user_id
    let user_id = match session.get_user_id() {
        Err(_) => return Ok(utils::redirect_to("/login")),
        Ok(user_id) => match user_id {
            None => return Ok(utils::redirect_to("/login")),
            Some(user_id) => user_id,
        },
    };
    // Get username
    let username = match routes::query_username(user_id, &pool).await {
        Err(_) => return Ok(utils::redirect_to("/login")),
        Ok(username) => username,
    };
    // Validate current password is valid
    let credentials = Credentials {
        username,
        password: password_data.current_password.clone(),
    };
    if let Err(error) = auth::validate_credentials(credentials, &pool).await {
        return match error {
            BizErrorEnum::InvalidUsername => {
                FlashMessage::error("The current username is incorrect.").send();
                Ok(utils::redirect_to("/login"))
            }
            BizErrorEnum::InvalidPassword(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(utils::redirect_to("/admin/password"))
            }
            _ => Err(error),
        };
    }
    // Validate new password equals to new password check
    if password_data.new_password.expose_secret()
        != password_data.new_password_check.expose_secret()
    {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
        return Ok(utils::redirect_to("/admin/password"));
    }
    // Validate the length of new password
    let length = password_data.new_password.expose_secret().length();
    if length < 6 || length > 128 {
        FlashMessage::error("The length of new password must >= 6 && <= 128 characters.").send();
        return Ok(utils::redirect_to("/admin/password"));
    }
    // Update new password
    auth::update_new_password(user_id, password_data.new_password, &pool).await?;

    FlashMessage::info("Your password has been changed.").send();
    Ok(utils::redirect_to("/admin/password"))
}
