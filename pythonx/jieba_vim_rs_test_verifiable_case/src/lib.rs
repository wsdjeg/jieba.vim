use assert_cmd::Command;
use core::{fmt, panic};
use jieba_vim_rs_test::cursor_marker::{CursorMarker, CursorPosition};
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs, io};
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};

enum Mode {
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

enum Motion {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

struct VerifiedCaseInput {
    group_id: Ident,
    test_name: Ident,
    before_cursor_position: CursorPosition,
    after_cursor_position: CursorPosition,
    buffers: Vec<String>,
    mode: Mode,
    operator: LitStr,
    motion: Motion,
}

impl Parse for VerifiedCaseInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let group_id: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let test_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        syn::bracketed!(content in input);
        let buffers: Vec<String> = content
            .parse_terminated(|s| s.parse::<LitStr>(), Token![,])?
            .into_iter()
            .map(|s| s.value())
            .collect();
        let parsed_buffers = match CursorMarker.strip_markers(buffers) {
            Err(err) => {
                return Err(input.error(format!(
                    "Failed to parse cursor positions from buffers: {:?}",
                    err
                )))
            }
            Ok(o) => o,
        };
        input.parse::<Token![,]>()?;

        let mode: LitStr = input.parse()?;
        let mode = match mode.value().as_str() {
            "n" => Mode::Normal,
            "xc" => Mode::VisualChar,
            "xl" => Mode::VisualLine,
            "xb" => Mode::VisualBlock,
            "o" => Mode::Operator,
            mode_str => {
                return Err(input.error(format!(
                    "Expecting 'n'/'xc'/'xl'/'xb'/'o' but found: {}",
                    mode_str
                )))
            }
        };
        input.parse::<Token![,]>()?;

        let operator: LitStr = input.parse()?;
        match &mode {
            Mode::Normal
            | Mode::VisualChar
            | Mode::VisualLine
            | Mode::VisualBlock => {
                if !operator.value().is_empty() {
                    return Err(input.error(
                        "When mode is not 'o', operator should be empty",
                    ));
                }
            }
            Mode::Operator => {
                if operator.value().is_empty() {
                    return Err(input.error(
                        "When mode is 'o', operator should not be empty",
                    ));
                }
            }
        }
        input.parse::<Token![,]>()?;

        let motion: LitStr = input.parse()?;
        static MOTION_RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^(\d+)?(w|W|e|E|b|B|ge|gE)$").unwrap());
        let motion = match MOTION_RE.captures(&motion.value()) {
            None => {
                return Err(input
                    .error(format!("Unexpected motion: {}", motion.value())))
            }
            Some(cap) => {
                let count = cap
                    .get(1)
                    .map(|s| s.as_str().parse::<usize>().unwrap())
                    .unwrap_or(0);
                match cap.get(2).unwrap().as_str() {
                    "w" => Motion::SmallW(count),
                    "W" => Motion::LargeW(count),
                    "e" => Motion::SmallE(count),
                    "E" => Motion::LargeE(count),
                    "b" => Motion::SmallB(count),
                    "B" => Motion::LargeB(count),
                    "ge" => Motion::SmallGe(count),
                    "gE" => Motion::LargeGe(count),
                    _ => panic!("Unexpected error"),
                }
            }
        };

        Ok(VerifiedCaseInput {
            group_id,
            test_name,
            before_cursor_position: parsed_buffers.before_cursor_position,
            after_cursor_position: parsed_buffers.after_cursor_position,
            buffers: parsed_buffers.striped_lines,
            mode,
            operator,
            motion,
        })
    }
}

fn write_vader_given_block<W: Write>(
    mut tofile: W,
    buffer_lines: &[String],
) -> io::Result<()> {
    writeln!(tofile, "Given:")?;
    for line in buffer_lines.iter() {
        if line.is_empty() {
            writeln!(tofile, "")?;
        } else {
            writeln!(tofile, "  {}", line)?;
        }
    }
    writeln!(tofile, "")?;
    Ok(())
}

