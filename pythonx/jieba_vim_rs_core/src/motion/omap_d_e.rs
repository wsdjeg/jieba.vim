use super::{index_tokens, BufferLike, WordMotion};
use crate::token::{self, JiebaPlaceholder, TokenLike, TokenType};

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
    /// with it and reuse most code from `nmap e`. Note also the special case
    /// `d-special` (https://vimhelp.org/change.txt.html#d-special). Therefore,
    /// we need to apply `o_v` in a case-by-case manner.
    ///
    /// Take in current `cursor_pos` (lnum, col), and return the new cursor
    /// position. Also return a bool indicating if `d-special` takes effect.
    /// Note that `lnum` is 1-indexed, and `col` is 0-indexed. We denote both
    /// `word` and `WORD` with the English word "word" below.
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
    pub fn omap_d_e<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        count: usize,
        word: bool,
    ) -> Result<((usize, usize), bool), B::Error> {
        let new_cursor_pos = self.nmap_e(buffer, cursor_pos, count, word)?;
        let (lnum, col) = cursor_pos;
        let (new_lnum, new_col) = new_cursor_pos;

        if lnum == new_lnum {
            return Ok((new_cursor_pos, false));
        }

        let tokens_cursor_line =
            token::parse_str(buffer.getline(lnum)?, &self.jieba, word);
        if !tokens_cursor_line.is_empty() {
            let i = index_tokens(&tokens_cursor_line, col).unwrap();
            if tokens_cursor_line[..i].iter().any(|tok| match tok.ty {
                TokenType::Space => false,
                TokenType::Word => true,
            }) {
                return Ok((new_cursor_pos, false));
            }
            let cursor_token = &tokens_cursor_line[i];
            if let TokenType::Word = cursor_token.ty {
                if col > cursor_token.first_char() {
                    return Ok((new_cursor_pos, false));
                }
            }
        }

        let tokens_new_cursor_line =
            token::parse_str(buffer.getline(new_lnum)?, &self.jieba, word);
        if !tokens_new_cursor_line.is_empty() {
            let j = index_tokens(&tokens_new_cursor_line, new_col).unwrap();
            if tokens_new_cursor_line[j + 1..]
                .iter()
                .any(|tok| match tok.ty {
                    TokenType::Space => false,
                    TokenType::Word => true,
                })
            {
                return Ok((new_cursor_pos, false));
            }
            let new_cursor_token = &tokens_new_cursor_line[j];
            if let TokenType::Word = new_cursor_token.ty {
                if new_col < new_cursor_token.last_char() {
                    return Ok((new_cursor_pos, false));
                }
            }
        }

        Ok((new_cursor_pos, true))
    }
}

#[cfg(test)]
mod tests {
    use super::super::WordMotion;
    use jieba_rs::Jieba;
    use jieba_vim_rs_test::assert_elapsed::AssertElapsed;
    use jieba_vim_rs_test::cursor_marker::CursorMarker;
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

                        // Check if d-special is active.
                        let output = CursorMarker
                            .strip_markers(vec![$($buffer_item.into()),*])
                            .map_err(|err| Error::InvalidCursorMarker {
                            inner: err,
                            group_id: "motion_omap_d_e".into(),
                            test_name: stringify!([<$test_name _word_ $index>]).into(),
                        })?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let (r, is_d_special) = motion.omap_d_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, true).unwrap();
                        timing.toc();

                        let _output = VerifiedCaseInput::new(
                            "motion_omap_d_e".into(),
                            stringify!([<$test_name _word_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "d".into(),
                            Motion::SmallE($count),
                            true,
                            is_d_special,
                        )?.verify_case()?;
                        assert_eq!(r, (ac.lnum, ac.col));
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

                        // Check if d-special is active.
                        let output = CursorMarker
                            .strip_markers(vec![$($buffer_item.into()),*])
                            .map_err(|err| Error::InvalidCursorMarker {
                            inner: err,
                            group_id: "motion_omap_d_e".into(),
                            test_name: stringify!([<$test_name _WORD_ $index>]).into(),
                        })?;
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        let timing = AssertElapsed::tic(50);
                        let (r, is_d_special) = motion.omap_d_e(&output.stripped_buffer, (bc.lnum, bc.col), $count, false).unwrap();
                        timing.toc();

                        let _output = VerifiedCaseInput::new(
                            "motion_omap_d_e".into(),
                            stringify!([<$test_name _WORD_ $index>]).into(),
                            vec![$($buffer_item.into()),*],
                            Mode::Operator,
                            "d".into(),
                            Motion::LargeE($count),
                            true,
                            is_d_special,
                        )?.verify_case()?;
                        assert_eq!(r, (ac.lnum, ac.col));
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
