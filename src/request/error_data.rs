use crate::error::BizErrorEnum;
use crate::startup::HmacSecret;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ErrorData {
    pub error: String,
    pub tag: String,
}

impl ErrorData {
    pub fn verify(self, secret: &HmacSecret) -> Result<String, BizErrorEnum> {
        let tag = hex::decode(self.tag).map_err(|e| BizErrorEnum::HexStringDecodedError(e))?;
        let query_param = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut hmac = Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
            .map_err(|e| BizErrorEnum::HmacGenerateError(e))?;
        hmac.update(query_param.as_bytes());
        hmac.verify_slice(&tag)
            .map_err(|e| BizErrorEnum::HmacVerifySliceError(e))?;

        Ok(self.error)
    }
}
