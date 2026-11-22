//! Records of another attempt, fence or step, or contradicting themselves.

use super::super::super::revision::Record;
use super::super::super::state::Disposition;
use super::super::Unexportable;
use super::Admitted;
use super::Outcome;
use super::binding;
use super::candidate;
use super::failed;
use super::repaired;

#[test]
fn every_recorded_receipt_and_revision_is_this_attempts() {
    let admitted = Admitted::new(binding("attempt-export", 7), super::super::super::tests::policy());
    let (revisions, steps) = admitted.run(&[failed(), Outcome::Passed]);
    let disposition = repaired(&revisions);
    assert_eq!(candidate(&admitted, &disposition, &revisions, &steps), Ok(&revisions[1]));
    type Tamper = fn(&mut Record);
    let tampers: [(&str, Tamper); 6] = [
        ("first receipt's attempt", |record| record.receipt.attempt_id = "attempt-other".into()),
        ("first receipt's fence", |record| record.receipt.fence = "8".into()),
        ("first receipt's workspace", |record| record.receipt.workspace_id = "workspace-other".into()),
        ("first receipt's base", |record| record.receipt.base_identity = "f".repeat(64)),
        ("first diff", |record| record.receipt.diff_identity = Some("d".repeat(64))),
        ("first execute bits", |record| {
            record.executables.insert("src/lib.txt".into(), 0o100);
        }),
    ];
    for (name, tamper) in tampers {
        let (mut revisions, mut steps) = (revisions.clone(), steps.clone());
        tamper(&mut revisions[0]);
        steps[0].candidate = Some(revisions[0].clone());
        assert_eq!(candidate(&admitted, &disposition, &revisions, &steps), Err(Unexportable::Contradicted), "{name}");
    }
    let mut unlisted = revisions.clone();
    unlisted[1].base_files.clear();
    let mut unlisted_steps = steps.clone();
    unlisted_steps[1].candidate = Some(unlisted[1].clone());
    let named = repaired(&unlisted);
    assert_eq!(candidate(&admitted, &named, &unlisted, &unlisted_steps), Err(Unexportable::Contradicted));
    for crossed in [binding("attempt-export", 8), binding("attempt-other", 7)] {
        let foreign = Admitted::new(crossed, super::super::super::tests::policy());
        let (revisions, steps) = foreign.run(&[failed(), Outcome::Passed]);
        let named = repaired(&revisions);
        assert_eq!(candidate(&foreign, &named, &revisions, &steps), Ok(&revisions[1]), "its own attempt");
        assert_eq!(candidate(&admitted, &named, &revisions, &steps), Err(Unexportable::Contradicted));
    }
}

#[test]
fn contradicted_records_name_no_candidate() {
    let admitted = Admitted::new(binding("attempt-export", 7), super::super::super::tests::policy());
    let (revisions, steps) = admitted.run(&[failed(), Outcome::Passed]);
    let last = revisions[1].revision.as_str();
    let named = |revision: &str, repairs| Disposition::RepairedCandidate {
        revision: revision.to_owned(),
        repairs,
    };
    let cases = [
        ("an earlier revision", named(&revisions[0].revision, 1)),
        ("a correct name for a repair", Disposition::CorrectCandidate { revision: last.into() }),
        ("no repairs", named(last, 0)),
        ("other repair count", named(last, 2)),
        ("an unrecorded revision", named(&"f".repeat(64), 1)),
    ];
    for (name, disposition) in &cases {
        assert_eq!(candidate(&admitted, disposition, &revisions, &steps), Err(Unexportable::Contradicted), "{name}");
    }
    let disposition = named(last, 1);
    assert_eq!(candidate(&admitted, &disposition, &[], &[]), Err(Unexportable::Contradicted), "no records");
    assert_eq!(candidate(&admitted, &disposition, &revisions, &steps[..1]), Err(Unexportable::Contradicted));
    let mut unlinked = steps.clone();
    unlinked[1].step.parent = None;
    assert_eq!(candidate(&admitted, &disposition, &revisions, &unlinked), Err(Unexportable::Contradicted));
    let mut unfailed = steps.clone();
    unfailed[1].step.failures.clear();
    assert_eq!(candidate(&admitted, &disposition, &revisions, &unfailed), Err(Unexportable::Contradicted));
}
