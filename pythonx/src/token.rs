use jieba_rs::Jieba;

use crate::utils;

/// Character types.
#[derive(Debug)]
enum CharType {
    /// Whitespace characters.
    Space,
    /// Word characters.
    Word(WordCharType),
    /// Non-word characters.
    NonWord(NonWordCharType),
}

/// Word character types.
#[derive(Debug)]
enum WordCharType {
    /// 汉字 characters.
    Hanzi,
    /// Other word characters.
    Other,
}

/// Non-word character types.
#[derive(Debug)]
enum NonWordCharType {
    /// Left-associated CJK punctuations. When a word character is followed by
    /// a [`NonWordCharType::LeftPunc`], an implicit space is added in between.
    LeftPunc,
    /// Right-associated CJK punctuations. When a word character follows a
    /// [`NonWordCharType::RightPunc`], an implicit space is added in between.
    RightPunc,
    /// Isolated CJK punctuations. When a word character is followed by or
    /// follows a [`NonWordCharType::IsolatedPunc`], an implicit space is added
    /// in between.
    IsolatedPunc,
    /// Other non-word characters.
    Other,
}

// The unicodes of CJK characters and punctuations are quoted from Github
// repository: https://github.com/tsroten/zhon.
// File: https://github.com/tsroten/zhon/blob/main/src/zhon/hanzi.py.
//
// The partition of CJK punctuations into left/right/isolated types are decided
// by myself, with help from https://www.compart.com/en/unicode. For CJK
// punctuations that I don't know how to categorize, I've marked them with `??`
// on the right.
fn categorize_char(c: char) -> CharType {
    match c {
        // Vim ASCII whitespace.
        ' ' | '\t'
        // CJK ideographic space, suggested by GPT. See also
        // https://www.compart.com/en/unicode/U+3000.
        | '\u{3000}'
        // CJK ideographic half fill space. See also
        // https://www.compart.com/en/unicode/block/U+3000.
        | '\u{303f}'
        => CharType::Space,

        // Ideographic number zero.
        | '\u{3007}'
        // CJK unified ideographs.
        | '\u{4e00}'..='\u{9fff}'
        // CJK unified ideographs extension A.
        | '\u{3400}'..='\u{4dbf}'
        // CJK compatibility ideographs.
        | '\u{f900}'..='\u{faff}'
        // CJK unified ideographs extension B.
        | '\u{20000}'..='\u{2a6df}'
        // CJK unified ideographs extension C.
        | '\u{2a700}'..='\u{2b73f}'
        // CJK unified ideographs extension D.
        | '\u{2b740}'..='\u{2b81f}'
        // CJK compatibility ideographs supplement.
        | '\u{2f800}'..='\u{2fa1f}'
        // Character code ranges for the Kangxi radicals and CJK radicals
        // supplement.
        | '\u{2f00}'..='\u{2fd5}'
        | '\u{2e80}'..='\u{2ef3}'
        => CharType::Word(WordCharType::Hanzi),

        // Default value of 'iskeyword' in Vim (ASCII range).
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_'
        // Default value of 'iskeyword' in Vim (extended ASCII range).
        | '\u{c0}'..='\u{ff}'
        => CharType::Word(WordCharType::Other),

        // Fullwidth ASCII variants.
        '\u{ff04}' | '\u{ff08}' | '\u{ff3b}' | '\u{ff5b}' | '\u{ff5f}'
        // Halfwidth CJK punctuation.
        | '\u{ff62}'
        // CJK angle and corner brackets.
        | '\u{3008}' | '\u{300a}' | '\u{300c}' | '\u{300e}' | '\u{3010}'
        // CJK brackets and symbols/punctuation.
        | '\u{3014}' | '\u{3016}' | '\u{3018}' | '\u{301a}' | '\u{301d}'
        // Quotation marks and apostrophe.
        | '\u{2018}' | '\u{201c}'
        => CharType::NonWord(NonWordCharType::LeftPunc),

        // Fullwidth ASCII variants.
        '\u{ff09}' | '\u{ff0c}' | '\u{ff1a}' | '\u{ff1b}' | '\u{ff3d}'
        | '\u{ff5d}' | '\u{ff60}' | '\u{ff05}'
        // Halfwidth CJK punctuation.
        | '\u{ff63}' | '\u{ff64}'
        // CJK symbols and punctuation.
        | '\u{3001}'
        // CJK angle and corner brackets.
        | '\u{3009}' | '\u{300b}' | '\u{300d}' | '\u{300f}' | '\u{3011}'
        // CJK brackets and symbols/punctuation.
        | '\u{3015}' | '\u{3017}' | '\u{3019}' | '\u{301b}' | '\u{301e}'
        // Quotation marks and apostrophe.
        | '\u{2019}' | '\u{201d}'
        // Small form variants.
        | '\u{fe51}' | '\u{fe54}'
        // Fullwidth full stop.
        | '\u{ff0e}'
        // Fullwidth exclamation mark.
        | '\u{ff01}'
        // Fullwidth question mark.
        | '\u{ff1f}'
        // Halfwidth ideographic full stop.
        | '\u{ff61}'
        // Ideographic full stop.
        | '\u{3002}'
        => CharType::NonWord(NonWordCharType::RightPunc),

        // Fullwidth ASCII variants.
        '\u{ff02}' | '\u{ff03}' |  '\u{ff06}'
        | '\u{ff07}' | '\u{ff0a}' | '\u{ff0b}'
        | '\u{ff0d}' | '\u{ff0f}'
        | '\u{ff1c}' | '\u{ff1d}' | '\u{ff1e}' | '\u{ff20}'
        | '\u{ff3c}' | '\u{ff3e}' | '\u{ff3f}' | '\u{ff40}'
        | '\u{ff5c}' | '\u{ff5e}'
        // CJK symbols and punctuation.
        | '\u{3003}' // ??
        // CJK brackets and symbols/punctuation.
        | '\u{301c}'
        | '\u{301f}' // ??
        // Other CJK symbols.
        | '\u{3030}'
        // Special CJK indicators.
        | '\u{303e}'
        // Dashes.
        | '\u{2013}' | '\u{2014}'
        // Quotation marks and apostrophe.
        | '\u{201b}' // ??
        | '\u{201e}' // ??
        | '\u{201f}' // ??
        // General punctuation.
        | '\u{2026}' | '\u{2027}'
        // Overscores and underscores.
        | '\u{fe4f}'
        // Latin punctuation.
        | '\u{00b7}'
        => CharType::NonWord(NonWordCharType::IsolatedPunc),

        _ => CharType::NonWord(NonWordCharType::Other),
    }
}

