use crate::token::{JiebaPlaceholder, Token};
use std::cmp::Ordering;

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

    pub use word_motion_tests;
}

#[cfg(test)]
pub mod test_utils {
    use super::BufferLike;

    /// Markers of cursor in a string.
    pub struct CursorMarker {
        /// The cursor before the motion.
        before: u8,
        /// The cursor after the motion.
        after: u8,
    }

    impl Default for CursorMarker {
        fn default() -> Self {
            Self {
                before: b'{',
                after: b'}',
            }
        }
    }

    impl CursorMarker {
        fn marker_predicate(&self, c: char) -> bool {
            c == self.before.into() || c == self.after.into()
        }

        fn strip_marker_str(
            &self,
            s: &mut String,
        ) -> (Option<usize>, Option<usize>) {
            let mut before_cursor_col = None;
            let mut after_cursor_col = None;
            for _ in 0..2 {
                if let Some(i) = s.find(|c| self.marker_predicate(c)) {
                    let c = s.drain(i..i + 1).next().unwrap();
                    if c == self.before.into() {
                        assert!(before_cursor_col.is_none());
                        before_cursor_col.get_or_insert(i);
                    } else {
                        assert!(after_cursor_col.is_none());
                        after_cursor_col.get_or_insert(i);
                    }
                }
            }
            assert!(s.find(|c| self.marker_predicate(c)).is_none());
            (before_cursor_col, after_cursor_col)
        }

        /// Strip the markers off `lines`, and return the cursor positions
        /// `(lnum, col)` before and after the underlying motion. Panics if the
        /// markers are not found or duplicate markers are detected.
        pub fn strip_markers(
            &self,
            lines: &mut [String],
        ) -> ((usize, usize), (usize, usize)) {
            let mut before_position = None;
            let mut after_position = None;
            for (lnum, line) in lines.iter_mut().enumerate() {
                let lnum = lnum + 1;
                let (before_col, after_col) = self.strip_marker_str(line);
                if let Some(i) = before_col {
                    assert!(before_position.is_none());
                    before_position.get_or_insert((lnum, i));
                }
                if let Some(j) = after_col {
                    assert!(after_position.is_none());
                    after_position.get_or_insert((lnum, j));
                }
            }
            (before_position.unwrap(), after_position.unwrap())
        }
    }

    impl BufferLike for Vec<&'static str> {
        type Error = ();

        fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
            self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
        }

        fn lines(&self) -> Result<usize, Self::Error> {
            Ok(self.len())
        }
    }

    impl BufferLike for Vec<String> {
        type Error = ();

        fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
            self.get(lnum - 1).map(|s| s.to_string()).ok_or(())
        }

        fn lines(&self) -> Result<usize, Self::Error> {
            Ok(self.len())
        }
    }

    fn into_vec_string<I: IntoIterator<Item = &'static str>>(
        v: I,
    ) -> Vec<String> {
        v.into_iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_cursor_marker_strip_markers() {
        let cm = CursorMarker::default();

        let mut lines = into_vec_string(["foo {bar", "hel}lo"]);
        assert_eq!(cm.strip_markers(&mut lines), ((1, 4), (2, 3)));
        assert_eq!(lines, vec!["foo bar", "hello"]);

        let mut lines = into_vec_string(["foo{ b}ar", "hello"]);
        assert_eq!(cm.strip_markers(&mut lines), ((1, 3), (1, 5)));
        assert_eq!(lines, vec!["foo bar", "hello"]);

        let mut lines = into_vec_string(["foo} b{ar", "hello"]);
        assert_eq!(cm.strip_markers(&mut lines), ((1, 5), (1, 3)));
        assert_eq!(lines, vec!["foo bar", "hello"]);

        let mut lines = into_vec_string(["fo{}o bar", "hello"]);
        assert_eq!(cm.strip_markers(&mut lines), ((1, 2), (1, 2)));
        assert_eq!(lines, vec!["foo bar", "hello"]);

        let mut lines = into_vec_string(["fo}{o bar", "hello"]);
        assert_eq!(cm.strip_markers(&mut lines), ((1, 2), (1, 2)));
        assert_eq!(lines, vec!["foo bar", "hello"]);

        let mut lines = into_vec_string(["hello", "foo{ b}ar"]);
        assert_eq!(cm.strip_markers(&mut lines), ((2, 3), (2, 5)));
        assert_eq!(lines, vec!["hello", "foo bar"]);

        let mut lines = into_vec_string(["hello", "foo} b{ar"]);
        assert_eq!(cm.strip_markers(&mut lines), ((2, 5), (2, 3)));
        assert_eq!(lines, vec!["hello", "foo bar"]);

        let mut lines = into_vec_string(["hello", "fo{}o bar"]);
        assert_eq!(cm.strip_markers(&mut lines), ((2, 2), (2, 2)));
        assert_eq!(lines, vec!["hello", "foo bar"]);

        let mut lines = into_vec_string(["hello", "fo}{o bar"]);
        assert_eq!(cm.strip_markers(&mut lines), ((2, 2), (2, 2)));
        assert_eq!(lines, vec!["hello", "foo bar"]);
    }

    #[test]
    #[should_panic]
    fn test_cursor_marker_strip_markers_invalid1() {
        let cm = CursorMarker::default();
        let mut lines = into_vec_string(["he}}{llo"]);
        cm.strip_markers(&mut lines);
    }

    #[test]
    #[should_panic]
    fn test_cursor_marker_strip_markers_invalid2() {
        let cm = CursorMarker::default();
        let mut lines = into_vec_string(["he{llo"]);
        cm.strip_markers(&mut lines);
    }

    #[test]
    #[should_panic]
    fn test_cursor_marker_strip_markers_invalid3() {
        let cm = CursorMarker::default();
        let mut lines = into_vec_string(["he}}llo"]);
        cm.strip_markers(&mut lines);
    }

    #[test]
    #[should_panic]
    fn test_cursor_marker_strip_markers_invalid4() {
        let cm = CursorMarker::default();
        let mut lines = into_vec_string(["hello"]);
        cm.strip_markers(&mut lines);
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
