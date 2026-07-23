//! Pure post-effect crash-profile classification tests.

mod common;

use common::PostEffectCrashError;
use common::PostEffectCrashObservation;
use common::PostEffectCrashProfile;
use common::PostEffectIngress;
use common::SCHEDULER_POST_EFFECT_PROFILE;
use common::WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE;
use common::classify_post_effect_crash;
use pretty_assertions::assert_eq;

fn matching_observation(profile: PostEffectCrashProfile) -> PostEffectCrashObservation {
    PostEffectCrashObservation {
        ingress: profile.ingress,
        checkpoint_observed: true,
        pre_restart_run_count: profile.pre_restart_run_count,
        schedule_remains_due: profile.schedule_remains_due,
        final_attempt_count: profile.final_attempt_count,
        final_run_count: profile.final_run_count,
        final_started_history_count: profile.final_started_history_count,
        final_rejected_history_count: profile.final_rejected_history_count,
        http_outcome: profile.http_outcome,
    }
}

// r[verify lattice.repository_validation.post_effect_crash_boundaries]
#[test]
fn reviewed_scheduler_and_webhook_profiles_accept_matching_observations() {
    assert_eq!(
        classify_post_effect_crash(SCHEDULER_POST_EFFECT_PROFILE, matching_observation(SCHEDULER_POST_EFFECT_PROFILE),),
        Ok(()),
    );
    assert_eq!(
        classify_post_effect_crash(
            WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE,
            matching_observation(WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE),
        ),
        Ok(()),
    );
}

// r[verify lattice.repository_validation.post_effect_crash_boundaries.scenario.missing-effect]
#[test]
fn classifier_rejects_death_before_the_effect_checkpoint() {
    let mut observation = matching_observation(SCHEDULER_POST_EFFECT_PROFILE);
    observation.checkpoint_observed = false;

    assert_eq!(
        classify_post_effect_crash(SCHEDULER_POST_EFFECT_PROFILE, observation),
        Err(PostEffectCrashError::MissingCheckpoint),
    );
}

// r[verify lattice.repository_validation.post_effect_crash_boundaries]
#[test]
fn classifier_rejects_premature_terminal_records_and_unreviewed_attempt_counts() {
    let mut premature = matching_observation(SCHEDULER_POST_EFFECT_PROFILE);
    premature.pre_restart_run_count = SCHEDULER_POST_EFFECT_PROFILE.final_run_count;
    let mut repeated_too_many = matching_observation(WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE);
    repeated_too_many.final_attempt_count =
        WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE.final_attempt_count.saturating_add(1);

    assert_eq!(
        classify_post_effect_crash(SCHEDULER_POST_EFFECT_PROFILE, premature),
        Err(PostEffectCrashError::PrematureTerminalRecord),
    );
    assert_eq!(
        classify_post_effect_crash(WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE, repeated_too_many),
        Err(PostEffectCrashError::AttemptCountMismatch),
    );
}

// r[verify lattice.repository_validation.post_effect_crash_boundaries.scenario.non-claim]
#[test]
fn classifier_rejects_cross_ingress_and_unsupported_response_outcomes() {
    let mut cross_ingress = matching_observation(SCHEDULER_POST_EFFECT_PROFILE);
    cross_ingress.ingress = PostEffectIngress::Webhook;
    let mut response_mismatch = matching_observation(WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE);
    response_mismatch.http_outcome = SCHEDULER_POST_EFFECT_PROFILE.http_outcome;

    assert_eq!(
        classify_post_effect_crash(SCHEDULER_POST_EFFECT_PROFILE, cross_ingress),
        Err(PostEffectCrashError::UnsupportedOutcome),
    );
    assert_eq!(
        classify_post_effect_crash(WEBHOOK_POST_EFFECT_DUPLICATE_RISK_PROFILE, response_mismatch),
        Err(PostEffectCrashError::HttpOutcomeMismatch),
    );
}
