//! SNBT (Stringified NBT) parser and formatter.

use crate::nbt::{Compound, NbtTag};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SnbtError {
    #[error("Unexpected end of SNBT string")]
    UnexpectedEof,
    #[error("Unexpected character '{0}' at index {1}")]
    UnexpectedChar(char, usize),
    #[error("Parse error: {0}")]
    Custom(String),
}

/// Parse SNBT string into an `NbtTag`.
pub fn parse_snbt(input: &str) -> Result<NbtTag, SnbtError> {
    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut pos = 0;
    skip_whitespace(&chars, &mut pos);
    parse_value(&chars, &mut pos)
}

fn skip_whitespace(chars: &[(usize, char)], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].1.is_whitespace() {
        *pos += 1;
    }
}

fn parse_value(chars: &[(usize, char)], pos: &mut usize) -> Result<NbtTag, SnbtError> {
    skip_whitespace(chars, pos);
    if *pos >= chars.len() {
        return Err(SnbtError::UnexpectedEof);
    }

    match chars[*pos].1 {
        '{' => parse_compound(chars, pos),
        '[' => parse_list_or_array(chars, pos),
        '"' | '\'' => parse_quoted_string(chars, pos).map(NbtTag::String),
        _ => parse_literal_or_number(chars, pos),
    }
}

fn parse_compound(chars: &[(usize, char)], pos: &mut usize) -> Result<NbtTag, SnbtError> {
    *pos += 1; // skip '{'
    let mut map: Compound = BTreeMap::new();

    loop {
        skip_whitespace(chars, pos);
        if *pos >= chars.len() {
            return Err(SnbtError::UnexpectedEof);
        }
        if chars[*pos].1 == '}' {
            *pos += 1;
            break;
        }

        let key = parse_key(chars, pos)?;
        skip_whitespace(chars, pos);
        if *pos >= chars.len() || chars[*pos].1 != ':' {
            let (idx, c) = chars.get(*pos).cloned().unwrap_or((0, '\0'));
            return Err(SnbtError::UnexpectedChar(c, idx));
        }
        *pos += 1; // skip ':'

        let val = parse_value(chars, pos)?;
        map.insert(key, val);

        skip_whitespace(chars, pos);
        if *pos < chars.len() && chars[*pos].1 == ',' {
            *pos += 1;
        }
    }
    Ok(NbtTag::Compound(map))
}

fn parse_list_or_array(chars: &[(usize, char)], pos: &mut usize) -> Result<NbtTag, SnbtError> {
    *pos += 1; // skip '['
    skip_whitespace(chars, pos);
    if *pos >= chars.len() {
        return Err(SnbtError::UnexpectedEof);
    }
    if chars[*pos].1 == ']' {
        *pos += 1;
        return Ok(NbtTag::List(Vec::new()));
    }

    // Check array prefix: e.g. [I; 1, 2, 3] or [B; 1b, 2b] or [L; 100L]
    if *pos + 1 < chars.len() && chars[*pos + 1].1 == ';' {
        let prefix = chars[*pos].1;
        *pos += 2;
        match prefix {
            'B' | 'b' => {
                let mut bytes = Vec::new();
                loop {
                    skip_whitespace(chars, pos);
                    if *pos >= chars.len() {
                        return Err(SnbtError::UnexpectedEof);
                    }
                    if chars[*pos].1 == ']' {
                        *pos += 1;
                        break;
                    }
                    if let NbtTag::Byte(b) = parse_value(chars, pos)? {
                        bytes.push(b as u8);
                    }
                    skip_whitespace(chars, pos);
                    if *pos < chars.len() && chars[*pos].1 == ',' {
                        *pos += 1;
                    }
                }
                return Ok(NbtTag::ByteArray(bytes));
            }
            'I' | 'i' => {
                let mut ints = Vec::new();
                loop {
                    skip_whitespace(chars, pos);
                    if *pos >= chars.len() {
                        return Err(SnbtError::UnexpectedEof);
                    }
                    if chars[*pos].1 == ']' {
                        *pos += 1;
                        break;
                    }
                    if let NbtTag::Int(i) = parse_value(chars, pos)? {
                        ints.push(i);
                    }
                    skip_whitespace(chars, pos);
                    if *pos < chars.len() && chars[*pos].1 == ',' {
                        *pos += 1;
                    }
                }
                return Ok(NbtTag::IntArray(ints));
            }
            'L' | 'l' => {
                let mut longs = Vec::new();
                loop {
                    skip_whitespace(chars, pos);
                    if *pos >= chars.len() {
                        return Err(SnbtError::UnexpectedEof);
                    }
                    if chars[*pos].1 == ']' {
                        *pos += 1;
                        break;
                    }
                    if let NbtTag::Long(l) = parse_value(chars, pos)? {
                        longs.push(l);
                    }
                    skip_whitespace(chars, pos);
                    if *pos < chars.len() && chars[*pos].1 == ',' {
                        *pos += 1;
                    }
                }
                return Ok(NbtTag::LongArray(longs));
            }
            _ => {}
        }
    }

    let mut list = Vec::new();
    loop {
        skip_whitespace(chars, pos);
        if *pos >= chars.len() {
            return Err(SnbtError::UnexpectedEof);
        }
        if chars[*pos].1 == ']' {
            *pos += 1;
            break;
        }
        list.push(parse_value(chars, pos)?);
        skip_whitespace(chars, pos);
        if *pos < chars.len() && chars[*pos].1 == ',' {
            *pos += 1;
        }
    }
    Ok(NbtTag::List(list))
}

