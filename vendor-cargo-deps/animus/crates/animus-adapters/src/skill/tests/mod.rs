use super::*;

mod negative;
mod positive;

const NAME: &str = "hermes-authoring";
const SECOND_NAME: &str = "review-reports";
const UNTRUSTED_NAME: &str = "project-local";
const MISSING_RELATED_NAME: &str = "missing-skill";
const DESCRIPTION: &str = "Helps authors write compact workflow skills";
const SECOND_DESCRIPTION: &str = "Summarizes review report workflow guidance";
const VERSION: &str = "1.2.3";
const BAD_VERSION: &str = "1.latest";
const CATEGORY: &str = "software-development";
const DATABASE_CATEGORY: &str = "database-bundle";
const TAG: &str = "workflow";
const SECOND_TAG: &str = "guidance";
const PROVENANCE: &str = "bundle:hermes-reference";
const DATABASE_PROVENANCE: &str = "database:trusted-skills";
const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const UPPERCASE_HASH: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const RAW_PATH_HANDLE: &str = "/home/example/.hermes/skills/secret/SKILL.md";
const INVALID_NAME: &str = "Bad Skill";
const INVALID_RELATED_NAME: &str = "../escape";
const SELECTED_BODY: &str = "# Skill\n\nUse the selected authoring workflow.";
const UNRELATED_BODY: &str = "# Other\n\nThis body must stay out of context.";
const EMPTY_BODY: &str = "   ";
const EXPECTED_SINGLE_ITEM: usize = 1;
const EXPECTED_TWO_ITEMS: usize = 2;
const DESCRIPTION_OVERFLOW_CHARS: usize = 1;

fn valid_draft() -> Draft<'static> {
    Draft {
        name: Some(NAME),
        description: Some(DESCRIPTION),
        version: Some(VERSION),
        category: Some(CATEGORY),
        tags: &[SECOND_TAG, TAG],
        related_skills: &[],
        provenance: Some(PROVENANCE),
        trust_status: Some(TRUSTED_LABEL),
        content_identity: Some(VALID_HASH),
    }
}

fn second_draft() -> Draft<'static> {
    Draft {
        name: Some(SECOND_NAME),
        description: Some(SECOND_DESCRIPTION),
        content_identity: Some(SECOND_HASH),
        related_skills: &[NAME],
        ..valid_draft()
    }
}

fn untrusted_draft() -> Draft<'static> {
    Draft {
        name: Some(UNTRUSTED_NAME),
        trust_status: Some(UNTRUSTED_LABEL),
        ..valid_draft()
    }
}

fn database_draft() -> Draft<'static> {
    Draft {
        category: Some(DATABASE_CATEGORY),
        provenance: Some(DATABASE_PROVENANCE),
        ..valid_draft()
    }
}

fn valid_manifest() -> Descriptor {
    Descriptor::validate(&valid_draft()).expect("valid skill manifest should validate")
}

fn second_manifest() -> Descriptor {
    Descriptor::validate(&second_draft()).expect("second skill manifest should validate")
}

fn untrusted_manifest() -> Descriptor {
    Descriptor::validate(&untrusted_draft()).expect("untrusted skill manifest should validate")
}
