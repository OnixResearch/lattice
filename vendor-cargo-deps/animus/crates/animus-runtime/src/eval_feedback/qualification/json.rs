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

/// A container whose members are still being parsed.
enum Open {
    List(Vec<Value>),
    Object {
        entries: Vec<(String, Value)>,
        /// Key of the member being parsed.
        key: String,
    },
}

impl Open {
    fn into_value(self) -> Value {
        match self {
            Self::List(items) => Value::List(items),
            Self::Object { entries, .. } => Value::Object(entries),
        }
    }
}

/// What starting a value produced.
enum Started {
    /// A scalar or an empty container.
    Complete(Value),
    /// A non-empty container was opened; its first member follows.
    Opened,
}

/// What attaching a complete value to the innermost open container produced.
enum Attached {
    /// The container continues; its next member follows.
    NextMember,
    /// The container closed into this value.
    Closed(Value),
    /// No container is open; this is the document.
    Root(Value),
}

/// Parses `bytes` as exactly one value of the subset.
///
/// The parser is iterative: an explicit stack holds the open containers, and
/// a value nested deeper than [`MAX_DEPTH`] is refused before it is read.
pub(crate) fn parse(bytes: &[u8]) -> Option<Value> {
    let mut parser = Parser { bytes, at: 0 };
    let mut open: Vec<Open> = Vec::new();
    loop {
        let mut value = match parser.start(&mut open)? {
            Started::Complete(value) => value,
            Started::Opened => continue,
        };
        loop {
            match parser.attach(&mut open, value)? {
                Attached::NextMember => break,
                Attached::Closed(closed) => value = closed,
                Attached::Root(root) => return (parser.at == bytes.len()).then_some(root),
            }
        }
    }
}

/// One pending piece of canonical output.
enum Emit<'a> {
    Value(&'a Value),
    Byte(u8),
    Text(&'a str),
}

/// Encodes `value` canonically, sorting object keys by byte order.
///
/// The encoder is iterative: containers schedule their members on an explicit
/// stack in reverse order.
pub(crate) fn encode(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pending: Vec<Emit<'_>> = alloc::vec![Emit::Value(value)];
    while let Some(next) = pending.pop() {
        match next {
            Emit::Value(value) => write_value(value, &mut out, &mut pending),
            Emit::Byte(byte) => out.push(byte),
            Emit::Text(text) => write_text(text, &mut out),
        }
    }
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

    /// Starts the value at the cursor, nested inside `open`. Scalars and empty
    /// containers complete at once; a non-empty container is pushed on `open`.
    fn start(&mut self, open: &mut Vec<Open>) -> Option<Started> {
        if open.len() > MAX_DEPTH {
            return None;
        }
        match self.peek()? {
            b'n' => self
                .expect(b"null")
                .map(|()| Started::Complete(Value::Null)),
            b't' => self
                .expect(b"true")
                .map(|()| Started::Complete(Value::Bool(true))),
            b'f' => self
                .expect(b"false")
                .map(|()| Started::Complete(Value::Bool(false))),
            b'"' => self.text().map(|text| Started::Complete(Value::Text(text))),
            b'[' => {
                self.expect(b"[")?;
                if self.peek()? == b']' {
                    self.at += 1;
                    return Some(Started::Complete(Value::List(Vec::new())));
                }
                open.push(Open::List(Vec::new()));
                Some(Started::Opened)
            }
            b'{' => {
                self.expect(b"{")?;
                if self.peek()? == b'}' {
                    self.at += 1;
                    return Some(Started::Complete(Value::Object(Vec::new())));
                }
                let key = self.member_key(&[])?;
                open.push(Open::Object {
                    entries: Vec::new(),
                    key,
                });
                Some(Started::Opened)
            }
            b'0'..=b'9' => self
                .number()
                .map(|number| Started::Complete(Value::Number(number))),
            _ => None,
        }
    }

    /// Attaches the complete `value` to the innermost open container, then
    /// reads the separator that continues or closes it.
    fn attach(&mut self, open: &mut Vec<Open>, value: Value) -> Option<Attached> {
        match open.last_mut() {
            None => Some(Attached::Root(value)),
            Some(Open::List(items)) => {
                items.push(value);
                match self.peek()? {
                    b',' => {
                        self.at += 1;
                        (items.len() < MAX_ITEMS).then_some(Attached::NextMember)
                    }
                    b']' => {
                        self.at += 1;
                        open.pop()
                            .map(|closed| Attached::Closed(closed.into_value()))
                    }
                    _ => None,
                }
            }
            Some(Open::Object { entries, key }) => {
                entries.push((core::mem::take(key), value));
                match self.peek()? {
                    b',' => {
                        self.at += 1;
                        *key = self.member_key(entries)?;
                        Some(Attached::NextMember)
                    }
                    b'}' => {
                        self.at += 1;
                        open.pop()
                            .map(|closed| Attached::Closed(closed.into_value()))
                    }
                    _ => None,
                }
            }
        }
    }

    /// Reads one object member's unique key and its `:`, refusing a member
    /// beyond [`MAX_ITEMS`].
    fn member_key(&mut self, entries: &[(String, Value)]) -> Option<String> {
        if entries.len() >= MAX_ITEMS {
            return None;
        }
        let key = self.text()?;
        if entries.iter().any(|(existing, _)| *existing == key) {
            return None;
        }
        self.expect(b":")?;
        Some(key)
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
}

/// Writes a scalar, or writes a container's opening byte and schedules its
/// members and closing byte on `pending` in reverse order.
fn write_value<'a>(value: &'a Value, out: &mut Vec<u8>, pending: &mut Vec<Emit<'a>>) {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(number) => out.extend_from_slice(alloc::format!("{number}").as_bytes()),
        Value::Text(text) => write_text(text, out),
        Value::List(items) => {
            out.push(b'[');
            pending.push(Emit::Byte(b']'));
            for (index, item) in items.iter().enumerate().rev() {
                pending.push(Emit::Value(item));
                if index > 0 {
                    pending.push(Emit::Byte(b','));
                }
            }
        }
        Value::Object(entries) => {
            let mut sorted: Vec<&(String, Value)> = entries.iter().collect();
            sorted.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            out.push(b'{');
            pending.push(Emit::Byte(b'}'));
            for (index, (key, item)) in sorted.into_iter().enumerate().rev() {
                pending.push(Emit::Value(item));
                pending.push(Emit::Byte(b':'));
                pending.push(Emit::Text(key));
                if index > 0 {
                    pending.push(Emit::Byte(b','));
                }
            }
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
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::Text(_) | Value::List(_) => {
                None
            }
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
