use crate::error::BizErrorEnum;
use crate::session_state::TypedSession;
use crate::utils;
use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

#[tracing::instrument(name = "Logout", skip(session))]
pub async fn log_out(session: TypedSession) -> Result<HttpResponse, BizErrorEnum> {
    let _user_id = match session.get_user_id() {
        Err(_) => return Ok(utils::redirect_to("/login")),
        Ok(user_id) => match user_id {
            None => return Ok(utils::redirect_to("/login")),
            Some(user_id) => user_id,
        },
    };

    session.log_out();

    FlashMessage::info("You have successfully logged out.").send();
    Ok(utils::redirect_to("/login"))
}
