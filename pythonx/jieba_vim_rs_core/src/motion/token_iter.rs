// Copyright 2024 Kaiwen Wu. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

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
    /// `true` if the cursor lies in a token at end-of-line.
    pub eol: bool,
}

/// Forward iterator of [`TokenIteratorItem`]s in a `buffer`. If the cursor
/// `col` is in a token, starts from that token; if `col` is to the right of
/// the last token in current line, starts from the next token in the buffer.
/// An empty line is regarded as a `None` token. If the cursor is at an empty
/// line, also starts from that empty line.
pub struct ForwardTokenIterator<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    jieba: &'p C,
    tokens: Vec<Token>,
    token_index: usize,
    lnum: usize,
    /// Number of lines in `buffer`.
    lines: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
    /// Whether current item is the cursor item or not.
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
        let cursor =
            (col == 0 && tokens.is_empty()) || token_index < tokens.len();
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
                let eol = self.token_index == self.tokens.len() - 1;
                self.token_index += 1;
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: Some(to_yield),
                    cursor: self.cursor,
                    eol,
                }))
            } else if self.cursor
                && self.tokens.is_empty()
                && self.token_index == 0
            {
                // The cursor line is empty.
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: None,
                    cursor: self.cursor,
                    eol: true,
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
                                eol: true,
                            }))
                        } else {
                            let to_yield = self
                                .tokens
                                .get(self.token_index)
                                .copied()
                                .unwrap();
                            let eol = self.token_index == self.tokens.len() - 1;
                            self.token_index += 1;
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: Some(to_yield),
                                cursor: self.cursor,
                                eol,
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

/// Backward iterator of [`TokenIteratorItem`]s in a `buffer`. If the cursor
/// `col` is in a token, starts from that token; if `col` is to the right of
/// the last token in current line, starts from that last token. An empty line
/// is regarded as a `None` token. If the cursor is at an empty line, also
/// starts from that empty line.
pub struct BackwardTokenIterator<'b, 'p, B: ?Sized, C> {
    buffer: &'b B,
    jieba: &'p C,
    tokens: Vec<Token>,
    token_index: usize,
    lnum: usize,
    /// Whether to cut into word (true) or WORD (false).
    word: bool,
    /// Whether current item is the cursor item or not.
    cursor: bool,
    /// Whether current item is the first item or not.
    first: bool,
}

impl<'b, 'p, B, C> BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    /// Construct a [`BackwardTokenIterator`], starting from the token where
    /// the cursor position `(lnum, col)` lies in.
    pub fn new(
        buffer: &'b B,
        jieba: &'p C,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> Result<Self, B::Error> {
        let tokens = token::parse_str(buffer.getline(lnum)?, jieba, word);
        let token_index = super::index_tokens(&tokens, col);
        let cursor = (col == 0 && tokens.is_empty()) || token_index.is_some();
        // One past the cursor token index.
        let token_index = token_index.map(|i| i + 1).unwrap_or(tokens.len());
        Ok(Self {
            buffer,
            jieba,
            tokens,
            token_index,
            lnum,
            word,
            cursor,
            first: true,
        })
    }

    fn fetch_prev_line(&mut self, lnum: usize) -> Result<(), B::Error> {
        self.tokens = token::parse_str(
            self.buffer.getline(lnum - 1)?,
            self.jieba,
            self.word,
        );
        Ok(())
    }
}

