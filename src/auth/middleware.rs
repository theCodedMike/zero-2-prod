use crate::session_state::TypedSession;
use crate::utils;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::{FromRequest, HttpMessage};
use actix_web_flash_messages::FlashMessage;
use actix_web_lab::middleware::Next;
use anyhow::anyhow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use uuid::Uuid;

/// from_fn takes an asynchronous function as argument and returns an actix-web middleware as output.
///
/// The asynchronous function must have the following signature and structure:
/// ```rust
/// use actix_web_lab::middleware::Next;
/// use actix_web::body::MessageBody;
/// use actix_web::dev::{ServiceRequest, ServiceResponse};
///
/// async fn my_middleware(
///     req: ServiceRequest,
///     next: Next<impl MessageBody>,
/// ) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
///     // before the handler is invoked
///
///     // Invoke handler
///     let response = next.call(req).await;
///
///     // after the handler was invoked
/// }
///
/// ```
///
pub async fn reject_anonymous_users(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let session = {
        let (http_request, payload) = req.parts_mut();
        TypedSession::from_request(http_request, payload).await
    }?;

    match session.get_user_id()? {
        Some(user_id) => {
            req.extensions_mut().insert(UserId(user_id));
            next.call(req).await
        }
        None => {
            let response = utils::redirect_to("/login");
            let error = anyhow!("The user has not logged in");
            Err(InternalError::from_response(error, response).into())
        }
    }
    /* match session.get_user_id()? {
        Some(user_id) => {
            req.extensions_mut().insert(UserId(user_id));
            next.call(req)
                .await
                .map_err(|_e| BizErrorEnum::ServiceCallError)
        }
        None => Err(BizErrorEnum::UserNotLoggedIn),
    }*/
}

#[derive(Copy, Clone, Debug)]
pub struct UserId(Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