/// The column location of a char or a token in a line.
#[derive(Debug, PartialEq, Eq)]
struct Col {
    /// The byte offset of the first char in the token.
    start_byte_index: usize,
    /// The byte offset of the last char in the token.
    incl_end_byte_index: usize,
    /// [`Col::incl_end_byte_index`] plus the byte length of the last char in
    /// utf-8.
    excl_end_byte_index: usize,
}

/// A Char token.
#[derive(Debug)]
struct Char {
    ch: char,
    col: Col,
    ty: CharType,
}

impl Char {
    fn new(ch: char, start_byte_index: usize) -> Self {
        Self {
            ch,
            col: Col {
                start_byte_index,
                incl_end_byte_index: start_byte_index,
                excl_end_byte_index: start_byte_index + ch.len_utf8(),
            },
            ty: categorize_char(ch),
        }
    }
}

/// The string `line` should not contain the end-of-line character. Return a
/// vec of `Char`s. An empty returned vec signifies that the `line` is empty.
fn parse_str_into_chars(line: &str) -> Vec<Char> {
    line.char_indices()
        .map(|(start_byte_index, ch)| Char::new(ch, start_byte_index))
        .collect()
}

/// Character group types.
#[derive(Debug, PartialEq, Eq)]
enum CharGroupType {
    /// A sequence of [`CharType::Space`] characters, or implicit whitespaces
    /// for auxiliary purpose.
    Space,
    /// A sequence of [`CharType::Word`] characters.
    Word(WordCharGroupType),
    /// A sequence of [`CharType::NonWord`] characters.
    NonWord(NonWordCharGroupType),
}

/// Word character group types.
#[derive(Debug, PartialEq, Eq)]
enum WordCharGroupType {
    /// A sequence of [`CharType::Word`] that contains [`WordCharType::Hanzi`].
    Hanzi,
    /// A sequence of [`CharType::Word`] that doesn't contain
    /// [`WordCharType::Hanzi`].
    Other,
}

