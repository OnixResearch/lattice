//! Key combination parsing and formatting.

const MIN_FUNCTION_KEY_NUMBER: u8 = 1;
const MAX_FUNCTION_KEY_NUMBER: u8 = 12;
const MODIFIER_PART_OFFSET: usize = 1;
const MAX_FORMATTED_MODIFIERS: usize = 3;

pub type Code = ratatui::crossterm::event::KeyCode;
pub type Event = ratatui::crossterm::event::KeyEvent;
pub type Modifiers = ratatui::crossterm::event::KeyModifiers;

/// A single key combination (e.g. `Ctrl+Shift+K`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chord {
    pub code: Code,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Chord {
    /// Create a new key combo.
    pub fn new(code: Code, ctrl: bool, alt: bool, shift: bool) -> Self {
        Self {
            code,
            ctrl,
            alt,
            shift,
        }
    }

    /// Create a key combo from a crossterm key event.
    pub fn from_event(event: &Event) -> Self {
        Self {
            code: event.code,
            ctrl: event.modifiers.contains(Modifiers::CONTROL),
            alt: event.modifiers.contains(Modifiers::ALT),
            shift: event.modifiers.contains(Modifiers::SHIFT),
        }
    }
}

// Key string parser lookup table.
const KEY_CODE_NAMES: &[(&str, Code)] = &[
    ("enter", Code::Enter),
    ("return", Code::Enter),
    ("cr", Code::Enter),
    ("esc", Code::Esc),
    ("escape", Code::Esc),
    ("tab", Code::Tab),
    ("backspace", Code::Backspace),
    ("bs", Code::Backspace),
    ("delete", Code::Delete),
    ("del", Code::Delete),
    ("up", Code::Up),
    ("down", Code::Down),
    ("left", Code::Left),
    ("right", Code::Right),
    ("home", Code::Home),
    ("end", Code::End),
    ("pageup", Code::PageUp),
    ("pgup", Code::PageUp),
    ("pagedown", Code::PageDown),
    ("pgdn", Code::PageDown),
    ("space", Code::Char(' ')),
    ("spc", Code::Char(' ')),
    ("/", Code::Char('/')),
];

/// Parse a human-readable key string like `"Ctrl+K"`, `"Alt+Enter"`, `"e"`.
pub fn parse_key_string(s: &str) -> Option<Chord> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    debug_assert!(!parts.is_empty(), "split always yields at least one part");
    debug_assert!(
        parts.len() >= MODIFIER_PART_OFFSET,
        "at least one key part must exist"
    );

    let key_str = parts.last()?;
    let modifier_end = parts.len().saturating_sub(MODIFIER_PART_OFFSET);

    let mut has_ctrl = false;
    let mut has_alt = false;
    let mut has_shift = false;

    for part in &parts[..modifier_end] {
        match part.to_lowercase().as_str() {
            "ctrl" => has_ctrl = true,
            "alt" => has_alt = true,
            "shift" => has_shift = true,
            _ => {}
        }
    }

    let key_lower = key_str.to_lowercase();
    let code = KEY_CODE_NAMES
        .iter()
        .find(|(name, _)| *name == key_lower)
        .map(|(_, code)| *code)
        .or_else(|| parse_literal_key_code(key_str))?;

    Some(Chord {
        code,
        ctrl: has_ctrl,
        alt: has_alt,
        shift: has_shift,
    })
}

fn parse_literal_key_code(key_str: &str) -> Option<Code> {
    if key_str.len() == MODIFIER_PART_OFFSET {
        return key_str.chars().next().map(Code::Char);
    }

    if !key_str.to_lowercase().starts_with('f') || key_str.len() <= MODIFIER_PART_OFFSET {
        return None;
    }

    let key_number = key_str[MODIFIER_PART_OFFSET..].parse::<u8>().ok()?;
    if !(MIN_FUNCTION_KEY_NUMBER..=MAX_FUNCTION_KEY_NUMBER).contains(&key_number) {
        return None;
    }

    Some(Code::F(key_number))
}

