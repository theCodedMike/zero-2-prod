use crate::error::BizErrorEnum;
use crate::utils;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

#[tracing::instrument(name = "/admin/password: Change password page", skip(flash_msgs))]
pub async fn change_password_form(
    flash_msgs: IncomingFlashMessages,
) -> Result<HttpResponse, BizErrorEnum> {
    // Verify if the user is logged in

    let mut msg_html = String::new();
    for msg in flash_msgs.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", msg.content()).unwrap();
    }
    let body = include_str!("change_password.html").replace("{}", &msg_html);
    Ok(utils::ok_to(body))
}
