use crate::token::{JiebaPlaceholder, Token};
use std::cmp::Ordering;

/// Any type that resembles a Vim buffer.
trait BufferLike {
    type Error;

    /// Get the line at line number `lnum` (1-indexed).
    fn getline(&self, lnum: usize) -> Result<String, Self::Error>;

    /// Get the total number of lines in the buffer.
    fn lines(&self) -> Result<usize, Self::Error>;
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
pub mod test_macros {
    #[macro_export]
    macro_rules! word_motion_tests {
        ( ($motion_fun:ident)  $((
                $test_name:ident $(< $timeout:literal)?:
                $($buffer:expr, ($lnum_before:expr, $col_before:expr), $count:expr, $word:literal,
                ($lnum_after:expr, $col_after:expr));* $(;)?
            )),* $(,)? ) => {
            $(
                #[test]
                $( #[ntest_timeout::timeout($timeout)] )?
                fn $test_name() {
                    let motion = WORD_MOTION.get().unwrap();
                    $(
                        let buffer = $buffer;
                        assert_eq!(motion.$motion_fun(&buffer, ($lnum_before, $col_before), $count, $word), Ok(($lnum_after, $col_after)));
                    )*
                }
            )*
        };
    }

    pub use word_motion_tests;
}