/// Non-word character group types.
#[derive(Debug, PartialEq, Eq)]
enum NonWordCharGroupType {
    /// A sequence of [`CharType::NonWord`] that starts with a
    /// [`NonWordCharType::LeftPunc`] or [`NonWordCharType::IsolatedPunc`],
    /// but does not end with a [`NonWordCharType::RightPunc`] or
    /// [`NonWordCharType::IsolatedPunc`].
    LeftPuncLeading,
    /// A sequence of [`CharType::NonWord`] that ends with a
    /// [`NonWordCharType::RightPunc`] or [`NonWordCharType::IsolatedPunc`]
    /// but does not start with a [`NonWordCharType::LeftPunc`] or
    /// [`NonWordCharType::IsolatedPunc`].
    RightPuncEnding,
    /// A sequence of [`CharType::NonWord`] that starts with a
    /// [`NonWordCharType::LeftPunc`] or [`NonWordCharType::IsolatedPunc`],
    /// and ends with a [`NonWordCharType::RightPunc`] or
    /// [`NonWordCharType::IsolatedPunc`].
    LeftPuncLeadingRightPuncEnding,
    /// A sequence of [`CharType::NonWord`] that neither starts with a
    /// [`NonWordCharType::LeftPunc`] or [`NonWordCharType::IsolatedPunc`],
    /// nor ends with a [`NonWordCharType::RightPunc`] or
    /// [`NonWordCharType::IsolatedPunc`].
    Other,
}

#[derive(Debug, PartialEq, Eq)]
struct CharGroup {
    chars: Vec<char>,
    col: Col,
    ty: CharGroupType,
}

impl From<Char> for CharGroup {
    fn from(c: Char) -> Self {
        Self {
            chars: vec![c.ch],
            col: c.col,
            ty: match c.ty {
                CharType::Space => CharGroupType::Space,
                CharType::Word(WordCharType::Hanzi) => {
                    CharGroupType::Word(WordCharGroupType::Hanzi)
                }
                CharType::Word(WordCharType::Other) => {
                    CharGroupType::Word(WordCharGroupType::Other)
                }
                CharType::NonWord(NonWordCharType::LeftPunc) => {
                    CharGroupType::NonWord(
                        NonWordCharGroupType::LeftPuncLeading,
                    )
                }
                CharType::NonWord(NonWordCharType::RightPunc) => {
                    CharGroupType::NonWord(
                        NonWordCharGroupType::RightPuncEnding,
                    )
                }
                CharType::NonWord(NonWordCharType::IsolatedPunc) => {
                    CharGroupType::NonWord(
                        NonWordCharGroupType::LeftPuncLeadingRightPuncEnding,
                    )
                }
                CharType::NonWord(NonWordCharType::Other) => {
                    CharGroupType::NonWord(NonWordCharGroupType::Other)
                }
            },
        }
    }
}

impl CharGroup {
    /// Construct an implicit whitespace.
    fn new_implicit_whitespace(start_byte_index: usize) -> Self {
        Self {
            chars: vec![],
            col: Col {
                start_byte_index,
                incl_end_byte_index: start_byte_index,
                excl_end_byte_index: start_byte_index,
            },
            ty: CharGroupType::Space,
        }
    }

