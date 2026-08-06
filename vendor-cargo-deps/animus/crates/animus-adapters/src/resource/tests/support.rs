use super::super::*;

pub(super) const NAME: &str = "skill-authoring";
pub(super) const SECOND_NAME: &str = "prompt-review";
pub(super) const PROJECT_NAME: &str = "project-local";
pub(super) const DESCRIPTION: &str = "Helps assemble focused agent guidance";
pub(super) const SECOND_DESCRIPTION: &str = "Reviews prompts before dispatch";
pub(super) const PROVENANCE: &str = "bundle:pi-reference";
pub(super) const PROJECT_PROVENANCE: &str = "project:workspace-guidance";
pub(super) const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const SECOND_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const RAW_PATH_HANDLE: &str = "/home/example/.pi/skills/secret/SKILL.md";
pub(super) const UPPERCASE_HASH: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
pub(super) const INVALID_NAME: &str = "Bad Resource";
pub(super) const UNSAFE_PROVENANCE: &str = "/home/example/project/.pi/skill";
pub(super) const UNSUPPORTED_KIND: &str = "theme";
pub(super) const SELECTED_BODY: &str = "# Skill\n\nUse the selected workflow.";
pub(super) const UNRELATED_BODY: &str = "# Other\n\nThis content must stay out of context.";
pub(super) const EMPTY_BODY: &str = "   ";
pub(super) const EXPECTED_SINGLE_ITEM: usize = 1;
pub(super) const EXPECTED_TWO_ITEMS: usize = 2;
pub(super) const EXPECTED_FOUR_ITEMS: usize = 4;
pub(super) const DESCRIPTION_OVERFLOW_CHARS: usize = 1;

#[derive(Clone, Copy)]
pub(super) struct DescriptorFixture {
    pub(super) kind: Kind,
    pub(super) name: &'static str,
    pub(super) description: &'static str,
}

pub(super) fn valid_descriptor() -> Descriptor {
    Descriptor::validate(&valid_draft()).expect("valid resource descriptor should validate")
}

pub(super) fn descriptor_with(fixture: DescriptorFixture) -> Descriptor {
    Descriptor::validate(&Draft {
        kind: Some(fixture.kind.as_str()),
        name: Some(fixture.name),
        description: Some(fixture.description),
        ..valid_draft()
    })
    .expect("descriptor variant should validate")
}

pub(super) fn valid_draft() -> Draft<'static> {
    Draft {
        kind: Some(SKILL_KIND_LABEL),
        name: Some(NAME),
        description: Some(DESCRIPTION),
        scope: Some(GLOBAL_SCOPE_LABEL),
        provenance: Some(PROVENANCE),
        trust_status: Some(TRUSTED_LABEL),
        content_identity: Some(VALID_HASH),
    }
}
