#![allow(unused)]
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<String> for Secret {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<&str> for Secret {
    fn from(val: &str) -> Self {
        Self(val.to_owned())
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("***")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_print_the_plaintext() {
        let secret = Secret::from("secret2");

        assert_eq!(format!("{secret:?}"), "***");
        assert!(!format!("{secret:?}").contains("secret2"));
        assert_eq!(secret.expose(), "secret2")
    }

    #[test]
    fn deserializes_transparently_from_a_bare_string() {
        let secret: Secret = serde_json::from_str("\"s3cr3t\"").unwrap();

        assert_eq!(secret.expose(), "s3cr3t");
        assert_eq!(secret.len(), 6);
        assert!(!secret.is_empty());
    }

    #[test]
    fn nested_debug_output_stays_redacted() {
        #[derive(Debug)]
        struct Holder {
            password: Option<Secret>,
        }

        let holder = Holder {
            password: Some(Secret::from("broker-pw")),
        };

        assert!(!format!("{holder:?}").contains("broker-pw"));
        assert_eq!(holder.password.map(|secret| secret.len()), Some(9));
    }
}