    /// Push a [`Char`]. Given back `c` if their types are not compatible
    /// in major class (space, word, nonword). `self`'s type may be modified
    /// accordingly, but it's guaranteed that the majar class of `self` will
    /// not be changed after push. Panics if there's gap between `self` and
    /// `c`.
    fn push(&mut self, c: Char) -> Result<(), Char> {
        assert_eq!(self.col.excl_end_byte_index, c.col.start_byte_index);

        use CharGroupType as G;
        use CharType::*;
        use NonWordCharGroupType as NG;
        use NonWordCharType as N;
        use WordCharGroupType as WG;
        use WordCharType as W;
        match (&self.ty, &c.ty) {
            (G::Space, Space) => (),

            (G::Word(WG::Hanzi), Word(_)) => (),

            (G::Word(WG::Other), Word(W::Hanzi)) => {
                self.ty = G::Word(WG::Hanzi);
            }
            (G::Word(WG::Other), Word(W::Other)) => (),

            (G::NonWord(NG::LeftPuncLeading), NonWord(N::LeftPunc))
            | (G::NonWord(NG::LeftPuncLeading), NonWord(N::Other)) => (),
            (G::NonWord(NG::LeftPuncLeading), NonWord(N::RightPunc))
            | (G::NonWord(NG::LeftPuncLeading), NonWord(N::IsolatedPunc)) => {
                self.ty = G::NonWord(NG::LeftPuncLeadingRightPuncEnding);
            }

            (G::NonWord(NG::RightPuncEnding), NonWord(N::LeftPunc))
            | (G::NonWord(NG::RightPuncEnding), NonWord(N::Other)) => {
                self.ty = G::NonWord(NG::Other);
            }
            (G::NonWord(NG::RightPuncEnding), NonWord(N::RightPunc))
            | (G::NonWord(NG::RightPuncEnding), NonWord(N::IsolatedPunc)) => (),

            (
                G::NonWord(NG::LeftPuncLeadingRightPuncEnding),
                NonWord(N::LeftPunc),
            )
            | (
                G::NonWord(NG::LeftPuncLeadingRightPuncEnding),
                NonWord(N::Other),
            ) => self.ty = G::NonWord(NG::LeftPuncLeading),
            (
                G::NonWord(NG::LeftPuncLeadingRightPuncEnding),
                NonWord(N::RightPunc),
            )
            | (
                G::NonWord(NG::LeftPuncLeadingRightPuncEnding),
                NonWord(N::IsolatedPunc),
            ) => (),

            (G::NonWord(NG::Other), NonWord(N::LeftPunc))
            | (G::NonWord(NG::Other), NonWord(N::Other)) => (),
            (G::NonWord(NG::Other), NonWord(N::RightPunc))
            | (G::NonWord(NG::Other), NonWord(N::IsolatedPunc)) => {
                self.ty = G::NonWord(NG::RightPuncEnding);
            }

            _ => return Err(c),
        }
        self.chars.push(c.ch);
        self.col.incl_end_byte_index = c.col.incl_end_byte_index;
        self.col.excl_end_byte_index = c.col.excl_end_byte_index;
        Ok(())
    }

    /// Append `group` after `self`. The type of `self` won't be changed.
    /// Panics if there's gap between `self` and `other`.
    fn append(&mut self, mut other: CharGroup) {
        assert_eq!(self.col.excl_end_byte_index, other.col.start_byte_index);
        self.chars.append(&mut other.chars);
        self.col.incl_end_byte_index = other.col.incl_end_byte_index;
        self.col.excl_end_byte_index = other.col.excl_end_byte_index;
    }
}

// `CharGroup` is not meant to be displayed. Therefore, I'm not implementing
// `std::fmt::Display`.
impl ToString for CharGroup {
    fn to_string(&self) -> String {
        self.chars.iter().collect()
    }
}

/// Group contiguous [`Char`]s of compatible major class into [`CharGroup`]s,
/// and insert implicit whitespaces in between as needed.
fn group_chars_rule(
    group: Option<CharGroup>,
    c: Char,
    _args: &(),
) -> Vec<CharGroup> {
    match group {
        None => vec![CharGroup::from(c)],
        Some(mut group) => match group.push(c) {
            Err(c) => {
                let c = CharGroup::from(c);
                // `group` and `c` are compatible in major type. We may need to
                // insert implicit whitespace in between. Since it's cheap, we
                // prepare one beforehand.
                let ispace =
                    CharGroup::new_implicit_whitespace(c.col.start_byte_index);
                use CharGroupType::*;
                use NonWordCharGroupType as N;
                match (&group.ty, &c.ty) {
                    // We never need to insert implicit space after a space.
                    (Space, Word(_)) | (Space, NonWord(_)) => vec![group, c],

                    (Word(_), Space) => vec![group, c],
                    (Word(_), NonWord(N::LeftPuncLeading))
                    | (Word(_), NonWord(N::LeftPuncLeadingRightPuncEnding)) => {
                        vec![group, ispace, c]
                    }
                    (Word(_), NonWord(_)) => vec![group, c],

                    (NonWord(_), Space) => vec![group, c],
                    (NonWord(N::RightPuncEnding), Word(_))
                    | (NonWord(N::LeftPuncLeadingRightPuncEnding), Word(_)) => {
                        vec![group, ispace, c]
                    }
                    (NonWord(_), Word(_)) => vec![group, c],

                    // Should not happen.
                    _ => panic!(),
                }
            }
            Ok(()) => vec![group],
        },
    }
}

