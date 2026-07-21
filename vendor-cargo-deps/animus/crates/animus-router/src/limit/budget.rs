#![allow(unknown_lints, path_segment_repetition)]

use super::Bounds;
use crate::{BudgetExhausted, BudgetKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetLedger {
    candidate_limit: u32,
    remaining_candidates: u32,
    remaining_parallelism: u32,
    remaining_synthesis_attempts: u32,
    remaining_repair_attempts: u32,
    tried_models: alloc::vec::Vec<alloc::string::String>,
}

impl From<&Bounds> for BudgetLedger {
    fn from(bounds: &Bounds) -> Self {
        Self {
            candidate_limit: bounds.max_candidates(),
            remaining_candidates: bounds.max_candidates(),
            remaining_parallelism: bounds.max_parallelism(),
            remaining_synthesis_attempts: bounds.max_synthesis_attempts(),
            remaining_repair_attempts: bounds.max_repair_attempts(),
            tried_models: alloc::vec::Vec::new(),
        }
    }
}

impl BudgetLedger {
    #[must_use]
    pub const fn candidate_limit(&self) -> u32 {
        self.candidate_limit
    }

    #[must_use]
    pub const fn remaining_candidates(&self) -> u32 {
        self.remaining_candidates
    }

    #[must_use]
    pub const fn remaining_parallelism(&self) -> u32 {
        self.remaining_parallelism
    }

    #[must_use]
    pub const fn remaining_synthesis_attempts(&self) -> u32 {
        self.remaining_synthesis_attempts
    }

    #[must_use]
    pub const fn remaining_repair_attempts(&self) -> u32 {
        self.remaining_repair_attempts
    }

    #[must_use]
    pub fn tried_models(&self) -> &[alloc::string::String] {
        self.tried_models.as_slice()
    }

    /// Consumes one candidate budget unit.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetExhausted`] when no candidate budget remains.
    pub fn consume_candidate(&self) -> Result<Self, BudgetExhausted> {
        self.consume(self.remaining_candidates, BudgetKind::Candidate, |next| {
            next.remaining_candidates -= 1;
        })
    }

    /// Consumes one parallel dispatch budget unit.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetExhausted`] when no parallelism budget remains.
    pub fn consume_parallelism(&self) -> Result<Self, BudgetExhausted> {
        self.consume(
            self.remaining_parallelism,
            BudgetKind::Parallelism,
            |next| {
                next.remaining_parallelism -= 1;
            },
        )
    }

    /// Consumes one synthesis-attempt budget unit.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetExhausted`] when no synthesis-attempt budget remains.
    pub fn consume_synthesis_attempt(&self) -> Result<Self, BudgetExhausted> {
        self.consume(
            self.remaining_synthesis_attempts,
            BudgetKind::Synthesis,
            |next| {
                next.remaining_synthesis_attempts -= 1;
            },
        )
    }

    /// Consumes one repair-attempt budget unit.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetExhausted`] when no repair-attempt budget remains.
    pub fn consume_repair_attempt(&self) -> Result<Self, BudgetExhausted> {
        self.consume(self.remaining_repair_attempts, BudgetKind::Repair, |next| {
            next.remaining_repair_attempts -= 1;
        })
    }

    /// Records that a model label has been attempted.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetExhausted`] when recording a new tried model would exceed
    /// the candidate budget cap.
    pub fn record_tried_model(&self, label: &str) -> Result<Self, BudgetExhausted> {
        if self
            .tried_models
            .iter()
            .any(|tried| tried.as_str() == label)
        {
            return Ok(self.clone());
        }
        let Ok(candidate_limit) = usize::try_from(self.candidate_limit) else {
            return Err(BudgetExhausted::new(
                BudgetKind::Candidate,
                self.remaining_candidates,
            ));
        };
        if self.tried_models.len() >= candidate_limit {
            return Err(BudgetExhausted::new(
                BudgetKind::Candidate,
                self.remaining_candidates,
            ));
        }
        let mut next = self.clone();
        next.tried_models.push(alloc::string::String::from(label));
        Ok(next)
    }

    fn consume(
        &self,
        remaining: u32,
        kind: BudgetKind,
        update: impl FnOnce(&mut Self),
    ) -> Result<Self, BudgetExhausted> {
        if remaining == 0 {
            return Err(BudgetExhausted::new(kind, remaining));
        }
        let mut next = self.clone();
        update(&mut next);
        Ok(next)
    }
}
