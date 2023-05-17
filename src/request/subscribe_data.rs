use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::error::BizErrorEnum;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SubscribeData {
    pub email: String,
    pub name: String,
}
impl TryFrom<SubscribeData> for NewSubscriber {
    type Error = BizErrorEnum;

    fn try_from(form: SubscribeData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;
        Ok(NewSubscriber::new(email, name))
    }
}
