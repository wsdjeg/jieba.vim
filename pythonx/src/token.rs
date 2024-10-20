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

    }
}

#[cfg(test)]
mod tests {
    use super::is_hanzi;

    #[test]
    fn test_is_hanzi_sanity_check() {
        assert!(is_hanzi('我'));
        assert!(is_hanzi('爱'));
        assert!(is_hanzi('你'));
    }

    // CJK whitespace is not a hanzi.
    #[test]
    fn test_is_hanzi_cjk_whitespace_not_hanzi() {
        assert!(!is_hanzi('\u{3000}'));
        assert!(!is_hanzi('\u{303f}'))
    }
}
