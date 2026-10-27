use alloc::{collections::BTreeMap, string::String};

/// One JSON number without lossy integer coercion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtifNumber {
    /// Signed integer.
    Signed(i64),
    /// Unsigned integer.
    Unsigned(u64),
    /// Finite or non-finite floating value supplied through typed construction.
    Float(f64),
}

/// One bounded JSON object used only for ATIF arguments and extensions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtifObject(BTreeMap<String, AtifValue>);

impl AtifObject {
    /// Construct an object from already-unique entries.
    #[must_use]
    pub fn new(entries: BTreeMap<String, AtifValue>) -> Self {
        Self(entries)
    }

    /// Return the object entries in deterministic key order.
    #[must_use]
    pub const fn entries(&self) -> &BTreeMap<String, AtifValue> {
        &self.0
    }

    /// Return the number of keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return true when the object is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// JSON value used by the authority-free ATIF DTOs.
#[derive(Clone, Debug, PartialEq)]
pub enum AtifValue {
    /// JSON null.
    Null,
    /// JSON Boolean.
    Bool(bool),
    /// JSON number.
    Number(AtifNumber),
    /// JSON string.
    String(String),
    /// JSON array.
    Array(alloc::vec::Vec<Self>),
    /// JSON object with deterministic unique keys.
    Object(AtifObject),
}

#[cfg(feature = "json-codec")]
mod serde_impl {
    use super::{AtifNumber, AtifObject, AtifValue};
    use alloc::{collections::BTreeMap, string::String, vec::Vec};
    use core::fmt;
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{Error as _, MapAccess, SeqAccess, Visitor},
        ser::{SerializeMap, SerializeSeq},
    };

    impl Serialize for AtifNumber {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Self::Signed(value) => serializer.serialize_i64(*value),
                Self::Unsigned(value) => serializer.serialize_u64(*value),
                Self::Float(value) => serializer.serialize_f64(*value),
            }
        }
    }

    impl Serialize for AtifObject {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.len()))?;
            for (key, value) in self.entries() {
                map.serialize_entry(key, value)?;
            }
            map.end()
        }
    }

    impl<'de> Deserialize<'de> for AtifObject {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(ObjectVisitor)
        }
    }

    impl Serialize for AtifValue {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                Self::Null => serializer.serialize_none(),
                Self::Bool(value) => serializer.serialize_bool(*value),
                Self::Number(value) => value.serialize(serializer),
                Self::String(value) => serializer.serialize_str(value),
                Self::Array(values) => {
                    let mut sequence = serializer.serialize_seq(Some(values.len()))?;
                    for value in values {
                        sequence.serialize_element(value)?;
                    }
                    sequence.end()
                }
                Self::Object(value) => value.serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for AtifValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ValueVisitor)
        }
    }

    struct ObjectVisitor;

    impl<'de> Visitor<'de> for ObjectVisitor {
        type Value = AtifObject;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a JSON object with unique keys")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut entries = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if entries.contains_key(&key) {
                    return Err(A::Error::custom("duplicate JSON object key"));
                }
                let value = map.next_value::<AtifValue>()?;
                entries.insert(key, value);
            }
            Ok(AtifObject::new(entries))
        }
    }

    struct ValueVisitor;

    impl<'de> Visitor<'de> for ValueVisitor {
        type Value = AtifValue;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a JSON value")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(AtifValue::Null)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(AtifValue::Null)
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
            Ok(AtifValue::Bool(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
            Ok(AtifValue::Number(AtifNumber::Signed(value)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
            Ok(AtifValue::Number(AtifNumber::Unsigned(value)))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
            Ok(AtifValue::Number(AtifNumber::Float(value)))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(AtifValue::String(value.into()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
            Ok(AtifValue::String(value))
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::new();
            while let Some(value) = sequence.next_element::<AtifValue>()? {
                values.push(value);
            }
            Ok(AtifValue::Array(values))
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            ObjectVisitor.visit_map(map).map(AtifValue::Object)
        }
    }
}
