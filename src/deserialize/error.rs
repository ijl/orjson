// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct DeserializeError<'a> {
    pub message: Cow<'a, str>,
    pub line: usize,   // start at 1
    pub column: usize, // start at 1
    pub data: &'a str,
}

impl<'a> DeserializeError<'a> {
    #[cold]
    pub fn new(message: Cow<'a, str>, line: usize, column: usize, data: &'a str) -> Self {
        DeserializeError {
            message,
            line,
            column,
            data,
        }
    }

    /// Return position of the error in the deserialized data
    #[cold]
    pub fn pos(&self) -> usize {
        if self.line == 0 {
            return 1;
        }

        self.data
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

                    let chars_count = s[..self.column - 1].chars().count();
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
            + (self.line - 1)
    }
}