impl<'b, 'p, B, C> Iterator for BackwardTokenIterator<'b, 'p, B, C>
where
    B: BufferLike + ?Sized,
    C: JiebaPlaceholder,
{
    type Item = Result<TokenIteratorItem, B::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_item = {
            if self.token_index > 0 {
                self.token_index -= 1;
                let eol = self.token_index == self.tokens.len() - 1;
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: Some(
                        self.tokens.get(self.token_index).copied().unwrap(),
                    ),
                    cursor: self.cursor,
                    eol,
                }))
            } else if self.first && self.tokens.is_empty() {
                // The cursor line is empty.
                Some(Ok(TokenIteratorItem {
                    lnum: self.lnum,
                    token: None,
                    cursor: self.cursor,
                    eol: true,
                }))
            } else if self.lnum > 1 {
                match self.fetch_prev_line(self.lnum) {
                    Err(err) => Some(Err(err)),
                    Ok(()) => {
                        self.lnum -= 1;
                        self.token_index = self.tokens.len();
                        if self.tokens.is_empty() {
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: None,
                                cursor: self.cursor,
                                eol: true,
                            }))
                        } else {
                            self.token_index -= 1;
                            let eol = self.token_index == self.tokens.len() - 1;
                            Some(Ok(TokenIteratorItem {
                                lnum: self.lnum,
                                token: Some(
                                    self.tokens
                                        .get(self.token_index)
                                        .copied()
                                        .unwrap(),
                                ),
                                cursor: self.cursor,
                                eol,
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
        if self.first {
            self.first = false;
        }
        next_item
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackwardTokenIterator, ForwardTokenIterator, TokenIteratorItem,
    };
    use crate::token::{test_macros, Token};
    use jieba_rs::Jieba;
    use once_cell::sync::OnceCell;

    impl From<(usize, Option<Token>, bool, bool)> for TokenIteratorItem {
        fn from(value: (usize, Option<Token>, bool, bool)) -> Self {
            Self {
                lnum: value.0,
                token: value.1,
                cursor: value.2,
                eol: value.3,
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
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, true, true).into())]
        );
        let it = get_forward_token_iterator(&buffer, 1, 1, true);
        assert!(it.collect::<Vec<_>>().is_empty());
        let it = get_forward_token_iterator(&buffer, 1, 2, true);
        assert!(it.collect::<Vec<_>>().is_empty());

        let buffer = vec!["", "", ""];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, None, true, true).into()),
                Ok((2, None, false, true).into()),
                Ok((3, None, false, true).into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 2, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, true, true).into()),
                Ok((3, None, false, true).into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, false, true).into()),
                Ok((3, None, false, true).into()),
            ]
        );

        let buffer = vec!["", " "];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, None, true, true).into()),
                Ok((2, Some(test_macros::token!(0, 0, 1, Space)), false, true)
                    .into()),
            ]
        );

        let buffer = vec!["aaa  "];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true, false)
                    .into()),
                Ok((1, Some(test_macros::token!(3, 4, 5, Space)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 1, 3, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(3, 4, 5, Space)),
                true,
                true
            )
                .into())]
        );
        let it = get_forward_token_iterator(&buffer, 1, 4, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(3, 4, 5, Space)),
                true,
                true
            )
                .into())]
        );
        let it = get_forward_token_iterator(&buffer, 1, 5, true);
        assert!(it.collect::<Vec<_>>().is_empty());

        let buffer = vec!["aaa aaa"];
        let it = get_forward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true, false)
                    .into()),
                Ok((
                    1,
                    Some(test_macros::token!(3, 3, 4, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((1, Some(test_macros::token!(4, 6, 7, Word)), false, true)
                    .into()),
            ]
        );

        let buffer = vec!["aaa", "aa aa", "", "  aaa"];
        let it = get_forward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), true, true)
                    .into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((3, None, false, true).into()),
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 1, 3, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((3, None, false, true).into()),
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 1, 4, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((3, None, false, true).into()),
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 3, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((3, None, true, true).into()),
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 3, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_forward_token_iterator(&buffer, 4, 5, true);
        assert!(it.collect::<Vec<_>>().is_empty());
        let it = get_forward_token_iterator(&buffer, 4, 6, true);
        assert!(it.collect::<Vec<_>>().is_empty());
    }

    fn get_backward_token_iterator<'b>(
        buffer: &'b Vec<&'static str>,
        lnum: usize,
        col: usize,
        word: bool,
    ) -> BackwardTokenIterator<'b, 'static, Vec<&'static str>, Jieba> {
        let jieba = JIEBA.get().unwrap();
        BackwardTokenIterator::new(buffer, jieba, lnum, col, word).unwrap()
    }

    #[test]
    fn test_backward_token_iterator() {
        let buffer = vec![""];
        let it = get_backward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, true, true).into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, false, true).into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 2, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, false, true).into())]
        );

        let buffer = vec!["", "", ""];
        let it = get_backward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, true, true).into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((1, None, false, true).into())]
        );
        let it = get_backward_token_iterator(&buffer, 2, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, true, true).into()),
                Ok((1, None, false, true).into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 2, 2, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, false, true).into()),
                Ok((1, None, false, true).into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 3, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((3, None, true, true).into()),
                Ok((2, None, false, true).into()),
                Ok((1, None, false, true).into()),
            ]
        );

        let buffer = vec![" ", ""];
        let it = get_backward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 0, 1, Space)),
                true,
                true
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 0, 1, Space)),
                false,
                true
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 2, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, true, true).into()),
                Ok((1, Some(test_macros::token!(0, 0, 1, Space)), false, true)
                    .into())
            ]
        );
        let it = get_backward_token_iterator(&buffer, 2, 2, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((2, None, false, true).into()),
                Ok((1, Some(test_macros::token!(0, 0, 1, Space)), false, true)
                    .into())
            ]
        );

        let buffer = vec!["aaa  "];
        let it = get_backward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 2, 3, Word)),
                true,
                false
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 4, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(3, 4, 5, Space)), true, true)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, false)
                    .into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 1, 5, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(3, 4, 5, Space)), false, true)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, false)
                    .into()),
            ]
        );

        let buffer = vec!["aaa aaa"];
        let it = get_backward_token_iterator(&buffer, 1, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 2, 3, Word)),
                true,
                false
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 5, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((1, Some(test_macros::token!(4, 6, 7, Word)), true, true)
                    .into()),
                Ok((
                    1,
                    Some(test_macros::token!(3, 3, 4, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, false)
                    .into()),
            ]
        );

        let buffer = vec!["aaa", "aa aa", "", "  aaa"];
        let it = get_backward_token_iterator(&buffer, 1, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 2, 3, Word)),
                true,
                true
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 1, 3, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![Ok((
                1,
                Some(test_macros::token!(0, 2, 3, Word)),
                false,
                true
            )
                .into())]
        );
        let it = get_backward_token_iterator(&buffer, 3, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((3, None, true, true).into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 3, 1, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((3, None, false, true).into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 4, 0, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((4, Some(test_macros::token!(0, 1, 2, Space)), true, false)
                    .into()),
                Ok((3, None, false, true).into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, true)
                    .into()),
            ]
        );
        let it = get_backward_token_iterator(&buffer, 4, 4, true);
        assert_eq!(
            it.collect::<Vec<_>>(),
            vec![
                Ok((4, Some(test_macros::token!(2, 4, 5, Word)), true, true)
                    .into()),
                Ok((
                    4,
                    Some(test_macros::token!(0, 1, 2, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((3, None, false, true).into()),
                Ok((2, Some(test_macros::token!(3, 4, 5, Word)), false, true)
                    .into()),
                Ok((
                    2,
                    Some(test_macros::token!(2, 2, 3, Space)),
                    false,
                    false
                )
                    .into()),
                Ok((2, Some(test_macros::token!(0, 1, 2, Word)), false, false)
                    .into()),
                Ok((1, Some(test_macros::token!(0, 2, 3, Word)), false, true)
                    .into()),
            ]
        );
    }
}
