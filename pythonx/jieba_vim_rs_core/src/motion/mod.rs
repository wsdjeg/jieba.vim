use crate::token::{JiebaPlaceholder, Token};
use std::cmp::Ordering;

mod nmap_e;
mod nmap_w;
mod token_iter;

/// Any type that resembles a Vim buffer.
pub trait BufferLike {
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
    macro_rules! setup_word_motion_tests {
        () => {
            static WORD_MOTION: once_cell::sync::OnceCell<
                crate::motion::WordMotion<jieba_rs::Jieba>,
            > = once_cell::sync::OnceCell::new();

            #[ctor::ctor]
            fn init() {
                WORD_MOTION.get_or_init(|| {
                    crate::motion::WordMotion::new(jieba_rs::Jieba::new())
                });
            }
        };
    }

    #[macro_export]
    macro_rules! word_motion_tests {
        ( ($motion_fun:ident)  $((
                $test_name:ident $(< $timeout:literal)?:
                $( [$($buffer_item:literal),*], $count:expr, $word:literal );* $(;)?
            )),* $(,)? ) => {
            $(
                #[test]
                $( #[ntest_timeout::timeout($timeout)] )?
                fn $test_name() {
                    let motion = WORD_MOTION.get().unwrap();
                    let cm = crate::motion::test_utils::CursorMarker::default();
                    $(
                        let mut buffer: Vec<String> = vec![$($buffer_item.into()),*];
                        let (before_cursor, after_cursor) = cm.strip_markers(&mut buffer);
                        assert_eq!(motion.$motion_fun(&buffer, before_cursor, $count, $word), Ok(after_cursor));
                    )*
                }
            )*
        };
    }

    pub use {setup_word_motion_tests, word_motion_tests};
}

#[cfg(test)]
impl BufferLike for Vec<&'static str> {
    type Error = ();

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }
}

#[cfg(test)]
impl BufferLike for Vec<String> {
    type Error = ();

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::index_tokens;

    #[test]
    fn test_index_tokens() {
        assert_eq!(index_tokens(&[], 0), None);
    }
}
