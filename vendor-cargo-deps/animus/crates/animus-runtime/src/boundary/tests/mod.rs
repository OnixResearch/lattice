use super::*;

mod negative;
mod positive;

const SANDBOX: &str = "workspace-readonly";
const CAPABILITY: &str = "exec:outside-sandbox";
const REASON: &str = "sandbox-escalation";
const ALLOW_CARGO_ID: &str = "allow-cargo";
const PROMPT_NIX_ID: &str = "prompt-nix";
const FORBID_RM_ID: &str = "forbid-rm";
const RATIONALE: &str = "bounded fixture rationale";
const SECRET_COMMAND: &str = "secret-tool";
const SECRET_TOKEN: &str = "token-value";
const PIPE_WRAPPER_SOURCE: &str = "cargo test | tee log";

fn sandbox() -> animus_adapters::ExecPolicySandboxContext {
    animus_adapters::ExecPolicySandboxContext::new(&animus_adapters::ExecPolicySandboxContextDraft {
        active_sandbox: Some(SANDBOX),
        requested_capability: Some(CAPABILITY),
        escalation_reason: Some(REASON),
    })
    .expect("sandbox context should validate")
}

fn command(argv: &'static [&'static str]) -> animus_adapters::ExecPolicyCommandIntent {
    animus_adapters::ExecPolicyCommandIntent::new(&animus_adapters::ExecPolicyCommandIntentDraft { argv })
        .expect("command should validate")
}

fn event_for(
    command: &animus_adapters::ExecPolicyCommandIntent,
    rules: &[animus_adapters::ExecPolicyRule],
) -> crate::RuntimeEvent {
    evaluate_sandbox_escalation(animus_adapters::ExecPolicyRequest {
        command,
        sandbox: &sandbox(),
        rules,
    })
}

fn allow_cargo_rule() -> animus_adapters::ExecPolicyRule {
    rule(&RuleFixture {
        id: ALLOW_CARGO_ID,
        decision: "allow",
        command_prefix: &["cargo"],
        positive_examples: &[animus_adapters::ExecPolicyCommandIntentDraft {
            argv: &["cargo", "metadata"],
        }],
        negative_examples: &[animus_adapters::ExecPolicyCommandIntentDraft { argv: &["rm"] }],
    })
}

fn prompt_nix_rule() -> animus_adapters::ExecPolicyRule {
    rule(&RuleFixture {
        id: PROMPT_NIX_ID,
        decision: "prompt",
        command_prefix: &["nix"],
        positive_examples: &[animus_adapters::ExecPolicyCommandIntentDraft {
            argv: &["nix", "build"],
        }],
        negative_examples: &[animus_adapters::ExecPolicyCommandIntentDraft { argv: &["cargo"] }],
    })
}

fn forbid_rm_rule() -> animus_adapters::ExecPolicyRule {
    rule(&RuleFixture {
        id: FORBID_RM_ID,
        decision: "forbid",
        command_prefix: &["rm"],
        positive_examples: &[animus_adapters::ExecPolicyCommandIntentDraft {
            argv: &["rm", "-rf", "target"],
        }],
        negative_examples: &[animus_adapters::ExecPolicyCommandIntentDraft { argv: &["cargo"] }],
    })
}

struct RuleFixture<'a> {
    id: &'a str,
    decision: &'a str,
    command_prefix: &'a [&'a str],
    positive_examples: &'a [animus_adapters::ExecPolicyCommandIntentDraft<'a>],
    negative_examples: &'a [animus_adapters::ExecPolicyCommandIntentDraft<'a>],
}

fn rule(fixture: &RuleFixture<'_>) -> animus_adapters::ExecPolicyRule {
    animus_adapters::ExecPolicyRule::validate(&animus_adapters::ExecPolicyRuleDraft {
        id: Some(fixture.id),
        decision: Some(fixture.decision),
        rationale: Some(RATIONALE),
        command_prefix: fixture.command_prefix,
        positive_examples: fixture.positive_examples,
        negative_examples: fixture.negative_examples,
    })
    .expect("rule should validate")
}
