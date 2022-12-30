use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        Self::try_from(s)
    }
}

impl TryFrom<String> for SubscriberName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = value.trim().is_empty();
        let is_too_long = value.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters =
            value.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name", value))
        } else {
            Ok(Self(value))
        }
    }
}
    // #[test]
    // fn valid_emails_are_parsed_succesfully() {
    //     let email = SafeEmail().fake();
    //     assert!(SubscriberEmail::parse(email).is_ok())
    // }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "e".repeat(256);
        assert!(SubscriberName::parse(name).is_ok())
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert!(SubscriberName::parse(name).is_err())
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert!(SubscriberName::parse(name).is_err())
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert!(SubscriberName::parse(name).is_err())
    }

    #[test]
    fn names_containing_invalid_characters_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert!(SubscriberName::parse(name).is_err())
        }
    }

    #[test]
    fn valid_name_is_parsed_succesfully() {
        let name = "Urusula le Guin".to_string();
        assert!(SubscriberName::parse(name).is_ok())
    }
}
