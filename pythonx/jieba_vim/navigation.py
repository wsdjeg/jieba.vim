"""
These names are dynamically defined in this module::

    - nmap_w
    - nmap_W
    - xmap_w
    - teardown_xmap_w
    - xmap_W
    - teardown_xmap_W
    - omap_w
    - teardown_omap_w
    - omap_W
    - teardown_omap_W
"""
import vim

from . import jieba_vim_rs

word_motion = None


def _init_word_motion():
    global word_motion
    if word_motion is not None:
        return
    user_dict = vim.eval('g:jieba_vim_user_dict') or None
    if int(vim.eval('g:jieba_vim_lazy')):
        word_motion = jieba_vim_rs.LazyWordMotion(user_dict)
    else:
        word_motion = jieba_vim_rs.WordMotion(user_dict)


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
        vim.command('set virtualedit=onemore')
        vim.current.window.cursor = method(vim.current.buffer,
                                           vim.current.window.cursor, count)

    def _teardown_wrapper():
        # The `m>gv` trick reference:
        # https://github.com/svermeulen/vim-NotableFt/blob/master/plugin/NotableFt.vim
        vim.command('normal! m>')
        vim.command('set virtualedit=')
        vim.command('normal! gv')

    return {
        fun_name: _motion_wrapper,
        'teardown_' + fun_name: _teardown_wrapper,
    }


def _vim_wrapper_factory_o(motion_name):
    fun_name = 'omap_' + motion_name

    def _motion_wrapper(operator, count):
        method = getattr(word_motion, fun_name)
        vim.command('set virtualedit=onemore')
        vim.current.window.cursor = method(vim.current.buffer,
                                           vim.current.window.cursor, operator,
                                           count)

    def _teardown_wrapper():
        vim.command('set virtualedit=')

    return {
        fun_name: _motion_wrapper,
        'teardown_' + fun_name: _teardown_wrapper,
    }


def _define_functions():
    for mo in ['w', 'W']:
        globals().update(_vim_wrapper_factory_n(mo))
        globals().update(_vim_wrapper_factory_x(mo))
        globals().update(_vim_wrapper_factory_o(mo))


_define_functions()
