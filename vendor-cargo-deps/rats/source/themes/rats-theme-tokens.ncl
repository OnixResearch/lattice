let HexColor = String in
let SemanticColors = {
  primary | HexColor,
  secondary | HexColor,
  background | HexColor,
  surface | HexColor,
  text | HexColor,
  text_muted | HexColor,
  success | HexColor,
  warning | HexColor,
  error | HexColor,
} in
let Metric = { amount | Number, unit | String } in
let ControlMetrics = {
  minimum_target | Metric,
  gap | Metric,
  padding | Metric,
} in
let ContrastProfile = {
  text_emphasis | Number,
  boundary_emphasis | Number,
  platform_color_deferred | Bool,
} in
let RendererMapping = {
  family | String,
  status | String,
  fallback | String,
  reason | String,
  unit | String,
} in
let AdaptiveTokens = {
  generation | Number,
  surfaces | {
    base | String,
    raised | String,
    overlay | String,
    modal | String,
  },
  typography | {
    body | String,
    label | String,
    heading | String,
    code | String,
  },
  density_profiles | {
    compact | ControlMetrics,
    comfortable | ControlMetrics,
    touch | ControlMetrics,
  },
  contrast_profiles | {
    standard | ContrastProfile,
    increased | ContrastProfile,
    forced_platform | ContrastProfile,
  },
  motion_profiles | { full | String, reduced | String },
  focus | { role | String, thickness | Metric, visibility | String },
  state_roles | {
    base | String,
    hovered | String,
    pressed | String,
    selected | String,
    focused | String,
    invalid | String,
    disabled | String,
  },
  state_precedence | Array String,
  renderer_mappings | {
    ratatui | Array RendererMapping,
    meteorite_css | Array RendererMapping,
    iced_pilot | Array RendererMapping,
  },
} in
let adaptive_defaults = {
  generation = 1,
  surfaces = {
    base = "flat",
    raised = "low",
    overlay = "medium",
    modal = "high",
  },
  typography = {
    body = "regular",
    label = "medium",
    heading = "strong",
    code = "monospace",
  },
  density_profiles = {
    compact = {
      minimum_target = { amount = 2, unit = "logical-step" },
      gap = { amount = 1, unit = "logical-step" },
      padding = { amount = 1, unit = "logical-step" },
    },
    comfortable = {
      minimum_target = { amount = 3, unit = "logical-step" },
      gap = { amount = 2, unit = "logical-step" },
      padding = { amount = 2, unit = "logical-step" },
    },
    touch = {
      minimum_target = { amount = 4, unit = "logical-step" },
      gap = { amount = 2, unit = "logical-step" },
      padding = { amount = 3, unit = "logical-step" },
    },
  },
  contrast_profiles = {
    standard = {
      text_emphasis = 1,
      boundary_emphasis = 1,
      platform_color_deferred = false,
    },
    increased = {
      text_emphasis = 2,
      boundary_emphasis = 3,
      platform_color_deferred = false,
    },
    forced_platform = {
      text_emphasis = 3,
      boundary_emphasis = 3,
      platform_color_deferred = true,
    },
  },
  motion_profiles = { full = "full", reduced = "existing-animation-policy" },
  focus = {
    role = "primary",
    thickness = { amount = 2, unit = "logical-step" },
    visibility = "keyboard-or-programmatic",
  },
  state_roles = {
    base = "text",
    hovered = "primary",
    pressed = "primary",
    selected = "primary",
    focused = "primary",
    invalid = "error",
    disabled = "text-muted",
  },
  state_precedence = ["disabled", "invalid", "pressed", "selected", "hovered", "focused", "base"],
  renderer_mappings = {
    ratatui = [
      { family = "surface-elevation", status = "approximated", fallback = "border-and-emphasis", reason = "no shadow elevation", unit = "" },
      { family = "typography", status = "approximated", fallback = "terminal-modifiers", reason = "font selection is renderer-owned", unit = "" },
      { family = "control-metrics", status = "supported", fallback = "", reason = "", unit = "terminal-cell" },
      { family = "focus-indicator", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "interaction-state", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "contrast", status = "approximated", fallback = "palette-and-modifier-emphasis", reason = "forced colors are unavailable", unit = "" },
      { family = "motion", status = "supported", fallback = "", reason = "", unit = "" },
    ],
    meteorite_css = [
      { family = "surface-elevation", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "typography", status = "supported", fallback = "", reason = "", unit = "css-pixel" },
      { family = "control-metrics", status = "supported", fallback = "", reason = "", unit = "css-pixel" },
      { family = "focus-indicator", status = "supported", fallback = "", reason = "", unit = "css-pixel" },
      { family = "interaction-state", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "contrast", status = "deferred-to-platform", fallback = "", reason = "forced-colors system values supersede authored colors", unit = "" },
      { family = "motion", status = "supported", fallback = "", reason = "", unit = "" },
    ],
    iced_pilot = [
      { family = "surface-elevation", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "typography", status = "approximated", fallback = "weight-and-size", reason = "font discovery and exact metrics remain platform-owned", unit = "native-logical-pixel" },
      { family = "control-metrics", status = "supported", fallback = "", reason = "", unit = "native-logical-pixel" },
      { family = "focus-indicator", status = "supported", fallback = "", reason = "", unit = "native-logical-pixel" },
      { family = "interaction-state", status = "supported", fallback = "", reason = "", unit = "" },
      { family = "contrast", status = "approximated", fallback = "authored-palette-emphasis", reason = "native forced-color observation is outside the pilot", unit = "" },
      { family = "motion", status = "supported", fallback = "", reason = "", unit = "" },
    ],
  },
} in
let ThemePreset = {
  name | String,
  variant | String,
  colors | SemanticColors,
  spacing | { compact | Number, normal | Number, relaxed | Number },
  borders | { radius | Number, width | Number },
  emphasis | { focused | String, disabled | String, selected | String },
  renderers | { ratatui | Array String, meteorite | Array String },
  adaptive | AdaptiveTokens,
} in
{
  schema = "rats-theme-tokens/v2",
  presets = [
    {
      name = "rats-terminal-dark",
      variant = "dark",
      colors = {
        primary = "#5ea1ff",
        secondary = "#66d9ef",
        background = "#101216",
        surface = "#1b1f27",
        text = "#f4f4f5",
        text_muted = "#9ca3af",
        success = "#22c55e",
        warning = "#facc15",
        error = "#ef4444",
      },
      spacing = { compact = 1, normal = 2, relaxed = 3 },
      borders = { radius = 1, width = 1 },
      emphasis = { focused = "bold", disabled = "dim", selected = "reverse" },
      renderers = {
        ratatui = ["Color::Rgb", "Modifier"],
        meteorite = ["css-custom-properties"],
      },
      adaptive = adaptive_defaults,
    } | ThemePreset,
    {
      name = "rats-light",
      variant = "light",
      colors = {
        primary = "#2563eb",
        secondary = "#0891b2",
        background = "#ffffff",
        surface = "#f8fafc",
        text = "#111827",
        text_muted = "#6b7280",
        success = "#16a34a",
        warning = "#d97706",
        error = "#dc2626",
      },
      spacing = { compact = 1, normal = 2, relaxed = 4 },
      borders = { radius = 2, width = 1 },
      emphasis = { focused = "bold", disabled = "dim", selected = "reverse" },
      renderers = {
        ratatui = ["Color::Rgb", "Modifier"],
        meteorite = ["css-custom-properties"],
      },
      adaptive = adaptive_defaults,
    } | ThemePreset,
  ],
}
