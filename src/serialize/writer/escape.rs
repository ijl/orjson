// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// This is an adaptation of `src/value/ser.rs` from serde-json.

use crate::serialize::writer::WriteExt;
use std::io;

pub enum CharEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped tab character (usually escaped as `\t`)
    Tab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\u00XX` where `XX` are two hex characters)
    AsciiControl(u8),
}

impl CharEscape {
    #[inline]
    fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
        match escape {
            self::BB => CharEscape::Backspace,
            self::TT => CharEscape::Tab,
            self::NN => CharEscape::LineFeed,
            self::FF => CharEscape::FormFeed,
            self::RR => CharEscape::CarriageReturn,
            self::QU => CharEscape::Quote,
            self::BS => CharEscape::ReverseSolidus,
            self::UU => CharEscape::AsciiControl(byte),
            _ => unreachable!(),
        }
    }
}

#[inline]
fn write_char_escape<W>(writer: &mut W, char_escape: CharEscape) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
{
    use CharEscape::*;

    let s = match char_escape {
        Quote => b"\\\"",
        ReverseSolidus => b"\\\\",
        Backspace => b"\\b",
        FormFeed => b"\\f",
        LineFeed => b"\\n",
        CarriageReturn => b"\\r",
        Tab => b"\\t",
        AsciiControl(byte) => {
            static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
            let bytes = &[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ];
            return unsafe { writer.write_reserved_fragment(bytes) };
        }
    };

    unsafe { writer.write_reserved_fragment(s) }
}

#[inline(never)]
pub fn format_escaped_str<W>(writer: &mut W, value: &str) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
{
    let len = value.len();

    if len == 0 {
        reserve_minimum!(writer);
        return unsafe { writer.write_reserved_fragment(b"\"\"") };
    }
    unsafe {
        let mut escapes: u8 = __;
        let mut idx = 0;
        let as_bytes = value.as_bytes();
        while idx < len.saturating_sub(8) {
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 1) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 2) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 3) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 4) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 5) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 6) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 7) as usize);
            if unlikely!(escapes != __) {
                return format_escaped_str_with_escapes(writer, as_bytes, idx);
            }
            idx += 8;
        }
        while idx < len {
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx) as usize);
            if unlikely!(escapes != __) {
                return format_escaped_str_with_escapes(writer, as_bytes, idx);
            }
            idx += 1;
        }
    }

    writer.write_str(value)
}

fn format_escaped_str_with_escapes<W>(
    writer: &mut W,
    value: &[u8],
    initial: usize,
) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
{
    writer.reserve((value.len() * 8) + 2);
    unsafe {
        writer.write_reserved_punctuation(b'"').unwrap();
        if initial > 0 {
            writer
                .write_reserved_fragment(value.get_unchecked(0..initial))
                .unwrap();
        }
        format_escaped_str_contents(writer, value.get_unchecked(initial..)).unwrap();
        writer.write_reserved_punctuation(b'"').unwrap();
    };
    Ok(())
}

fn format_escaped_str_contents<W>(writer: &mut W, bytes: &[u8]) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
{
    let len = bytes.len();
    let mut start = 0;
    let mut idx = 0;

    let mut escape: u8;
    loop {
        if idx < len.saturating_sub(4) {
            escape = 0;
            unsafe {
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 1) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 2) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 3) as usize);
            }
            if escape == 0 {
                idx += 4;
                continue;
            }
        }

        let byte = unsafe { *bytes.get_unchecked(idx) };
        escape = unsafe { *ESCAPE.get_unchecked(byte as usize) };
        if escape == 0 {
            idx += 1;
            if idx == len {
                break;
            } else {
                continue;
            }
        }

        if start < idx {
            unsafe {
                writer
                    .write_reserved_fragment(bytes.get_unchecked(start..idx))
                    .unwrap()
            };
        }

        let char_escape = CharEscape::from_escape_table(escape, byte);
        write_char_escape(writer, char_escape)?;

        idx += 1;
        start = idx;
        if idx == len {
            break;
        }
    }

    if start != len {
        unsafe {
            writer
                .write_reserved_fragment(bytes.get_unchecked(start..len))
                .unwrap()
        };
    }
    Ok(())
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
const ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];
