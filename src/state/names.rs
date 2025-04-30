use std::fmt;

use maddie_wtf::OptionExt as _;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use thiserror::Error;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct TagName(String);

impl TryFrom<String> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        use ParseTagNameError::*;

        // Look for any characters that are not lowercase ASCII-alphabetic or
        // dashes. If any are found, this is an invalid group name, and the
        // invalid char will be returned in Some().
        raw.chars()
            .find(|&c| !(c.is_ascii_lowercase() || c == '-'))
            .map(|inv| InvalidChar(raw.clone(), inv))
            .err_or(TagName(raw))
    }
}

impl TryFrom<&str> for TagName {
    type Error = ParseTagNameError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::try_from(raw.to_owned())
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum ParseTagNameError {
    #[error("tag name \"{0}\" contains invalid char '{1}'")]
    InvalidChar(String, char),
}

struct TagNameVisitor;

impl Visitor<'_> for TagNameVisitor {
    type Value = TagName;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a string containing only lowercase ASCII-alphabetic characters or dashes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        TagName::try_from(v).map_err(E::custom)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TagNameVisitor)
    }
}
