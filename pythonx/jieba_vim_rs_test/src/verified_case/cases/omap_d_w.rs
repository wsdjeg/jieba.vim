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

use super::super::Count;
use super::{utils, VerifiableCase, TEMPLATES};
use crate::cursor_marker::{self, CursorMarker};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

#[derive(PartialEq, Serialize, Deserialize)]
pub struct OmapDWCase {
    pub lnum_before: usize,
    pub col_before: usize,
    pub lnum_after: usize,
    pub col_after: usize,
    pub buffer: Vec<String>,
    pub count: Count,
    pub word: bool,
}

impl OmapDWCase {
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
            "w"
        } else {
            "W"
        }
    }
}

impl VerifiableCase for OmapDWCase {
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
            o_v => false,
            d_special => false,
        );
        TEMPLATES
            .get_template("execute_omap_d")
            .unwrap()
            .render_to_write(ctx, &mut writer)
            .unwrap();
    }
}

impl fmt::Display for OmapDWCase {
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
        write!(f, "{}", out)
    }
}
