const COUNT_INCREMENT: u32 = 1;

/// Safe replay metadata counts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReplayCounts {
    entries: u32,
    model_visible_items: u32,
    summaries: u32,
    usage_observations: u32,
    safe_receipts: u32,
}

impl ReplayCounts {
    pub(super) fn record_entry(&mut self) {
        self.entries = self.entries.saturating_add(COUNT_INCREMENT);
    }

    pub(super) fn record_model_visible(&mut self) {
        self.model_visible_items = self.model_visible_items.saturating_add(COUNT_INCREMENT);
    }

    pub(super) fn record_summary(&mut self) {
        self.summaries = self.summaries.saturating_add(COUNT_INCREMENT);
    }

    pub(super) fn reset_model_visible_items(&mut self) {
        self.model_visible_items = 0;
    }

    pub(super) fn record_usage(&mut self) {
        self.usage_observations = self.usage_observations.saturating_add(COUNT_INCREMENT);
    }

    pub(super) fn record_receipt(&mut self) {
        self.safe_receipts = self.safe_receipts.saturating_add(COUNT_INCREMENT);
    }

    /// Returns the number of replayed entries.
    #[must_use]
    pub const fn entries(self) -> u32 {
        self.entries
    }

    /// Returns the number of model-visible context items.
    #[must_use]
    pub const fn model_visible_items(self) -> u32 {
        self.model_visible_items
    }

    /// Returns the number of summary entries.
    #[must_use]
    pub const fn summaries(self) -> u32 {
        self.summaries
    }

    /// Returns the number of usage observations.
    #[must_use]
    pub const fn usage_observations(self) -> u32 {
        self.usage_observations
    }

    /// Returns the number of safe receipt entries.
    #[must_use]
    pub const fn safe_receipts(self) -> u32 {
        self.safe_receipts
    }
}
