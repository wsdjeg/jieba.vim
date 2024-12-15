// Copyright 2024 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

use crate::token::{JiebaPlaceholder, Token};
#[cfg(test)]
use jieba_vim_rs_test::verified_case::cases::MotionOutput as TestMotionOutput;
use std::cmp::Ordering;

mod d_special;
mod nmap_b;
mod nmap_e;
mod nmap_ge;
mod nmap_w;
mod omap_b;
mod omap_c_w;
mod omap_d_e;
mod omap_e;
mod omap_w;
mod token_iter;
mod xmap_b;
mod xmap_e;
mod xmap_ge;
mod xmap_w;

/// Any type that resembles a Vim buffer.
pub trait BufferLike {
    type Error;

    /// Get the line at line number `lnum` (1-indexed).
    fn getline(&self, lnum: usize) -> Result<String, Self::Error>;

    /// Get the total number of lines in the buffer.
    fn lines(&self) -> Result<usize, Self::Error>;
}

/// The motion return type.
#[derive(Debug)]
pub struct MotionOutput {
    /// The new cursor position after the motion.
    pub new_cursor_pos: (usize, usize),
    /// Whether the motion induces d-special. Should be false when not in
    /// operator-pending mode
    pub d_special: bool,
    /// Whether the motion should prevent changes, where the operation is
    /// silently aborted. Should be false when not in operator-pending mode
    pub prevent_change: bool,
}

#[cfg(test)]
impl PartialEq<TestMotionOutput> for MotionOutput {
    fn eq(&self, other: &TestMotionOutput) -> bool {
        self.new_cursor_pos == other.new_cursor_pos
            && self.d_special == other.d_special
            && self.prevent_change == other.prevent_change
    }
}

/// Get the index of the token in `tokens` that covers `col`. Return `None` if
/// `col` is to the right of the last token.
fn index_tokens(tokens: &[Token], col: usize) -> Option<usize> {
    tokens
        .binary_search_by(|tok| {
            if col < tok.col.start_byte_index {
                Ordering::Greater
            } else if col >= tok.col.excl_end_byte_index {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .ok()
}

pub struct WordMotion<C> {
    jieba: C,
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    pub fn new(jieba: C) -> Self {
        Self { jieba }
    }
}

#[cfg(test)]
static WORD_MOTION: once_cell::sync::Lazy<WordMotion<jieba_rs::Jieba>> =
    once_cell::sync::Lazy::new(|| WordMotion::new(jieba_rs::Jieba::new()));

#[cfg(test)]
impl<C> WordMotion<C> {
    fn _noop(&self) {}
}

#[cfg(test)]
#[ctor::ctor]
fn init_word_motion() {
    WORD_MOTION._noop(); // force initialization
}

#[cfg(test)]
impl BufferLike for Vec<&'static str> {
    type Error = ();

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }
}

#[cfg(test)]
impl BufferLike for Vec<String> {
    type Error = ();

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::index_tokens;

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }
}
