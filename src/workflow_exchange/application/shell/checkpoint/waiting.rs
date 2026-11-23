//! The serving handler's index of adopted executions waiting for a slot (Lattice #12).

use std::collections::BTreeMap;

use lattice_workflow_contract::Identity;

/// The store reads behind [`WaitingAdoptions`]; a seam that lets tests count them.
pub(super) trait AdoptionReads {
    /// A waiting adoption, loaded to start.
    type Waiting;
    type Error;

    /// Equal values mean no adoption was added in between.
    fn generation(&self) -> Result<u64, Self::Error>;
    /// Run id and execution of every adoption waiting now.
    fn scan(&self) -> Result<Vec<(u64, Identity)>, Self::Error>;
    /// `execution` loaded when it still waits, `None` once it no longer does.
    fn load(&self, execution: &Identity) -> Result<Option<Self::Waiting>, Self::Error>;
}

/// Waiting adopted executions by run id, valid for the adoption generation they were scanned at.
#[derive(Default)]
pub(in super::super) struct WaitingAdoptions {
    generation: Option<u64>,
    executions: BTreeMap<u64, Identity>,
}

impl WaitingAdoptions {
    /// Up to `count` waiting adoptions, lowest run id first. The index is rescanned only when the
    /// adoption generation moved, so a request with no free slot, or with no new adoption and
    /// nothing waiting, reads no adoption. A store error drops the index; the next call rescans.
    pub(super) fn take<R: AdoptionReads>(&mut self, reads: &R, count: usize) -> Result<Vec<R::Waiting>, R::Error> {
        if count == 0 {
            return Ok(Vec::new());
        }
        let taken = self.take_from(reads, count);
        if taken.is_err() {
            self.generation = None;
        }
        taken
    }

    fn take_from<R: AdoptionReads>(&mut self, reads: &R, count: usize) -> Result<Vec<R::Waiting>, R::Error> {
        let generation = reads.generation()?;
        if self.generation != Some(generation) {
            self.executions = reads.scan()?.into_iter().collect();
            self.generation = Some(generation);
        }
        let mut taken = Vec::new();
        while taken.len() < count
            && let Some((_run_id, execution)) = self.executions.pop_first()
        {
            taken.extend(reads.load(&execution)?);
        }
        Ok(taken)
    }
}

#[cfg(test)]
mod tests;
