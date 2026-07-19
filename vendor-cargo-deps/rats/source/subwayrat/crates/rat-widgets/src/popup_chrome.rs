//! Shared rat-chrome overlay helpers for rat-widgets popup shells.

use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle};
use ratatui::style::Style;

use crate::theme::WidgetTheme;

pub const POPUP_MARGIN: u16 = 4;
pub const TOAST_OFFSET_X: i16 = -2;
pub const TOAST_OFFSET_Y: i16 = 1;

pub fn centered_overlay_model(width: u16, height: u16, title: impl Into<String>) -> OverlayModel {
    OverlayModel::default()
        .with_anchor(OverlayAnchor::Center)
        .with_width(OverlaySize::Fixed(width))
        .with_height(OverlaySize::Fixed(height))
        .with_clear(true)
        .with_title(title)
}

pub fn top_right_overlay_model(width: u16, height: u16, title: impl Into<String>) -> OverlayModel {
    OverlayModel::default()
        .with_anchor(OverlayAnchor::TopRight)
        .with_offsets(TOAST_OFFSET_X, TOAST_OFFSET_Y)
        .with_width(OverlaySize::Fixed(width))
        .with_height(OverlaySize::Fixed(height))
        .with_clear(true)
        .with_title(title)
}

pub fn themed_overlay_style(theme: &WidgetTheme, border: ratatui::style::Color) -> OverlayStyle {
    OverlayStyle::default()
        .with_border(Style::default().fg(border))
        .with_title(Style::default().fg(border))
        .with_fill(Style::default().bg(theme.background))
}

pub fn title(text: &str) -> String {
    format!(" {text} ")
}

pub fn clamped_axis(available: u16, preferred: u16, margin: u16) -> u16 {
    preferred.min(available.saturating_sub(margin))
}

pub fn usize_to_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
}
