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

use crate::cursor_marker;
use crate::cursor_marker::{CursorMarker, CursorPosition};
use assert_cmd::Command;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fmt, io};

#[derive(Eq, PartialEq, Clone, Deserialize, Serialize)]
pub enum Mode {
    Normal,
    VisualChar,
    VisualLine,
    VisualBlock,
    Operator,
}

impl AsRef<str> for Mode {
    fn as_ref(&self) -> &str {
        match self {
            Mode::Normal => "n",
            Mode::VisualChar => "xc",
            Mode::VisualLine => "xl",
            Mode::VisualBlock => "xb",
            Mode::Operator => "o",
        }
    }
}

#[derive(Eq, PartialEq, Clone, Deserialize, Serialize)]
pub enum Motion {
    SmallW(usize),
    LargeW(usize),
    SmallE(usize),
    LargeE(usize),
    SmallB(usize),
    LargeB(usize),
    SmallGe(usize),
    LargeGe(usize),
}

impl fmt::Display for Motion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Motion::SmallW(c) if c == &0 => write!(f, "w"),
            Motion::SmallW(c) => write!(f, "{}w", c),
            Motion::LargeW(c) if c == &0 => write!(f, "W"),
            Motion::LargeW(c) => write!(f, "{}W", c),
            Motion::SmallE(c) if c == &0 => write!(f, "e"),
            Motion::SmallE(c) => write!(f, "{}e", c),
            Motion::LargeE(c) if c == &0 => write!(f, "E"),
            Motion::LargeE(c) => write!(f, "{}E", c),
            Motion::SmallB(c) if c == &0 => write!(f, "b"),
            Motion::SmallB(c) => write!(f, "{}b", c),
            Motion::LargeB(c) if c == &0 => write!(f, "B"),
            Motion::LargeB(c) => write!(f, "{}B", c),
            Motion::SmallGe(c) if c == &0 => write!(f, "ge"),
            Motion::SmallGe(c) => write!(f, "{}ge", c),
            Motion::LargeGe(c) if c == &0 => write!(f, "gE"),
            Motion::LargeGe(c) => write!(f, "{}gE", c),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct VerifiedCaseInput {
    pub group_id: String,
    pub test_name: String,
    pub before_cursor_position: CursorPosition,
    pub after_cursor_position: CursorPosition,
    pub buffer: Vec<String>,
    pub stripped_buffer: Vec<String>,
    pub mode: Mode,
    pub operator: String,
    pub motion: Motion,
    pub o_v: bool,
    pub d_special: bool,
}

#[derive(Deserialize, Serialize)]
struct VerifiedCaseInputResult {
    input: VerifiedCaseInput,
    verified: bool,
}

