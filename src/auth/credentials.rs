use crate::request::LoginData;
use secrecy::Secret;

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl From<LoginData> for Credentials {
    fn from(value: LoginData) -> Self {
        Self {
            username: value.username,
            password: value.password,
        }
    }
}
