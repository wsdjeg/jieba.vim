# jieba.vim: Vim 的中文按词跳转插件

*在[此](https://github.com/kkew3/jieba.vim/tree/dev/rust)关注最新开发进度。*

*For English, see below.*

## 简介

Vim (以及很多其它文本编辑器) 使用 [word motions][1] 在一行内移动光标。对于英语等使用空格分隔单词的语言，它能很好地工作，但对于像中文一样不使用空格分隔单词的语言则很难使用。

[jieba][2] 是一个用于中文分词的 Python 包。已经有很多插件项目诸如 [Jieba][3] (VSCode)、[Deno bridge jieba][4] (Emacs)、[jieba_nvim][5] (neovim) 将其用以更好地编辑中文文本。然而我还没有发现 Vim 8 上的 jieba 插件，因此我开发了这个插件。

## 安装

本插件使用 Python3 + Rust 开发，Vim 需要 `+python3` 特性以正常使用。

对于 [vim-plug][6]，使用如下代码安装：

```vim
Plug 'kkew3/jieba.vim', { 'branch': 'rust', 'do': './build.sh' }
```

可能需要进入插件目录调整 `pythonx/Cargo.toml` 中的 pyo3 python ABI 版本，以匹配 vim 中 python3 的版本。
可以在终端使用

```bash
vim +"py3 print(sys.version)"
```

查看 vim 的 python3 版本。

## 功能

1. 增强八个 Vim word motion 的功能，即 `b`、`B`、`ge`、`gE`、`w`、`W`、`e`、`E`，使其能用于中文分词（同时也保留其按空格分词的功能）。其行为与默认行为相似，例如 `w` 不会跳过中文标点而 `W` 会跳过中文标点等。
2. 预览 word motion 的跳转位置。

由于中文分词有时存在歧义，即使没有歧义也会有人类与 jieba 的对齐问题，因此有时中文 word motion 的跳转位置并不显然。这时用户可能想提前预览将要进行的跳转将会跳转到哪些位置。

## 使用

本插件设计为非侵入式，即默认不映射任何按键，但提供一些命令与 `<Plug>(...)` 映射供使用者自行配置。提供一个命令：

- `JiebaPreviewCancel`：用于取消按词跳转位置预览

提供以下 `<Plug>()` 映射，其中 `X` 表示上文所述的八个 Vim word motion 按键，即 `b`、`B`、`ge`、`gE`、`w`、`W`、`e`、`E`：

- `<Plug>(Jieba_preview_cancel)`：即 `JiebaPreviewCancel` 命令
- `<Plug>(Jieba_preview_X)`：预览增强了的 `X` 的跳转位置
- `<Plug>(Jieba_X)`: 增强了的 `X`，同时在 normal、operator-pending、visual (除 visual line 模式外) 三种模式下可用，以及可与 count 协同使用。例如假设 `w` 被映射到 `<Plug>(Jieba_w)`，那么 `3w` 将是向后跳三个词，`d3w` 是删除后三个词

用户可自行在 `.vimrc` 中将按键映射到这些 `<Plug>()` 映射。例如：

```vim
nmap <LocalLeader>jw <Plug>(Jieba_preview_w)
" 等等，以及
map w <Plug>(Jieba_w)
" 等等
```

### 例子 1 -- 灵活启用/禁用 jieba.vim

在 `.vimrc` 中加入以下代码：

```vim
function! s:JiebaMapKeys()
    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'nmap <buffer> <LocalLeader>j' . ky . ' <Plug>(Jieba_preview_' . ky . ')'
        for md in modes
            execute md . 'map ' . ky . ' <Plug>(Jieba_' . ky . ')'
        endfor
    endfor
endfunction

function! s:JiebaUnmapKeys()
    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'silent! nunmap <buffer> <LocalLeader>j' . ky
        for md in modes
            execute 'silent! ' . md . 'unmap ' . ky
        endfor
    endfor
endfunction

command! JiebaEnable call s:JiebaMapKeys()
command! JiebaDisable call s:JiebaUnmapKeys()
```

该代码在调用 `JiebaEnable` 命令时启用 jieba.vim，而调用 `JiebaDisable` 命令时禁用 jieba.vim。启用时 `<LocalLeader>jX` 会被映射为 `<Plug>(Jieba_preview_X)`、`X` 会被映射为 `<Plug>(Jieba_X)`。

### 例子 2 -- 我的配置

```vim
function! s:JiebaMapKeys()
    if exists("b:jieba_enabled")
        return
    endif
    let b:jieba_enabled = 1

    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'nmap <buffer> <LocalLeader>j' . ky . ' <Plug>(Jieba_preview_' . ky . ')'
        for md in modes
            execute md . 'map ' . ky . ' <Plug>(Jieba_' . ky . ')'
        endfor
    endfor
endfunction

function! s:JiebaUnmapKeys()
    if exists("b:jieba_enabled")
        unlet b:jieba_enabled
    else
        return
    endif

    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'silent! nunmap <buffer> <LocalLeader>j' . ky
        for md in modes
            execute 'silent! ' . md . 'unmap ' . ky
        endfor
    endfor
endfunction

function! s:JiebaToggle()
    if exists("b:jieba_enabled")
        call s:JiebaUnmapKeys()
    else
        call s:JiebaMapKeys()
    endif
endfunction

command! JiebaEnable call s:JiebaMapKeys()
command! JiebaDisable call s:JiebaUnmapKeys()
command! JiebaToggle call s:JiebaToggle()

nnoremap <Leader>jj :<C-u>JiebaToggle<CR>

augroup jieba_group
    autocmd!
    autocmd FileType text :JiebaEnable
    autocmd FileType markdown :JiebaEnable
    autocmd FileType tex :JiebaEnable
augroup END
```

## Bug

- 无法对行末文字做 `onoremap` 操作。详见[这个提问][onoremap-question]。

## 对于开发者

若想在本地运行针对 rust 实现的测试，

```bash
cd pythonx
cargo test
```

---

# jieba.vim: Facilitate better word motions when editing Chinese text in Vim

## Introduction

Vim (and many other text editors) use [word motions][1] to move the cursor within a line.
It works well for space-delimited language like English, but not quite well for language like Chinese, where there's no space between words.

[jieba][2] is a Python library for Chinese word segmentation.
It has been used in various projects (e.g. [Jieba][3] (for VSCode), [Deno bridge jieba][4] (for Emacs), [jieba_nvim][5] (for neovim)) to facilitate better word motions when editing Chinese.
However, I haven't seen one for Vim.
That's why I develop this plugin.

## Installation

This plugin was developed using Python3 + Rust.
`+python3` features is required for Vim to use the jieba.vim.

For [vim-plug][6],

```vim
Plug 'kkew3/jieba.vim', { 'branch': 'rust', 'do': './build.sh' }
```

User may need to adjust the pyo3 python ABI in `pythonx/Cargo.toml` under the plugin directory after downloading the plugin, in order to match with the python3 version vim is compiled against.
The vim's python3 version may be checked by the following command at terminal:

```bash
vim +"py3 print(sys.version)"
```

## Functions

1. Augment eight Vim word motions (i.e. `b`, `B`, `ge`, `gE`, `w`, `W`, `e`, `E`) such that they can be used in Chinese text and English text at the same time. The augmented behavior remains similar. For example, augmented `w` won't jump over Chinese punctuation whereas `W` will.
2. Preview the destination of the word motions beforehand.

Since there's sometimes ambiguity in Chinese word segmentation, and since even when there's no ambiguity, jieba library may not align well with human users, it's not always evident where a word motion will jump to.
In such circumstance, user may want to preview jumps beforehand.

## Usage

This plugin is designed to be nonintrusive, i.e. not providing any default keymaps.
However, various commands and `<Plug>(...)` mappings are provided for users to manually configure to their needs.
Provided commands:

- `JiebaPreviewCancel`: used to clear up the preview markup

Provided `<Plug>()` mappings, wherein `X` denotes the eight Vim word motion keys, i.e. `b`, `B`, `ge`, `gE`, `w`, `W`, `e`, `E`:

- `<Plug>(Jieba_preview_cancel)`: same as the command `JiebaPreviewCancel`
- `<Plug>(Jieba_preview_X)`: preview the destination of the augmented `X`
- `<Plug>(Jieba_X)`: the augmented `X`. This mapping is usable in normal, operator-pending and visual modes (except for visual line mode), and can be used together with count. For example, assuming that `w` has been mapped to `<Plug>(Jieba_w)`, then `3w` will jump three words forward, `d3w` will delete three words forward

User may map keys to these `<Plug>()` mappings on their own.
For example,

```vim
nmap <LocalLeader>jw <Plug>(Jieba_preview_w)
" etc., and
map w <Plug>(Jieba_w)
" etc.
```

### Usage example 1 -- enable and disable jieba.vim anytime

Insert the following code in `.vimrc`:

```vim
function! s:JiebaMapKeys()
    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'nmap <buffer> <LocalLeader>j' . ky . ' <Plug>(Jieba_preview_' . ky . ')'
        for md in modes
            execute md . 'map ' . ky . ' <Plug>(Jieba_' . ky . ')'
        endfor
    endfor
endfunction

function! s:JiebaUnmapKeys()
    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'silent! nunmap <buffer> <LocalLeader>j' . ky
        for md in modes
            execute 'silent! ' . md . 'unmap ' . ky
        endfor
    endfor
endfunction

command! JiebaEnable call s:JiebaMapKeys()
command! JiebaDisable call s:JiebaUnmapKeys()
```

This snippet enable jieba.vim key mappings when `JiebaEnable` command is called, and disable those mappings when `JiebaDisable` is called.
When enabling jieba.vim, `<LocalLeader>jX` will be mapped to `<Plug>(Jieba_preview_X)`, and `X` will be mapped to `<Plug>(Jieba_X)`.

### Usage example 2 -- my configuration

```vim
function! s:JiebaMapKeys()
    if exists("b:jieba_enabled")
        return
    endif
    let b:jieba_enabled = 1

    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'nmap <buffer> <LocalLeader>j' . ky . ' <Plug>(Jieba_preview_' . ky . ')'
        for md in modes
            execute md . 'map ' . ky . ' <Plug>(Jieba_' . ky . ')'
        endfor
    endfor
endfunction

function! s:JiebaUnmapKeys()
    if exists("b:jieba_enabled")
        unlet b:jieba_enabled
    else
        return
    endif

    let keys = ["b", "B", "ge", "gE", "w", "W", "e", "E",]
    let modes = ["n", "o", "x",]
    for ky in keys
        execute 'silent! nunmap <buffer> <LocalLeader>j' . ky
        for md in modes
            execute 'silent! ' . md . 'unmap ' . ky
        endfor
    endfor
endfunction

function! s:JiebaToggle()
    if exists("b:jieba_enabled")
        call s:JiebaUnmapKeys()
    else
        call s:JiebaMapKeys()
    endif
endfunction

command! JiebaEnable call s:JiebaMapKeys()
command! JiebaDisable call s:JiebaUnmapKeys()
command! JiebaToggle call s:JiebaToggle()

nnoremap <Leader>jj :<C-u>JiebaToggle<CR>

augroup jieba_group
    autocmd!
    autocmd FileType text :JiebaEnable
    autocmd FileType markdown :JiebaEnable
    autocmd FileType tex :JiebaEnable
augroup END
```

## Bug

- `onoremap` does not apply to the end of line. For details see [this question][onoremap-question].

## For developers

To run tests against rust implementation locally,

```bash
cd pythonx
cargo test
```



[1]: https://vimdoc.sourceforge.net/htmldoc/motion.html#word-motions
[2]: https://github.com/fxsjy/jieba
[3]: https://marketplace.visualstudio.com/items?itemName=StephanosKomnenos.jieba
[4]: https://github.com/ginqi7/deno-bridge-jieba
[5]: https://github.com/cathaysia/jieba_nvim
[6]: https://github.com/junegunn/vim-plug
[onoremap-question]: https://stackoverflow.com/q/79082971/7881370
