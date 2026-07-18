//! Optional terminal IDE shell composition core.
//!
//! The shell core computes deterministic regions, validates application adapter
//! availability, and projects shared action metadata for command surfaces. It is
//! pure data plumbing; renderer crates own ratatui drawing and applications own
//! project data, editor buffers, diagnostics, previews, and task output.

const DEFAULT_WIDTH: u16 = 120;
const DEFAULT_HEIGHT: u16 = 40;
const NAVIGATION_WIDTH: u16 = 28;
const AUXILIARY_WIDTH: u16 = 32;
const COMMAND_HEIGHT: u16 = 3;
const STATUS_HEIGHT: u16 = 1;
const MIN_MAIN_WIDTH: u16 = 10;
const MIN_MAIN_HEIGHT: u16 = 5;

mod commands;
pub use commands::IdeCommandSurfaces;

/// Deterministic rectangle used by the IDE shell core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IdeRect {
    /// Left coordinate.
    pub x: u16,
    /// Top coordinate.
    pub y: u16,
    /// Width in terminal cells.
    pub width: u16,
    /// Height in terminal cells.
    pub height: u16,
}

impl IdeRect {
    /// Creates a rectangle.
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Shell region layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeShellRegions {
    /// Navigation/sidebar region.
    pub navigation: IdeRect,
    /// Main editor/content region.
    pub main: IdeRect,
    /// Auxiliary/preview region.
    pub auxiliary: IdeRect,
    /// Command palette/input surface region.
    pub command: IdeRect,
    /// Status bar region.
    pub status: IdeRect,
}

/// Optional IDE shell configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeShellConfig {
    /// Shell viewport width.
    pub width: u16,
    /// Shell viewport height.
    pub height: u16,
    /// Whether the navigation sidebar is visible.
    pub navigation_visible: bool,
    /// Whether the auxiliary panel is visible.
    pub auxiliary_visible: bool,
    /// Optional persisted layout version.
    pub persisted_layout_version: Option<u32>,
}

impl Default for IdeShellConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            navigation_visible: true,
            auxiliary_visible: true,
            persisted_layout_version: Some(IdeLayoutVersion::CURRENT.value()),
        }
    }
}

/// Supported persisted layout version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdeLayoutVersion(u32);

impl IdeLayoutVersion {
    /// Current persisted layout version.
    pub const CURRENT: Self = Self(1);

    /// Returns the numeric version.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Application adapter boundary for terminal IDE data providers.
pub trait IdeDataAdapters {
    /// Returns adapter availability for startup validation.
    fn adapter_set(&self) -> IdeAdapterSet;
}

/// Adapter availability supplied by an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeAdapterSet {
    /// Project tree adapter exists.
    pub project_tree: bool,
    /// Editor buffer adapter exists.
    pub buffers: bool,
    /// Diagnostics adapter exists.
    pub diagnostics: bool,
    /// Preview adapter exists.
    pub preview: bool,
    /// Task output adapter exists.
    pub tasks: bool,
}

impl IdeAdapterSet {
    /// Creates an adapter set with all required adapters available.
    #[must_use]
    pub const fn all_available() -> Self {
        Self {
            project_tree: true,
            buffers: true,
            diagnostics: true,
            preview: true,
            tasks: true,
        }
    }
}

/// Stable IDE diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdeIssueCode {
    /// A required data adapter is missing.
    MissingAdapter,
    /// A shared action registry issue was found.
    ActionRegistryIssue,
    /// Persisted layout version is invalid.
    LayoutRestoreFailed,
    /// Computed layout is too small for the safe shell.
    LayoutTooSmall,
}

