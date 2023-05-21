use crate::error::BizErrorEnum;
use crate::session_state::TypedSession;
use uuid::Uuid;

pub fn validate_session(session: &TypedSession) -> Result<Uuid, BizErrorEnum> {
    match session.get_user_id() {
        Err(_) => Err(BizErrorEnum::UserNotLoggedIn),
        Ok(user_id) => match user_id {
            None => Err(BizErrorEnum::UserNotLoggedIn),
            Some(user_id) => Ok(user_id),
        },
    }
}
