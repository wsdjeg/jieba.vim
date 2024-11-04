if exists("g:loaded_jieba_vim")
    finish
endif
let g:loaded_jieba_vim = 1

let g:jieba_vim_lazy = get(g:, 'jieba_vim_lazy', 1)
let g:jieba_vim_user_dict = get(g:, 'jieba_vim_user_dict', '')

if !has('python3')
    echoerr "python3 is required by jieba.vim"
    finish
endif

py3 import jieba_vim
py3 import jieba_vim.navigation


command! JiebaPreviewCancel py3 jieba_vim.preview_cancel()

let s:motions = ["w", "W"]

for ky in s:motions
    execute 'nnoremap <silent> <Plug>(Jieba_preview_' . ky . ') :<C-u>py3 jieba_vim.preview(jieba_vim.navigation.word_motion.nmap_' . ky . ')<CR>'
endfor
nnoremap <silent> <Plug>(Jieba_preview_cancel) :<C-u>py3 jieba_vim.preview_cancel()<CR>

for ky in s:motions
    execute 'nnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.nmap_' . ky . '(" . v:count1 . ")<CR>"'
    execute 'onoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.omap_' . ky . '(" . v:operator . ", " . v:count1 . ")<CR>:py3 jieba_vim.navigation.teardown_omap_' . ky . '()<CR>"'
    execute 'xnoremap <expr> <silent> <Plug>(Jieba_' . ky . ') ":<C-u>py3 jieba_vim.navigation.xmap_' . ky . '(" . v:count1 . ")<CR>:py3 jieba_vim.navigation.teardown_xmap_' . ky . '()<CR>"'
endfor
