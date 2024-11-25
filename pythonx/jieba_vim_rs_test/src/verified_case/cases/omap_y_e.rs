use super::super::Count;
use super::{utils, VerifiableCase, TEMPLATES};
use crate::cursor_marker::{self, CursorMarker};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct OmapYECase {
    pub lnum_before: usize,
    pub col_before: usize,
    pub lnum_after: usize,
    pub col_after: usize,
    pub buffer: Vec<String>,
    pub count: Count,
    pub word: bool,
}

impl OmapYECase {
    /// Create a new case. `count` equals 0 means 1 but without explicit count.
    pub fn new<C: Into<Count>>(
        marked_buffer: Vec<String>,
        count: C,
        word: bool,
    ) -> Result<Self, cursor_marker::Error> {
        let output = CursorMarker.strip_markers(marked_buffer)?;
        Ok(Self {
            lnum_before: output.before_cursor_position.lnum,
            col_before: output.before_cursor_position.col,
            lnum_after: output.after_cursor_position.lnum,
            col_after: output.after_cursor_position.col,
            buffer: output.stripped_buffer,
            count: count.into(),
            word,
        })
    }

    fn motion_str(&self) -> &'static str {
        if self.word {
            "e"
        } else {
            "E"
        }
    }
}

impl VerifiableCase for OmapYECase {
    fn to_vader(&self, path: &Path) {
        let mut writer = BufWriter::new(File::create(path).unwrap());
        let buffer = &self.buffer;
        let lnum_before = self.lnum_before;
        let col_before = utils::to_vim_col(self.col_before);
        let lnum_after = self.lnum_after;
        let col_after = utils::to_vim_col(self.col_after);
        let count = self.count.to_string();
        let motion = self.motion_str();

        let ctx = minijinja::context!(buffer);
        TEMPLATES
            .get_template("setup_omap")
            .unwrap()
            .render_to_write(ctx, &mut writer)
            .unwrap();
        let ctx = minijinja::context!(
            lnum_before,
            col_before,
            lnum_after,
            col_after,
            count,
            motion,
            o_v => true,
            d_special => false,
        );
        TEMPLATES
            .get_template("execute_omap_y")
            .unwrap()
            .render_to_write(ctx, &mut writer)
            .unwrap();
    }
}

impl fmt::Display for OmapYECase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str("\nBuffer:\n");
        out.push_str(&utils::display_buffer(&self.buffer));
        out.push_str("\nExpected motion: ");
        out.push_str(&format!(
            "({}, {}) -y{}{}-> ({}, {})\n",
            self.lnum_before,
            self.col_before,
            self.count.to_string(),
            self.motion_str(),
            self.lnum_after,
            self.col_after
        ));
        write!(f, "{}", out)
    }
}
