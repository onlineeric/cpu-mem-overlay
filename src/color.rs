//! Hex-string color parsing for the `background_color` config key.
//!
//! Accepts `"#RRGGBB"` (alpha defaults to `0xFF`) and `"#RRGGBBAA"`,
//! case-insensitive. Returns an opaque `Err` when the input does not match
//! either shape — callers fall back to the default per FR-004.

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ColorParseError {
    MissingHash,
    BadLength,
    NonHexDigit,
}

pub(crate) fn parse_hex_color(s: &str) -> Result<[u8; 4], ColorParseError> {
    let rest = s.strip_prefix('#').ok_or(ColorParseError::MissingHash)?;
    let (r, g, b, a) = match rest.len() {
        6 => (
            parse_byte(&rest[0..2])?,
            parse_byte(&rest[2..4])?,
            parse_byte(&rest[4..6])?,
            0xFF,
        ),
        8 => (
            parse_byte(&rest[0..2])?,
            parse_byte(&rest[2..4])?,
            parse_byte(&rest[4..6])?,
            parse_byte(&rest[6..8])?,
        ),
        _ => return Err(ColorParseError::BadLength),
    };
    Ok([r, g, b, a])
}

fn parse_byte(pair: &str) -> Result<u8, ColorParseError> {
    if !pair.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(ColorParseError::NonHexDigit);
    }
    u8::from_str_radix(pair, 16).map_err(|_| ColorParseError::NonHexDigit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_digit_lowercase() {
        assert_eq!(parse_hex_color("#1b1b1b"), Ok([0x1B, 0x1B, 0x1B, 0xFF]));
    }

    #[test]
    fn six_digit_uppercase() {
        assert_eq!(parse_hex_color("#101820"), Ok([0x10, 0x18, 0x20, 0xFF]));
    }

    #[test]
    fn eight_digit_lowercase() {
        assert_eq!(parse_hex_color("#101820cc"), Ok([0x10, 0x18, 0x20, 0xCC]));
    }

    #[test]
    fn eight_digit_uppercase_partial_alpha() {
        assert_eq!(parse_hex_color("#101820CC"), Ok([0x10, 0x18, 0x20, 0xCC]));
    }

    #[test]
    fn alpha_zero_fully_transparent() {
        assert_eq!(parse_hex_color("#00000000"), Ok([0, 0, 0, 0]));
    }

    #[test]
    fn missing_hash_rejected() {
        assert_eq!(parse_hex_color("1b1b1b"), Err(ColorParseError::MissingHash));
    }

    #[test]
    fn wrong_length_four_rejected() {
        assert_eq!(parse_hex_color("#1b1b"), Err(ColorParseError::BadLength));
    }

    #[test]
    fn wrong_length_five_rejected() {
        assert_eq!(parse_hex_color("#1b1b1"), Err(ColorParseError::BadLength));
    }

    #[test]
    fn wrong_length_seven_rejected() {
        assert_eq!(parse_hex_color("#1b1b1bf"), Err(ColorParseError::BadLength));
    }

    #[test]
    fn wrong_length_nine_rejected() {
        assert_eq!(
            parse_hex_color("#1b1b1bff0"),
            Err(ColorParseError::BadLength)
        );
    }

    #[test]
    fn non_hex_digit_rejected() {
        assert_eq!(
            parse_hex_color("#zzzzzz"),
            Err(ColorParseError::NonHexDigit)
        );
    }

    #[test]
    fn empty_string_rejected() {
        assert_eq!(parse_hex_color(""), Err(ColorParseError::MissingHash));
    }

    #[test]
    fn just_hash_rejected() {
        assert_eq!(parse_hex_color("#"), Err(ColorParseError::BadLength));
    }

    #[test]
    fn mixed_case_succeeds() {
        assert_eq!(parse_hex_color("#aBcDeF12"), Ok([0xAB, 0xCD, 0xEF, 0x12]));
    }
}
