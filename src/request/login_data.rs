use secrecy::Secret;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: Secret<String>,
}