// TODO buggy implementation
impl CharGroup {
    /// Split `self` into subgroups, whose types will be recategorized. Panics
    /// if `self.chars.len() != sizes.sum()`.
    fn split_into_subgroups(self, sizes: Vec<usize>) -> Vec<CharGroup> {
        assert_eq!(self.chars.len(), sizes.iter().sum());
        let mut sub_groups = Vec::with_capacity(sizes.len());
        let mut chars = self.chars.into_iter();
        let mut start = self.col.start_byte_index;
        for sz in sizes {
            let mut sub_chars = (0..sz).map(|_| {
                let ch = chars.next().unwrap();
                let ch = Char::new(ch, start);
                start = ch.col.excl_end_byte_index;
                ch
            });
            if let Some(ch) = sub_chars.next() {
                let mut sub_group = CharGroup::from(ch);
                for ch in sub_chars {
                    sub_group.push(ch).unwrap();
                }
                sub_groups.push(sub_group);
            }
        }
        sub_groups
    }
}

/// Concatenate contiguous [`WordCharGroupType::Other`] groups, and insert
/// implict whitespace in between otherwise. We assume that both `prev_group`
/// and `group` are of major class [`CharGroupType::Word`]. Panics if they
/// aren't.
fn insert_implicit_whitespace_in_cut_result_rule(
    prev_group: Option<CharGroup>,
    group: CharGroup,
    _args: &(),
) -> Vec<CharGroup> {
    match prev_group {
        None => vec![group],
        Some(mut prev_group) => {
            use CharGroupType::*;
            use WordCharGroupType as W;
            match (&prev_group.ty, &group.ty) {
                (Word(W::Other), Word(W::Other)) => {
                    // Concatenate `group` after `prev_group`.
                    prev_group.append(group);
                    vec![prev_group]
                }
                (Word(_), Word(_)) => {
                    let ispace = CharGroup::new_implicit_whitespace(
                        group.col.start_byte_index,
                    );
                    vec![prev_group, ispace, group]
                }
                // Shouldn't happend.
                _ => panic!(),
            }
        }
    }
}

/// Cut [`CharGroup`]s of type [`WordCharGroupType::Hanzi`] into sub groups,
/// and insert implicit whitespaces in between. Since this merging rule
/// ought to be used after [`group_chars_rule`], we won't need to care about
/// `prev_group` -- we need only to prepend it to the returned vec. One caveat
/// of `jieba` is that it may also cut text with no 汉字 at all. For example,
/// "B超abc_def" will become ("B超", "abc", "_", "def"). Therefore, after the
/// cut operation, we are required to concatenate contiguous [`CharGroup`]s
/// of type [`WordCharGroupType::Other`] before inserting the implicit
/// whitespaces. [`insert_implicit_whitespace_in_cut_result_rule`] achieves the
/// concatenation and insertion implicit whitespaces work.
fn cut_hanzi_rule(
    prev_group: Option<CharGroup>,
    group: CharGroup,
    jieba: &Jieba,
) -> Vec<CharGroup> {
    use CharGroupType::*;
    use WordCharGroupType as W;
    match group.ty {
        Word(W::Hanzi) => {
            let s = group.to_string();
            let n_chars: Vec<_> = jieba
                .cut(&s, true)
                .into_iter()
                .map(|part| part.chars().count())
                .collect();
            let sub_groups = group.split_into_subgroups(n_chars);
            utils::chain_into_vec(
                prev_group,
                utils::stack_merge(
                    sub_groups,
                    &(),
                    insert_implicit_whitespace_in_cut_result_rule,
                ),
            )
        }

        // Otherwise, return as is.
        _ => utils::chain_into_vec(prev_group, [group]),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    col: Col,
    ty: TokenType,
}

/// Token types.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    /// Either a word or a WORD token, depending on the context.
    ///
    /// A word, by definition of Vim, is either a sequence of word characters
    /// or a sequence of non-word non-whitespace characters. A WORD, in
    /// contrast, is a sequence of either word or non-word non-whitespace
    /// characters.
    Word,
    Space,
}

impl From<CharGroup> for Token {
    fn from(g: CharGroup) -> Self {
        Self {
            col: g.col,
            ty: match g.ty {
                CharGroupType::Space => TokenType::Space,
                _ => TokenType::Word,
            },
        }
    }
}

