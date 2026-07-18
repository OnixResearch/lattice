/// Pointer phase after renderer normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerPhase {
    /// Pointer was pressed.
    Down,
    /// Pointer moved without an explicit drag phase.
    Move,
    /// Pointer entered a target surface.
    Enter,
    /// Pointer left a target surface.
    Leave,
    /// Pointer wheel or scroll event.
    Wheel,
    /// Pointer was released.
    Up,
    /// Drag began with a captured target.
    DragStart,
    /// Drag updated after capture.
    DragUpdate,
    /// Drag release committed after capture.
    DragCommit,
    /// Pointer interaction was canceled.
    Cancel,
    /// Drag interaction was canceled.
    DragCancel,
    /// Renderer reported a phase the shared router does not understand.
    Unsupported(String),
}

/// Pointer source after renderer normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerSource {
    /// Mouse source.
    Mouse,
    /// Touch source.
    Touch,
    /// Pen or stylus source.
    Pen,
    /// Terminal mouse source.
    Terminal,
    /// Ratzilla browser abstraction source.
    Ratzilla,
    /// DOM pointer or mouse source.
    Dom,
    /// Renderer reported a source the shared router does not understand.
    Unsupported(String),
}

/// Click count after renderer normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PointerClickCount {
    /// Event does not carry click count metadata.
    None,
    /// Single-click event.
    Single,
    /// Double-click event.
    Double,
    /// Renderer supplied a higher click count.
    Multiple(u8),
    /// Renderer supplied malformed click-count metadata.
    Unsupported(String),
}

/// Keyboard modifiers active for a pointer event.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PointerModifiers {
    /// Control modifier.
    pub ctrl: bool,
    /// Alt or option modifier.
    pub alt: bool,
    /// Shift modifier.
    pub shift: bool,
    /// Meta, command, or super modifier.
    pub meta: bool,
}
