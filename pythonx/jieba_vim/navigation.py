# Copyright 2024 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy
# of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.
"""
These names are dynamically defined in this module::

    - nmap_w
    - nmap_W
    - xmap_w
    - teardown_xmap_w
    - xmap_W
    - teardown_xmap_W
    - omap_w
    - omap_W
    - nmap_e
    - nmap_E
    - xmap_e
    - teardown_xmap_e
    - xmap_E
    - teardown_xmap_E
    - omap_e
    - omap_E
    - nmap_b
    - nmap_B
    - xmap_b
    - teardown_xmap_b
    - xmap_B
    - teardown_xmap_B
    - omap_b
    - omap_B
"""
import vim

from . import jieba_vim_rs

word_motion = None


def _init_word_motion():
    global word_motion
    if word_motion is not None:
        return
    user_dict = vim.eval('g:jieba_vim_user_dict') or None
    try:
        if int(vim.eval('g:jieba_vim_lazy')):
            word_motion = jieba_vim_rs.LazyWordMotion(user_dict)
        else:
            word_motion = jieba_vim_rs.WordMotion(user_dict)
    except (IOError, ValueError):
        vim.command('echoerr "jieba.vim: failed to load user dict: {}"'.format(
            user_dict))


_init_word_motion()


def _vim_wrapper_factory_n(motion_name):
    fun_name = 'nmap_' + motion_name

    def _motion_wrapper(count):
        method = getattr(word_motion, fun_name)
        vim.current.window.cursor = method(vim.current.buffer,
                                           vim.current.window.cursor, count)

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_x(motion_name):
    fun_name = 'xmap_' + motion_name

    def _motion_wrapper(count):
        method = getattr(word_motion, fun_name)
        # I tried `let s:jieba_vim_previous_virtualedit = &virtualedit` but got
        # error "Illegal variable name: s:jieba_vim_previous_virtualedit". Will
        # the use of global variable lead to race condition when there are
        # multiple instances of Vim open?
        vim.command('let g:jieba_vim_previous_virtualedit = &virtualedit')
        vim.command('set virtualedit=onemore')
        vim.current.window.cursor = method(vim.current.buffer,
                                           vim.current.window.cursor, count)

    def _teardown_wrapper():
        # The `m>gv` trick reference:
        # https://github.com/svermeulen/vim-NotableFt/blob/01732102c1d8c7b7bd6e221329e37685aa4ab41a/plugin/NotableFt.vim#L32
        vim.command('normal! m>')
        vim.command(
            'execute "set virtualedit=" . g:jieba_vim_previous_virtualedit')
        vim.command('normal! gv')

    return {
        fun_name: _motion_wrapper,
        'teardown_' + fun_name: _teardown_wrapper,
    }


def _vim_wrapper_factory_o(motion_name):
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(operator, count):
        method = getattr(word_motion, fun_name)
        # virtualedit trick reference:
        # https://github.com/svermeulen/vim-NotableFt/blob/01732102c1d8c7b7bd6e221329e37685aa4ab41a/plugin/NotableFt.vim#L242-L256
        #
        # I tried `let s:jieba_vim_previous_virtualedit = &virtualedit` but got
        # error "Illegal variable name: s:jieba_vim_previous_virtualedit". Will
        # the use of global variable lead to race condition when there are
        # multiple instances of Vim open?
        vim.command('let g:jieba_vim_previous_virtualedit = &virtualedit')
        vim.command('set virtualedit=onemore')
        vim.current.window.cursor = method(vim.current.buffer,
                                           vim.current.window.cursor, operator,
                                           count)
        vim.command(
            'augroup jieba_vim_reset_virtualedit '
            '| autocmd! '
            '| autocmd TextChanged,CursorMoved <buffer> '
            'execute "set virtualedit=" . g:jieba_vim_previous_virtualedit '
            '| autocmd! jieba_vim_reset_virtualedit '
            '| augroup END')

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_e(motion_name):
    assert motion_name in ['e', 'E']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(operator, count):
        method = getattr(word_motion, fun_name)
        # virtualedit trick reference:
        # https://github.com/svermeulen/vim-NotableFt/blob/01732102c1d8c7b7bd6e221329e37685aa4ab41a/plugin/NotableFt.vim#L242-L256
        #
        # I tried `let s:jieba_vim_previous_virtualedit = &virtualedit` but got
        # error "Illegal variable name: s:jieba_vim_previous_virtualedit". Will
        # the use of global variable lead to race condition when there are
        # multiple instances of Vim open?
        vim.command('let g:jieba_vim_previous_virtualedit = &virtualedit')
        vim.command('set virtualedit=onemore')
        vim.current.window.cursor, is_prev_d_special = method(
            vim.current.buffer, vim.current.window.cursor, operator, count)
        vim.command(
            'augroup jieba_vim_reset_virtualedit '
            '| autocmd! '
            '| autocmd TextChanged,CursorMoved <buffer> '
            'execute "set virtualedit=" . g:jieba_vim_previous_virtualedit '
            '| autocmd! jieba_vim_reset_virtualedit '
            '| augroup END')
        # This patch breaks `.` (see https://vimhelp.org/repeat.txt.html#.).
        # Need help on fixing this issue.
        if is_prev_d_special:
            vim.command('augroup jieba_vim_teardown_d_special '
                        '| autocmd! '
                        '| autocmd TextChanged <buffer> execute "normal! dd" '
                        '| autocmd! jieba_vim_teardown_d_special '
                        '| augroup END')

    return {fun_name: _motion_wrapper}


def _vim_wrapper_factory_omap_b(motion_name):
    assert motion_name in ['b', 'B']
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(operator, count):
        method = getattr(word_motion, fun_name)
        new_cursor = method(vim.current.buffer, vim.current.window.cursor,
                            count)
        if operator == 'c' and new_cursor == vim.current.window.cursor:
            vim.command(
                'augroup jieba_vim_prevent_change '
                '| autocmd! '
                '| autocmd ModeChanged <buffer> call feedkeys("\\<Esc>") '
                '| autocmd! jieba_vim_prevent_change '
                '| augroup END')
        else:
            vim.current.window.cursor = new_cursor

    return {fun_name: _motion_wrapper}


def _define_functions():
    for mo in ['w', 'W', 'e', 'E', 'b', 'B']:
        globals().update(_vim_wrapper_factory_n(mo))
        globals().update(_vim_wrapper_factory_x(mo))
        if mo in ['e', 'E']:
            globals().update(_vim_wrapper_factory_omap_e(mo))
        elif mo in ['b', 'B']:
            globals().update(_vim_wrapper_factory_omap_b(mo))
        else:
            globals().update(_vim_wrapper_factory_o(mo))


_define_functions()
