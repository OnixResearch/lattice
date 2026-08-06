/// Declarative split axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutAxis {
    /// Horizontal split.
    Horizontal,
    /// Vertical split.
    Vertical,
}

impl LayoutAxis {
    /// Parses a split-axis label.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] for unsupported labels.
    pub fn parse(value: &str) -> Result<Self, HostControlError> {
        parse_value(AXIS_FIELD, value, |normalized| match normalized {
            AXIS_HORIZONTAL => Some(Self::Horizontal),
            AXIS_VERTICAL => Some(Self::Vertical),
            _unsupported => None,
        })
    }
}

/// Borrowed command descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutCommandDescriptorDraft<'a> {
    /// Program or host-local command label.
    pub command: &'a str,
    /// Bounded argument labels.
    pub args: &'a [&'a str],
}

/// Declarative command descriptor for a pane layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutCommandDescriptor {
    command: String,
    args: Vec<String>,
}

impl LayoutCommandDescriptor {
    /// Validates a pane command descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when command text is empty or embeds host-specific paths/URIs.
    pub fn validate(draft: LayoutCommandDescriptorDraft<'_>) -> Result<Self, HostControlError> {
        let command = required_text(COMMAND_FIELD, draft.command)?;
        if is_unsafe_host_reference(&command) {
            return Err(HostControlError::InvalidLayout);
        }
        Ok(Self {
            command,
            args: draft
                .args
                .iter()
                .map(|arg| required_text(COMMAND_ARG_FIELD, arg))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    /// Returns the command label.
    #[must_use]
    pub fn command(&self) -> &str {
        self.command.as_str()
    }

    /// Returns command argument labels.
    #[must_use]
    pub fn args(&self) -> &[String] {
        self.args.as_slice()
    }
}

/// Borrowed pane layout metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutPaneDraft<'a> {
    /// Pane node identity.
    pub node_id: &'a str,
    /// Optional pane label.
    pub label: Option<&'a str>,
    /// Optional cwd summary.
    pub cwd_summary: Option<&'a str>,
    /// Optional command descriptor.
    pub command: Option<LayoutCommandDescriptorDraft<'a>>,
    /// Whether this pane should be focused after apply.
    pub focused: bool,
    /// Whether this pane should be zoomed after apply.
    pub zoomed: bool,
}

/// Declarative host layout node.
// r[impl agent_harness_runtime.host_control.layout_contract]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutNode {
    /// Pane leaf node.
    Pane {
        /// Pane node identity.
        node_id: String,
        /// Optional pane label.
        label: Option<String>,
        /// Optional cwd summary.
        cwd_summary: Option<String>,
        /// Optional command descriptor.
        command: Option<LayoutCommandDescriptor>,
        /// Focus state.
        focused: bool,
        /// Zoom state.
        zoomed: bool,
    },
    /// Split node with child nodes.
    Split {
        /// Split node identity.
        node_id: String,
        /// Split axis.
        axis: LayoutAxis,
        /// Child nodes.
        children: Vec<LayoutNode>,
    },
}

impl LayoutNode {
    /// Builds a pane layout node.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the node id is invalid.
    pub fn pane(node_id: &str) -> Result<Self, HostControlError> {
        Self::pane_with(LayoutPaneDraft {
            node_id,
            label: None,
            cwd_summary: None,
            command: None,
            focused: false,
            zoomed: false,
        })
    }

    /// Builds a pane layout node with metadata.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when pane metadata is invalid.
    pub fn pane_with(draft: LayoutPaneDraft<'_>) -> Result<Self, HostControlError> {
        Ok(Self::Pane {
            node_id: required_text(NODE_ID_FIELD, draft.node_id)?,
            label: optional_text(LABEL_FIELD, draft.label)?,
            cwd_summary: optional_text(CWD_FIELD, draft.cwd_summary)?,
            command: draft
                .command
                .map(LayoutCommandDescriptor::validate)
                .transpose()?,
            focused: draft.focused,
            zoomed: draft.zoomed,
        })
    }

    /// Builds a split layout node.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the node id, axis, or child list is invalid.
    pub fn split(
        node_id: &str,
        axis: &str,
        children: Vec<LayoutNode>,
    ) -> Result<Self, HostControlError> {
        if children.is_empty() {
            return Err(HostControlError::InvalidLayout);
        }
        Ok(Self::Split {
            node_id: required_text(NODE_ID_FIELD, node_id)?,
            axis: LayoutAxis::parse(axis)?,
            children,
        })
    }

    /// Returns node identity.
    #[must_use]
    pub fn node_id(&self) -> &str {
        match self {
            Self::Pane { node_id, .. } | Self::Split { node_id, .. } => node_id.as_str(),
        }
    }

    /// Returns whether this layout node is focused.
    #[must_use]
    pub const fn focused(&self) -> bool {
        match self {
            Self::Pane { focused, .. } => *focused,
            Self::Split { .. } => false,
        }
    }
}

/// Declarative layout apply request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutApplyRequest {
    root: LayoutNode,
    preserve_live_processes: bool,
}

impl LayoutApplyRequest {
    /// Validates a layout apply request.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the request claims live process preservation.
    pub fn new(root: LayoutNode, preserve_live_processes: bool) -> Result<Self, HostControlError> {
        if preserve_live_processes {
            return Err(HostControlError::InvalidLayout);
        }
        Ok(Self {
            root,
            preserve_live_processes,
        })
    }

    /// Returns the layout root.
    #[must_use]
    pub const fn root(&self) -> &LayoutNode {
        &self.root
    }

    /// Returns whether the request claims live process preservation.
    #[must_use]
    pub const fn preserve_live_processes(&self) -> bool {
        self.preserve_live_processes
    }
}

/// Layout apply response produced by a host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutApplyResponse {
    created_topology: Vec<HostSelector>,
    focused_selector: Option<HostSelector>,
    preserved_live_processes: bool,
}

impl LayoutApplyResponse {
    /// Validates a layout apply response.
    ///
    /// # Errors
    ///
    /// Returns [`HostControlError`] when the response falsely claims live process preservation.
    pub fn new(
        created_topology: &[HostSelector],
        focused_selector: Option<&HostSelector>,
        preserved_live_processes: bool,
    ) -> Result<Self, HostControlError> {
        if preserved_live_processes {
            return Err(HostControlError::InvalidLayout);
        }
        Ok(Self {
            created_topology: created_topology.to_vec(),
            focused_selector: focused_selector.cloned(),
            preserved_live_processes,
        })
    }

    /// Returns created topology selectors.
    #[must_use]
    pub fn created_topology(&self) -> &[HostSelector] {
        self.created_topology.as_slice()
    }

    /// Returns focused selector when present.
    #[must_use]
    pub const fn focused_selector(&self) -> Option<&HostSelector> {
        self.focused_selector.as_ref()
    }

    /// Returns whether live process preservation was claimed.
    #[must_use]
    pub const fn preserved_live_processes(&self) -> bool {
        self.preserved_live_processes
    }
}
