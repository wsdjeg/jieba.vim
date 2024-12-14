use std::fmt;
use std::path::Path;

/// A test case that can be verified through Vim vader test (see
/// https://github.com/junegunn/vader.vim). The test should implement Display
/// so that it can be pretty-printed on test error.
pub trait VerifiableCase: fmt::Display + Clone + Into<MotionOutput> {
    /// Write the test case to a file that can be used by vader.vim. Panics if
    /// the file cannot be written.
    fn to_vader(&self, path: &Path);
}

/// A mirror definition of `MotionOutput` defined in `jieba_vim_rs_core` crate.
#[derive(Debug)]
pub struct MotionOutput {
    pub new_cursor_pos: (usize, usize),
    pub d_special: bool,
    pub prevent_change: bool,
}