fn write_vader_given_block<W: Write>(
    mut tofile: W,
    buffer_lines: &[String],
) -> io::Result<()> {
    writeln!(tofile, "Given:")?;
    for line in buffer_lines.iter() {
        if line.is_empty() {
            writeln!(tofile, "  ")?;
        } else {
            writeln!(tofile, "  {}", line)?;
        }
    }
    writeln!(tofile, "")?;
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidCursorMarker {
        inner: cursor_marker::Error,
        group_id: String,
        test_name: String,
    },
    InvalidArgument(String),
    CannotVerify {
        group_id: String,
        test_name: String,
    },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl VerifiedCaseInput {
    fn write_vader<W: Write>(&self, mut tofile: W) -> io::Result<()> {
        let buffer_lines = &self.stripped_buffer;
        let lnum_before = self.before_cursor_position.lnum;
        let col_before = self.before_cursor_position.col + 1;
        let lnum_after = self.after_cursor_position.lnum;
        let col_after = self.after_cursor_position.col + 1;
        let operator = &self.operator;
        let motion = &self.motion;

        match self.mode {
            Mode::Normal => {
                write_vader_given_block(&mut tofile, &buffer_lines)?;
                write!(
                    tofile,
                    r#"
Execute:
  call cursor({lnum_before}, {col_before})
  normal! {motion}
  let g:groundtruth_lnum = line(".")
  let g:groundtruth_col = col(".")

Then:
  AssertEqual g:groundtruth_lnum, {lnum_after}
  AssertEqual g:groundtruth_col, {col_after}
"#
                )?;
            }
            Mode::VisualChar | Mode::VisualLine | Mode::VisualBlock => {
                write_vader_given_block(&mut tofile, &buffer_lines)?;
                let v = match self.mode {
                    Mode::VisualChar => "v",
                    Mode::VisualLine => "V",
                    Mode::VisualBlock => r"\<C-v>",
                    _ => panic!("Unexpected error"),
                };
                write!(
                    tofile,
                    r#"
Execute:
  call cursor({lnum_before}, {col_before})
  execute "normal! {v}{motion}" | execute "normal! mx\<Esc>"
  let g:groundtruth_lnum = line("'x")
  let g:groundtruth_col = col("'x")

Then:
  AssertEqual g:groundtruth_lnum, {lnum_after}
  AssertEqual g:groundtruth_col, {col_after}
"#
                )?;
            }
            Mode::Operator => {
                write!(
                    tofile,
                    r#"
Before:
  function! VeCursor(lnum, col)
    set virtualedit=onemore
    call cursor(a:lnum, a:col)
  endfunction

"#
                )?;
                write_vader_given_block(&mut tofile, &buffer_lines)?;
                if !(operator == "d" || operator == "c" || operator == "y") {
                    panic!("Unsupported operator: {}", operator);
                }
                let o_v = if self.o_v { "v" } else { "" };
                if operator == "y" {
                    write!(
                        tofile,
                        r#"
Execute:
  call cursor({lnum_before}, {col_before})
  let @b = ""
  let @x = ""
  normal! "x{operator}{motion}
  let g:groundtruth_lnum = line(".")
  let g:groundtruth_col = col(".")
  $put x
  1,$y b
  let g:groundtruth_buffer = @b

  silent! normal! u
  let @b = ""
  let @x = ""
  call cursor({lnum_before}, {col_before})
  execute 'normal! "x{operator}{o_v}:call VeCursor({lnum_after}, {col_after})' . "\<cr>"
  set virtualedit=
  let g:rust_lnum = line(".")
  let g:rust_col = col(".")
  $put x
  1,$y b
  let g:rust_buffer = @b

Then:
  AssertEqual g:groundtruth_lnum, g:rust_lnum
  AssertEqual g:groundtruth_col, g:rust_col
  AssertEqual g:groundtruth_buffer, g:rust_buffer

Before:
    "#
                    )?;
                } else if operator == "c" {
                    write!(
                        tofile,
                        r#"
Execute:
  call cursor({lnum_before}, {col_before})
  let @b = ""
  normal! {operator}{motion}XXX
  let g:groundtruth_lnum = line(".")
  let g:groundtruth_col = col(".")
  1,$y b
  let g:groundtruth_buffer = @b

  silent! normal! u
  let @b = ""
  call cursor({lnum_before}, {col_before})
  execute "normal! {operator}{o_v}:call VeCursor({lnum_after}, {col_after})\<cr>XXX"
  set virtualedit=
  let g:rust_lnum = line(".")
  let g:rust_col = col(".")
  1,$y b
  let g:rust_buffer = @b

Then:
  AssertEqual g:groundtruth_lnum, g:rust_lnum
  AssertEqual g:groundtruth_col, g:rust_col
  AssertEqual g:groundtruth_buffer, g:rust_buffer

Before:
    "#
                    )?;
                } else {
                    let dd = if self.d_special { "normal! dd" } else { "" };
                    write!(
                        tofile,
                        r#"
Execute:
  call cursor({lnum_before}, {col_before})
  let @b = ""
  normal! {operator}{motion}
  let g:groundtruth_lnum = line(".")
  let g:groundtruth_col = col(".")
  1,$y b
  let g:groundtruth_buffer = @b

  silent! normal! u
  let @b = ""
  call cursor({lnum_before}, {col_before})
  execute "normal! {operator}{o_v}:call VeCursor({lnum_after}, {col_after})\<cr>"
  {dd}
  set virtualedit=
  let g:rust_lnum = line(".")
  let g:rust_col = col(".")
  1,$y b
  let g:rust_buffer = @b

Then:
  AssertEqual g:groundtruth_lnum, g:rust_lnum
  AssertEqual g:groundtruth_col, g:rust_col
  AssertEqual g:groundtruth_buffer, g:rust_buffer

Before:
    "#
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn new(
        group_id: String,
        test_name: String,
        marked_buffer: Vec<String>,
        mode: Mode,
        operator: String,
        motion: Motion,
        o_v: bool,
        d_special: bool,
    ) -> Result<Self, Error> {
        let parsed_buffer = CursorMarker
            .strip_markers(marked_buffer.clone())
            .map_err(|err| Error::InvalidCursorMarker {
            inner: err,
            group_id: group_id.clone(),
            test_name: test_name.clone(),
        })?;
        match &mode {
            Mode::Operator => {
                if operator.is_empty() {
                    return Err(Error::InvalidArgument(
                        "When mode is Mode::Operator, operator should not be empty".into()));
                }
            }
            _ => {
                if !operator.is_empty() {
                    return Err(Error::InvalidArgument(
                        "When mode is not Mode::Operator, operator should be empty".into()));
                }
            }
        }

        Ok(VerifiedCaseInput {
            group_id,
            test_name,
            before_cursor_position: parsed_buffer.before_cursor_position,
            after_cursor_position: parsed_buffer.after_cursor_position,
            buffer: marked_buffer,
            stripped_buffer: parsed_buffer.stripped_buffer,
            mode,
            operator,
            motion,
            o_v,
            d_special,
        })
    }

    pub fn verify_case(self) -> Result<Self, Error> {
        // Create the working directory if not exists.
        let basedir: PathBuf = [
            env::var("CARGO_MANIFEST_DIR").unwrap(),
            ".verified_cases".into(),
        ]
        .iter()
        .collect();
        fs::create_dir(&basedir).ok();

        // Form the unique case identifier.
        let case_name = format!("{}-{}", self.group_id, self.test_name);

        // Try loading verification input and result.
        let verified_input_result_file: PathBuf =
            [&basedir, Path::new(&format!("{}-io.json", case_name))]
                .iter()
                .collect();
        if let Ok(verified_input_result_str) =
            fs::read_to_string(&verified_input_result_file)
        {
            if let Ok(verified_input_result) =
                serde_json::from_str::<VerifiedCaseInputResult>(
                    &verified_input_result_str,
                )
            {
                if &verified_input_result.input == &self {
                    if !verified_input_result.verified {
                        return Err(Error::CannotVerify {
                            group_id: self.group_id.clone(),
                            test_name: self.test_name.clone(),
                        });
                    } else {
                        return Ok(self);
                    }
                }
            }
        }

        // Create a minimal vimrc if not already exists.
        let vimrc_file_path: PathBuf =
            [&basedir, Path::new("vimrc")].iter().collect();
        if let Ok(mut vimrc_file) = File::create_new(vimrc_file_path) {
            vimrc_file
                .write_all("set rtp+=~/.vim/bundle/vader.vim\n".as_bytes())?;
        }

        // Create the vim vader test file.
        let vader_file_name = format!("{}.vader", case_name);
        let vader_file_path: PathBuf =
            [&basedir, Path::new(&vader_file_name)].iter().collect();
        self.write_vader(BufWriter::new(File::create(
            vader_file_path.clone(),
        )?))?;

        // Run vader test with vim, and see if the case can be verified.
        let assert = Command::new("vim")
            .args(&[
                "-N",
                "-u",
                "vimrc",
                &format!("+:Vader! {}", vader_file_name),
            ])
            .current_dir(&basedir)
            .timeout(Duration::from_secs(5))
            .assert();
        let verified_result = assert.try_success().is_ok();

        // Try dumping result to json.
        let verified_input_result = VerifiedCaseInputResult {
            input: self.clone(),
            verified: verified_result,
        };
        if let Ok(contents) = serde_json::to_string(&verified_input_result) {
            fs::write(verified_input_result_file, contents).ok();
        }

        if !verified_result {
            return Err(Error::CannotVerify {
                group_id: self.group_id.clone(),
                test_name: self.test_name.clone(),
            });
        }

        Ok(self)
    }
}
