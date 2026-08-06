//! Bounded, navigable in-memory command history.
//!
//! Persistence location, retention, privacy, encoding, and I/O belong to a
//! named application adapter. This module exchanges generation-bound data and
//! typed storage outcomes only.

pub const COMMAND_HISTORY_CODEC_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHistorySnapshot {
    pub base_generation: u64,
    pub codec_version: u16,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandHistoryLoadOutcome {
    Loaded(CommandHistorySnapshot),
    Missing,
    Denied,
    Malformed,
    FutureVersion,
    ReadFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistorySaveOutcome {
    Saved { generation: u64 },
    Denied,
    WriteFailed,
    FlushFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistoryDiagnostic {
    Missing,
    Denied,
    Malformed,
    FutureVersion,
    ReadFailed,
    WriteFailed,
    FlushFailed,
    Stale,
    CapacityExceeded,
    InvalidEntries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHistoryApplyResult {
    Applied,
    Preserved(CommandHistoryDiagnostic),
}

/// A bounded command history buffer with navigation support.
///
/// # Example
///
/// ```rust
/// use rat_widgets::CommandHistory;
///
/// let mut history = CommandHistory::new(5);
/// history.add("ls -la".to_string());
/// history.add("cd /home".to_string());
///
/// // Navigate backward
/// assert_eq!(history.prev(), Some("cd /home"));
/// assert_eq!(history.prev(), Some("ls -la"));
///
/// // Navigate forward
/// assert_eq!(history.next_entry(), Some("cd /home"));
/// assert_eq!(history.next_entry(), None); // Back to empty input
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHistory {
    entries: Vec<String>,
    index: usize,
    browsing: bool,
    max_entries: usize,
    generation: u64,
}

impl CommandHistory {
    /// Creates a new command history with the specified maximum number of entries.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            index: 0,
            browsing: false,
            max_entries,
            generation: 0,
        }
    }

    /// Creates a command history from existing entries.
    ///
    /// Useful for restoring from persistence. Entries are taken as-is,
    /// no deduplication or filtering is performed.
    pub fn with_entries(entries: Vec<String>, max_entries: usize) -> Self {
        let index = entries.len();
        Self {
            entries,
            index,
            browsing: false,
            max_entries,
            generation: 0,
        }
    }

    /// Adds an entry to the history.
    ///
    /// - Skips empty or whitespace-only entries
    /// - Skips consecutive duplicates
    /// - Evicts oldest entries if over max_entries limit
    /// - Resets browsing state
    pub fn add(&mut self, entry: String) {
        if entry.trim().is_empty() {
            return;
        }

        // Skip consecutive duplicate
        if self.entries.last().map(|s| s == &entry).unwrap_or(false) {
            return;
        }

        self.entries.push(entry);
        self.generation = self.generation.saturating_add(1);

        // Evict oldest if over max
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        // Reset navigation
        self.index = self.entries.len();
        self.browsing = false;
    }

    /// Navigate to the previous entry (Up key behavior).
    ///
    /// Returns the entry text. First call starts browsing from the end.
    /// If already at the first entry, stays there and returns it.
    pub fn prev(&mut self) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        if !self.browsing {
            self.browsing = true;
            self.index = self.entries.len();
        }

        if self.index > 0 {
            self.index -= 1;
            Some(&self.entries[self.index])
        } else {
            Some(&self.entries[0]) // stay at first
        }
    }

    /// Navigate to the next entry (Down key behavior).
    ///
    /// Returns the entry text, or None when past the newest entry
    /// (meaning "back to empty input"). Stops browsing when None is returned.
    pub fn next_entry(&mut self) -> Option<&str> {
        if !self.browsing || self.entries.is_empty() {
            return None;
        }

        if self.index < self.entries.len().saturating_sub(1) {
            self.index += 1;
            Some(&self.entries[self.index])
        } else {
            // Past the end — stop browsing
            self.index = self.entries.len();
            self.browsing = false;
            None
        }
    }

    /// Stops browsing and resets index to end.
    pub fn reset_browse(&mut self) {
        self.browsing = false;
        self.index = self.entries.len();
    }

    /// Returns true if currently browsing through history.
    pub fn is_browsing(&self) -> bool {
        self.browsing
    }

    /// Returns the current entry if browsing.
    pub fn current(&self) -> Option<&str> {
        if self.browsing && self.index < self.entries.len() {
            Some(&self.entries[self.index])
        } else {
            None
        }
    }

    /// Returns all entries (for persistence or inspection).
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Returns the number of entries in the history.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clears all entries and resets state.
    pub fn clear(&mut self) {
        if !self.entries.is_empty() {
            self.generation = self.generation.saturating_add(1);
        }
        self.entries.clear();
        self.index = 0;
        self.browsing = false;
    }

    /// Current immutable state generation.
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Produces plain data for an application-owned codec/storage adapter.
    pub fn storage_snapshot(&self) -> CommandHistorySnapshot {
        CommandHistorySnapshot {
            base_generation: self.generation,
            codec_version: COMMAND_HISTORY_CODEC_VERSION,
            entries: self.entries.clone(),
        }
    }

    /// Applies a later load message only when it is current and valid.
    pub fn apply_load_outcome(&mut self, outcome: CommandHistoryLoadOutcome) -> CommandHistoryApplyResult {
        let snapshot = match outcome {
            CommandHistoryLoadOutcome::Loaded(snapshot) => snapshot,
            CommandHistoryLoadOutcome::Missing => {
                return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Missing);
            }
            CommandHistoryLoadOutcome::Denied => {
                return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Denied);
            }
            CommandHistoryLoadOutcome::Malformed => {
                return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Malformed);
            }
            CommandHistoryLoadOutcome::FutureVersion => {
                return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::FutureVersion);
            }
            CommandHistoryLoadOutcome::ReadFailed => {
                return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::ReadFailed);
            }
        };
        if snapshot.base_generation != self.generation {
            return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Stale);
        }
        if snapshot.codec_version > COMMAND_HISTORY_CODEC_VERSION {
            return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::FutureVersion);
        }
        if snapshot.entries.len() > self.max_entries {
            return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::CapacityExceeded);
        }
        if snapshot.entries.iter().any(|entry| entry.trim().is_empty())
            || snapshot.entries.windows(2).any(|pair| pair[0] == pair[1])
        {
            return CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::InvalidEntries);
        }
        self.entries = snapshot.entries;
        self.index = self.entries.len();
        self.browsing = false;
        self.generation = self.generation.saturating_add(1);
        CommandHistoryApplyResult::Applied
    }

    /// Classifies a later save/flush result without changing accepted history.
    pub fn classify_save_outcome(&self, outcome: CommandHistorySaveOutcome) -> CommandHistoryApplyResult {
        match outcome {
            CommandHistorySaveOutcome::Saved { generation } if generation == self.generation => {
                CommandHistoryApplyResult::Applied
            }
            CommandHistorySaveOutcome::Saved { .. } => {
                CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Stale)
            }
            CommandHistorySaveOutcome::Denied => CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::Denied),
            CommandHistorySaveOutcome::WriteFailed => {
                CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::WriteFailed)
            }
            CommandHistorySaveOutcome::FlushFailed => {
                CommandHistoryApplyResult::Preserved(CommandHistoryDiagnostic::FlushFailed)
            }
        }
    }

    /// Returns the current position for display purposes.
    ///
    /// Returns (current_index + 1, total) for display like "[3/15]".
    /// When not browsing, returns (total, total).
    pub fn position(&self) -> (usize, usize) {
        if self.browsing && !self.entries.is_empty() {
            (self.index + 1, self.entries.len())
        } else {
            (self.entries.len(), self.entries.len())
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_len() {
        let mut history = CommandHistory::new(5);
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());

        history.add("command1".to_string());
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        history.add("command2".to_string());
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_skip_empty() {
        let mut history = CommandHistory::new(5);

        history.add("".to_string());
        history.add("   ".to_string());
        history.add("\t\n".to_string());

        assert_eq!(history.len(), 0);

        history.add("real_command".to_string());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_skip_consecutive_dupes() {
        let mut history = CommandHistory::new(5);

        history.add("command".to_string());
        history.add("command".to_string());

        assert_eq!(history.len(), 1);
        assert_eq!(history.entries(), &["command"]);
    }

    #[test]
    fn test_non_consecutive_dupes_allowed() {
        let mut history = CommandHistory::new(5);

        history.add("a".to_string());
        history.add("b".to_string());
        history.add("a".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.entries(), &["a", "b", "a"]);
    }

    #[test]
    fn test_max_entries_eviction() {
        let mut history = CommandHistory::new(3);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.add("cmd3".to_string());
        assert_eq!(history.entries(), &["cmd1", "cmd2", "cmd3"]);

        history.add("cmd4".to_string());
        assert_eq!(history.entries(), &["cmd2", "cmd3", "cmd4"]);

        history.add("cmd5".to_string());
        assert_eq!(history.entries(), &["cmd3", "cmd4", "cmd5"]);
    }

    #[test]
    fn test_prev_next_navigation() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.add("cmd3".to_string());

        // Navigate backward
        assert_eq!(history.prev(), Some("cmd3"));
        assert_eq!(history.prev(), Some("cmd2"));
        assert_eq!(history.prev(), Some("cmd1"));

        // Navigate forward
        assert_eq!(history.next_entry(), Some("cmd2"));
        assert_eq!(history.next_entry(), Some("cmd3"));
        assert_eq!(history.next_entry(), None); // Back to empty
    }

    #[test]
    fn test_prev_at_start_stays() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());

        assert_eq!(history.prev(), Some("cmd2"));
        assert_eq!(history.prev(), Some("cmd1"));
        assert_eq!(history.prev(), Some("cmd1")); // Stay at first
        assert_eq!(history.prev(), Some("cmd1"));
    }

    #[test]
    fn test_next_past_end_stops_browsing() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());

        assert_eq!(history.prev(), Some("cmd1"));
        assert!(history.is_browsing());

        assert_eq!(history.next_entry(), None); // Past end
        assert!(!history.is_browsing());
    }

    #[test]
    fn test_add_resets_browsing() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());

        assert_eq!(history.prev(), Some("cmd1"));
        assert!(history.is_browsing());

        history.add("cmd2".to_string());
        assert!(!history.is_browsing());
    }

    #[test]
    fn test_position_display() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());
        history.add("cmd3".to_string());

        // Not browsing
        assert_eq!(history.position(), (3, 3));

        // Browse backward
        assert_eq!(history.prev(), Some("cmd3"));
        assert_eq!(history.position(), (3, 3));

        assert_eq!(history.prev(), Some("cmd2"));
        assert_eq!(history.position(), (2, 3));

        assert_eq!(history.prev(), Some("cmd1"));
        assert_eq!(history.position(), (1, 3));

        // Browse forward
        assert_eq!(history.next_entry(), Some("cmd2"));
        assert_eq!(history.position(), (2, 3));

        assert_eq!(history.next_entry(), Some("cmd3"));
        assert_eq!(history.position(), (3, 3));

        assert_eq!(history.next_entry(), None); // Back to not browsing
        assert_eq!(history.position(), (3, 3));
    }

    #[test]
    fn test_clear() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());

        assert_eq!(history.prev(), Some("cmd2"));
        assert!(history.is_browsing());

        history.clear();

        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        assert!(!history.is_browsing());
        assert_eq!(history.current(), None);
        assert_eq!(history.position(), (0, 0));
    }

    #[test]
    fn test_current() {
        let mut history = CommandHistory::new(5);

        history.add("cmd1".to_string());
        history.add("cmd2".to_string());

        assert_eq!(history.current(), None); // Not browsing

        assert_eq!(history.prev(), Some("cmd2"));
        assert_eq!(history.current(), Some("cmd2"));

        assert_eq!(history.prev(), Some("cmd1"));
        assert_eq!(history.current(), Some("cmd1"));

        history.reset_browse();
        assert_eq!(history.current(), None); // Not browsing
    }

    #[test]
    fn test_empty_history_navigation() {
        let mut history = CommandHistory::new(5);

        assert_eq!(history.prev(), None);
        assert_eq!(history.next_entry(), None);
        assert_eq!(history.current(), None);
        assert_eq!(history.position(), (0, 0));
    }

    #[test]
    fn test_with_entries() {
        let entries = vec!["cmd1".to_string(), "cmd2".to_string()];
        let history = CommandHistory::with_entries(entries.clone(), 5);

        assert_eq!(history.entries(), &entries);
        assert_eq!(history.len(), 2);
        assert!(!history.is_browsing());
    }

    #[test]
    fn storage_rejections_preserve_accepted_history() {
        let mut history = CommandHistory::new(3);
        history.add("accepted".to_string());
        let accepted = history.clone();

        for outcome in [
            CommandHistoryLoadOutcome::Missing,
            CommandHistoryLoadOutcome::Denied,
            CommandHistoryLoadOutcome::Malformed,
            CommandHistoryLoadOutcome::FutureVersion,
            CommandHistoryLoadOutcome::ReadFailed,
        ] {
            assert!(matches!(history.apply_load_outcome(outcome), CommandHistoryApplyResult::Preserved(_)));
            assert_eq!(history, accepted);
        }
        for outcome in [
            CommandHistorySaveOutcome::Denied,
            CommandHistorySaveOutcome::WriteFailed,
            CommandHistorySaveOutcome::FlushFailed,
        ] {
            assert!(matches!(history.classify_save_outcome(outcome), CommandHistoryApplyResult::Preserved(_)));
            assert_eq!(history, accepted);
        }
    }

    #[test]
    fn stale_future_duplicate_and_excessive_snapshots_fail_closed() {
        let mut history = CommandHistory::new(2);
        history.add("accepted".to_string());
        let accepted = history.clone();
        let snapshots = [
            CommandHistorySnapshot {
                base_generation: history.generation().saturating_sub(1),
                codec_version: COMMAND_HISTORY_CODEC_VERSION,
                entries: vec!["new".to_string()],
            },
            CommandHistorySnapshot {
                base_generation: history.generation(),
                codec_version: COMMAND_HISTORY_CODEC_VERSION + 1,
                entries: vec!["new".to_string()],
            },
            CommandHistorySnapshot {
                base_generation: history.generation(),
                codec_version: COMMAND_HISTORY_CODEC_VERSION,
                entries: vec!["same".to_string(), "same".to_string()],
            },
            CommandHistorySnapshot {
                base_generation: history.generation(),
                codec_version: COMMAND_HISTORY_CODEC_VERSION,
                entries: vec!["one".to_string(), "two".to_string(), "three".to_string()],
            },
        ];

        for snapshot in snapshots {
            assert!(matches!(
                history.apply_load_outcome(CommandHistoryLoadOutcome::Loaded(snapshot)),
                CommandHistoryApplyResult::Preserved(_)
            ));
            assert_eq!(history, accepted);
        }
    }

    #[test]
    fn current_loaded_snapshot_and_explicit_reset_apply_as_data() {
        let mut history = CommandHistory::new(3);
        history.add("old".to_string());
        let loaded = CommandHistorySnapshot {
            base_generation: history.generation(),
            codec_version: COMMAND_HISTORY_CODEC_VERSION,
            entries: vec!["new".to_string()],
        };
        assert_eq!(
            history.apply_load_outcome(CommandHistoryLoadOutcome::Loaded(loaded)),
            CommandHistoryApplyResult::Applied
        );
        assert_eq!(history.entries(), &["new"]);
        let reset = CommandHistorySnapshot {
            base_generation: history.generation(),
            codec_version: COMMAND_HISTORY_CODEC_VERSION,
            entries: Vec::new(),
        };
        assert_eq!(
            history.apply_load_outcome(CommandHistoryLoadOutcome::Loaded(reset)),
            CommandHistoryApplyResult::Applied
        );
        assert!(history.is_empty());
    }
}
