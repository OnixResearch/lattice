use super::*;

mod negative;
mod positive;

const SANDBOX: &str = "workspace-readonly";
const REQUESTED_CAPABILITY: &str = "exec:outside-sandbox";
const ESCALATION_REASON: &str = "process-adapter-request";
const ALLOW_CARGO_ID: &str = "allow-cargo-readonly";
const PROMPT_NIX_ID: &str = "prompt-nix-build";
const FORBID_RM_ID: &str = "forbid-destructive-remove";
const FORBID_CARGO_TEST_ID: &str = "forbid-cargo-test";
const SAFE_RATIONALE: &str = "safe bounded command prefix";
const PROMPT_RATIONALE: &str = "host approval required";
const FORBID_RATIONALE: &str = "destructive prefix denied";
const CARGO_TEST_WRAPPER: &[&str] = &["sh", "-c", "cargo test && rm -rf target"];
const UNSUPPORTED_PIPE_WRAPPER: &[&str] = &["sh", "-c", "cargo test | tee log"];
const RAW_PIPE_SOURCE: &str = "cargo test | tee log";
const RAW_REMOVE_SOURCE: &str = "rm -rf target";

fn sandbox() -> SandboxContext {
    SandboxContext::new(&SandboxContextDraft {
        active_sandbox: Some(SANDBOX),
        requested_capability: Some(REQUESTED_CAPABILITY),
        escalation_reason: Some(ESCALATION_REASON),
    })
    .expect("sandbox fixture should validate")
}

fn command(argv: &'static [&'static str]) -> CommandIntent {
    CommandIntent::new(&CommandIntentDraft { argv }).expect("command fixture should validate")
}

fn evaluate_command(command: &CommandIntent, rules: &[Rule]) -> Evaluation {
    evaluate(Request {
        command,
        sandbox: &sandbox(),
        rules,
    })
}

fn allow_cargo_rule() -> Rule {
    rule(&RuleFixture {
        id: ALLOW_CARGO_ID,
        decision: ALLOW_LABEL,
        rationale: SAFE_RATIONALE,
        command_prefix: &["cargo"],
        positive_examples: &[CommandIntentDraft {
            argv: &["cargo", "metadata"],
        }],
        negative_examples: &[CommandIntentDraft { argv: &["rm"] }],
    })
}

fn prompt_nix_rule() -> Rule {
    rule(&RuleFixture {
        id: PROMPT_NIX_ID,
        decision: PROMPT_LABEL,
        rationale: PROMPT_RATIONALE,
        command_prefix: &["nix"],
        positive_examples: &[CommandIntentDraft {
            argv: &["nix", "build"],
        }],
        negative_examples: &[CommandIntentDraft { argv: &["cargo"] }],
    })
}

fn forbid_rm_rule() -> Rule {
    rule(&RuleFixture {
        id: FORBID_RM_ID,
        decision: FORBID_LABEL,
        rationale: FORBID_RATIONALE,
        command_prefix: &["rm"],
        positive_examples: &[CommandIntentDraft {
            argv: &["rm", "-rf", "target"],
        }],
        negative_examples: &[CommandIntentDraft { argv: &["cargo"] }],
    })
}

fn forbid_cargo_test_rule() -> Rule {
    rule(&RuleFixture {
        id: FORBID_CARGO_TEST_ID,
        decision: FORBID_LABEL,
        rationale: FORBID_RATIONALE,
        command_prefix: &["cargo", "test"],
        positive_examples: &[CommandIntentDraft {
            argv: &["cargo", "test"],
        }],
        negative_examples: &[CommandIntentDraft {
            argv: &["cargo", "metadata"],
        }],
    })
}

struct RuleFixture<'a> {
    id: &'a str,
    decision: &'a str,
    rationale: &'a str,
    command_prefix: &'a [&'a str],
    positive_examples: &'a [CommandIntentDraft<'a>],
    negative_examples: &'a [CommandIntentDraft<'a>],
}

fn rule(fixture: &RuleFixture<'_>) -> Rule {
    Rule::validate(&RuleDraft {
        id: Some(fixture.id),
        decision: Some(fixture.decision),
        rationale: Some(fixture.rationale),
        command_prefix: fixture.command_prefix,
        positive_examples: fixture.positive_examples,
        negative_examples: fixture.negative_examples,
    })
    .expect("rule fixture should validate")
}
