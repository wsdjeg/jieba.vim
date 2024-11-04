use super::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use super::{BufferLike, WordMotion};
use crate::token::{JiebaPlaceholder, TokenLike, TokenType};

/// Test if a token is stoppable for `omap_c_w`.
fn is_stoppable(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => true,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

/// Test if a token is stoppable for `omap_c_w` when cursor starts at a word.
fn is_stoppable_ce_mode(item: &TokenIteratorItem) -> bool {
    match item.token {
        None => false,
        Some(token) => match token.ty {
            TokenType::Word => true,
            TokenType::Space => false,
        },
    }
}

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `w` (if `word` is `true`) or `W` (if `word` is `false`)
    /// in operator-pending mode while used with operator `c`. Since Vim's help
    /// states in section "exclusive-linewise" that:
    ///
    /// > When using ":" any motion becomes characterwise exclusive.
    ///
    /// with plain onoremap we won't be able to operate on the last character
    /// in a line. Therefore, we assume that `+virtualedit` feature is enabled
    /// and `set virtualedit=onemore` temporarily to circumvent this issue.
    /// See also about this trick at https://vimhelp.org/intro.txt.html#%7Bmotion%7D
    /// and https://github.com/svermeulen/vim-NotableFt/blob/master/plugin/NotableFt.vim.
    ///
    /// Take in current `cursor_pos` (lnum, col), and return the new cursor
    /// position. Note that `lnum` is 1-indexed, and `col` is 0-indexed. We
    /// denote both `word` and `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `w`/`W` jumps to the first character of next word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If there is no next word to the right of current cursor, jump to one
    ///   character after the last token in the buffer (`virtualedit`).
    /// - Quoted from Vim's help section "WORD": "cw" and "cW" are treated like
    ///   "ce" and "cE" if the cursor is on a non-blank. This is because "cw"
    ///   is interpreted as change-word, and a word does not include the
    ///   following white space (see also cw).
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn omap_c_w<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        mut count: usize,
        word: bool,
    ) -> Result<(usize, usize), B::Error> {
        // ["{abcd}  "], 1;
        let (mut lnum, mut col) = cursor_pos;
        let mut it =
            ForwardTokenIterator::new(buffer, &self.jieba, lnum, col, word)?
                .peekable();
        let mut cursor_starts_at_word: Option<bool> = None;
        while count > 0 && it.peek().is_some() {
            let item = it.next().unwrap()?;
            let ce_mode = *cursor_starts_at_word
                .get_or_insert(item.cursor && is_stoppable_ce_mode(&item));
            if ce_mode {
                if !is_stoppable_ce_mode(&item) {
                    lnum = item.lnum;
                    if it.peek().is_none() {
                        col = item.token.last_char1();
                    } else {
                        col = item.token.last_char()
                    }
                } else {
                    lnum = item.lnum;
                    col = item.token.last_char1();
                    count -= 1;
                }
            } else {
                if !is_stoppable(&item) {
                    lnum = item.lnum;
                    if it.peek().is_none() || (count == 1 && item.eol) {
                        col = item.token.last_char1();
                        count -= 1;
                    } else {
                        col = item.token.last_char();
                    }
                } else {
                    if !item.cursor {
                        lnum = item.lnum;
                        col = item.token.first_char();
                        count -= 1;
                    }
                    if count > 0 && it.peek().is_none() {
                        col = item.token.last_char1();
                    } else if count == 1 && item.eol && it.peek().is_some() {
                        let next_item = it.next().unwrap()?;
                        lnum = next_item.lnum;
                        col = next_item.token.first_char();
                        count -= 1;
                    }
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
                            motion_omap_c_w,
                            [<$test_name _word_ $index>],
                            [$($buffer_item),*],
                            "o", "c", $count, "w");
                        let buffer: Vec<String> = buffer.iter().map(|s| s.to_string()).collect();
                        let output = cm.strip_markers(buffer).unwrap();
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        assert_eq!(
                            motion.omap_c_w(&output.striped_lines, (bc.lnum, bc.col), $count, true),
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
                            motion_omap_c_w,
                            [<$test_name _WORD_ $index>],
                            [$($buffer_item),*],
                            "o", "c", $count, "W");
                        let buffer: Vec<String> = buffer.iter().map(|s| s.to_string()).collect();
                        let output = cm.strip_markers(buffer).unwrap();
                        let bc = output.before_cursor_position;
                        let ac = output.after_cursor_position;
                        assert_eq!(
                            motion.omap_c_w(&output.striped_lines, (bc.lnum, bc.col), $count, true),
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
        test_empty_empty (word):
        (1) ["{", "}"], 1;
    );

    word_motion_tests!(
        test_space_newline (word):
        (1) ["   { }", ""], 1;
        (2) ["   { }", "  "], 1;
        (3) ["{   }", ""], 1;
        (4) ["{   }", "  "], 1;
    );

    word_motion_tests!(
        test_empty_space_empty (word):
        (1) ["{", "}       ", ""], 1;
        (2) ["{", "}       abcd", ""], 1;
        (3) ["{", "}abcd", ""], 1;
        (4) ["{", "   abcd", "}       ", "  ef"], 2;
        (5) ["{", "   abcd", "}         efg", "  hi"], 2;
    );

    word_motion_tests!(
        test_empty_word (word):
        (1) ["{", "}abc  def"], 1;
        (2) ["{", "abc  }def"], 2;
    );

    word_motion_tests!(
        test_one_word (word):
        (1) ["{abcd}"], 1;
        (2) ["a{bcd}"], 1;
        (3) ["abc{d}"], 1;
    );

    word_motion_tests!(
        test_one_word_space (word):
        (1) ["{abcd}  "], 1;
        (2) ["ab{cd}  "], 1;
        (3) ["abc{d}  "], 1;
        (4) ["abcd{  }"], 1;
        (5) ["ab{cd  }"], 2;
    );

    word_motion_tests!(
        test_space_word (word):
        (1) ["{    }abc"], 1;
        (2) [" {   }abc"], 1;
        (3) ["{    abc  }def"], 2;
        (4) ["{    abc  def}"], 3;
    );

    word_motion_tests!(
        test_two_words (word):
        (1) ["{abcd}   efg"], 1;
        (2) ["ab{cd}    efg"], 1;
        (3) ["abc{d}    efg"], 1;
        (4) ["abcd{    }efg"], 1;
        (5) ["abcd {   }efg"], 1;
        (6) ["abcd   { }efg"], 1;
    );

    word_motion_tests!(
        test_three_words (word):
        (1) ["{abcd}   efgh   ijkl"], 1;
        (2) ["abc{d}   efgh   ijkl"], 1;
        (3) ["abcd{   }efgh   ijkl"], 1;
        (4) ["{abcd   efgh}   ijkl"], 2;
        (5) ["abc{d   efgh}   ijkl"], 2;
        (6) ["abcd{   efgh   }ijkl"], 2;
        (7) ["{abcd   efgh    ijkl}"], 3;
        (8) ["abcd{   efgh    ijkl}"], 3;
    );

    word_motion_tests!(
        test_three_words_space (word):
        (1) ["{abcd   efgh    ijkl}   "], 3;
        (2) ["abcd{   efgh    ijkl   }"], 3;
    );

    word_motion_tests!(
        test_word_newline (word):
        (1) ["abcd   {efgh}", ""], 1;
        (2) ["abcd   e{fgh}", ""], 1;
        (3) ["abcd   {efgh}", "  "], 1;
        (4) ["abcd   efg{h}", "  "], 1;
        (5) ["abcd   {efgh}", "  ijkl"], 1;
        (6) ["abcd   efg{h}", "  ijkl"], 1;
        (7) ["abcd   {efgh}", "ijkl  "], 1;
        (8) ["abcd   efg{h}", "ijkl  "], 1;
        (9) ["abcd   {efgh", "   ijkl}"], 2;
        (10) ["abcd   {efgh", "ijkl}   "], 2;
        (11) ["abcd   {efgh", "   ijkl}   "], 2;
    );

    word_motion_tests!(
        test_word_space_newline (word):
        (1) ["abcd  {   }", ""], 1;
        (2) ["abcd  {   ", "}"], 2;
        (3) ["abcd  {   ", "}"], 10;
    );

    word_motion_tests!(
        test_space_newline_space (word):
        (1) ["    {  }", "       "], 1;
        (2) ["    {  ", "       }"], 2;
        (3) ["  {    ", "   ", "    }"], 2;
        (4) ["  {    ", "   ", "", "}    "], 2;
        (5) ["  {    ", "   ", "", "    }"], 3;
    );

    word_motion_tests!(
        test_word_space_newline_space (word):
        (1) ["a{bcd}     ", "    "], 1;
        (2) ["a{bcd     ", "     }"], 2;
        (3) ["a{bcd     ", "      ", "  }"], 2;
    );

    word_motion_tests!(
        test_word_newline_counts (word):
        (1) ["ab{cd  efg", " ", "  hij}", ""], 3;
        (2) ["ab{cd  efg", "", "  hij}"], 3;
        (3) ["ab{cd  efg", "", "  hij}", ""], 3;
        (4) ["ab{cd  efg}", ""], 2;
        (5) ["ab{cd  efg}", " ", "  ", "  ", "  hij"], 2;
        (6) ["ab{cd  efg", " ", "  ", "  ", "  hij}", "  ", ""], 3;
        (7) ["ab{cd  efg", "", " ", "  hij}"], 3;
        (8) ["ab{cd  efg", "", " ", "  hij}   "], 3;
        (9) ["ab{cd  efg", " ", "  hij}   ", ""], 3;
        (10) ["ab{cd  efg", " ", "  ", "  ", "  hij}  ", "  ", ""], 3;
    );
}
