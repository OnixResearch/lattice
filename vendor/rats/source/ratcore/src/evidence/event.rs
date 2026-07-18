const EVENT_KIND_KEY: &str = "key";
const EVENT_KIND_MOUSE: &str = "mouse";
const EVENT_KIND_RESIZE: &str = "resize";
const EVENT_KIND_TEXT: &str = "text";
const EVENT_KIND_COMMAND: &str = "command";
const EVENT_KIND_WAIT: &str = "wait";

/// Logical input event in a renderer-neutral replay script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayEvent {
    /// Logical key press such as `Enter`, `ArrowDown`, or `Ctrl+K`.
    Key {
        /// Target focus owner for the event.
        target: String,
        /// Logical key label.
        key: String,
    },
    /// Logical mouse input against a target and cell/pixel-like coordinates.
    Mouse {
        /// Target hit-test owner for the event.
        target: String,
        /// Horizontal coordinate in the target coordinate space.
        x: u16,
        /// Vertical coordinate in the target coordinate space.
        y: u16,
    },
    /// Logical viewport resize.
    Resize {
        /// New viewport width.
        width: u16,
        /// New viewport height.
        height: u16,
    },
    /// Text input for the current target.
    Text {
        /// Target focus owner for text insertion.
        target: String,
        /// Text to insert.
        value: String,
    },
    /// Command/action activation.
    Command {
        /// Stable action ID.
        action_id: String,
    },
    /// Deterministic wait condition over explicit target state.
    Wait {
        /// Wait condition evaluated without sleeping or polling external systems.
        condition: super::ReplayWaitCondition,
    },
    /// Unsupported event marker used by validators and negative fixtures.
    Unsupported {
        /// Unsupported event kind label.
        kind: String,
    },
}

impl ReplayEvent {
    /// Returns the stable event kind label.
    #[must_use]
    pub fn kind(&self) -> &str {
        match self {
            Self::Key { .. } => EVENT_KIND_KEY,
            Self::Mouse { .. } => EVENT_KIND_MOUSE,
            Self::Resize { .. } => EVENT_KIND_RESIZE,
            Self::Text { .. } => EVENT_KIND_TEXT,
            Self::Command { .. } => EVENT_KIND_COMMAND,
            Self::Wait { .. } => EVENT_KIND_WAIT,
            Self::Unsupported { kind } => kind.as_str(),
        }
    }
}
