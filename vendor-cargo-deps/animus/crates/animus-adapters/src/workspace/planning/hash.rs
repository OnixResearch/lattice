type Capability = crate::workspace::adapter::Capability;
type Descriptor = crate::workspace::intent::Descriptor;

pub(super) const SCHEMA: &str = "animus.workspace-plan.v1";
const SEPARATOR: &str = "|";
const ABSENT_PARENT_LABEL: &str = "none";
const RECEIPT_COUNT_LIMIT: u32 = u32::MAX;

struct Writer {
    text: alloc::string::String,
}

impl Writer {
    fn new() -> Self {
        Self {
            text: alloc::string::String::new(),
        }
    }

    fn push(&mut self, part: &str) {
        if !self.text.is_empty() {
            self.text.push_str(SEPARATOR);
        }
        self.text.push_str(part);
    }

    fn digest(self) -> alloc::string::String {
        alloc::format!("{}", blake3::hash(self.text.as_bytes()).to_hex())
    }
}

pub(super) fn descriptor(descriptor: &Descriptor) -> alloc::string::String {
    debug_assert!(!descriptor.source_id().is_empty());
    debug_assert!(!descriptor.workspace_id().is_empty());
    let mut writer = Writer::new();
    writer.push(SCHEMA);
    writer.push(descriptor.source_id());
    writer.push(descriptor.workspace_id());
    writer.push(descriptor.parent_workspace_id().unwrap_or(ABSENT_PARENT_LABEL));
    for root in descriptor.writable_roots() {
        writer.push(root.as_str());
    }
    writer.push(descriptor.file_inclusion().as_str());
    writer.push(descriptor.operation().as_str());
    writer.push(descriptor.commit_target().as_str());
    writer.push(descriptor.abort_policy().as_str());
    writer.push(descriptor.conflict_policy().as_str());
    writer.push(descriptor.cleanup_policy().as_str());
    for capability in descriptor.required_capabilities() {
        writer.push(capability.as_str());
    }
    writer.digest()
}

pub(super) fn operation(descriptor: &Descriptor) -> alloc::string::String {
    parts(&[
        SCHEMA,
        descriptor.workspace_id(),
        descriptor.operation().as_str(),
        descriptor.commit_target().as_str(),
        descriptor.abort_policy().as_str(),
        descriptor.conflict_policy().as_str(),
        descriptor.cleanup_policy().as_str(),
    ])
}

pub(in crate::workspace) fn parts(parts: &[&str]) -> alloc::string::String {
    debug_assert!(!parts.is_empty());
    let mut writer = Writer::new();
    for part in parts {
        writer.push(part);
    }
    writer.digest()
}

pub(in crate::workspace) fn capability_labels(capabilities: &[Capability]) -> alloc::vec::Vec<alloc::string::String> {
    capabilities.iter().map(|capability| alloc::string::String::from(capability.as_str())).collect()
}

pub(in crate::workspace) fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(RECEIPT_COUNT_LIMIT)
}
