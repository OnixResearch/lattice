use super::*;

const CONTRACT: &str = "animus.runtime";
const MODEL_RESOURCE: &str = "model:default";
const MODEL_PREFIX: &str = "model:";
const MODEL_ABILITY: &str = "call";
const TOOL_RESOURCE: &str = "tool:read";
const TOOL_PREFIX: &str = "tool:";
const TOOL_ABILITY: &str = "invoke";
const FILE_RESOURCE: &str = "fs:workspace/readme";
const FILE_PREFIX: &str = "fs:workspace/";
const FILE_ABILITY: &str = "read";
const EXEC_RESOURCE: &str = "exec:cargo-test";
const EXEC_PREFIX: &str = "exec:";
const EXEC_ABILITY: &str = "run";
const NETWORK_RESOURCE: &str = "net:api.example";
const NETWORK_PREFIX: &str = "net:";
const NETWORK_ABILITY: &str = "connect";
const STEEL_PREFIX: &str = "steel:";
const STEEL_ABILITY: &str = "evaluate";
const AGENT_ROLE_PREFIX: &str = "role:";
const AGENT_ROLE_ABILITY: &str = "activate";
const OTHER_ABILITY: &str = "delete";
const EMPTY_FIELD: &str = " ";
const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SAFE_RECEIPT_TEXT: &str = "status=allowed issue=none resource_hash=abc redaction=hash-only";
const LEAKING_RECEIPT_TEXT: &str = "status=allowed RAW_PROMPT=send credential=secret";
const GUARDED_ACTION_COUNT: usize = 7;

macro_rules! capability_request {
    ($action:expr, $resource:expr, $ability:expr) => {
        CapabilityRequest::new($action, CONTRACT, $resource, $ability)
            .expect("request should build")
    };
}

macro_rules! policy_contract {
    ($action:expr, $resource_prefix:expr, $ability:expr) => {
        CapabilityPolicyContract::new($action, CONTRACT, $resource_prefix, $ability)
            .expect("contract should build")
    };
}

macro_rules! ucan_grant {
    ($resource_prefix:expr, $ability:expr) => {
        UcanGrant::new(CONTRACT, $resource_prefix, $ability).expect("grant should build")
    };
}

mod equivalence;
mod negative;
mod positive;

fn authorize(context: AuthorizationContext<'_>) -> AuthorizationReceipt {
    authorize_capability(&context)
}

fn safe_hash() -> super::super::SafeHash {
    super::super::SafeHash::new(VALID_HASH).expect("hash should parse")
}
