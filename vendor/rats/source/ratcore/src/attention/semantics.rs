const ROLE_BUTTON: &str = "Button";
const ROLE_MENU_ITEM: &str = "Menu item";
const ROLE_DIALOG: &str = "Dialog";
const ROLE_LIST_ITEM: &str = "List item";
const ROLE_TABLE_CELL: &str = "Table cell";
const ROLE_TREE_ITEM: &str = "Tree item";
const ROLE_FORM_FIELD: &str = "Form field";
const ROLE_PANEL: &str = "Panel";

/// Semantic focus role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FocusRole {
    /// Button or command control.
    Button,
    /// Menu item.
    MenuItem,
    /// Dialog container or dialog action.
    Dialog,
    /// List row or option.
    ListItem,
    /// Table cell or row.
    TableCell,
    /// Tree item.
    TreeItem,
    /// Form field.
    FormField,
    /// Panel or region.
    Panel,
    /// Application-specific role.
    Custom(String),
}

impl FocusRole {
    /// Returns a human-readable role label.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::Button => ROLE_BUTTON,
            Self::MenuItem => ROLE_MENU_ITEM,
            Self::Dialog => ROLE_DIALOG,
            Self::ListItem => ROLE_LIST_ITEM,
            Self::TableCell => ROLE_TABLE_CELL,
            Self::TreeItem => ROLE_TREE_ITEM,
            Self::FormField => ROLE_FORM_FIELD,
            Self::Panel => ROLE_PANEL,
            Self::Custom(label) => label.as_str(),
        }
    }
}

/// High-level widget family exposing shared focus metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FocusWidgetKind {
    /// Command palette result or input item.
    CommandPalette,
    /// Leader-key overlay item.
    LeaderKey,
    /// Chrome menu item.
    ChromeMenu,
    /// Dialog action or body control.
    Dialog,
    /// Table row/cell.
    Table,
    /// Tree row.
    Tree,
    /// Form field.
    Form,
}

impl FocusWidgetKind {
    /// Returns the default role for the widget kind.
    #[must_use]
    pub fn default_role(self) -> FocusRole {
        match self {
            Self::CommandPalette | Self::LeaderKey | Self::ChromeMenu => FocusRole::MenuItem,
            Self::Dialog => FocusRole::Dialog,
            Self::Table => FocusRole::TableCell,
            Self::Tree => FocusRole::TreeItem,
            Self::Form => FocusRole::FormField,
        }
    }
}
