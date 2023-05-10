use crate::domain::{SubscriberEmail, SubscriberName};

#[derive(Debug)]
pub struct NewSubscriber {
    email: SubscriberEmail,
    name: SubscriberName,
}
impl NewSubscriber {
    pub fn new(email: SubscriberEmail, name: SubscriberName) -> Self {
        Self { email, name }
    }

    pub fn email(&self) -> &str {
        self.email.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_email(&self) -> &SubscriberEmail {
        &self.email
    }

    pub fn get_name(&self) -> &SubscriberName {
        &self.name
    }
}

#[derive(Debug)]
pub enum InvalidReason {
    NameIsEmpty,
    NameIsTooLong,
    NameContainsForbiddenCharacters,

    EmailIsEmpty,
    EmailMissingAtSymbol,
    EmailMissingSubject,
    EmailMissingDomain,
    EmailFormatWrong,
}
impl InvalidReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvalidReason::NameIsEmpty => "Name is empty",
            InvalidReason::NameIsTooLong => "Name is too long",
            InvalidReason::NameContainsForbiddenCharacters => "Name contains forbidden characters",

            InvalidReason::EmailIsEmpty => "Email is empty",
            InvalidReason::EmailMissingAtSymbol => "Email missing @ symbol",
            InvalidReason::EmailMissingSubject => "Email missing subject",
            InvalidReason::EmailMissingDomain => "Email missing domain",
            InvalidReason::EmailFormatWrong => "Email's format is not correct",
        }
    }
}
