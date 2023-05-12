use crate::error::BizErrorEnum;
use unicode_segmentation::UnicodeSegmentation;

const FORBIDDEN_CHARACTERS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

#[derive(Debug)]
pub struct SubscriberName(String);
impl SubscriberName {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.
    /// It panics otherwise.
    pub fn parse(name: String) -> Result<Self, BizErrorEnum> {
        // `.trim()` returns a view over the input `s` without trailing
        // whitespace-like characters.
        // `.is_empty` checks if the view contains any character.
        if name.trim().is_empty() {
            return Err(BizErrorEnum::SubscriberNameIsEmpty);
        };

        // A grapheme is defined by the Unicode standard as a "user-perceived"
        // character: `å` is a single grapheme, but it is composed of two characters
        // (`a` and `̊`).
        //
        // `graphemes` returns an iterator over the graphemes in the input `s`.
        // `true` specifies that we want to use the extended grapheme definition set,
        // the recommended one.
        if name.graphemes(true).count() > 256 {
            return Err(BizErrorEnum::SubscriberNameIsTooLong);
        };

        // Iterate over all characters in the input `s` to check if any of them
        // matches one of the characters in the forbidden array.
        if name
            .chars()
            .any(|a_char| FORBIDDEN_CHARACTERS.contains(&a_char))
        {
            return Err(BizErrorEnum::SubscriberNameContainsIllegalCharacter);
        };

        Ok(Self(name))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::subscriber_name::{SubscriberName, FORBIDDEN_CHARACTERS};
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &FORBIDDEN_CHARACTERS {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
