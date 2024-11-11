use super::{BufferLike, WordMotion};
use crate::motion::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

/// Test if a token is stoppable for `xmap_e`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => false,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `e` (if `word` is `true`) or `E` (if `word` is `false`)
    /// in visual mode. Take in current `cursor_pos` (lnum, col), and return
    /// the new cursor position. Note that `lnum` is 1-indexed, and `col`
    /// is 0-indexed. We denote both `word` and `WORD` with the English word
    /// "word" below.
    ///
    /// # Basics
    ///
    /// `e`/`E` jumps to the last character of current word, if cursor is not
    /// already on the last character, or the last character of the next word.
    /// Empty line is *not* considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the one character to the right of the last
    ///   character of the last token in the buffer, no further jump should
    ///   be made.
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer.
    pub fn xmap_e<B: BufferLike + ?Sized>(
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
                if it.peek().is_some() {
                    col = item.token.last_char();
                } else {
                    col = item.token.last_char1();
                }
            } else if !(item.cursor && col == item.token.last_char()) {
                lnum = item.lnum;
                col = item.token.last_char();
                count -= 1;
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char1();
                }
            } else if it.peek().is_none() {
                col = item.token.last_char1();
                count -= 1;
            }
        }
        Ok((lnum, col))
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
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xc_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualChar,
                            "".into(),
                            Motion::SmallE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xl_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualLine,
                            "".into(),
                            Motion::SmallE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xb_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualBlock,
                            "".into(),
                            Motion::SmallE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
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
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xc_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualChar,
                            "".into(),
                            Motion::LargeE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xl_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualLine,
                            "".into(),
                            Motion::LargeE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_e".into(),
                            stringify!([<$test_name _xb_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualBlock,
                            "".into(),
                            Motion::LargeE($count),
                            false,
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
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
        (1) ["aaa{a}"], 1;
        (2) ["a{aa}a"], 1;
        (3) ["a{aaa}"], 2;
    );

    word_motion_tests!(
        test_one_word_space (word):
        (1) ["a{aa}a    "], 1;
        (2) ["aaa{a    }"], 1;
        (3) ["aaaa {   }"], 1;
    );

    word_motion_tests!(
        test_two_words (word):
        (1) ["a{aa}a  aaa"], 1;
        (2) ["a{aaa  aa}a"], 2;
        (3) ["a{aaa  aaa}"], 3;
        (4) ["aaa{a aa}a"], 1;
        (5) ["aaa{a aaa}"], 2;
    );

    word_motion_tests!(
        test_one_word_newline (word):
        (1) ["a{aa}a", ""], 1;
        (2) ["a{aaa", "}"], 2;
        (3) ["aaa{a", "}"], 1;
    );

    word_motion_tests!(
        test_one_word_space_newline (word):
        (1) ["a{aa}a    ", ""], 1;
        (2) ["aaa{a     ", "}"], 1;
        (3) ["aaaa{    ", "}"], 1;
        (4) ["aaaa {   ", "}"], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space (word):
        (1) ["aaa{a", "    }"], 1;
        (2) ["aaa{a", "  ", "    }"], 1;
        (3) ["aaaa", "{  ", "    }"], 1;
        (4) ["aaa{a", "", "    }"], 1;
    );

    word_motion_tests!(
        test_one_word_newline_space_newline (word):
        (1) ["aaa{a", " ", "}"], 1;
        (2) ["aaa{a", " ", "  }"], 1;
        (3) ["aaa{a", " ", " ", "}"], 1;
        (4) ["aaa{a", " ", " ", "  }"], 1;
        (5) ["aaa{a", "", " ", "}"], 1;
        (6) ["aaa{a", "", " ", "  }"], 1;
        (7) ["aaa{a", " ", "", "}"], 1;
        (8) ["aaa{a", " ", "", "  }"], 1;
        (9) ["aaa{a", "", "", "}"], 1;
        (10) ["aaa{a", "", "", "  }"], 1;
    );

    word_motion_tests!(
        test_word_newline_word (word):
        (1) ["a{aa}a", "", " ", "", "aaa"], 1;
        (2) ["a{aaa", "", " ", "", "aa}a"], 2;
        (3) ["a{aaa", "", " ", "", "aaa   }"], 3;
        (4) ["aaa{a", "", " ", "", "aa}a"], 1;
        (5) ["aaa{a", "  ", "", " ", "aaa}a"], 1;
        (6) ["aaa{a", "", "aa}a", "", "aaaa"], 1;
        (7) ["aaa{a", "", "aaa", "", "aaa}a"], 2;
    );

    word_motion_tests!(
        test_large_unnecessary_count (word):
        (1) ["{}"], 10293949403;
        (2) ["a{aa aaaa}"], 10293949403;
        (3) ["aaa {aaaa}"], 10293949403;
    );
}
