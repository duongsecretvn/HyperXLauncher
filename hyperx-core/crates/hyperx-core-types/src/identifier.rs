//! Minecraft Identifier / NamespacedKey implementation (e.g., `minecraft:dirt`).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum IdentifierError {
    #[error("Invalid character in namespace: '{0}'")]
    InvalidNamespaceChar(char),
    #[error("Invalid character in path: '{0}'")]
    InvalidPathChar(char),
    #[error("Empty namespace or path")]
    EmptyComponent,
    #[error("Too many colons in identifier string")]
    TooManyColons,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";

    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Result<Self, IdentifierError> {
        let ns = namespace.into();
        let p = path.into();
        Self::validate_namespace(&ns)?;
        Self::validate_path(&p)?;
        Ok(Self {
            namespace: ns,
            path: p,
        })
    }

    /// Construct an identifier in the default `minecraft` namespace.
    pub fn minecraft(path: impl Into<String>) -> Result<Self, IdentifierError> {
        Self::new(Self::DEFAULT_NAMESPACE, path)
    }

    /// Construct an unchecked identifier for hot paths where keys are static.
    pub const fn new_unchecked(namespace: String, path: String) -> Self {
        Self { namespace, path }
    }

    #[inline]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[inline]
    pub fn is_minecraft(&self) -> bool {
        self.namespace == Self::DEFAULT_NAMESPACE
    }

    fn validate_namespace(ns: &str) -> Result<(), IdentifierError> {
        if ns.is_empty() {
            return Err(IdentifierError::EmptyComponent);
        }
        for c in ns.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' || c == '.') {
                return Err(IdentifierError::InvalidNamespaceChar(c));
            }
        }
        Ok(())
    }

    fn validate_path(path: &str) -> Result<(), IdentifierError> {
        if path.is_empty() {
            return Err(IdentifierError::EmptyComponent);
        }
        for c in path.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' || c == '.' || c == '/') {
                return Err(IdentifierError::InvalidPathChar(c));
            }
        }
        Ok(())
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(':') {
            Some((ns, path)) => {
                if path.contains(':') {
                    return Err(IdentifierError::TooManyColons);
                }
                Self::new(ns, path)
            }
            None => Self::minecraft(s),
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({}:{})", self.namespace, self.path)
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Identifier::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_parsing() {
        let id = Identifier::from_str("stone").unwrap();
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");

        let id2 = Identifier::from_str("hyperx:custom_block/variant").unwrap();
        assert_eq!(id2.namespace(), "hyperx");
        assert_eq!(id2.path(), "custom_block/variant");
    }
}
