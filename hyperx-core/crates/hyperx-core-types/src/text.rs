//! Minecraft Text Component parser and representation.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextComponent {
    String(String),
    Structured(Box<StructuredComponent>),
    Array(Vec<TextComponent>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredComponent {
    #[serde(default)]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<TextComponent>,
}

impl TextComponent {
    pub fn literal(text: impl Into<String>) -> Self {
        TextComponent::Structured(Box::new(StructuredComponent {
            text: text.into(),
            ..Default::default()
        }))
    }

    /// Extract flat plain-text string representation (stripping styling).
    pub fn to_plain_text(&self) -> String {
        let mut buf = String::new();
        self.append_plain_text(&mut buf);
        buf
    }

    fn append_plain_text(&self, out: &mut String) {
        match self {
            TextComponent::String(s) => out.push_str(s),
            TextComponent::Structured(comp) => {
                out.push_str(&comp.text);
                for sub in &comp.extra {
                    sub.append_plain_text(out);
                }
            }
            TextComponent::Array(arr) => {
                for item in arr {
                    item.append_plain_text(out);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_component_json() {
        let json = r#"{"text":"Hello, ","color":"green","extra":[{"text":"HyperX!","bold":true}]}"#;
        let comp: TextComponent = serde_json::from_str(json).unwrap();
        assert_eq!(comp.to_plain_text(), "Hello, HyperX!");
    }
}