impl VerifiedCaseInput {
    fn write_vader<W: Write>(&self, mut tofile: W) -> io::Result<()> {
        let buffer_lines = &self.buffers;
        let lnum_before = self.before_cursor_position.lnum;
        let col_before = self.before_cursor_position.col + 1;
        let lnum_after = self.after_cursor_position.lnum;
        let col_after = self.after_cursor_position.col + 1;
        let operator = self.operator.value();
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
                let reg = match motion {
                    Motion::SmallW(_)
                    | Motion::LargeW(_)
                    | Motion::SmallE(_)
                    | Motion::LargeE(_) => "'>",
                    Motion::SmallB(_)
                    | Motion::LargeB(_)
                    | Motion::SmallGe(_)
                    | Motion::LargeGe(_) => "'<",
                };
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
  execute "normal! {v}{motion}\<cr>"
  let g:groundtruth_lnum = line("{reg}")
  let g:groundtruth_col = col("{reg}")

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
    setlocal virtualedit=onemore
    call cursor(a:lnum, a:col)
  endfunction

"#
                )?;
                write_vader_given_block(&mut tofile, &buffer_lines)?;
                write!(
                    tofile,
                    r#"
Execute:
  call cursor({lnum_before}, {col_before})
  normal! {operator}{motion}
  let g:groundtruth_lnum = line(".")
  let g:groundtruth_col = col(".")
  1,$y b
  let g:groundtruth_buffer = @b

  silent! normal! u
  call cursor({lnum_before}, {col_before})
  execute "normal! {operator}:call VeCursor({lnum_after}, {col_after})\<cr>"
  setlocal virtualedit=
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

        Ok(())
    }
}

/// Usage: `verified_case_dry_run!(group_id, test_name, buffer_lines, mode,
/// operator, motion)`.
///
/// For example,
///
/// ```norun
/// verified_case!(motion_nmap_w, test_empty, ["{abc }def"], "n", "", "w")
/// ```
#[proc_macro]
pub fn verified_case(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as VerifiedCaseInput);

    match verify_case(&input) {
        Ok(verified) => {
            if !verified {
                let err_msg = format!(
                    "Can't verify `{}` from group `{}`",
                    input.test_name, input.group_id
                );
                let quoted = quote! {
                    compile_error!(#err_msg);
                };
                return quoted.into();
            }
        }
        Err(err) => {
            let quoted = quote! {
                compile_error!("Error: {}", #err);
            };
            return quoted.into();
        }
    }

    let buffers = input.buffers;
    let quoted = quote! {
        [#(#buffers),*]
    };
    quoted.into()
}

/// Check the macro input only without actually verifying the test case.
///
/// Usage: `verified_case_dry_run!(group_id, test_name, buffer_lines, mode,
/// operator, motion)`.
///
/// For example,
///
/// ```norun
/// verified_case!(motion_nmap_w, test_empty, ["{abc }def"], "n", "", "w")
/// ```
#[proc_macro]
pub fn verified_case_dry_run(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as VerifiedCaseInput);
    let buffers = input.buffers;
    let expanded = quote! {
        [#(#buffers),*]
    };
    expanded.into()
}

fn verify_case(case_info: &VerifiedCaseInput) -> Result<bool, String> {
    // Create the working directory if not exists.
    let basedir: PathBuf = [
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        ".verified_cases".into(),
    ]
    .iter()
    .collect();
    fs::create_dir(&basedir).ok();

    // Create a minimal vimrc if not already exists.
    let vimrc_file_path: PathBuf =
        [&basedir, Path::new("vimrc")].iter().collect();
    if let Ok(mut vimrc_file) = File::create_new(vimrc_file_path) {
        vimrc_file
            .write_all("set rtp+=~/.vim/bundle/vader.vim\n".as_bytes())
            .map_err(|_| format!("Failed to write vimrc file"))?;
    }

    // Form the unique case identifier.
    let case_name = format!("{}-{}", case_info.group_id, case_info.test_name);

    // Create the vim vader test file.
    let vader_file_name = format!("{}.vader", case_name);
    let vader_file_path: PathBuf =
        [&basedir, Path::new(&vader_file_name)].iter().collect();
    let mut vader_file =
        BufWriter::new(File::create(vader_file_path.clone()).map_err(
            |_| format!("Failed to create vader file: {:?}", vader_file_path),
        )?);
    case_info.write_vader(&mut vader_file).map_err(|_| {
        format!("Failed to write vader file: {:?}", vader_file_path)
    })?;

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
    Ok(assert.try_success().is_ok())
}
