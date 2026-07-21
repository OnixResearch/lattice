const BACKSLASH: char = '\\';
const DOUBLE_QUOTE: char = '"';
const NEWLINE: char = '\n';
const CARRIAGE_RETURN: char = '\r';
const TAB: char = '\t';

pub(crate) fn scheme_string(value: &str) -> String {
    let mut quoted = String::from("\"");
    for character in value.chars() {
        match character {
            BACKSLASH => quoted.push_str("\\\\"),
            DOUBLE_QUOTE => quoted.push_str("\\\""),
            NEWLINE => quoted.push_str("\\n"),
            CARRIAGE_RETURN => quoted.push_str("\\r"),
            TAB => quoted.push_str("\\t"),
            other => quoted.push(other),
        }
    }
    quoted.push(DOUBLE_QUOTE);
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN_TEXT: &str = "hello";
    const QUOTED_PLAIN_TEXT: &str = "\"hello\"";
    const SPECIAL_TEXT: &str = "line\n\"quoted\"\\path";
    const QUOTED_SPECIAL_TEXT: &str = "\"line\\n\\\"quoted\\\"\\\\path\"";

    #[test]
    fn quotes_plain_text() {
        assert_eq!(scheme_string(PLAIN_TEXT), QUOTED_PLAIN_TEXT);
    }

    #[test]
    fn escapes_special_text() {
        assert_eq!(scheme_string(SPECIAL_TEXT), QUOTED_SPECIAL_TEXT);
    }
}
