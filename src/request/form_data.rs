use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::error::BizErrorEnum;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}
impl TryFrom<FormData> for NewSubscriber {
    type Error = BizErrorEnum;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber::new(email, name))
    }
}
