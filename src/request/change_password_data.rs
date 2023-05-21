use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordData {
    pub current_password: Secret<String>,
    pub new_password: Secret<String>,
    pub new_password_check: Secret<String>,
}
