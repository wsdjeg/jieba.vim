use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Count;
use super::{utils, MotionOutput, VerifiableCase, TEMPLATES};
use crate::cursor_marker::{self, CursorMarker};

#[derive(PartialEq, Clone, Serialize, Deserialize)]
pub struct OmapDGeCase {
    pub lnum_before: usize,
    pub col_before: usize,
    pub lnum_after: usize,
    pub col_after: usize,
    pub buffer: Vec<String>,
    pub count: Count,
    pub word: bool,
    pub d_special: bool,
    pub prevent_change: bool,
}

impl OmapDGeCase {
    /// Create a new case. `count` equals 0 means 1 but without explicit count.
    pub fn new<C: Into<Count>>(
        marked_buffer: Vec<String>,
        count: C,
        word: bool,
        d_special: bool,
        prevent_change: bool,
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
            d_special,
            prevent_change,
        })
    }

    fn motion_str(&self) -> &'static str {
        if self.word {
            "ge"
        } else {
            "gE"
        }
    }
}

impl VerifiableCase for OmapDGeCase {
    fn to_vader(&self, path: &Path) {
        let mut writer = BufWriter::new(File::create(path).unwrap());
        let buffer = &self.buffer;
        let lnum_before = self.lnum_before;
        let col_before = utils::to_vim_col(self.col_before);
        let lnum_after = self.lnum_after;
        let col_after = utils::to_vim_col(self.col_after);
        let count = self.count.to_string();
        let motion = self.motion_str();
        let d_special = self.d_special;
        let prevent_change = self.prevent_change;

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
            d_special,
            prevent_change,
        );
        TEMPLATES
            .get_template("execute_omap_d")
            .unwrap()
            .render_to_write(ctx, &mut writer)
            .unwrap();
    }
}

impl fmt::Display for OmapDGeCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str("\nBuffer:\n");
        out.push_str(&utils::display_buffer(&self.buffer));
        out.push_str("\nExpected motion: ");
        out.push_str(&format!(
            "({}, {}) -d{}{}-> ({}, {})\n",
            self.lnum_before,
            self.col_before,
            self.count.to_string(),
            self.motion_str(),
            self.lnum_after,
            self.col_after
        ));
        if self.d_special {
            out.push_str("\nd-special on\n");
        } else {
            out.push_str("\nd-special off\n");
        }
        if self.prevent_change {
            out.push_str("\nprevent-change on\n");
        } else {
            out.push_str("\nprevent-change off\n");
        }
        write!(f, "{}", out)
    }
}

impl Into<MotionOutput> for OmapDGeCase {
    fn into(self) -> MotionOutput {
        MotionOutput {
            new_cursor_pos: (self.lnum_after, self.col_after),
            d_special: self.d_special,
            prevent_change: self.prevent_change,
        }
    }
}
