// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::borrow::Cow;

pub struct DeserializeError<'a> {
    pub message: Cow<'a, str>,
    #[cfg(not(feature = "yyjson"))]
    pub line: usize, // start at 1
    #[cfg(not(feature = "yyjson"))]
    pub column: usize, // start at 1
    pub data: Option<&'a str>,
    pub pos: i64,
}

impl<'a> DeserializeError<'a> {
    #[cold]
    pub fn invalid(message: Cow<'a, str>) -> Self {
        DeserializeError {
            message: message,
            #[cfg(not(feature = "yyjson"))]
            line: 0,
            #[cfg(not(feature = "yyjson"))]
            column: 0,
            data: None,
            pos: 0,
        }
    }

    #[cold]
    #[cfg(not(feature = "yyjson"))]
    pub fn from_json(message: Cow<'a, str>, line: usize, column: usize, data: &'a str) -> Self {
        DeserializeError {
            message,
            line,
            column,
            data: Some(data),
            pos: 0,
        }
    }

    #[cold]
    #[cfg(feature = "yyjson")]
    pub fn from_yyjson(message: Cow<'a, str>, pos: i64, data: &'a str) -> Self {
        DeserializeError {
            message: message,
            data: Some(data),
            pos: pos,
        }
    }

    /// Return position of the error in the deserialized data
    #[cold]
    #[cfg(feature = "yyjson")]
    pub fn pos(&self) -> i64 {
        match self.data {
            Some(as_str) => bytecount::num_chars(&as_str.as_bytes()[0..self.pos as usize]) as i64,
            None => 0,
        }
    }

    /// Return position of the error in the deserialized data
    #[cold]
    #[cfg(not(feature = "yyjson"))]
    #[cfg_attr(feature = "unstable-simd", optimize(size))]
    pub fn pos(&self) -> i64 {
        if self.line == 0 || self.data.is_none() {
            return 1;
        }

        let val = self.data.unwrap()
            .split('\n')
            // take only the relevant lines
            .take(self.line)
            .enumerate()
            .map(|(idx, s)| {
                if idx == self.line - 1 {
                    // Last line: only characters until the column of the error are relevant.
                    // Note: Rust uses bytes whereas Python uses chars: we hence cannot
                    //       directly use the `column` field
                    if self.column == 0 { return 0; }

                    // Find a column we can safely slice on
                    let mut column = self.column - 1;
                    while column > 0 && !s.is_char_boundary(column) {
                        column -= 1;
                    }

                    let chars_count = s[..column].chars().count();
                    if chars_count == s.chars().count() - 1 {
                        chars_count + 1
                    } else {
                        chars_count
                    }
                } else {
                    // Other lines
                    s.chars().count()
                }
            })
            .sum::<usize>()
            // add missed '\n' characters
            + (self.line - 1);
        val as i64
    }
}
