use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;

use lattice_workflow_contract::Identity;

use super::AdoptionReads;
use super::WaitingAdoptions;

#[derive(Debug, PartialEq, Eq)]
struct Failed;

/// An adoption store that counts its reads: a scan finds `waiting`, and loading an execution
/// starts it, so it waits no longer.
#[derive(Default)]
struct Counted {
    generation: Cell<u64>,
    waiting: RefCell<BTreeMap<u64, Identity>>,
    is_load_failing: Cell<bool>,
    scans: Cell<usize>,
    loads: Cell<usize>,
}

impl Counted {
    fn adopt(&self, run_id: u64) {
        self.waiting.borrow_mut().insert(run_id, execution(run_id));
        self.generation.set(self.generation.get() + 1);
    }

    /// Scans and loads so far.
    fn reads(&self) -> (usize, usize) {
        (self.scans.get(), self.loads.get())
    }
}

impl AdoptionReads for Counted {
    type Waiting = u64;
    type Error = Failed;

    fn generation(&self) -> Result<u64, Failed> {
        Ok(self.generation.get())
    }

    fn scan(&self) -> Result<Vec<(u64, Identity)>, Failed> {
        self.scans.set(self.scans.get() + 1);
        Ok(self.waiting.borrow().iter().map(|(run_id, execution)| (*run_id, execution.clone())).collect())
    }

    fn load(&self, execution: &Identity) -> Result<Option<u64>, Failed> {
        self.loads.set(self.loads.get() + 1);
        if self.is_load_failing.get() {
            return Err(Failed);
        }
        let mut waiting = self.waiting.borrow_mut();
        let run_id = waiting.iter().find(|(_run_id, candidate)| *candidate == execution).map(|(run_id, _)| *run_id);
        Ok(run_id.filter(|run_id| waiting.remove(run_id).is_some()))
    }
}

fn execution(run_id: u64) -> Identity {
    Identity::new(format!("{}{run_id:064}", lattice_workflow_contract::BLAKE3_PREFIX)).expect("test identity")
}

#[test]
fn a_newly_waiting_adoption_is_taken_once_a_slot_frees() {
    let reads = Counted::default();
    reads.adopt(1);
    reads.adopt(2);
    let mut index = WaitingAdoptions::default();
    assert_eq!(index.take(&reads, 1), Ok(vec![1]), "the lowest run id starts first");
    reads.adopt(3);
    assert_eq!(index.take(&reads, 0), Ok(vec![]), "no slot is free");
    assert_eq!(index.take(&reads, 2), Ok(vec![2, 3]), "an adoption added after the index was built is taken");
    assert_eq!(reads.reads(), (2, 3), "one rescan for the one new adoption");
}

#[test]
fn no_free_slot_or_no_new_adoption_reads_no_adoption() {
    let reads = Counted::default();
    reads.adopt(1);
    let mut index = WaitingAdoptions::default();
    assert_eq!(index.take(&reads, 0), Ok(vec![]));
    assert_eq!(reads.reads(), (0, 0), "a request with no free slot reads no adoption");
    assert_eq!(index.take(&reads, 2), Ok(vec![1]));
    assert_eq!(reads.reads(), (1, 1));
    for _ in 0..3 {
        assert_eq!(index.take(&reads, 2), Ok(vec![]));
    }
    assert_eq!(reads.reads(), (1, 1), "requests after no new adoption read no adoption");
    reads.adopt(2);
    assert_eq!(index.take(&reads, 0), Ok(vec![]));
    assert_eq!(reads.reads(), (1, 1), "a new adoption is not read before a slot frees");
}

#[test]
fn a_failed_load_drops_the_index_for_a_rescan() {
    let reads = Counted::default();
    reads.adopt(1);
    let mut index = WaitingAdoptions::default();
    reads.is_load_failing.set(true);
    assert_eq!(index.take(&reads, 1), Err(Failed));
    reads.is_load_failing.set(false);
    assert_eq!(index.take(&reads, 1), Ok(vec![1]), "the adoption whose load failed is found again");
    assert_eq!(reads.reads(), (2, 2));
}
