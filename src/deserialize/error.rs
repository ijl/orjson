// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct DeserializeError<'a> {
    pub message: Cow<'a, str>,
    pub line: usize,   // start at 1
    pub column: usize, // start at 1
    pub data: &'a str,
    pub pos: i64,
}

impl<'a> DeserializeError<'a> {
    #[cold]
    pub fn new(message: Cow<'a, str>, line: usize, column: usize, data: &'a str) -> Self {
        DeserializeError {
            message,
            line,
            column,
            data,
            pos: 0,
        }
    }
    #[cold]
    #[cfg(feature = "yyjson")]
    pub fn from_yyjson(message: Cow<'a, str>, pos: i64, data: &'a str) -> Self {
        DeserializeError {
            message,
            line: 0,
            column: 0,
            data,
            pos,
        }
    }

    /// Return position of the error in the deserialized data
    #[cold]
    #[cfg_attr(feature = "unstable-simd", optimize(size))]
    pub fn pos(&self) -> i64 {
        if self.pos != 0 {
            // yyjson
            return bytecount::num_chars(&self.data.as_bytes()[0..self.pos as usize]) as i64;
        }
        if self.line == 0 {
            return 1;
        }

        let val = self.data
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