/// Format a key combo into a human-readable string.
pub fn format(k: &Chord) -> String {
    let mut parts = Vec::new();
    debug_assert!(parts.is_empty(), "formatter starts with no modifier parts");

    if k.ctrl {
        parts.push("Ctrl".to_string());
    }
    if k.alt {
        parts.push("Alt".to_string());
    }
    if k.shift {
        parts.push("Shift".to_string());
    }
    debug_assert!(
        parts.len() <= MAX_FORMATTED_MODIFIERS,
        "only supported modifiers are formatted"
    );

    parts.push(format_code(k.code));
    parts.join("+")
}

fn format_code(code: Code) -> String {
    match code {
        Code::Char(' ') => "Space".to_string(),
        Code::Char(c) => c.to_string(),
        Code::Enter => "Enter".to_string(),
        Code::Esc => "Esc".to_string(),
        Code::Tab => "Tab".to_string(),
        Code::Backspace => "Backspace".to_string(),
        Code::Delete => "Delete".to_string(),
        Code::Up => "Up".to_string(),
        Code::Down => "Down".to_string(),
        Code::Left => "Left".to_string(),
        Code::Right => "Right".to_string(),
        Code::Home => "Home".to_string(),
        Code::End => "End".to_string(),
        Code::PageUp => "PageUp".to_string(),
        Code::PageDown => "PageDown".to_string(),
        Code::F(n) => format!("F{}", n),
        other => format!("{:?}", other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_FUNCTION_KEY: u8 = 1;
    const OUT_OF_RANGE_FUNCTION_KEY: &str = "F13";

    #[test]
    fn parse_simple_key() {
        let combo = parse_key_string("k").unwrap();
        assert_eq!(combo.code, Code::Char('k'));
        assert!(!combo.ctrl);
        assert!(!combo.alt);
        assert!(!combo.shift);
    }

    #[test]
    fn parse_ctrl_key() {
        let combo = parse_key_string("Ctrl+c").unwrap();
        assert_eq!(combo.code, Code::Char('c'));
        assert!(combo.ctrl);
        assert!(!combo.alt);
        assert!(!combo.shift);
    }

    #[test]
    fn parse_ctrl_shift_function_key() {
        let combo = parse_key_string("Ctrl+Shift+F1").unwrap();
        assert_eq!(combo.code, Code::F(FIRST_FUNCTION_KEY));
        assert!(combo.ctrl);
        assert!(!combo.alt);
        assert!(combo.shift);
    }

    #[test]
    fn parse_special_keys() {
        assert_eq!(parse_key_string("enter").unwrap().code, Code::Enter);
        assert_eq!(parse_key_string("esc").unwrap().code, Code::Esc);
        assert_eq!(parse_key_string("space").unwrap().code, Code::Char(' '));
    }

    #[test]
    fn parse_rejects_unknown_key_name() {
        assert_eq!(parse_key_string("definitely-not-a-key"), None);
    }

    #[test]
    fn parse_rejects_out_of_range_function_key() {
        assert_eq!(parse_key_string(OUT_OF_RANGE_FUNCTION_KEY), None);
    }

    #[test]
    fn format_simple_key() {
        let combo = Chord::new(Code::Char('q'), false, false, false);
        assert_eq!(format(&combo), "q");
    }

    #[test]
    fn format_ctrl_key() {
        let combo = Chord::new(Code::Char('c'), true, false, false);
        assert_eq!(format(&combo), "Ctrl+c");
    }

    #[test]
    fn format_ctrl_shift_function_key() {
        let combo = Chord::new(Code::F(FIRST_FUNCTION_KEY), true, false, true);
        assert_eq!(format(&combo), "Ctrl+Shift+F1");
    }

    #[test]
    fn from_event() {
        let event = Event::new(Code::Char('k'), Modifiers::CONTROL);
        let combo = Chord::from_event(&event);
        assert_eq!(combo.code, Code::Char('k'));
        assert!(combo.ctrl);
        assert!(!combo.alt);
        assert!(!combo.shift);
    }
}
