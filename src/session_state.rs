use crate::constant::SESSION_USER_ID;
use crate::error::BizErrorEnum;
use actix_session::{Session, SessionExt};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::Ready;
use uuid::Uuid;

/// Customize actix-web Extractor
pub struct TypedSession(Session);

impl TypedSession {
    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), BizErrorEnum> {
        self.0
            .insert(SESSION_USER_ID, user_id)
            .map_err(|e| BizErrorEnum::ActixSessionInsertError(e))
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, BizErrorEnum> {
        self.0
            .get(SESSION_USER_ID)
            .map_err(|e| BizErrorEnum::ActixSessionGetError(e))
    }

    pub fn log_out(&self) {
        self.0.purge()
    }
}
/// Make TypedSession as an actix-web extractor
impl FromRequest for TypedSession {
    // This is a complicated way of saying
    // "We return the same error returned by the
    // implementation of `FromRequest` for `Session`".
    type Error = <Session as FromRequest>::Error;
    // Rust does not yet support the `async` syntax in traits.
    // From request expects a `Future` as return type to allow for extractors
    // that need to perform asynchronous operations (e.g. a HTTP call)
    // We do not have a `Future`, because we don't perform any I/O,
    // so we wrap `TypedSession` into `Ready` to convert it into a `Future` that
    // resolves to the wrapped value the first time it's polled by the executor.
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        std::future::ready(Ok(TypedSession(req.get_session())))
    }
}
