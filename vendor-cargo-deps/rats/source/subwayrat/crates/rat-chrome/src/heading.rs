//! Title bar model, layout, and renderer.
//!
//! The model is pure state: label text, alignment, indicator labels, and the
//! currently focused indicator. Rendering is a thin shell around the pure
//! layout/truncation helpers in this module.

use ratatui::layout::Rect;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

const TITLE_BAR_ROW_HEIGHT: u16 = 1;
const INDICATOR_GAP: u16 = 1;
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;

/// Horizontal title alignment inside the label region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TitleAlignment {
    /// Align the label to the left edge.
    #[default]
    Left,
    /// Center the label in the space left after right-aligned indicators.
    Center,
    /// Align the label against the indicator gap or right edge.
    Right,
}

/// Pure data model for a title bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleBarModel {
    /// Main title text.
    pub label: String,
    /// Label alignment in the available label region.
    pub alignment: TitleAlignment,
    /// Right-aligned indicator labels such as minimize/maximize/close glyphs.
    pub indicators: Vec<String>,
    /// Focused indicator index, when indicator keyboard focus is active.
    pub focused_indicator: Option<usize>,
}

impl TitleBarModel {
    /// Create a title bar model with a label and no indicators.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            alignment: TitleAlignment::Left,
            indicators: Vec::new(),
            focused_indicator: None,
        }
    }

    /// Set the title label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the label alignment.
    pub fn with_alignment(mut self, alignment: TitleAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set the right-aligned indicators.
    pub fn with_indicators<I, S>(mut self, indicators: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.indicators = indicators.into_iter().map(Into::into).collect();
        self.clamp_focus();
        self
    }

    /// Focus an indicator, clamping to the nearest valid index.
    pub fn with_focused_indicator(mut self, index: usize) -> Self {
        self.focus_indicator(index);
        self
    }

    /// Focus an indicator, clamping to the nearest valid index.
    pub fn focus_indicator(&mut self, index: usize) -> Option<usize> {
        let last_index = self.last_indicator_index()?;
        self.focused_indicator = Some(index.min(last_index));
        self.focused_indicator
    }

    /// Move indicator focus one step left, clamping at the first indicator.
    pub fn move_left(&mut self) -> Option<usize> {
        let last_index = self.last_indicator_index()?;
        self.focused_indicator = Some(match self.focused_indicator {
            Some(index) => index.saturating_sub(1),
            None => last_index,
        });
        self.focused_indicator
    }

    /// Move indicator focus one step right, clamping at the last indicator.
    pub fn move_right(&mut self) -> Option<usize> {
        let last_index = self.last_indicator_index()?;
        self.focused_indicator = Some(match self.focused_indicator {
            Some(index) => (index + 1).min(last_index),
            None => 0,
        });
        self.focused_indicator
    }

    /// Clear indicator focus.
    pub fn clear_focus(&mut self) {
        self.focused_indicator = None;
    }

    fn clamp_focus(&mut self) {
        let Some(index) = self.focused_indicator else {
            return;
        };
        self.focused_indicator = self.last_indicator_index().map(|last| index.min(last));
    }

    fn last_indicator_index(&self) -> Option<usize> {
        self.indicators.len().checked_sub(1)
    }
}

/// Visual style for a title bar.
#[derive(Debug, Clone)]
pub struct TitleBarStyle {
    /// Style used to fill the full title bar area.
    pub background: ratatui::style::Style,
    /// Style applied to the label.
    pub label: ratatui::style::Style,
    /// Style applied to non-focused indicators.
    pub indicator: ratatui::style::Style,
    /// Style applied to the focused indicator.
    pub focused_indicator: ratatui::style::Style,
}

/// Layout information for one rendered indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TitleBarIndicatorLayout {
    /// Index into `TitleBarModel::indicators`.
    pub index: usize,
    /// Indicator rectangle.
    pub rect: Rect,
}

/// Computed title bar layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleBarLayout {
    /// Label rectangle after truncation and alignment.
    pub label: Rect,
    /// Label text after truncation.
    pub rendered_label: String,
    /// Right-aligned indicator rectangles.
    pub indicators: Vec<TitleBarIndicatorLayout>,
}

/// Compute title bar label and indicator layout without rendering.
pub fn compute_title_bar_layout(area: Rect, model: &TitleBarModel) -> TitleBarLayout {
    if area.width == 0 || area.height == 0 {
        return TitleBarLayout::default();
    }

    let indicators = compute_indicator_layouts(area, &model.indicators);
    let label_region_right = indicators
        .first()
        .map(|indicator| indicator.rect.x.saturating_sub(INDICATOR_GAP).max(area.x))
        .unwrap_or_else(|| rect_right(area));
    let label_region_width = label_region_right.saturating_sub(area.x);
    let rendered_label = truncate_to_width(&model.label, label_region_width);
    let rendered_label_width = text_width_u16(&rendered_label).min(label_region_width);
    let label_x = align_label_x(area.x, label_region_width, rendered_label_width, model.alignment);

    TitleBarLayout {
        label: Rect::new(label_x, area.y, rendered_label_width, TITLE_BAR_ROW_HEIGHT),
        rendered_label,
        indicators,
    }
}

fn compute_indicator_layouts(area: Rect, indicators: &[String]) -> Vec<TitleBarIndicatorLayout> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let mut cursor_right = rect_right(area);
    let mut layouts = Vec::with_capacity(indicators.len());

    for (index, label) in indicators.iter().enumerate().rev() {
        let width = text_width_u16(label);
        if width == 0 {
            continue;
        }

        let remaining = cursor_right.saturating_sub(area.x);
        if width > remaining {
            break;
        }

        let x = cursor_right - width;
        layouts.push(TitleBarIndicatorLayout {
            index,
            rect: Rect::new(x, area.y, width, TITLE_BAR_ROW_HEIGHT),
        });
        cursor_right = x.saturating_sub(INDICATOR_GAP).max(area.x);
    }

    layouts.reverse();
    layouts
}

fn align_label_x(label_region_x: u16, label_region_width: u16, label_width: u16, alignment: TitleAlignment) -> u16 {
    match alignment {
        TitleAlignment::Left => label_region_x,
        TitleAlignment::Center => label_region_x + label_region_width.saturating_sub(label_width) / 2,
        TitleAlignment::Right => label_region_x + label_region_width.saturating_sub(label_width),
    }
}

fn truncate_to_width(text: &str, max_width: u16) -> String {
    let max_width = usize::from(max_width);
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    if max_width <= ELLIPSIS_WIDTH {
        return ELLIPSIS.to_string();
    }

    let content_width = max_width - ELLIPSIS_WIDTH;
    let mut used_width = 0usize;
    let mut truncated = String::new();

    for character in text.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if used_width + character_width > content_width {
            break;
        }
        used_width += character_width;
        truncated.push(character);
    }

    truncated.push_str(ELLIPSIS);
    truncated
}

fn text_width_u16(text: &str) -> u16 {
    let width = UnicodeWidthStr::width(text);
    width.min(usize::from(u16::MAX)) as u16
}

fn rect_right(rect: Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}
