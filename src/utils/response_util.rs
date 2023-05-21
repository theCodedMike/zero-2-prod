use actix_web::http::header::{ContentType, LOCATION};
use actix_web::HttpResponse;

pub fn redirect_to(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        //.insert_header(("Set-Cookie", format!("login_error_msg={}", error)))
        // 等价于
        //.cookie(Cookie::new(LOGIN_ERROR_MSG, error.to_string()))
        .finish()
}

pub fn ok_to(body: String) -> HttpResponse {
    // Response Headers:
    // set-cookie: login_error_msg=; Max-Age=0
    /*response
    .add_removal_cookie(&Cookie::new(LOGIN_ERROR_MSG, ""))
    .map_err(|e| {
        tracing::error!("Failed to add removal cookie: {:?}", e);
    })
    .unwrap();*/
    HttpResponse::Ok()
        .content_type(ContentType::html())
        // 等价于
        //.cookie(
        //    Cookie::build(LOGIN_ERROR_MSG, "")
        //        .max_age(Duration::ZERO)
        //        .finish(),
        //)
        .body(body)
}
