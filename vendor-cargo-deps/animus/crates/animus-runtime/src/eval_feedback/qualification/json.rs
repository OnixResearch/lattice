//! Strict canonical JSON subset for qualification attachments.
//!
//! The subset has no whitespace, no string escapes, no signed or fractional
//! numbers, and objects whose keys are unique and sorted by byte order. Strings
//! hold printable ASCII only. Decoding then re-encoding a document must
//! reproduce its exact bytes, so one attachment has exactly one spelling.

use alloc::string::String;
use alloc::vec::Vec;

/// Deepest nesting an attachment uses.
const MAX_DEPTH: usize = 6;
/// Longest string any attachment field may hold.
const MAX_STRING_BYTES: usize = 128;
/// Most items one list or object may hold.
const MAX_ITEMS: usize = 64;
/// Decimal digits of `u64::MAX`.
const MAX_NUMBER_DIGITS: usize = 20;
/// Radix of decimal numbers.
const DECIMAL_RADIX: u64 = 10;

/// One value of the canonical subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(u64),
    Text(String),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

/// Parses `bytes` as exactly one value of the subset.
pub(crate) fn parse(bytes: &[u8]) -> Option<Value> {
    let mut parser = Parser { bytes, at: 0 };
    let value = parser.value(0)?;
    (parser.at == bytes.len()).then_some(value)
}

/// Encodes `value` canonically, sorting object keys by byte order.
pub(crate) fn encode(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    write_value(value, &mut out);
    out
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn expect(&mut self, literal: &[u8]) -> Option<()> {
        let end = self.at.checked_add(literal.len())?;
        (self.bytes.get(self.at..end)? == literal).then(|| self.at = end)
    }

    fn value(&mut self, depth: usize) -> Option<Value> {
        if depth > MAX_DEPTH {
            return None;
        }
        match self.peek()? {
            b'n' => self.expect(b"null").map(|()| Value::Null),
            b't' => self.expect(b"true").map(|()| Value::Bool(true)),
            b'f' => self.expect(b"false").map(|()| Value::Bool(false)),
            b'"' => self.text().map(Value::Text),
            b'[' => self.list(depth),
            b'{' => self.object(depth),
            b'0'..=b'9' => self.number().map(Value::Number),
            _ => None,
        }
    }

    fn number(&mut self) -> Option<u64> {
        let start = self.at;
        let mut value: u64 = 0;
        while let Some(byte @ b'0'..=b'9') = self.peek() {
            if self.at.saturating_sub(start) >= MAX_NUMBER_DIGITS {
                return None;
            }
            value = value
                .checked_mul(DECIMAL_RADIX)?
                .checked_add(u64::from(byte - b'0'))?;
            self.at += 1;
        }
        let digits = self.at.saturating_sub(start);
        let leading_zero = digits > 1 && self.bytes.get(start) == Some(&b'0');
        (!leading_zero).then_some(value)
    }

    fn text(&mut self) -> Option<String> {
        self.expect(b"\"")?;
        let start = self.at;
        while let Some(byte) = self.peek() {
            if byte == b'"' {
                let text = core::str::from_utf8(self.bytes.get(start..self.at)?).ok()?;
                self.at += 1;
                return Some(String::from(text));
            }
            let printable = (b' '..=b'~').contains(&byte) && byte != b'\\';
            if !printable || self.at.saturating_sub(start) >= MAX_STRING_BYTES {
                return None;
            }
            self.at += 1;
        }
        None
    }

    fn list(&mut self, depth: usize) -> Option<Value> {
        self.expect(b"[")?;
        let mut items = Vec::new();
        if self.peek()? == b']' {
            self.at += 1;
            return Some(Value::List(items));
        }
        loop {
            if items.len() >= MAX_ITEMS {
                return None;
            }
            items.push(self.value(depth.checked_add(1)?)?);
            match self.peek()? {
                b',' => self.at += 1,
                b']' => {
                    self.at += 1;
                    return Some(Value::List(items));
                }
                _ => return None,
            }
        }
    }

    fn object(&mut self, depth: usize) -> Option<Value> {
        self.expect(b"{")?;
        let mut entries: Vec<(String, Value)> = Vec::new();
        if self.peek()? == b'}' {
            self.at += 1;
            return Some(Value::Object(entries));
        }
        loop {
            if entries.len() >= MAX_ITEMS {
                return None;
            }
            let key = self.text()?;
            if entries.iter().any(|(existing, _)| *existing == key) {
                return None;
            }
            self.expect(b":")?;
            let value = self.value(depth.checked_add(1)?)?;
            entries.push((key, value));
            match self.peek()? {
                b',' => self.at += 1,
                b'}' => {
                    self.at += 1;
                    return Some(Value::Object(entries));
                }
                _ => return None,
            }
        }
    }
}

fn write_value(value: &Value, out: &mut Vec<u8>) {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(number) => out.extend_from_slice(alloc::format!("{number}").as_bytes()),
        Value::Text(text) => write_text(text, out),
        Value::List(items) => {
            out.push(b'[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_value(item, out);
            }
            out.push(b']');
        }
        Value::Object(entries) => {
            let mut sorted: Vec<&(String, Value)> = entries.iter().collect();
            sorted.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            out.push(b'{');
            for (index, (key, item)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_text(key, out);
                out.push(b':');
                write_value(item, out);
            }
            out.push(b'}');
        }
    }
}

fn write_text(text: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    out.extend_from_slice(text.as_bytes());
    out.push(b'"');
}

/// Typed access to one decoded object; every key must be consumed.
pub(crate) struct Fields {
    entries: Vec<(String, Value)>,
}

impl Fields {
    pub(crate) fn new(value: Value) -> Option<Self> {
        match value {
            Value::Object(entries) => Some(Self { entries }),
            _ => None,
        }
    }

    pub(crate) fn take(&mut self, key: &str) -> Option<Value> {
        let index = self.entries.iter().position(|(name, _)| name == key)?;
        Some(self.entries.swap_remove(index).1)
    }

    pub(crate) fn finish(self) -> Option<()> {
        self.entries.is_empty().then_some(())
    }
}

/// Builds an object value; the encoder sorts its keys.
pub(crate) fn object(entries: Vec<(&str, Value)>) -> Value {
    Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (String::from(key), value))
            .collect(),
    )
}

pub(crate) fn text(value: &str) -> Value {
    Value::Text(String::from(value))
}

pub(crate) fn optional_text(value: Option<&str>) -> Value {
    value.map_or(Value::Null, text)
}

pub(crate) fn optional_number(value: Option<u64>) -> Value {
    value.map_or(Value::Null, Value::Number)
}

pub(crate) fn texts(values: &[String]) -> Value {
    Value::List(values.iter().map(|value| text(value)).collect())
}
