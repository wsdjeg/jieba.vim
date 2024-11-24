use core::fmt;
use std::path::Path;

/// A test case that can be verified through Vim vader test (see
/// https://github.com/junegunn/vader.vim). The test should implement Display
/// so that it can be pretty-printed on test error.
pub trait VerifiableCase: fmt::Display {
    /// Write the test case to a file that can be used by vader.vim. Panics if
    /// the file cannot be written.
    fn to_vader<P: AsRef<Path>>(&self, path: P);
}
