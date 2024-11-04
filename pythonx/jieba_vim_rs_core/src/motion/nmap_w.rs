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

use super::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use super::{BufferLike, WordMotion};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

/// Test if a token is stoppable for `nmap_w`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    if item.cursor {
        false
    } else {
        match item.token {
            None => true,
            Some(token) => match token.ty {
                TokenType::Word => true,
                TokenType::Space => false,
            },
        }
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in normal mode. Take in current `cursor_pos` (lnum, col), and return
    /// the new cursor position. Note that `lnum` is 1-indexed, and `col`
    /// is 0-indexed. We denote both `word` and `WORD` with the English word
    /// "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the last character of the last token in the
    ///   buffer, no further jump should be made.
    /// - If there is no next word to the right of current cursor, jump to the
    ///   last character of the last token in the buffer.
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn nmap_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        mut count: usize,
        word: bool,
    ) -> Result<(usize, usize), B::Error> {
        let (mut lnum, mut col) = cursor_pos;
        let mut it =
            ForwardTokenIterator::new(buffer, &self.jieba, lnum, col, word)?
                .peekable();
        while count > 0 && it.peek().is_some() {
            let item = it.next().unwrap()?;
            if !is_stoppable(&item) {
                lnum = item.lnum;
                col = item.token.last_char();
            } else {
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char();
                }
            }
        }
        Ok((lnum, col))
    }
}

#[cfg(test)]
mod tests {
    use super::super::WordMotion;
    use jieba_rs::Jieba;
    use jieba_vim_rs_test::cursor_marker::CursorMarker;
    #[cfg(feature = "verifiable_case")]
    use jieba_vim_rs_test_verifiable_case::verified_case;
    #[cfg(not(feature = "verifiable_case"))]
    use jieba_vim_rs_test_verifiable_case::verified_case_dry_run as verified_case;
    use once_cell::sync::OnceCell;

    static WORD_MOTION: OnceCell<WordMotion<Jieba>> = OnceCell::new();

    #[ctor::ctor]
    fn init() {
        WORD_MOTION.get_or_init(|| WordMotion::new(Jieba::new()));
    }

    macro_rules! word_motion_tests {
        (
            $test_name:ident (word):
            $(
                ($index:literal) [$($buffer_item:literal),*], $count:literal
            );* $(;)?
        ) => {
            $(
                paste::paste! {
                    #[test]
                    #[ntest_timeout::timeout(50)]
                    fn [<$test_name _word_ $index>]() {
                        let motion = WORD_MOTION.get().unwrap();
                        let cm = CursorMarker;
                        let buffer = verified_case!(
                            motion_nmap_w,
                            [<$test_name _word_ $index>],
                            [$($buffer_item),*],
                            "n", "", $count, "w");
                        let buffer: Vec<String> = buffer.iter().map(|s| s.to_string()).collect();
                        let output = cm.strip_markers(buffer).unwrap();
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        assert_eq!(
                            motion.nmap_w(&output.striped_lines, (bc.lnum, bc.col), $count, true),
                            Ok((ac.lnum, ac.col))
                        );
                    }
                }
            )*
        };
        (
            $test_name:ident (WORD):
            $(
                ($index:literal) [$($buffer_item:literal),*], $count:literal
            );* $(;)?
        ) => {
            $(
                paste::paste! {
                    #[test]
                    #[ntest_timeout::timeout(50)]
                    fn [<$test_name _WORD_ $index>]() {
                        let motion = WORD_MOTION.get().unwrap();
                        let cm = CursorMarker;
                        let buffer = verified_case!(
                            motion_nmap_w,
                            [<$test_name _WORD_ $index>],
                            [$($buffer_item),*],
                            "n", "", $count, "W");
                        let buffer: Vec<String> = buffer.iter().map(|s| s.to_string()).collect();
                        let output = cm.strip_markers(buffer).unwrap();
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        assert_eq!(
                            motion.nmap_w(&output.striped_lines, (bc.lnum, bc.col), $count, false),
                            Ok((ac.lnum, ac.col))
                        );
                    }
                }
            )*
        };
    }

    word_motion_tests!(
        test_empty (word):
        (1) ["{}"], 1;
    );

    word_motion_tests!(
        test_space (word):
        (1) ["{} "], 1;
        (2) ["{    } "], 1;
    );

    word_motion_tests!(
        test_one_word (word):
        (1) ["aaa{}a"], 1;
        (2) ["a{aa}a"], 1;
        (3) ["a{aa}a"], 2;
    );

    word_motion_tests!(
        test_one_word_space (word):
        (1) ["a{aaa   } "], 1;
        (2) ["aaa{a   } "], 1;
        (3) ["aaaa {  } "], 1;
    );

    word_motion_tests!(
        test_two_words (word):
        (1) ["a{aaa  }aaa"], 1;
        (2) ["a{aaa  aa}a"], 2;
    );

    word_motion_tests!(
        test_one_word_newline (word):
        (1) ["a{aaa", "}"], 1;
    );

    word_motion_tests!(
        test_one_word_space_newline (word):
        (1) ["a{aaa    ", "}"], 1;
        (2) ["aaaa{    ", "}"], 1;
        (3) ["aaaa {   ", "}"], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space (word):
        (1) ["a{aaa", "   } "], 1;
        (2) ["a{aaa", "  ", "   } "], 1;
        (3) ["aaaa", "{  ", "   } "], 1;
        (4) ["a{aa", "}", "   "], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space_newline (word):
        (1) ["a{aaa", " ", "}"], 1;
        (2) ["a{aaa", " ", " ", "}", "  "], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space_word (word):
        (1) ["a{aaa", " ", " ", "}aaa"], 1;
        (2) ["a{aaa", " ", " ", "   }aaa"], 1;
    );

    word_motion_tests!(
        test_large_unnecessary_count (word):
        (1) ["{}"], 10293949403;
        (2) ["a{aa aaa}a"], 10293949403;
        (3) ["aaa {aaa}a"], 10293949403;
    );
}
