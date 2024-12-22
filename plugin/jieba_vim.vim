" Copyright 2024 Kaiwen Wu. All Rights Reserved.
"
" Licensed under the Apache License, Version 2.0 (the "License"); you may not
" use this file except in compliance with the License. You may obtain a copy
" of the License at
"
"     http://www.apache.org/licenses/LICENSE-2.0
"
" Unless required by applicable law or agreed to in writing, software
" distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
" WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
" License for the specific language governing permissions and limitations
" under the License.


""
" @section Introduction, intro
" @stylized jieba
" @library
" @order intro config
" jieba.vim 是一个基于 jieba 中文分词插件.


if exists("g:loaded_jieba_vim")
    finish
endif
let g:loaded_jieba_vim = 1
""
"  (默认 1)：是/否 (1/0) 延迟加载 jieba 词典直到有中文出现。
let g:jieba_vim_lazy = get(g:, 'jieba_vim_lazy', 1)

""
" (默认空)：若为非空字符串，加载此文件路径所指向的用户自定义词典。
let g:jieba_vim_user_dict = get(g:, 'jieba_vim_user_dict', '')


""
" (默认 0)：是/否 (1/0) 自动开启 keymap。
let g:jieba_vim_keymap = get(g:, 'jieba_vim_keymap', 0)

if !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif

py3 import jieba_vim
py3 import jieba_vim.navigation


command! JiebaPreviewCancel py3 jieba_vim.preview_cancel()

let s:motions = ["w", "W", "e", "E", "b", "B", "ge", "gE"]

for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>py3 jieba_vim.preview(jieba_vim.navigation.word_motion.preview_nmap_' . ky . ')<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>py3 jieba_vim.preview_cancel()<CR>

for ky in s:motions
    execute 'nnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.nmap_' . ky . '(" . v:count1 . ")<CR>"'
    if ky ==# "e" || ky ==# "E"
        execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') "v:<C-u>py3 jieba_vim.navigation.omap_' . ky . '(\"" . v:operator . "\", " . v:count1 . ")<CR>"'
    elseif ky ==# "b" || ky ==# "B" || ky ==# "ge" || ky ==# "gE"
        execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') "<Esc>:<C-u>py3 jieba_vim.navigation.omap_' . ky . '(\"" . v:operator . "\", " . v:count1 . ")<CR>"'
    else
        execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.omap_' . ky . '(\"" . v:operator . "\", " . v:count1 . ")<CR>"'
    endif
    execute 'xnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') "<Esc>:<C-u>py3 jieba_vim.navigation.xmap_' . ky . '(" . v:count1 . ")<CR>:py3 jieba_vim.navigation.teardown_xmap_' . ky . '()<CR>"'
endfor

let s:modes = ["n", "x", "o"]
if g:jieba_vim_keymap
    for ky in s:motions
        for md in s:modes
            execute md . "map " . ky . " <Plug>(Jieba_" . ky . ")"
        endfor
    endfor
endif
