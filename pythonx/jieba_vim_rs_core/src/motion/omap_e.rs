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

use super::{BufferLike, WordMotion};
use crate::token::JiebaPlaceholder;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`) in
    /// operator-pending mode while used with operator `d`. Since Vim's help
    /// states in section "exclusive-linewise" that:
    ///
    /// > When using ":" any motion becomes characterwise exclusive,
    ///
    /// But since `e`/`E` is itself inclusive, and `o_v`
    /// (https://vimhelp.org/motion.txt.html#o_v) can be used to invert
    /// exclusiveness to inclusiveness, we may use prefix the colon command
    /// with it and reuse most code from `nmap e`.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
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
    pub fn omap_e<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        count: usize,
        word: bool,
    ) -> Result<(usize, usize), B::Error> {
        self.nmap_e(buffer, cursor_pos, count, word)
    }
}

#[cfg(test)]
mod tests {
    use super::super::WordMotion;
    use jieba_rs::Jieba;
    use jieba_vim_rs_test::assert_elapsed::AssertElapsed;
    use jieba_vim_rs_test::verified_case::{
        Error, Mode, Motion, VerifiedCaseInput,
    };
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
                    #[serial_test::serial]
                    fn [<$test_name _word_ $index>]() -> Result<(), Error> {
                        let motion = WORD_MOTION.get().unwrap();

                        let output = VerifiedCaseInput::new(
                            "motion_omap_c_e".into(),
                            stringify!([<$test_name _word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "c".into(),
                            Motion::SmallE($count),
                            true,
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.omap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_omap_y_e".into(),
                            stringify!([<$test_name _word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "y".into(),
                            Motion::SmallE($count),
                            true,
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.omap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        Ok(())
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
                    #[serial_test::serial]
                    fn [<$test_name _WORD_ $index>]() -> Result<(), Error> {
                        let motion = WORD_MOTION.get().unwrap();

                        let output = VerifiedCaseInput::new(
                            "motion_omap_c_e".into(),
                            stringify!([<$test_name _WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "c".into(),
                            Motion::LargeE($count),
                            true,
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.omap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_omap_y_e".into(),
                            stringify!([<$test_name _WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "y".into(),
                            Motion::LargeE($count),
                            true,
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.omap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        Ok(())
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
        test_one_word (word):
        (1) ["abc{}d"], 1;
        (2) ["abc{}d"], 2;
        (3) ["a{bc}d"], 1;
        (4) ["a{bc}d"], 2;
    );

    word_motion_tests!(
        test_one_word_space (word):
        (1) ["a{bc}d    "], 1;
        (2) ["a{bcd   } "], 2;
        (3) ["abc{d   } "], 1;
        (4) ["abc{d   } "], 2;
        (5) ["abcd {  } "], 1;
        (6) ["abcd {  } "], 2;
    );

    word_motion_tests!(
        test_two_words (word):
        (1) ["a{bc}d  efg"], 1;
        (2) ["a{bcd  ef}g"], 2;
        (3) ["a{bcd  ef}g"], 3;
        (4) ["abc{d ef}g"], 1;
        (5) ["abc{d ef}g"], 2;
        (6) ["abc{d efg  } "], 3;
    );

    word_motion_tests!(
        test_one_word_newline (word):
        (1) ["a{bc}d", ""], 1;
        (2) ["a{bcd", "}"], 2;
        (3) ["abc{d", "}"], 1;
    );

    word_motion_tests!(
        test_word_newline_newline (word):
        (1) ["abcd", "{   ", "  } "], 1;
        (2) ["abcd", "{   ", "  } "], 2;
    );

    word_motion_tests!(
        test_one_word_space_newline (word):
        (1) ["a{bc}d    ", ""], 1;
        (2) ["abc{d     ", "}"], 1;
        (3) ["abcd{    ", "}"], 1;
        (4) ["abcd {   ", "}"], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space (word):
        (1) ["abc{d", "   } "], 1;
        (2) ["abc{d", "  ", "   } "], 1;
        (3) ["abcd", "{  ", "   } "], 1;
        (4) ["abc{d", "", "   } "], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space_newline (word):
        (1) ["abc{d", " ", "}"], 1;
        (2) ["abc{d", " ", " ", "}"], 1;
        (3) ["abc{d", "", " ", "}"], 1;
        (4) ["abc{d", " ", "", "}"], 1;
        (5) ["abc{d", "", "", "}"], 1
    );

    word_motion_tests!(
        test_word_newline_word (word):
        (1) ["a{bc}d", "", " ", "", "efg"], 1;
        (2) ["abc{d", "", " ", "", "ef}g  "], 1;
        (3) ["abc{d", "  ", "", " ", "efg}h"], 1;
        (4) ["abc{d", "", "ef}g", "", "efgh"], 1;
        (5) ["abc{d", "", "efg", "", "efg}h"], 2;
        (6) ["abc{d", "", "efg", "", "efg}h  "], 2;
    );

    word_motion_tests!(
        test_large_unnecessary_count (word):
        (1) ["{}"], 10293949403;
        (2) ["a{bc def}g"], 10293949403;
        (3) ["abc {def}g"], 10293949403;
    );
}
