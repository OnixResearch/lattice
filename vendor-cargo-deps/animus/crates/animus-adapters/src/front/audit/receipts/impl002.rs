
/// Validated event window identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditWindow {
    window_id: String,
    event_window_hash: crate::artifact::Hash,
}
