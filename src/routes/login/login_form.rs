use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use actix_web_flash_messages::Level::Error;
// use tracing_log::log::trace;
use crate::util;
use std::fmt::Write;

#[tracing::instrument(name = "Get login page", skip(flash_msgs))]
pub async fn login_form(flash_msgs: IncomingFlashMessages) -> HttpResponse {
    // HMAC to verify integrity and provenance for our query parameters
    /*let error_msg = match query {
        None => "".into(),
        Some(query) => match query.into_inner().verify(&secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to verify query parameters using the HMAC tag: {}",
                    e
                );
                "".into()
            }
        },
    };*/
    // Use cookie
    /*let error_msg = match request.cookie(LOGIN_ERROR_MSG) {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
    };*/

    // Use Flash msg
    let mut error_msg = String::new();
    for msg in flash_msgs.iter().filter(|msg| msg.level() == Error) {
        writeln!(error_msg, "<p><i>{}</i></p>", msg.content()).unwrap();
    }

    let login_page = include_str!("login.html").replace("{}", &error_msg);
    util::ok_to_return(login_page)
}
