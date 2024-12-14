use super::{BufferLike, MotionOutput, WordMotion};
use crate::token::JiebaPlaceholder;

impl<C: JiebaPlaceholder> WordMotion<C> {
    /// Vim motion `ge` (if `word` is `true`) or `gE` (if `word` is `false`)
    /// in visual mode. Take in `cursor_pos` (lnum, col), and return the new
    /// cursor position. Note that `lnum` is 1-indexed, and `col` is 0-indexed.
    /// We denote both `word` and `WORD` with the English word "word" below.
    ///
    /// # Basics
    ///
    /// `ge`/`gE` jumps to the last character of previous word. Empty line is
    /// considered as a word.
    ///
    /// # Edge cases
    ///
    /// - If current cursor is on the first character of the first token in the
    ///   buffer, no further jump should be made.
    /// - If there is no previous word to the left of current cursor, jump to
    ///   the first character of the first token in the buffer.
    ///
    /// # Panics
    ///
    /// - If current cursor `col` is to the right of the last token in current
    ///   line of the buffer.
    pub fn xmap_ge<B: BufferLike + ?Sized>(
        &self,
        buffer: &B,
        cursor_pos: (usize, usize),
        count: u64,
        word: bool,
    ) -> Result<MotionOutput, B::Error> {
        self.nmap_ge(buffer, cursor_pos, count, word)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "verifiable_case")]
    use jieba_vim_rs_test_macro::verified_cases;
    #[cfg(not(feature = "verifiable_case"))]
    use jieba_vim_rs_test_macro::verified_cases_dry_run as verified_cases;

    // Copied from nmap_ge.
    #[verified_cases(
        mode = "xc",
        motion = "ge",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["}{"])]
    #[vcase(name = "space", buffer = ["}{ "])]
    #[vcase(name = "space", buffer = ["}   { "])]
    #[vcase(name = "newline_newline", buffer = ["}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "   {  "])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "   {  "])]
    #[vcase(name = "one_word", buffer = ["}{aaaa"])]
    #[vcase(name = "one_word", buffer = ["}aa{aa"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["aaa}a{   "])]
    #[vcase(name = "one_word_space", buffer = ["aaa}a  { "])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["}   {aaaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  {aaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  aa{a"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  aa{a"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a  { "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa  { "], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a{   "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa{   "], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa}a", "{"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa}a    ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], count = 2)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   {aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "  { aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["", "   aaa}a  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaaa  { "], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "aaa{a"])]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "", "aaa{a"], count = 3)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", " ", " ", "aaa{a"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", " ", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aa}a aaaa", " ", "  ", "{"], count = 2)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaaa", "}", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", "", "  ", "{"], count = 2)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["", "  ", "}", "aa{a"])]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 2)]
    #[vcase(name = "two_words_newline_one_word", buffer = ["aaaa aa}a", "", "  ", "{aaa"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}{"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}aaa  aaa{aa"], count = 10293949403)]
    mod motion_xcmap_ge {}

    // Copied from xcmap_ge above.
    #[verified_cases(
        mode = "xl",
        motion = "ge",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["}{"])]
    #[vcase(name = "space", buffer = ["}{ "])]
    #[vcase(name = "space", buffer = ["}   { "])]
    #[vcase(name = "newline_newline", buffer = ["}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "   {  "])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "   {  "])]
    #[vcase(name = "one_word", buffer = ["}{aaaa"])]
    #[vcase(name = "one_word", buffer = ["}aa{aa"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["aaa}a{   "])]
    #[vcase(name = "one_word_space", buffer = ["aaa}a  { "])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["}   {aaaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  {aaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  aa{a"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  aa{a"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a  { "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa  { "], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a{   "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa{   "], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa}a", "{"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa}a    ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], count = 2)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   {aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "  { aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["", "   aaa}a  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaaa  { "], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "aaa{a"])]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "", "aaa{a"], count = 3)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", " ", " ", "aaa{a"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", " ", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aa}a aaaa", " ", "  ", "{"], count = 2)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaaa", "}", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", "", "  ", "{"], count = 2)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["", "  ", "}", "aa{a"])]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 2)]
    #[vcase(name = "two_words_newline_one_word", buffer = ["aaaa aa}a", "", "  ", "{aaa"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}{"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}aaa  aaa{aa"], count = 10293949403)]
    mod motion_xlmap_ge {}

    // Copied from xcmap_ge above.
    #[verified_cases(
        mode = "xb",
        motion = "ge",
        timeout = 50,
        backend_path = "crate::motion::WORD_MOTION"
    )]
    #[vcase(name = "empty", buffer = ["}{"])]
    #[vcase(name = "space", buffer = ["}{ "])]
    #[vcase(name = "space", buffer = ["}   { "])]
    #[vcase(name = "newline_newline", buffer = ["}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "  ", "{"])]
    #[vcase(name = "newline_space_newline", buffer = ["}  ", "   {  "])]
    #[vcase(name = "newline_space_newline", buffer = ["  ", "}", "   {  "])]
    #[vcase(name = "one_word", buffer = ["}{aaaa"])]
    #[vcase(name = "one_word", buffer = ["}aa{aa"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"])]
    #[vcase(name = "one_word", buffer = ["}aaa{a"], count = 2)]
    #[vcase(name = "one_word_space", buffer = ["aaa}a{   "])]
    #[vcase(name = "one_word_space", buffer = ["aaa}a  { "])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"])]
    #[vcase(name = "space_one_word", buffer = ["}   aaa{a"], count = 2)]
    #[vcase(name = "space_one_word", buffer = ["}   {aaaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  {aaa"])]
    #[vcase(name = "two_words", buffer = ["aaa}a  aa{a"])]
    #[vcase(name = "two_words", buffer = ["}aaaa  aa{a"], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a  { "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa  { "], count = 2)]
    #[vcase(name = "space_one_word_space", buffer = ["   aaa}a{   "])]
    #[vcase(name = "space_one_word_space", buffer = ["}   aaaa{   "], count = 2)]
    #[vcase(name = "one_word_newline", buffer = ["aaa}a", "{"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"])]
    #[vcase(name = "newline_one_word", buffer = ["}", "aaa{a"], count = 2)]
    #[vcase(name = "one_word_space_newline", buffer = ["aaa}a    ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "{"])]
    #[vcase(name = "two_words_space_newline", buffer = ["aaaa aa}a    ", "  ", "  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaa{a"], count = 2)]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   {aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "  { aaaa"])]
    #[vcase(name = "newline_space_one_word", buffer = ["", "   aaa}a  { "])]
    #[vcase(name = "newline_space_one_word", buffer = ["}", "   aaaa  { "], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "aaa{a"])]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["     ", "}", "", "aaa{a"], count = 2)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", "", "", "aaa{a"], count = 3)]
    #[vcase(name = "space_newline_one_word", buffer = ["}     ", " ", " ", "aaa{a"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", " ", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aa}a aaaa", " ", "  ", "{"], count = 2)]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaaa", "}", "  ", "{"])]
    #[vcase(name = "two_words_newline_space_newline", buffer = ["aaa aaa}a", "", "  ", "{"], count = 2)]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["", "  ", "}", "aa{a"])]
    #[vcase(name = "newline_space_newline_one_word", buffer = ["}", "  ", "", "aa{a"], count = 2)]
    #[vcase(name = "two_words_newline_one_word", buffer = ["aaaa aa}a", "", "  ", "{aaa"], count = 2)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}{"], count = 10293949403)]
    #[vcase(name = "large_unnecessary_count", buffer = ["}aaa  aaa{aa"], count = 10293949403)]
    mod motion_xbmap_ge {}
}
