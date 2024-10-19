/// Test if a character is a 汉字.
///
/// Adapted from Github repository: https://github.com/tsroten/zhon.
/// File: https://github.com/tsroten/zhon/blob/main/src/zhon/hanzi.py.
pub fn is_hanzi(c: char) -> bool {
    match c {
        // Ideographic number zero.
        '\u{3007}'
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
        | '\u{2e80}'..='\u{2ef3}' => true,
        _ => false,
    }
}

pub fn is_space(c: char) -> bool {
    match c {
        // ASCII whitespace.
        ' ' | '\t'
        // CJK ideographic space, suggested by GPT. See also
        // https://www.compart.com/en/unicode/U+3000.
        | '\u{3000}'
        // CJK ideographic half fill space. See also
        // https://www.compart.com/en/unicode/block/U+3000.
        | '\u{303f}' => true,
        _ => false,
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
