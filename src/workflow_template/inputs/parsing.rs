//! Raw input collection from `--input` arguments and one `--input-file` JSON object. Pure.

/// Collect raw inputs from both sources, unique by name across and within them.
///
/// `cli` holds each `--input NAME=VALUE` argument. `file` holds the `--input-file` bytes,
/// which must be one JSON object. A repeated name fails with [`super::InputError::DuplicateInput`].
pub fn collect(
    cli: &[String],
    file: Option<&[u8]>,
) -> Result<std::collections::BTreeMap<String, super::RawInput>, super::InputError> {
    let mut entries = parse_cli(cli)?;
    if let Some(bytes) = file {
        entries.extend(parse_file(bytes)?);
    }
    let entry_count = entries.len();
    let mut merged = std::collections::BTreeMap::new();
    for (name, value) in entries {
        if merged.insert(name, value).is_some() {
            return Err(super::InputError::DuplicateInput);
        }
    }
    debug_assert_eq!(merged.len(), entry_count);
    Ok(merged)
}

fn parse_cli(cli: &[String]) -> Result<Vec<(String, super::RawInput)>, super::InputError> {
    cli.iter()
        .map(|argument| match argument.split_once('=') {
            Some((name, value)) if !name.is_empty() => Ok((name.to_owned(), super::RawInput::Text(value.to_owned()))),
            Some(_) | None => Err(super::InputError::MalformedInput),
        })
        .collect()
}

fn parse_file(bytes: &[u8]) -> Result<Vec<(String, super::RawInput)>, super::InputError> {
    let object = serde_json::from_slice::<Entries>(bytes).map_err(|_| super::InputError::MalformedFile)?;
    Ok(object.0.into_iter().map(|(name, value)| (name, super::RawInput::Json(value))).collect())
}

/// A JSON object's entries in order, keeping repeated keys so they can be refused.
struct Entries(Vec<(String, serde_json::Value)>);

impl<'de> serde::Deserialize<'de> for Entries {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(EntriesVisitor)
    }
}

struct EntriesVisitor;

impl<'de> serde::de::Visitor<'de> for EntriesVisitor {
    type Value = Entries;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON object of input values")
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries = Vec::with_capacity(map.size_hint().unwrap_or_default().min(super::super::params::MAX_PARAMS));
        while let Some((name, value)) = map.next_entry::<String, serde_json::Value>()? {
            entries.push((name, value));
        }
        Ok(Entries(entries))
    }
}

#[cfg(test)]
mod tests {
    fn collected(cli: &[&str], file: Option<&str>) -> Result<Vec<String>, super::super::InputError> {
        let cli = cli.iter().map(|argument| (*argument).to_owned()).collect::<Vec<_>>();
        super::collect(&cli, file.map(str::as_bytes)).map(|merged| merged.into_keys().collect())
    }

    #[test]
    fn both_sources_merge_and_any_repeated_name_is_refused() {
        assert_eq!(collected(&["a=1", "b=x=y"], Some(r#"{"c": true}"#)).unwrap(), ["a", "b", "c"]);
        assert_eq!(collected(&["a="], None).unwrap(), ["a"]);
        let duplicate = Err(super::super::InputError::DuplicateInput);
        assert_eq!(collected(&["a=1", "a=2"], None), duplicate);
        assert_eq!(collected(&["a=1"], Some(r#"{"a": 1}"#)), duplicate);
        assert_eq!(collected(&[], Some(r#"{"a": 1, "a": 2}"#)), duplicate);
        let malformed = Err(super::super::InputError::MalformedInput);
        assert_eq!(collected(&["novalue"], None), malformed);
        assert_eq!(collected(&["=1"], None), malformed);
        for file in ["[]", "1", "\"a\"", "{", "null"] {
            assert_eq!(collected(&[], Some(file)), Err(super::super::InputError::MalformedFile), "{file}");
        }
    }
}
