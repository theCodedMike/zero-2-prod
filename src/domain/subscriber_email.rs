use crate::domain::new_subscriber::InvalidReason;

#[derive(Debug)]
pub struct SubscriberEmail(String);
impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, InvalidReason> {
        if email.trim().is_empty() {
            return Err(InvalidReason::EmailIsEmpty);
        }

        if !email.contains('@') {
            return Err(InvalidReason::EmailMissingAtSymbol);
        }

        let split = email.rsplitn(2, '@').collect::<Vec<&str>>();
        // domain part
        if split[0].is_empty() {
            return Err(InvalidReason::EmailMissingDomain);
        }
        // user part
        if split[1].is_empty() {
            return Err(InvalidReason::EmailMissingSubject);
        }

        if !validator::validate_email(&email) {
            return Err(InvalidReason::EmailFormatWrong);
        }

        Ok(Self(email))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::{Arbitrary, Gen};

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail().fake();
        println!("fake email: {}", email);
        assert_ok!(SubscriberEmail::parse(email));
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully_2(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
