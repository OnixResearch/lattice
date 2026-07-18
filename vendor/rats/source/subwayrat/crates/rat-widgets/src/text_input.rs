//! Inline single-line text input widget.
//!
//! Unlike [`InputDialog`](crate::InputDialog) (which renders as a centered popup),
//! this widget renders within its given `Rect` — suitable for search boxes,
//! filter inputs, and other inline text fields.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub type Completer = Box<dyn Fn(&str) -> Vec<String>>;

/// Pure data model for text input — no ratatui dependency.
pub struct TextInputModel {
    pub value: String,
    pub cursor_pos: usize,
    pub completer: Option<Completer>,
}

impl TextInputModel {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor_pos: 0,
            completer: None,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, v: &str) {
        self.value = v.to_string();
        self.cursor_pos = self.value.len();
    }

    pub fn type_char(&mut self, c: char) {
        self.value.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        // Walk back one char boundary.
        let prev = self.value[..self.cursor_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.value.remove(prev);
        self.cursor_pos = prev;
    }

    pub fn delete(&mut self) {
        if self.cursor_pos >= self.value.len() {
            return;
        }
        self.value.remove(self.cursor_pos);
    }

    pub fn move_left(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        self.cursor_pos = self.value[..self.cursor_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    pub fn move_right(&mut self) {
        if self.cursor_pos >= self.value.len() {
            return;
        }
        self.cursor_pos = self.value[self.cursor_pos..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor_pos + i)
            .unwrap_or(self.value.len());
    }

    pub fn move_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_pos = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_pos = 0;
    }

    pub fn complete(&mut self) -> Vec<String> {
        let Some(ref completer) = self.completer else {
            return Vec::new();
        };

        let matches = completer(&self.value);

        match matches.len() {
            0 => Vec::new(),
            1 => {
                // Single match: replace value and move cursor to end
                self.value = matches[0].clone();
                self.cursor_pos = self.value.len();
                matches
            }
            _ => {
                // Multiple matches: find longest common prefix
                let common_prefix = longest_common_prefix(&matches);
                if !common_prefix.is_empty() && common_prefix != self.value {
                    self.value = common_prefix;
                    self.cursor_pos = self.value.len();
                }
                matches
            }
        }
    }

    pub fn submit(&mut self) -> String {
        let val = std::mem::take(&mut self.value);
        self.cursor_pos = 0;
        val
    }
}

impl Default for TextInputModel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TextInput {
    pub model: TextInputModel,
    focused: bool,
    focused_border: Color,
    unfocused_border: Color,
    text_style: Style,
    cursor_style: Style,
    placeholder: String,
    placeholder_style: Style,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            model: TextInputModel::new(),
            focused: false,
            focused_border: Color::Cyan,
            unfocused_border: Color::DarkGray,
            text_style: Style::default().fg(Color::White),
            cursor_style: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::SLOW_BLINK),
            placeholder: String::new(),
            placeholder_style: Style::default().fg(Color::DarkGray),
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.model.set_value(&value.into());
        self
    }

    pub fn with_placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_focused_border(mut self, c: Color) -> Self {
        self.focused_border = c;
        self
    }

    pub fn with_unfocused_border(mut self, c: Color) -> Self {
        self.unfocused_border = c;
        self
    }

    pub fn with_text_style(mut self, s: Style) -> Self {
        self.text_style = s;
        self
    }

    pub fn with_completer(mut self, c: Completer) -> Self {
        self.model.completer = Some(c);
        self
    }

    // -- focus management --

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    // -- delegation methods --

    pub fn value(&self) -> &str {
        self.model.value()
    }

    pub fn set_value(&mut self, v: &str) {
        self.model.set_value(v);
    }

    pub fn type_char(&mut self, c: char) {
        self.model.type_char(c);
    }

    pub fn backspace(&mut self) {
        self.model.backspace();
    }

    pub fn delete(&mut self) {
        self.model.delete();
    }

    pub fn move_left(&mut self) {
        self.model.move_left();
    }

    pub fn move_right(&mut self) {
        self.model.move_right();
    }

    pub fn move_home(&mut self) {
        self.model.move_home();
    }

    pub fn move_end(&mut self) {
        self.model.move_end();
    }

    pub fn clear(&mut self) {
        self.model.clear();
    }

    pub fn complete(&mut self) -> Vec<String> {
        self.model.complete()
    }

    pub fn submit(&mut self) -> String {
        self.model.submit()
    }

    // -- rendering --

    pub fn render(&self, frame: &mut Frame, area: Rect, block: Option<Block>) {
        let border_color = if self.focused {
            self.focused_border
        } else {
            self.unfocused_border
        };

        let block = block.unwrap_or_else(|| {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        });

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let visible_width = inner.width as usize;

        // Empty + unfocused → placeholder
        if self.model.value.is_empty() && !self.focused {
            let shown: String = self.placeholder.chars().take(visible_width).collect();
            let line = Line::from(Span::styled(shown, self.placeholder_style));
            frame.render_widget(Paragraph::new(line), inner);
            return;
        }

        // Compute scroll offset so the cursor stays visible.
        let char_count = self.model.value[..self.model.cursor_pos].chars().count();
        let scroll_offset = if char_count >= visible_width {
            char_count - visible_width + 1
        } else {
            0
        };

        // Build visible slice: skip `scroll_offset` chars, take `visible_width`.
        let visible_chars: Vec<char> = self.model.value.chars().skip(scroll_offset).collect();
        let cursor_col = char_count - scroll_offset; // column within visible region

        if self.focused {
            let before: String = visible_chars[..cursor_col].iter().collect();
            let cursor_char = visible_chars
                .get(cursor_col)
                .map(|c| c.to_string())
                .unwrap_or_else(|| " ".to_string());
            let after_start = (cursor_col + 1).min(visible_chars.len());
            let after: String = visible_chars[after_start..]
                .iter()
                .take(visible_width.saturating_sub(cursor_col + 1))
                .collect();

            let spans = vec![
                Span::styled(before, self.text_style),
                Span::styled(cursor_char, self.cursor_style),
                Span::styled(after, self.text_style),
            ];
            frame.render_widget(Paragraph::new(Line::from(spans)), inner);
        } else {
            let shown: String = visible_chars.iter().take(visible_width).collect();
            let line = Line::from(Span::styled(shown, self.text_style));
            frame.render_widget(Paragraph::new(line), inner);
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    if strings.len() == 1 {
        return strings[0].clone();
    }

    let mut prefix = String::new();
    let first_chars: Vec<char> = strings[0].chars().collect();

    for (i, &ch) in first_chars.iter().enumerate() {
        if strings.iter().skip(1).all(|s| s.chars().nth(i) == Some(ch)) {
            prefix.push(ch);
        } else {
            break;
        }
    }

    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_char_inserts_at_cursor() {
        let mut input = TextInput::new();
        input.type_char('a');
        input.type_char('b');
        input.type_char('c');
        assert_eq!(input.value(), "abc");
        assert_eq!(input.model.cursor_pos, 3);
    }

    #[test]
    fn type_char_mid_string() {
        let mut input = TextInput::new().with_value("ac");
        input.move_home();
        input.move_right(); // after 'a'
        input.type_char('b');
        assert_eq!(input.value(), "abc");
        assert_eq!(input.model.cursor_pos, 2);
    }

    #[test]
    fn backspace_deletes_before_cursor() {
        let mut input = TextInput::new().with_value("abc");
        input.backspace();
        assert_eq!(input.value(), "ab");
        assert_eq!(input.model.cursor_pos, 2);
    }

    #[test]
    fn backspace_at_start_is_noop() {
        let mut input = TextInput::new().with_value("abc");
        input.move_home();
        input.backspace();
        assert_eq!(input.value(), "abc");
        assert_eq!(input.model.cursor_pos, 0);
    }

    #[test]
    fn delete_removes_at_cursor() {
        let mut input = TextInput::new().with_value("abc");
        input.move_home();
        input.delete();
        assert_eq!(input.value(), "bc");
        assert_eq!(input.model.cursor_pos, 0);
    }

    #[test]
    fn delete_at_end_is_noop() {
        let mut input = TextInput::new().with_value("abc");
        input.delete();
        assert_eq!(input.value(), "abc");
    }

    #[test]
    fn cursor_movement() {
        let mut input = TextInput::new().with_value("hello");
        assert_eq!(input.model.cursor_pos, 5);

        input.move_home();
        assert_eq!(input.model.cursor_pos, 0);

        input.move_right();
        assert_eq!(input.model.cursor_pos, 1);

        input.move_end();
        assert_eq!(input.model.cursor_pos, 5);

        input.move_left();
        assert_eq!(input.model.cursor_pos, 4);
    }

    #[test]
    fn move_left_at_start_stays() {
        let mut input = TextInput::new().with_value("x");
        input.move_home();
        input.move_left();
        assert_eq!(input.model.cursor_pos, 0);
    }

    #[test]
    fn move_right_at_end_stays() {
        let mut input = TextInput::new().with_value("x");
        input.move_right();
        assert_eq!(input.model.cursor_pos, 1);
    }

    #[test]
    fn submit_clears_and_returns() {
        let mut input = TextInput::new().with_value("query");
        let val = input.submit();
        assert_eq!(val, "query");
        assert_eq!(input.value(), "");
        assert_eq!(input.model.cursor_pos, 0);
    }

    #[test]
    fn clear_empties() {
        let mut input = TextInput::new().with_value("stuff");
        input.clear();
        assert_eq!(input.value(), "");
        assert_eq!(input.model.cursor_pos, 0);
    }

    #[test]
    fn set_value_puts_cursor_at_end() {
        let mut input = TextInput::new();
        input.set_value("hi");
        assert_eq!(input.value(), "hi");
        assert_eq!(input.model.cursor_pos, 2);
    }

    #[test]
    fn focus_and_blur() {
        let mut input = TextInput::new();
        assert!(!input.is_focused());
        input.focus();
        assert!(input.is_focused());
        input.blur();
        assert!(!input.is_focused());
    }

    #[test]
    fn placeholder_preserved() {
        let input = TextInput::new().with_placeholder("Search...");
        assert_eq!(input.placeholder, "Search...");
    }

    #[test]
    fn multibyte_chars() {
        let mut input = TextInput::new().with_value("café");
        // "café" = 5 bytes (é is 2 bytes)
        assert_eq!(input.model.cursor_pos, 5);

        input.backspace(); // remove 'é'
        assert_eq!(input.value(), "caf");
        assert_eq!(input.model.cursor_pos, 3);

        input.type_char('é');
        assert_eq!(input.value(), "café");
        assert_eq!(input.model.cursor_pos, 5);
    }

    #[test]
    fn delete_multibyte_mid() {
        let mut input = TextInput::new().with_value("aéb");
        input.move_home();
        input.move_right(); // past 'a', cursor at byte 1
        input.delete(); // remove 'é'
        assert_eq!(input.value(), "ab");
        assert_eq!(input.model.cursor_pos, 1);
    }

    #[test]
    fn builder_chain() {
        let input = TextInput::new()
            .with_value("init")
            .with_placeholder("hint")
            .with_focused(true)
            .with_focused_border(Color::Green)
            .with_unfocused_border(Color::Red)
            .with_text_style(Style::default().fg(Color::Yellow));

        assert_eq!(input.value(), "init");
        assert!(input.is_focused());
        assert_eq!(input.placeholder, "hint");
        assert_eq!(input.focused_border, Color::Green);
        assert_eq!(input.unfocused_border, Color::Red);
    }

    #[test]
    fn complete_no_completer_returns_empty() {
        let mut input = TextInput::new().with_value("test");
        let matches = input.complete();
        assert!(matches.is_empty());
        assert_eq!(input.value(), "test");
    }

    #[test]
    fn complete_no_matches_returns_empty() {
        let mut input = TextInput::new()
            .with_value("xyz")
            .with_completer(Box::new(|_| vec![]));
        let matches = input.complete();
        assert!(matches.is_empty());
        assert_eq!(input.value(), "xyz");
    }

    #[test]
    fn complete_single_match_replaces_value() {
        let mut input = TextInput::new()
            .with_value("he")
            .with_completer(Box::new(|s| {
                if s.starts_with("he") {
                    vec!["hello".to_string()]
                } else {
                    vec![]
                }
            }));
        let matches = input.complete();
        assert_eq!(matches, vec!["hello"]);
        assert_eq!(input.value(), "hello");
        assert_eq!(input.model.cursor_pos, 5);
    }

    #[test]
    fn complete_multiple_matches_common_prefix() {
        let mut input = TextInput::new()
            .with_value("te")
            .with_completer(Box::new(|s| {
                if s.starts_with("te") {
                    vec![
                        "test".to_string(),
                        "testing".to_string(),
                        "temp".to_string(),
                    ]
                } else {
                    vec![]
                }
            }));
        let matches = input.complete();
        assert_eq!(matches, vec!["test", "testing", "temp"]);
        assert_eq!(input.value(), "te"); // common prefix is just "te"
    }

    #[test]
    fn complete_multiple_matches_longer_prefix() {
        let mut input = TextInput::new()
            .with_value("test")
            .with_completer(Box::new(|s| {
                if s.starts_with("test") {
                    vec!["testing".to_string(), "tester".to_string()]
                } else {
                    vec![]
                }
            }));
        let matches = input.complete();
        assert_eq!(matches, vec!["testing", "tester"]);
        assert_eq!(input.value(), "test"); // common prefix is still just "test"
    }

    #[test]
    fn longest_common_prefix_empty() {
        assert_eq!(longest_common_prefix(&[]), "");
    }

    #[test]
    fn longest_common_prefix_single() {
        assert_eq!(longest_common_prefix(&["hello".to_string()]), "hello");
    }

    #[test]
    fn longest_common_prefix_multiple() {
        assert_eq!(
            longest_common_prefix(&[
                "test".to_string(),
                "testing".to_string(),
                "tester".to_string()
            ]),
            "test"
        );
    }

    #[test]
    fn longest_common_prefix_no_common() {
        assert_eq!(
            longest_common_prefix(&["abc".to_string(), "def".to_string()]),
            ""
        );
    }
}
