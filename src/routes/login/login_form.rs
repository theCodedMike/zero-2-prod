use crate::constant::LOGIN_ERROR_MSG;
use actix_web::cookie::Cookie;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse};
use tracing_log::log::trace;

#[tracing::instrument(name = "Get login page", skip(request))]
pub async fn login_form(request: HttpRequest) -> HttpResponse {
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
    let error_msg = match request.cookie(LOGIN_ERROR_MSG) {
        None => "".into(),
        Some(cookie) => format!("<p><i>{}</i></p>", cookie.value()),
    };
    let login_page = include_str!("login.html").replace("{}", &error_msg);

    // Response Headers:
    // set-cookie: login_error_msg=; Max-Age=0
    let mut response = HttpResponse::Ok()
        .content_type(ContentType::html())
        // 等价于
        //.cookie(
        //    Cookie::build(LOGIN_ERROR_MSG, "")
        //        .max_age(Duration::ZERO)
        //        .finish(),
        //)
        .body(login_page);

    response
        .add_removal_cookie(&Cookie::new(LOGIN_ERROR_MSG, ""))
        .map_err(|e| {
            trace!("Failed to add removal cookie: {:?}", e);
        })
        .unwrap();

    response
}