/// Deterministic IDE shell diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeIssue {
    /// Stable issue code.
    pub code: IdeIssueCode,
    /// Affected field or adapter name.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl IdeIssue {
    fn new(code: IdeIssueCode, subject: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Validated shell model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeShellModel {
    /// Computed regions.
    pub regions: IdeShellRegions,
    /// Projected command surfaces.
    pub commands: IdeCommandSurfaces,
    /// Deterministic diagnostics.
    pub issues: Vec<IdeIssue>,
}

impl IdeShellModel {
    /// Returns true when no diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Builds a validated terminal IDE shell model.
#[must_use]
pub fn build_ide_shell(
    config: &IdeShellConfig,
    adapters: &IdeAdapterSet,
    actions: Vec<crate::action_registry::ActionDescriptor>,
) -> IdeShellModel {
    let regions = compute_ide_regions(config);
    let registry = crate::action_registry::build_registry(actions);
    let mut issues = validate_ide_shell(config, adapters, &regions);
    issues.extend(commands::registry_issues(&registry));
    let commands = commands::command_surfaces(&registry);
    IdeShellModel {
        regions,
        commands,
        issues,
    }
}

/// Computes stable shell regions for a viewport.
#[must_use]
pub fn compute_ide_regions(config: &IdeShellConfig) -> IdeShellRegions {
    let chrome_height = COMMAND_HEIGHT.saturating_add(STATUS_HEIGHT);
    assert!(chrome_height >= COMMAND_HEIGHT);
    assert!(chrome_height >= STATUS_HEIGHT);

    let usable_height = config.height.saturating_sub(chrome_height);
    let navigation_width = if config.navigation_visible {
        NAVIGATION_WIDTH.min(config.width)
    } else {
        0
    };
    let auxiliary_width = if config.auxiliary_visible {
        AUXILIARY_WIDTH.min(config.width.saturating_sub(navigation_width))
    } else {
        0
    };
    let main_x = navigation_width;
    let main_width = config
        .width
        .saturating_sub(navigation_width)
        .saturating_sub(auxiliary_width);
    let auxiliary_x = main_x.saturating_add(main_width);
    let command_y = usable_height;
    let status_y = config.height.saturating_sub(STATUS_HEIGHT);

    IdeShellRegions {
        navigation: IdeRect::new(0, 0, navigation_width, usable_height),
        main: IdeRect::new(main_x, 0, main_width, usable_height),
        auxiliary: IdeRect::new(auxiliary_x, 0, auxiliary_width, usable_height),
        command: IdeRect::new(
            0,
            command_y,
            config.width,
            COMMAND_HEIGHT.min(config.height),
        ),
        status: IdeRect::new(0, status_y, config.width, STATUS_HEIGHT.min(config.height)),
    }
}

fn validate_ide_shell(
    config: &IdeShellConfig,
    adapters: &IdeAdapterSet,
    regions: &IdeShellRegions,
) -> Vec<IdeIssue> {
    let mut issues = adapter_issues(adapters);
    if config.persisted_layout_version != Some(IdeLayoutVersion::CURRENT.value()) {
        issues.push(IdeIssue::new(
            IdeIssueCode::LayoutRestoreFailed,
            "persisted_layout_version",
            "persisted IDE layout version is unsupported",
        ));
    }
    if regions.main.width < MIN_MAIN_WIDTH || regions.main.height < MIN_MAIN_HEIGHT {
        issues.push(IdeIssue::new(
            IdeIssueCode::LayoutTooSmall,
            "main",
            "main IDE region is too small; fallback shell should render",
        ));
    }
    issues
}

fn adapter_issues(adapters: &IdeAdapterSet) -> Vec<IdeIssue> {
    [
        ("project_tree", adapters.project_tree),
        ("buffers", adapters.buffers),
        ("diagnostics", adapters.diagnostics),
        ("preview", adapters.preview),
        ("tasks", adapters.tasks),
    ]
    .into_iter()
    .filter(|(_, available)| !available)
    .map(|(name, _)| {
        IdeIssue::new(
            IdeIssueCode::MissingAdapter,
            name,
            format!("required IDE adapter `{name}` is missing"),
        )
    })
    .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
