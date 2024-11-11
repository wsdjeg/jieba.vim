use super::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use super::{BufferLike, WordMotion};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

/// Test if a token is stoppable for `xmap_w`.
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
    /// in visual mode. Take in current `cursor_pos` (lnum, col), and return
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
    /// - If current cursor is on the one character to the right of the last
    ///   character of the last token in the buffer, no further jump should be
    ///   made.
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character to the right of the last character of the last token in the
    ///   buffer.
    pub fn xmap_w<B: BufferLike + ?Sized>(
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
            } else {
                lnum = item.lnum;
                col = item.token.first_char();
                count -= 1;
                if count > 0 && it.peek().is_none() {
                    col = item.token.last_char1();
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
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xc_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualChar,
                            "".into(),
                            Motion::SmallW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xl_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualLine,
                            "".into(),
                            Motion::SmallW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xb_word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualBlock,
                            "".into(),
                            Motion::SmallW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, true);
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
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xc_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualChar,
                            "".into(),
                            Motion::LargeW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xl_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualLine,
                            "".into(),
                            Motion::LargeW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
                        timing.toc();
                        assert_eq!(r, Ok((ac.lnum, ac.col)));

                        let output = VerifiedCaseInput::new(
                            "motion_xmap_w".into(),
                            stringify!([<$test_name _xb_WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::VisualBlock,
                            "".into(),
                            Motion::LargeW($count),
                        )?.verify_case()?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let r = motion.xmap_w(&output.stripped_buffer, (bc.lnum, bc.col), $count, false);
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
        test_space (word):
        (1) ["{ }"], 1;
        (2) ["{     }"], 1;
    );

    word_motion_tests!(
        test_one_word (word):
        (1) ["aaa{a}"], 1;
        (2) ["a{aaa}"], 1;
        (3) ["a{aaa}"], 2;
    );

    word_motion_tests!(
        test_one_word_space (word):
        (1) ["a{aaa    }"], 1;
        (2) ["aaa{a    }"], 1;
        (3) ["aaaa {   }"], 1;
    );

    word_motion_tests!(
        test_two_words (word):
        (1) ["a{aaa  }aaa"], 1;
        (2) ["a{aaa  aaa}"], 2;
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
        (1) ["a{aaa", "    }"], 1;
        (2) ["a{aaa", "  ", "    }"], 1;
        (3) ["aaaa", "{  ", "    }"], 1;
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
}
