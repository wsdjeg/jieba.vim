use super::token_iter::{ForwardTokenIterator, TokenIteratorItem};
use super::{BufferLike, MotionOutput, WordMotion};
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
        mut count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
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
        Ok(MotionOutput {
            new_cursor_pos: (lnum, col),
            d_special: false,
            prevent_change: false,
        })
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "verifiable_case")]
    use jieba_vim_rs_test_macro::verified_cases;
    #[cfg(not(feature = "verifiable_case"))]
    use jieba_vim_rs_test_macro::verified_cases_dry_run as verified_cases;

    #[verified_cases(
        mode = "xc",
        motion = "w",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "space", buffer = ["{ }"])]
    #[vcase(name = "space", buffer = ["{     }"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aaa    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaaa}"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["   { }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaaa}"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  }aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaaa   }"])]
    #[vcase(name = "space_one_word_space", buffer = [" {   }aaaa   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aaa    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aa", "}", "   "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", " ", "}", "  "])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "}aaa"])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "   }aaa"])]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xcmap_w {}

    // Copied from xcmap_w above.
    #[verified_cases(
        mode = "xl",
        motion = "w",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "space", buffer = ["{ }"])]
    #[vcase(name = "space", buffer = ["{     }"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aaa    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaaa}"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["   { }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaaa}"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  }aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaaa   }"])]
    #[vcase(name = "space_one_word_space", buffer = [" {   }aaaa   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aaa    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aa", "}", "   "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", " ", "}", "  "])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "}aaa"])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "   }aaa"])]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xlmap_w {}

    // Copied from xcmap_w above.
    #[verified_cases(
        mode = "xb",
        motion = "w",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["{}"])]
    #[vcase(name = "space", buffer = ["{ }"])]
    #[vcase(name = "space", buffer = ["{     }"])]
    #[vcase(name = "one_word", buffer = ["aaa{a}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"])]
    #[vcase(name = "one_word", buffer = ["a{aaa}"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["a{aaa    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaa{a    }"])]
    #[vcase(name = "one_word_space", buffer = ["aaaa {   }"])]
    #[vcase(name = "space_one_word", buffer = ["{    }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["{    aaaa}"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["   { }aaaa"])]
    #[vcase(name = "space_one_word", buffer = ["    {aaaa}"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  }aaa"])]
    #[vcase(name = "two_words", buffer = ["a{aaa  aaa}"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["    {aaaa   }"])]
    #[vcase(name = "space_one_word_space", buffer = [" {   }aaaa   "])]
    #[vcase(name = "space_one_word_space", buffer = [" {   aaaa   }"], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["a{aaa", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["a{aaa    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa{    ", "}"])]
    #[vcase(name = "one_word_space_newline", buffer = ["aaaa {   ", "}"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aaa", "  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["aaaa", "{  ", "    }"])]
    #[vcase(name = "one_word_newline_space", buffer = ["a{aa", "}", "   "])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", "}"])]
    #[vcase(name = "one_word_newline_space_newline", buffer = ["a{aaa", " ", " ", "}", "  "])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "}aaa"])]
    #[vcase(name = "one_word_newline_space_word", buffer = ["a{aaa", " ", " ", "   }aaa"])]
    #[vcase(name = "large_unnecessary_count", buffer = ["{}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["a{aa aaaa}"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["aaa {aaaa}"], count = 10293949403)]
    mod motion_xbmap_w {}
}
