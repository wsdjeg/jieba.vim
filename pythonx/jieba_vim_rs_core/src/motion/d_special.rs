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

use super::{index_tokens, BufferLike};
use crate::token::{self, JiebaPlaceholder, TokenLike, TokenType};

/// Check if current motion satisfies d-special case. See
/// https://vimhelp.org/change.txt.html#d-special.
///
/// `cursor_pos` is the position before motion, and `new_cursor_pos` is the one
/// after motion.
pub fn is_d_special<B: BufferLike + ?Sized, C: JiebaPlaceholder>(
    buffer: &B,
    jieba: &C,
    cursor_pos: (usize, usize),
    new_cursor_pos: (usize, usize),
    word: bool,
) -> Result<bool, B::Error> {
    let (lnum, col) = cursor_pos;
    let (new_lnum, new_col) = new_cursor_pos;

    if lnum == new_lnum {
        return Ok(false);
    }

    let tokens_cursor_line =
        token::parse_str(buffer.getline(lnum)?, jieba, word);
    if !tokens_cursor_line.is_empty() {
        let i = index_tokens(&tokens_cursor_line, col).unwrap();
        if tokens_cursor_line[..i].iter().any(|tok| match tok.ty {
            TokenType::Space => false,
            TokenType::Word => true,
        }) {
            return Ok(false);
        }
        let cursor_token = &tokens_cursor_line[i];
        if let TokenType::Word = cursor_token.ty {
            if col > cursor_token.first_char() {
                return Ok(false);
            }
        }
    }

    let tokens_new_cursor_line =
        token::parse_str(buffer.getline(new_lnum)?, jieba, word);
    if !tokens_new_cursor_line.is_empty() {
        let j = index_tokens(&tokens_new_cursor_line, new_col).unwrap();
        if tokens_new_cursor_line[j + 1..]
            .iter()
            .any(|tok| match tok.ty {
                TokenType::Space => false,
                TokenType::Word => true,
            })
        {
            return Ok(false);
        }
        let new_cursor_token = &tokens_new_cursor_line[j];
        if let TokenType::Word = new_cursor_token.ty {
            if new_col < new_cursor_token.last_char() {
                return Ok(false);
            }
        }
    }

    Ok(true)
}
