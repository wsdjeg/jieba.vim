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
    use super::super::test_macros::{
        setup_word_motion_tests, word_motion_tests,
    };

    setup_word_motion_tests!();

    word_motion_tests! { (nmap_w)
        (
            test_empty:
            ["{}"], 1, true;
            ["{}"], 1, false;
        ),
        (
            test_one_word:
            ["aaa{}a"], 1, true;
            ["aaa{}a"], 1, false;
            ["a{aa}a"], 1, true;
            ["a{aa}a"], 1, false;
            ["a{aa}a"], 2, true;
            ["a{aa}a"], 2, false;
            ["{你}好"], 1, true;
            ["{你}好"], 1, false;
            ["{你}好"], 1, true;
            ["{你}好"], 1, false;
        ),
        (
            test_one_word_space:
            ["a{aaa   } "], 1, true;
            ["a{aaa   } "], 1, false;
            ["aaa{a   } "], 1, true;
            ["aaa{a   } "], 1, false;
            ["aaaa {  } "], 1, true;
            ["aaaa {  } "], 1, false;
            ["{你好  } "], 1, true;
            ["{你好  } "], 1, false;
            ["你好 { } "], 1, true;
            ["你好 { } "], 1, false;
        ),
        (
            test_two_words:
            ["a{aaa  }aaa"], 1, true;
            ["a{aaa  }aaa"], 1, false;
            ["a{aaa  aa}a"], 2, true;
            ["a{aaa  aa}a"], 2, false;
            ["{你好}世界"], 1, true;
            ["{你好}世界"], 1, false;
            ["{你好世}界"], 2, true;
            ["{你好世}界"], 2, false;
            ["{你好  }世界"], 1, true;
            ["{你好  }世界"], 1, false;
            ["{你好  世}界"], 2, true;
            ["{你好  世}界"], 2, false;
        ),
        (
            test_one_word_new_line:
            ["a{aaa", "}"], 1, true;
            ["a{aaa", "}"], 1, false;
            ["{你好", "}"], 1, true;
            ["你{好", "}"], 1, true;
            ["{你好", "}"], 1, false;
            ["你{好", "}"], 1, false;
        ),
        (
            test_one_word_space_new_line:
            ["a{aaa    ", "}"], 1, true;
            ["a{aaa    ", "}"], 1, false;
            ["aaaa{    ", "}"], 1, true;
            ["aaaa {   ", "}"], 1, true;
            ["{你好    ", "}"], 1, true;
            ["{你好    ", "}"], 1, false;
        ),
        (
            test_one_word_new_line_space:
            ["a{aaa", "   } "], 1, true;
            ["a{aaa", "   } "], 1, false;
            ["a{aaa", "  ", "   } "], 1, true;
            ["a{aaa", "  ", "   } "], 1, false;
            ["aaaa", "{  ", "   } "], 1, true;
            ["a{aa", "}", "   "], 1, true;
            ["a{aaa", "}", "   "], 1, false;
            ["{你好", "  ", "   } "], 1, true;
            ["你{好", "  ", "   } "], 1, true;
        ),
        (
            test_one_word_new_line_space_new_line:
            ["a{aaa", " ", "}"], 1, true;
            ["a{aaa", " ", "}"], 1, false;
            ["a{aaa", " ", " ", "}", "  "], 1, true;
            ["a{aaa", " ", " ", "}", "  "], 1, false;
            ["{你好", " ", " ", "}", "  "], 1, true;
            ["{你好", " ", " ", "}", "  "], 1, false;
        ),
        (
            test_large_unnecessary_count < 100:
            ["{}"], 10293949403, true;
            ["{}"], 10293949403, false;
            ["a{aa aaa}a"], 10293949403, true;
            ["a{aa aaa}a"], 10293949403, false;
            ["aaa {aaa}a"], 10293949403, true;
            ["aaa {aaa}a"], 10293949403, false;
        ),
        (
            test_jieba_efficiency1 < 50:
            ["{你好}，世界"], 1, true;
        ),
        (
            test_jieba_efficiency2 < 50:
            ["{你好，}世界"], 1, false;
        ),
    }
}
