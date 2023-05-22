use crate::error::BizErrorEnum;
use crate::session_state::TypedSession;
use crate::utils;
use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

#[tracing::instrument(name = "Logout", skip(session))]
pub async fn log_out(session: TypedSession) -> Result<HttpResponse, BizErrorEnum> {
    // Verify if the user is logged in

    session.log_out();

    FlashMessage::info("You have successfully logged out.").send();
    Ok(utils::redirect_to("/login"))
}
