//! Toast notification overlay

use std::time::Duration;

use rat_chrome::overlay_frame;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;

use crate::popup_chrome::POPUP_MARGIN;
use crate::popup_chrome::clamped_axis;
use crate::popup_chrome::themed_overlay_style;
use crate::popup_chrome::top_right_overlay_model;
use crate::theme::WidgetTheme;

const INFO_TTL_SECS: u64 = 3;
const WARNING_TTL_SECS: u64 = 5;
const ERROR_TTL_SECS: u64 = 8;
const NOTIFICATION_WIDTH: u16 = 50;
const NOTIFICATION_HEIGHT: u16 = 3;
const INFO_TITLE: &str = " Info ";
const WARNING_TITLE: &str = " Warning ";
const ERROR_TITLE: &str = " Error ";

pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

/// Caller-owned monotonic observation in milliseconds from an application epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NotificationMoment(u64);

impl NotificationMoment {
    pub const fn from_millis(milliseconds: u64) -> Self {
        Self(milliseconds)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created: NotificationMoment,
    pub ttl: Duration,
}

impl Notification {
    pub fn info_at(message: impl Into<String>, created: NotificationMoment) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Info,
            created,
            ttl: Duration::from_secs(INFO_TTL_SECS),
        }
    }

    pub fn warning_at(message: impl Into<String>, created: NotificationMoment) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Warning,
            created,
            ttl: Duration::from_secs(WARNING_TTL_SECS),
        }
    }

    pub fn error_at(message: impl Into<String>, created: NotificationMoment) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Error,
            created,
            ttl: Duration::from_secs(ERROR_TTL_SECS),
        }
    }

    pub fn is_expired_at(&self, observed: NotificationMoment) -> bool {
        let elapsed = observed.as_millis().saturating_sub(self.created.as_millis());
        Duration::from_millis(elapsed) >= self.ttl
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        self.render_themed(frame, area, &WidgetTheme::default());
    }

    pub fn render_themed(&self, frame: &mut Frame, area: Rect, theme: &WidgetTheme) {
        let chrome = notification_chrome(&self.level, theme);
        let model = notification_overlay_model(area, chrome.title);
        let style = themed_overlay_style(theme, chrome.border_color);
        let layout = overlay_frame(frame, area, &model, &style);

        if layout.inner.width == 0 || layout.inner.height == 0 {
            return;
        }

        let paragraph = Paragraph::new(Line::from(self.message.clone())).wrap(Wrap { trim: true });
        frame.render_widget(paragraph, layout.inner);
    }
}

struct NotificationChrome {
    border_color: ratatui::style::Color,
    title: &'static str,
}

fn notification_chrome(level: &NotificationLevel, theme: &WidgetTheme) -> NotificationChrome {
    match level {
        NotificationLevel::Info => NotificationChrome {
            border_color: theme.primary,
            title: INFO_TITLE,
        },
        NotificationLevel::Warning => NotificationChrome {
            border_color: theme.warning,
            title: WARNING_TITLE,
        },
        NotificationLevel::Error => NotificationChrome {
            border_color: theme.error,
            title: ERROR_TITLE,
        },
    }
}

fn notification_overlay_model(area: Rect, title: impl Into<String>) -> rat_chrome::OverlayModel {
    let width = clamped_axis(area.width, NOTIFICATION_WIDTH, POPUP_MARGIN);
    let height = NOTIFICATION_HEIGHT.min(area.height);

    top_right_overlay_model(width, height, title)
}

#[cfg(test)]
mod tests {
    use rat_chrome::compute_overlay_layout;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::Color;
    use ratatui::text::Line;

    use super::*;

    const VIEWPORT_WIDTH: u16 = 80;
    const VIEWPORT_HEIGHT: u16 = 12;
    const TINY_VIEWPORT_WIDTH: u16 = 1;
    const TINY_VIEWPORT_HEIGHT: u16 = 1;
    const BACKGROUND_CELL: &str = "x";
    const MESSAGE: &str = "Saved";

    #[test]
    fn warning_notification_uses_top_right_chrome() {
        let notification = Notification::warning_at(MESSAGE, moment(0));

        let buffer = render_notification(&notification, viewport());
        let layout = notification_layout(viewport(), &notification);

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Yellow);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), "S");
    }

    #[test]
    fn error_notification_uses_error_border() {
        let notification = Notification::error_at(MESSAGE, moment(0));

        let buffer = render_notification(&notification, viewport());
        let layout = notification_layout(viewport(), &notification);

        assert_eq!(buffer[(layout.outer.x, layout.outer.y)].fg, Color::Red);
    }

    #[test]
    fn tiny_notification_area_does_not_panic() {
        let notification = Notification::info_at(MESSAGE, moment(0));

        let buffer = render_notification(&notification, tiny_viewport());

        assert_eq!(buffer.area.width, TINY_VIEWPORT_WIDTH);
        assert_eq!(buffer.area.height, TINY_VIEWPORT_HEIGHT);
    }

    #[test]
    fn explicit_time_controls_expiry_without_an_ambient_read() {
        let created = moment(10_000);
        let notification = Notification::info_at(MESSAGE, created);

        assert!(!notification.is_expired_at(created));
        assert!(!notification.is_expired_at(moment(9_000)));
        assert!(notification.is_expired_at(moment(13_000)));
    }

    const fn moment(milliseconds: u64) -> NotificationMoment {
        NotificationMoment::from_millis(milliseconds)
    }

    fn render_notification(notification: &Notification, viewport: Rect) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_test_background(frame, area);
                notification.render(frame, area);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_test_background(frame: &mut Frame, area: Rect) {
        let line = Line::from(BACKGROUND_CELL.repeat(usize::from(area.width)));
        let rows = vec![line; usize::from(area.height)];
        frame.render_widget(Paragraph::new(rows), area);
    }

    fn notification_layout(viewport: Rect, notification: &Notification) -> rat_chrome::OverlayLayout {
        let theme = WidgetTheme::default();
        let chrome = notification_chrome(&notification.level, &theme);
        compute_overlay_layout(viewport, &notification_overlay_model(viewport, chrome.title))
    }

    fn viewport() -> Rect {
        Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn tiny_viewport() -> Rect {
        Rect::new(0, 0, TINY_VIEWPORT_WIDTH, TINY_VIEWPORT_HEIGHT)
    }
}
