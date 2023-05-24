use crate::error::BizErrorEnum;
use validator::HasLen;

#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey {
    type Error = BizErrorEnum;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(BizErrorEnum::IdempotencyKeyIsBlank);
        }
        let min_length = 10;
        if value.length() < min_length {
            return Err(BizErrorEnum::IdempotencyKeyIsTooShort);
        }
        let max_length = 50;
        if value.length() > max_length {
            return Err(BizErrorEnum::IdempotencyKeyIsTooLong);
        }

        Ok(IdempotencyKey(value))
    }
}

impl From<IdempotencyKey> for String {
    fn from(value: IdempotencyKey) -> Self {
        value.0
    }
}

impl AsRef<str> for IdempotencyKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