fn remove_implicit_whitespace_rule(
    prev_group: Option<CharGroup>,
    group: CharGroup,
    _args: &(),
) -> Vec<CharGroup> {
    match group.ty {
        CharGroupType::Space => {
            if group.col.start_byte_index == group.col.excl_end_byte_index {
                // Remove this implicit whitespace.
                utils::chain_into_vec(prev_group, [])
            } else {
                // Otherwise, return as is.
                utils::chain_into_vec(prev_group, [group])
            }
        }
        // Return as is.
        _ => utils::chain_into_vec(prev_group, [group]),
    }
}

/// Parse a vec of [`Char`]s into `word`s and space.
fn parse_chars_into_words(chars: Vec<Char>, jieba: &Jieba) -> Vec<Token> {
    let groups = utils::stack_merge(chars, &(), group_chars_rule);
    let groups = utils::stack_merge(groups, jieba, cut_hanzi_rule);
    let groups =
        utils::stack_merge(groups, &(), remove_implicit_whitespace_rule);
    groups.into_iter().map(Token::from).collect()
}

/// Concatenate contiguous non-space [`CharGroup`]s.
fn concat_nonspace_groups_rule(
    prev_group: Option<CharGroup>,
    group: CharGroup,
    _args: &(),
) -> Vec<CharGroup> {
    match prev_group {
        None => vec![group],
        Some(mut prev_group) => {
            use CharGroupType::*;
            match (&prev_group.ty, &group.ty) {
                (Space, _) | (_, Space) => vec![prev_group, group],
                _ => {
                    // Concatenate `group` after `prev_group`.
                    prev_group.append(group);
                    vec![prev_group]
                }
            }
        }
    }
}

/// Parse a vec of [`Char`]s into `WORD`s and space.
#[allow(non_snake_case)]
fn parse_chars_into_WORDs(chars: Vec<Char>, jieba: &Jieba) -> Vec<Token> {
    let groups = utils::stack_merge(chars, &(), group_chars_rule);
    let groups = utils::stack_merge(groups, jieba, cut_hanzi_rule);
    let groups = utils::stack_merge(groups, &(), concat_nonspace_groups_rule);
    let groups =
        utils::stack_merge(groups, &(), remove_implicit_whitespace_rule);
    groups.into_iter().map(Token::from).collect()
}

