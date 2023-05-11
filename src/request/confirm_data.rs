use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfirmData {
    pub subscription_token: String,
}