fn parse_key(chars: &[(usize, char)], pos: &mut usize) -> Result<String, SnbtError> {
    if chars[*pos].1 == '"' || chars[*pos].1 == '\'' {
        parse_quoted_string(chars, pos)
    } else {
        let mut key = String::new();
        while *pos < chars.len() {
            let c = chars[*pos].1;
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '+' {
                key.push(c);
                *pos += 1;
            } else {
                break;
            }
        }
        if key.is_empty() {
            return Err(SnbtError::Custom("Empty compound key".into()));
        }
        Ok(key)
    }
}

fn parse_quoted_string(chars: &[(usize, char)], pos: &mut usize) -> Result<String, SnbtError> {
    let quote = chars[*pos].1;
    *pos += 1;
    let mut out = String::new();
    let mut escaped = false;

    while *pos < chars.len() {
        let c = chars[*pos].1;
        *pos += 1;
        if escaped {
            out.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == quote {
            return Ok(out);
        } else {
            out.push(c);
        }
    }
    Err(SnbtError::UnexpectedEof)
}

fn parse_literal_or_number(chars: &[(usize, char)], pos: &mut usize) -> Result<NbtTag, SnbtError> {
    let mut s = String::new();
    while *pos < chars.len() {
        let c = chars[*pos].1;
        if c.is_alphanumeric() || c == '.' || c == '-' || c == '+' || c == '_' {
            s.push(c);
            *pos += 1;
        } else {
            break;
        }
    }

    if s.eq_ignore_ascii_case("true") {
        return Ok(NbtTag::Byte(1));
    }
    if s.eq_ignore_ascii_case("false") {
        return Ok(NbtTag::Byte(0));
    }

    let last = s.chars().last().unwrap_or('\0');
    match last {
        'b' | 'B' => {
            if let Ok(v) = s[..s.len() - 1].parse::<i8>() {
                return Ok(NbtTag::Byte(v));
            }
        }
        's' | 'S' => {
            if let Ok(v) = s[..s.len() - 1].parse::<i16>() {
                return Ok(NbtTag::Short(v));
            }
        }
        'l' | 'L' => {
            if let Ok(v) = s[..s.len() - 1].parse::<i64>() {
                return Ok(NbtTag::Long(v));
            }
        }
        'f' | 'F' => {
            if let Ok(v) = s[..s.len() - 1].parse::<f32>() {
                return Ok(NbtTag::Float(v));
            }
        }
        'd' | 'D' => {
            if let Ok(v) = s[..s.len() - 1].parse::<f64>() {
                return Ok(NbtTag::Double(v));
            }
        }
        _ => {
            if let Ok(v) = s.parse::<i32>() {
                return Ok(NbtTag::Int(v));
            }
            if let Ok(v) = s.parse::<f64>() {
                return Ok(NbtTag::Double(v));
            }
        }
    }

    Ok(NbtTag::String(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_snbt() {
        let snbt = r#"{name:"HyperX",count:64b,tags:[1,2,3],active:true}"#;
        let tag = parse_snbt(snbt).expect("parse failed");
        let comp = tag.as_compound().expect("expected compound");
        assert_eq!(comp.get("name").unwrap().as_str(), Some("HyperX"));
        assert_eq!(comp.get("count").unwrap().as_i32(), Some(64));
        assert_eq!(comp.get("active").unwrap().as_i32(), Some(1));
    }
}
