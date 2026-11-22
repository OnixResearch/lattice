// r[verify lattice.implementation_attempt.repair_export.scenario.passing]
// r[verify lattice.implementation_attempt.repair_export.scenario.refused]

use super::super::revision::Record;
use super::super::state::Disposition;
use super::Unexportable;

fn binding(attempt_id: &str, fence: u64) -> super::super::StepBinding {
    super::super::StepBinding {
        envelope: "e".repeat(64),
        work_id: "work-export".into(),
        work_version: 1,
        task_spec_digest: "c".repeat(64),
        attempt_id: attempt_id.into(),
        fence,
    }
}

fn receipt(binding: &super::super::StepBinding, text: &str) -> crate::candidate::CandidateReceipt {
    let tree = |bytes: &[u8]| std::collections::BTreeMap::from([("src/lib.txt".to_owned(), bytes.to_vec())]);
    let fence = binding.fence.to_string();
    let bindings = crate::candidate::Bindings {
        work_id: &binding.work_id,
        attempt_id: &binding.attempt_id,
        fence: &fence,
        workspace_id: "workspace-export",
    };
    crate::candidate::produced_receipt_from_snapshots(bindings, &tree(b"broken"), &tree(text.as_bytes()))
        .expect("a produced receipt")
}

/// The revisions a loop over `binding` records for `texts`, in order.
fn chain(binding: &super::super::StepBinding, texts: &[&str]) -> Vec<Record> {
    let mut parent: Option<String> = None;
    let mut records = Vec::with_capacity(texts.len());
    for (iteration, text) in (1..).zip(texts) {
        let receipt = receipt(binding, text);
        let revision = super::super::revision::identity(super::super::revision::Input {
            binding,
            iteration,
            parent: parent.as_deref(),
            candidate_identity: receipt.candidate_identity.as_deref().expect("candidate"),
            diff_identity: receipt.diff_identity.as_deref().expect("diff"),
        });
        records.push(Record {
            iteration,
            revision: revision.clone(),
            parent: parent.replace(revision),
            receipt,
        });
    }
    records
}

fn candidate<'a>(
    binding: &super::super::StepBinding,
    disposition: &'a Disposition,
    steps: u32,
    revisions: &'a [Record],
) -> Result<&'a Record, Unexportable> {
    super::candidate(binding, super::Retained {
        disposition,
        steps,
        revisions,
    })
}

#[test]
fn a_passing_disposition_names_its_exact_last_revision() {
    let bound = binding("attempt-export", 7);
    let first = chain(&bound, &["fixed"]);
    let correct = Disposition::CorrectCandidate {
        revision: first[0].revision.clone(),
    };
    assert_eq!(candidate(&bound, &correct, 1, &first), Ok(&first[0]));
    let repaired_chain = chain(&bound, &["still broken", "fixed"]);
    let repaired = Disposition::RepairedCandidate {
        revision: repaired_chain[1].revision.clone(),
        repairs: 1,
    };
    assert_eq!(candidate(&bound, &repaired, 2, &repaired_chain), Ok(&repaired_chain[1]));
    assert_eq!(repaired_chain[1].parent.as_deref(), Some(repaired_chain[0].revision.as_str()));
}

#[test]
fn non_passing_dispositions_name_no_candidate() {
    let bound = binding("attempt-export", 7);
    let revisions = chain(&bound, &["still broken"]);
    let stops = [
        Disposition::BudgetExhausted {
            bound: super::super::state::Bound::Iterations,
        },
        Disposition::Cancelled,
        Disposition::Uncertain {
            cause: super::super::state::Uncertainty::CheckOutcome,
        },
        Disposition::NotRepairable {
            classes: vec!["lint".into()],
        },
        Disposition::ChecksRejected {
            defect: super::super::checks::Defect::MissingCheck { check: "unit".into() },
        },
        Disposition::KilnOwned {
            continuation: super::super::KilnOwned::ChangedWork,
        },
        Disposition::FenceSuperseded,
        Disposition::AuthorityWithdrawn {
            cause: super::super::state::Authority::ManagedRevision,
        },
    ];
    for stop in &stops {
        assert_eq!(candidate(&bound, stop, 1, &revisions), Err(Unexportable::NotCandidate), "{stop:?}");
    }
}

#[test]
fn contradicted_or_crossed_records_name_no_candidate() {
    let bound = binding("attempt-export", 7);
    let revisions = chain(&bound, &["still broken", "fixed"]);
    let repaired = |revision: &str, repairs| Disposition::RepairedCandidate {
        revision: revision.to_owned(),
        repairs,
    };
    let last = revisions[1].revision.as_str();
    let cases: [(&str, Disposition, u32, Vec<Record>); 7] = [
        ("an earlier revision", repaired(&revisions[0].revision, 1), 2, revisions.clone()),
        (
            "a correct name for a repair",
            Disposition::CorrectCandidate { revision: last.into() },
            2,
            revisions.clone(),
        ),
        ("no repairs", repaired(last, 0), 2, revisions.clone()),
        ("other step count", repaired(last, 1), 3, revisions.clone()),
        ("an unrecorded revision", repaired(&"f".repeat(64), 1), 2, revisions.clone()),
        ("no revisions", repaired(last, 1), 2, Vec::new()),
        ("a broken parent link", repaired(last, 1), 2, {
            let mut unlinked = revisions.clone();
            unlinked[1].parent = None;
            unlinked
        }),
    ];
    for (name, disposition, steps, records) in &cases {
        assert_eq!(candidate(&bound, disposition, *steps, records), Err(Unexportable::Contradicted), "{name}");
    }
    let mut tampered = revisions.clone();
    tampered[1].receipt.diff_identity = Some("d".repeat(64));
    assert_eq!(candidate(&bound, &repaired(last, 1), 2, &tampered), Err(Unexportable::Contradicted));
    for crossed in [binding("attempt-export", 8), binding("attempt-other", 7)] {
        let foreign = chain(&crossed, &["still broken", "fixed"]);
        let named = repaired(&foreign[1].revision, 1);
        assert_eq!(candidate(&crossed, &named, 2, &foreign), Ok(&foreign[1]), "its own attempt exports it");
        assert_eq!(candidate(&bound, &named, 2, &foreign), Err(Unexportable::Contradicted), "{crossed:?}");
    }
}