/// Parse `line` into tokens. If `into_word` is `true`, the non-space tokens
/// will be interpretable as `word`s; otherwise, they will be `WORD`s.
pub fn parse_str(line: &str, jieba: &Jieba, into_word: bool) -> Vec<Token> {
    let chars = parse_str_into_chars(line);
    if into_word {
        parse_chars_into_words(chars, jieba)
    } else {
        parse_chars_into_WORDs(chars, jieba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use proptest::prelude::*;
    use std::io::BufReader;

    #[test]
    fn test_categorize_char_sanity_check() {
        assert!(matches!(
            categorize_char('-'),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char(','),
            CharType::NonWord(NonWordCharType::Other)
        ));
        assert!(matches!(
            categorize_char('我'),
            CharType::Word(WordCharType::Hanzi)
        ));
        assert!(matches!(
            categorize_char('，'),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('。'),
            CharType::NonWord(NonWordCharType::RightPunc)
        ));
        assert!(matches!(
            categorize_char('（'),
            CharType::NonWord(NonWordCharType::LeftPunc)
        ));
        assert!(matches!(
            categorize_char('—'),
            CharType::NonWord(NonWordCharType::IsolatedPunc)
        ));
        assert!(matches!(categorize_char('\u{3000}'), CharType::Space));
    }

    #[test]
    fn test_char_group_split_into_subgroups() {
        let cg = CharGroup {
            chars: vec!['h', 'e', 'l', 'l', 'o'],
            col: Col {
                start_byte_index: 0,
                incl_end_byte_index: 4,
                excl_end_byte_index: 5,
            },
            ty: CharGroupType::Word(WordCharGroupType::Other),
        };
        let groups = cg.split_into_subgroups(vec![2, 2, 1]);
        assert_eq!(
            groups,
            vec![
                CharGroup {
                    chars: vec!['h', 'e'],
                    col: Col {
                        start_byte_index: 0,
                        incl_end_byte_index: 1,
                        excl_end_byte_index: 2,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
                CharGroup {
                    chars: vec!['l', 'l'],
                    col: Col {
                        start_byte_index: 2,
                        incl_end_byte_index: 3,
                        excl_end_byte_index: 4,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
                CharGroup {
                    chars: vec!['o'],
                    col: Col {
                        start_byte_index: 4,
                        incl_end_byte_index: 4,
                        excl_end_byte_index: 5,
                    },
                    ty: CharGroupType::Word(WordCharGroupType::Other),
                },
            ]
        );
    }

    static JIEBA: Lazy<Jieba> = Lazy::new(|| {
        let mut dict = BufReader::new(crate::DICT.as_bytes());
        Jieba::with_dict(&mut dict).unwrap()
    });

    fn parse_str_test(s: &str, into_word: bool) -> Vec<Token> {
        parse_str(s, &JIEBA, into_word)
    }

    proptest! {
        #[test]
        fn parse_str_tokens_are_nonempty_contiguous_word(s in "\\PC*") {
            let tokens = parse_str_test(&s, true);
            let mut start = 0;
            for tok in tokens {
                assert_eq!(tok.col.start_byte_index, start);
                assert!(tok.col.start_byte_index < tok.col.excl_end_byte_index);
                assert!(tok.col.start_byte_index <= tok.col.incl_end_byte_index);
                assert!(tok.col.incl_end_byte_index < tok.col.excl_end_byte_index);
                start = tok.col.excl_end_byte_index;
            }
        }
    }

    proptest! {
        #[test]
        #[allow(non_snake_case)]
        fn parse_str_tokens_are_nonempty_contiguous_WORD(s in "\\PC*") {
            let tokens = parse_str_test(&s, false);
            let mut start = 0;
            for tok in tokens {
                assert_eq!(tok.col.start_byte_index, start);
                assert!(tok.col.start_byte_index < tok.col.excl_end_byte_index);
                assert!(tok.col.start_byte_index <= tok.col.incl_end_byte_index);
                assert!(tok.col.incl_end_byte_index < tok.col.excl_end_byte_index);
                start = tok.col.excl_end_byte_index;
            }
        }
    }

    macro_rules! token {
        ($i:literal, $j:literal, $k:literal, $t:ident) => {
            Token {
                col: Col {
                    start_byte_index: $i,
                    incl_end_byte_index: $j,
                    excl_end_byte_index: $k,
                },
                ty: TokenType::$t,
            }
        };
    }

    #[test]
    fn test_parse_empty() {
        let tokens = parse_str_test("", true);
        assert!(tokens.is_empty());

        let tokens = parse_str_test("", false);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_parse_en_only_word() {
        let tokens = parse_str_test("hello, world", true);
        assert_eq!(
            tokens,
            vec![
                token!(0, 4, 5, Word),
                token!(5, 5, 6, Word),
                token!(6, 6, 7, Space),
                token!(7, 11, 12, Word),
            ]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_parse_en_only_WORD() {
        let tokens = parse_str_test("hello, world", false);
        assert_eq!(
            tokens,
            vec![
                token!(0, 5, 6, Word),
                token!(6, 6, 7, Space),
                token!(7, 11, 12, Word),
            ]
        );
    }

    #[test]
    fn test_parse_hanzi_and_en_1_word() {
        let tokens = parse_str_test("B超foo_bar", true);
        assert_eq!(
            tokens,
            vec![token!(0, 1, 4, Word), token!(4, 10, 11, Word)]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_parse_hanzi_and_en_1_WORD() {
        let tokens = parse_str_test("B超foo_bar", false);
        assert_eq!(
            tokens,
            vec![token!(0, 1, 4, Word), token!(4, 10, 11, Word)]
        );
    }

    #[test]
    fn test_parse_hanzi_and_en_2_word() {
        let tokens = parse_str_test("B超，foo。。。", true);
        assert_eq!(
            tokens,
            vec![
                token!(0, 1, 4, Word),
                token!(4, 4, 7, Word),
                token!(7, 9, 10, Word),
                token!(10, 16, 19, Word),
            ]
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_parse_hanzi_and_en_2_WORD() {
        let tokens = parse_str_test("B超，foo。。。", false);
        assert_eq!(
            tokens,
            vec![token!(0, 4, 7, Word), token!(7, 16, 19, Word)]
        );
    }
}
