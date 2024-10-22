//! Token iterators.

use super::{BufferLike, JiebaPlaceholder};
use crate::token::{self, Token};

/// Item type yieled by token iterators.
#[derive(Debug, PartialEq, Eq)]
pub struct TokenIteratorItem {
    /// The `lnum` of current token.
    pub lnum: usize,
    pub token: Option<Token>,
    /// `true` if the cursor lies in current token.
    pub cursor: bool,
}

/// Forward iterator of `(lnum, token)`s in a `buffer`. If the cursor `col` is
/// in a token, starts from that token; if `col` is to the right of the last
/// token in current line, starts from the next token in the buffer. An empty
/// line is regarded as a `None` token.
///
/// Implication: if the cursor starts at an empty line, the first token yielded
/// by the iterator will be the next token, not current empty token.
pub struct ForwardTokenIterator<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    jieba: &'p C,
    tokens: Vec<Token>,
    token_index: usize,
    lnum: usize,
    lines: usize,
    word: bool,
    cursor: bool,
}

impl<'b, 'p, B, C> ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`ForwardTokenIterator`], starting from the token where the
    /// cursor position `(lnum, col)` lies in.
    pub fn new(
        buffer: &'b B,
        jieba: &'p C,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> Result<Self, B::Error> {
        let tokens = token::parse_str(buffer.getline(lnum)?, jieba, word);
        let token_index =
            super::index_tokens(&tokens, col).unwrap_or(tokens.len());
        let cursor = token_index < tokens.len();
        let lines = buffer.lines()?;
        Ok(Self {
            buffer,
            jieba,
            tokens,
            token_index,
            lnum,
            lines,
            word,
            cursor,
        })
    }

    fn fetch_next_line(&mut self, lnum: usize) -> Result<(), B::Error> {
        self.tokens = token::parse_str(
            self.buffer.getline(lnum + 1)?,
            self.jieba,
            self.word,
        );
        Ok(())
    }
}

impl<'b, 'p, B, C> Iterator for ForwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = {
            if self.token_index < self.tokens.len() {
                let to_yield =
                    self.tokens.get(self.token_index).copied().unwrap();
                self.token_index += 1;
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: Some(to_yield),
                    cursor: self.cursor,
                }))
            } else if self.lnum < self.lines {
                match self.fetch_next_line(self.lnum) {
                    Err(err) => Some(Err(err)),
                    Ok(()) => {
                        self.lnum += 1;
                        self.token_index = 0;
                        if self.tokens.is_empty() {
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: None,
                                cursor: self.cursor,
                            }))
                        } else {
                            let to_yield = self
                                .tokens
                                .get(self.token_index)
                                .copied()
                                .unwrap();
                            self.token_index += 1;
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: Some(to_yield),
                                cursor: self.cursor,
                            }))
                        }
                    }
                }
            } else {
                None
            }
        };
        if self.cursor {
            self.cursor = false;
        }
        next_item
    }
}

#[cfg(test)]
mod tests {
    use super::{ForwardTokenIterator, TokenIteratorItem};
    use crate::token::{test_macros, Token};
    use jieba_rs::Jieba;
    use once_cell::sync::OnceCell;

    impl From<(usize, Option<Token>, bool)> for TokenIteratorItem {
        fn from(value: (usize, Option<Token>, bool)) -> Self {
            Self {
                lnum: value.0,
                token: value.1,
                cursor: value.2,
            }
        }
    }

    static JIEBA: OnceCell<Jieba> = OnceCell::new();

    #[ctor::ctor]
    fn init() {
        JIEBA.get_or_init(|| Jieba::new());
    }

    fn get_forward_token_iterator<'b>(
        buffer: &'b Vec<&'static str>,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> ForwardTokenIterator<'b, 'static, Vec<&'static str>, Jieba> {
        let jieba = JIEBA.get().unwrap();
        ForwardTokenIterator::new(buffer, jieba, lnum, col, word).unwrap()
    }

    #[test]
    fn test_forward_token_iterator() {
        let buffer = vec![""];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert!(it.collect::<Vec<_>>().is_empty());

        let buffer = vec!["", ""];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(it.collect::<Vec<_>>(), vec![Ok((2, None, false).into())]);

        let buffer = vec!["", " "];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok(
                (2, Some(test_macros::token!(0, 0, 1, Space)), false).into()
            )]
        );

        let buffer = vec!["aaa  "];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true).into()),
                Ok((1, Some(test_macros::token!(3, 4, 5, Space)), false)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 1, 3, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok(
                (1, Some(test_macros::token!(3, 4, 5, Space)), true).into()
            )]
        );
        let it = get_forward_token_iterator(&buffer, 1, 4, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok(
                (1, Some(test_macros::token!(3, 4, 5, Space)), true).into()
            )]
        );
        let it = get_forward_token_iterator(&buffer, 1, 5, true);
        assert!(it.collect::<Vec<_>>().is_empty());

        let buffer = vec!["aaa aaa"];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true).into()),
                Ok((1, Some(test_macros::token!(3, 3, 4, Space)), false)
                    .into()),
                Ok((1, Some(test_macros::token!(4, 6, 7, Word)), false).into()),
            ]
        );

        let buffer = vec!["aaa", "aa aa", "", "  aaa"];
        let it = get_forward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true).into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false).into()),
                Ok((2, Some(test_macros::token!(2, 2, 3, Space)), false)
                    .into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false).into()),
                Ok((3, None, false).into()),
                Ok((4, Some(test_macros::token!(0, 1, 2, Space)), false)
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false).into()),
            ]
        )
    }
}
