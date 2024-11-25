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
        count: u64,
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
    #[cfg(feature = "verifiable_case")]
    use jieba_vim_rs_test_macro::verified_cases;
    #[cfg(not(feature = "verifiable_case"))]
    use jieba_vim_rs_test_macro::verified_cases_dry_run as verified_cases;

    #[verified_cases(
        mode = "o",
        operator = "d",
        motion = "e",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"])]
    #[vcase(name = "one_word", buffer = ["abc{}d"], count = 2)]
    #[vcase(name = "one_word", buffer = ["a{bc}d"])]
    #[vcase(name = "one_word", buffer = ["a{bc}d"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{bc}d    "])]
    #[vcase(name = "one_word_space", buffer = ["a{bcd   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "])]
    #[vcase(name = "one_word_space", buffer = ["abc{d   } "], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "])]
    #[vcase(name = "one_word_space", buffer = ["abcd {  } "], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    ab}c"])]
    #[vcase(name = "space_word", buffer = [" {   ab}c"])]
    #[vcase(name = "space_word", buffer = ["{    ab}c  def"])]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 2)]
    #[vcase(name = "space_word", buffer = ["{    abc  de}f"], count = 3)]
    #[vcase(name = "two_words", buffer = ["a{bc}d  efg"])]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["a{bcd  ef}g"], count = 3)]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"])]
    #[vcase(name = "two_words", buffer = ["abc{d ef}g"], count = 2)]
    #[vcase(name = "two_words", buffer = ["abc{d efg  } "], count = 3)]
    #[vcase(name = "one_word_newline", buffer = ["a{bc}d", ""])]
    #[vcase(name = "one_word_newline", buffer = ["a{bcd", "}"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["abc{d", "}"])]
    #[vcase(name = "newline_one_word", buffer = ["{", "abc}d"], d_special)]
    #[vcase(name = "newline_one_word", buffer = ["{", "", "abc}d"], d_special)]
    #[vcase(name = "newline_one_word", buffer = ["{", "  ", "abc}d"], d_special)]
    #[vcase(name = "newline_two_words", buffer = ["{", "", "abc}d", "efg"], d_special)]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "], d_special)]
    #[vcase(name = "word_newline_newline", buffer = ["abcd", "{   ", "  } "], count = 2, d_special)]
    #[vcase(name = "one_word_space_newline", buffer = ["a{bc}d    ", ""])]
    #[vcase(name = "one_word_space_newline", buffer = ["abc{d     ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["abcd {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "  ", "   } "])]
    #[vcase(name = "one_word_newline_space", buffer = ["abc{d", "", "   } "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", " ", "", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["abc{d", "", "", "}"])]
    #[vcase(name = "word_newline_word", buffer = ["a{bc}d", "", " ", "", "efg"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", " ", "", "ef}g  "])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "  ", "", " ", "efg}h"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "ef}g", "", "efgh"])]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h"], count = 2)]
    #[vcase(name = "word_newline_word", buffer = ["abc{d", "", "efg", "", "efg}h  "], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{bc def}g"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["abc {def}g"], count = 10293949403)]
    mod motion_omap_d_e {}
}
