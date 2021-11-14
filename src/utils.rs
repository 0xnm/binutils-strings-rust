use super::strings::EncodingKind;

pub(crate) fn char_is_printable(c: char, encoding: EncodingKind,
                                include_all_whitespace: bool) -> bool {
    return c >= '\x00' && c <= '\u{ff}' &&
        (c == '\t' ||
            is_printable_ascii(c) ||
            (matches!(encoding, EncodingKind::Bit8) && c > '\x7f') ||
            (include_all_whitespace && (c.is_ascii_whitespace() || c == '\x0b')));
}

pub(crate) fn to_little_endian_32(symbol: u32) -> u32 {
    return ((symbol & 0xff) << 24) | ((symbol & 0xff00) << 8) |
        ((symbol & 0xff0000) >> 8) | ((symbol & 0xff000000) >> 24);
}

pub(crate) fn to_little_endian_16(symbol: u32) -> u32 {
    return ((symbol & 0xff) << 8) | ((symbol & 0xff00) >> 8);
}

fn is_printable_ascii(c: char) -> bool {
    return match c {
        '\x20'..='\x7e' => true,
        _ => false
    };
}

/**
If non-zero, then number of bytes it is using
 */
pub(crate) fn is_valid_utf8(buffer: &[u8]) -> u8 {
    if buffer[0] < 0xc0 {
        return 0;
    }

    if buffer.len() < 2 {
        return 0;
    }

    if (buffer[1] & 0xc0) != 0x80 {
        return 0;
    }

    if (buffer[0] & 0x20) == 0 {
        return 2;
    }

    if buffer.len() < 3 {
        return 0;
    }

    if (buffer[2] & 0xc0) != 0x80 {
        return 0;
    }

    if (buffer[0] & 0x10) == 0 {
        return 3;
    }

    if buffer.len() < 4 {
        return 0;
    }

    if (buffer[3] & 0xc0) != 0x80 {
        return 0;
    }

    return 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_is_printable() {
        for c in ' '..='~' {
            assert!(is_printable_ascii(c))
        }
    }

    #[test]
    fn test_char_is_not_printable() {
        for c in '\0'..' ' {
            assert!(!is_printable_ascii(c))
        }
        assert!(!is_printable_ascii(0x7f as char))
    }

    #[test]
    fn test_char_is_graphic_whitespace() {
        let chars = vec!['\n', '\x0C', '\r', '\x0b'];

        for char in chars {
            assert!(char_is_printable(char, EncodingKind::Bit7, true));
            assert!(!char_is_printable(char, EncodingKind::Bit7, false));
        }
    }

    #[test]
    fn test_char_is_graphic_tab() {
        assert!(char_is_printable('\t', EncodingKind::Bit7, false));
    }

    #[test]
    fn test_char_is_graphic_printable_char() {
        for c in ' '..='~' {
            assert!(char_is_printable(c, EncodingKind::Bit7, false));
        }
    }

    #[test]
    fn test_char_not_is_graphic_unicode_char() {
        assert!(!char_is_printable('\u{100}', EncodingKind::Bit7, false));
    }

    #[test]
    fn test_char_is_graphic_bit8() {
        for char in '\u{80}'..='\u{ff}' {
            assert!(!char_is_printable(char, EncodingKind::Bit7, false));
            assert!(char_is_printable(char, EncodingKind::Bit8, false));
        }
    }
}
