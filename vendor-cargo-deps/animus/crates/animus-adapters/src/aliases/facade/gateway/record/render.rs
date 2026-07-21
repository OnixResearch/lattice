const RECEIPT_FIELD_SEPARATOR: char = '\n';
const RECEIPT_KEY_SEPARATOR: &str = "=";

#[derive(Clone, Copy)]
pub(super) struct LabelPayload<'a> {
    pub(super) label: &'a str,
    pub(super) value: &'a str,
}

#[derive(Clone, Copy)]
struct Field<'a> {
    key: &'a str,
    value: &'a str,
}

struct Payload {
    text: alloc::string::String,
}

impl Payload {
    fn new() -> Self {
        Self {
            text: alloc::string::String::new(),
        }
    }

    fn push(&mut self, field: Field<'_>) {
        debug_assert!(!field.key.is_empty());
        debug_assert!(!field.value.is_empty());
        self.text.push_str(field.key);
        self.text.push_str(RECEIPT_KEY_SEPARATOR);
        self.text.push_str(field.value);
        self.text.push(RECEIPT_FIELD_SEPARATOR);
    }

    fn push_hash(&mut self, key: &'static str, hash: Option<&crate::ArtifactHash>) {
        if let Some(value) = hash {
            self.push(Field {
                key,
                value: value.as_str(),
            });
        }
    }

    fn into_text(self) -> alloc::string::String {
        self.text
    }
}

pub(super) fn label_payload(input: LabelPayload<'_>) -> alloc::string::String {
    debug_assert!(!input.label.is_empty());
    debug_assert!(!input.value.is_empty());
    let mut payload = Payload::new();
    payload.push(Field {
        key: "label",
        value: input.label,
    });
    payload.push(Field {
        key: "value",
        value: input.value,
    });
    payload.into_text()
}

pub(super) fn safe_text(receipt: &super::AdmissionReceipt) -> alloc::string::String {
    debug_assert!(!receipt.resource().is_empty());
    debug_assert!(!receipt.ability().is_empty());
    let mut payload = Payload::new();
    payload.push(Field {
        key: "verdict",
        value: receipt.verdict().as_str(),
    });
    if let Some(reason) = receipt.reason() {
        payload.push(Field {
            key: "reason",
            value: reason.as_str(),
        });
    }
    payload.push(Field {
        key: "entrypoint",
        value: receipt.entrypoint().as_str(),
    });
    payload.push(Field {
        key: "action",
        value: receipt.action().as_str(),
    });
    payload.push(Field {
        key: "resource",
        value: receipt.resource(),
    });
    payload.push(Field {
        key: "ability",
        value: receipt.ability(),
    });
    push_hashes(&mut payload, receipt);
    payload.push(Field {
        key: "caveat",
        value: receipt.caveat().as_str(),
    });
    push_status(&mut payload, receipt);
    payload.push(Field {
        key: "receipt_hash",
        value: receipt.receipt_hash().as_str(),
    });
    payload.into_text()
}

fn push_hashes(payload: &mut Payload, receipt: &super::AdmissionReceipt) {
    payload.push(Field {
        key: "audience_hash",
        value: receipt.audience_hash().as_str(),
    });
    payload.push(Field {
        key: "actor_hash",
        value: receipt.actor_hash().as_str(),
    });
    payload.push_hash("credential_hash", receipt.credential_hash());
    payload.push_hash("policy_hash", receipt.policy_hash());
}

fn push_status(payload: &mut Payload, receipt: &super::AdmissionReceipt) {
    if let Some(expiry) = receipt.expiry() {
        payload.push(Field {
            key: "expiry",
            value: expiry.as_str(),
        });
    }
    if let Some(status) = receipt.verifier_status() {
        payload.push(Field {
            key: "verifier",
            value: status.as_str(),
        });
    }
}
