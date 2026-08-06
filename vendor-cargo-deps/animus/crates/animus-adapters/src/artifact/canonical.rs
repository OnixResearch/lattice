use alloc::string::ToString;

const PREFIX: &str = "animus.artifact.envelope";
const VERSION_FIELD: &str = "version";
const KIND_FIELD: &str = "kind";
const REDACTION_FIELD: &str = "redaction";
const PAYLOAD_FIELD: &str = "payload";
const DEPENDENCY_FIELD: &str = "dependency";
const LENGTH_SEPARATOR: &str = ":";
const FIELD_SEPARATOR: char = '\n';
const NUL_CHARACTER: char = '\0';
const ORDER_PAIR_LEN: usize = 2;
const ORDER_LEFT_INDEX: usize = 0;
const ORDER_RIGHT_INDEX: usize = 1;

#[derive(Clone, Copy)]
struct Field<'a> {
    label: &'static str,
    value: &'a str,
}

struct Writer {
    text: alloc::string::String,
}

impl Writer {
    fn new() -> Self {
        Self {
            text: alloc::string::String::new(),
        }
    }

    fn push(&mut self, field: Field<'_>) {
        assert!(!field.label.is_empty(), "canonical field label must not be empty");
        assert!(field.value.len() >= field.value.chars().count(), "field byte length covers chars");
        self.text.push_str(field.label);
        self.text.push_str(LENGTH_SEPARATOR);
        self.text.push_str(&field.value.len().to_string());
        self.text.push_str(LENGTH_SEPARATOR);
        self.text.push_str(field.value);
        self.text.push(FIELD_SEPARATOR);
    }

    fn into_string(self) -> alloc::string::String {
        self.text
    }
}

pub(super) fn text(
    kind: super::Kind,
    redaction: super::RedactionClass,
    payload: &str,
    dependencies: &[super::Hash],
) -> alloc::string::String {
    assert!(!payload.contains(NUL_CHARACTER), "canonical payload must already be NUL-free");
    assert!(
        dependencies.windows(ORDER_PAIR_LEN).all(|pair| pair[ORDER_LEFT_INDEX] <= pair[ORDER_RIGHT_INDEX]),
        "canonical dependencies must be sorted"
    );
    let mut writer = Writer::new();
    writer.push(Field {
        label: PREFIX,
        value: "",
    });
    writer.push(Field {
        label: VERSION_FIELD,
        value: &super::env::VERSION.to_string(),
    });
    writer.push(Field {
        label: KIND_FIELD,
        value: kind.as_str(),
    });
    writer.push(Field {
        label: REDACTION_FIELD,
        value: redaction.as_str(),
    });
    writer.push(Field {
        label: PAYLOAD_FIELD,
        value: payload,
    });
    for dependency in dependencies {
        writer.push(Field {
            label: DEPENDENCY_FIELD,
            value: dependency.as_str(),
        });
    }
    writer.into_string()
}
