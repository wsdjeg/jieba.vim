"""
Property-based integration tests based on junegunn/vader.vim
(https://github.com/junegunn/vader.vim).
"""

import os
import typing as ty
import contextlib
import uuid
from pathlib import Path
import subprocess

import hypothesis
from hypothesis import strategies as st

Path('cases').mkdir(exist_ok=True, parents=False)


def the_strategy():
    paragraph_st = st.lists(st.sampled_from(['a', ',', ' ', '\n']), min_size=1)
    mode_st = st.sampled_from(['n', 'o', 'xchar', 'xline', 'xblock'])
    simple_move_st = st.sampled_from(['h', 'j', 'k', 'l'])
    basic_jieba_motion_st = st.sampled_from(
        ['w', 'W', 'e', 'E', 'b', 'B', 'ge', 'gE'])
    count_st = paragraph_st.flatmap(
        lambda para: st.integers(min_value=1, max_value=len(para)))
    count_jieba_motion_st = st.tuples(
        count_st, basic_jieba_motion_st).map(lambda x: f'{x[0]}{x[1]}')
    jieba_motion_st = st.one_of(basic_jieba_motion_st, count_jieba_motion_st)

    def just_singleton_list(e):
        return st.lists(st.just(e), min_size=1, max_size=1)

    def cat(*lists_st):
        def cat_lists(lists):
            r = []
            for l in lists:
                r.extend(l)
            return r

        return st.tuples(*lists_st).map(cat_lists)

    def generate_based_on_mode(mode):
        if mode == 'n':
            setup_key_seq_st = st.lists(simple_move_st)
            jieba_key_seq_st = st.lists(jieba_motion_st, min_size=1)
            teardown_key_seq_st = st.just(None)
        elif mode == 'o':
            setup_key_seq_st = st.lists(simple_move_st)
            jieba_key_seq_st = cat(
                just_singleton_list('"xy'),
                st.lists(jieba_motion_st, min_size=1, max_size=1))
            teardown_key_seq_st = st.just(None)
        elif mode == 'xchar':
            setup_key_seq_st = cat(
                st.lists(simple_move_st), just_singleton_list('v'),
                st.lists(simple_move_st))
            jieba_key_seq_st = st.lists(jieba_motion_st, min_size=1)
            teardown_key_seq_st = just_singleton_list('"xy')
        elif mode == 'xline':
            setup_key_seq_st = cat(
                st.lists(simple_move_st), just_singleton_list('V'),
                st.lists(simple_move_st))
            jieba_key_seq_st = st.lists(jieba_motion_st, min_size=1)
            teardown_key_seq_st = just_singleton_list('"xy')
        else:
            setup_key_seq_st = cat(
                st.lists(simple_move_st),
                st.just(['\\<C-v>']),
                st.lists(simple_move_st),
            )
            jieba_key_seq_st = st.lists(jieba_motion_st, min_size=1)
            teardown_key_seq_st = just_singleton_list('"xy')
        return st.tuples(paragraph_st, st.just(mode), setup_key_seq_st,
                         jieba_key_seq_st, teardown_key_seq_st)

    return mode_st.flatmap(generate_based_on_mode)


class VaderBlock:
    def __init__(self, outfile: ty.TextIO, label: str, comment: str = ''):
        self.outfile = outfile
        self.label = label
        self.comment = comment or None

    def __enter__(self):
        if self.comment:
            self.outfile.write(f'{self.label} ({self.comment}):\n')
        else:
            self.outfile.write(f'{self.label}:\n')
        return self

    def print(self, string: str = ''):
        if string:
            self.outfile.write(f'  {string}\n')
        else:
            self.outfile.write('\n')

    def __exit__(self, _a, _b, _c):
        self.outfile.write('\n')


@contextlib.contextmanager
def write_vader_hooks(
    outfile: ty.TextIO,
    mode: str,
):
    jieba_keys = ['w', 'W', 'e', 'E', 'b', 'B', 'ge', 'gE']
    with VaderBlock(outfile, 'Before') as block:
        block.print(f'Log "{mode[0]}map jieba keys"')
        for k in jieba_keys:
            block.print(f'{mode[0]}map {k} <Plug>(Jieba_{k})')
    with VaderBlock(outfile, 'After') as block:
        block.print(f'Log "{mode[0]}unmap jieba keys"')
        for k in jieba_keys:
            block.print(f'{mode[0]}unmap {k}')
    yield
    with VaderBlock(outfile, 'Before'):
        pass
    with VaderBlock(outfile, 'After'):
        pass


def write_vader_given_block(outfile: ty.TextIO, paragraph: list[str]):
    with VaderBlock(outfile, 'Given') as block:
        for line in ''.join(paragraph).splitlines():
            block.print(line)


def write_key_sequence(block: VaderBlock, keys: list[str], bang: bool = True):
    if bang:
        for key in keys:
            block.print(f'normal! {key}')
    else:
        for key in keys:
            block.print(f'normal {key}')


def write_vader_execute_then_block(
    outfile: ty.TextIO,
    mode: str,
    setup_keys: list[str] | None,
    jieba_keys: list[str] | None,
    teardown_keys: list[str] | None,
):
    if mode[0] == 'n':
        with VaderBlock(outfile, 'Execute') as block:
            # Record ground truth
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys)
            block.print('let g:proptest_groundtruth_line_after = line(".")')
            block.print('let g:proptest_groundtruth_col_after = col(".")')
            # Record jieba
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys, bang=False)
            block.print('let g:proptest_jieba_line_after = line(".")')
            block.print('let g:proptest_jieba_col_after = col(".")')
        with VaderBlock(outfile, 'Then') as block:
            block.print('AssertEqual '
                        'g:proptest_groundtruth_line_after, '
                        'g:proptest_jieba_line_after')
            block.print('AssertEqual '
                        'g:proptest_groundtruth_col_after, '
                        'g:proptest_jieba_col_after')
    elif mode[0] == 'o':
        with VaderBlock(outfile, 'Execute') as block:
            # Record groundtruth
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys)
            block.print('let g:propttest_groundtruth_yanked = @x')
            # Record jieba
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys, bang=False)
            block.print('let g:proptest_jieba_yanked = @x')
        with VaderBlock(outfile, 'Then') as block:
            block.print('AssertEqual '
                        'g:propttest_groundtruth_yanked, '
                        'g:proptest_jieba_yanked')
    else:
        with VaderBlock(outfile, 'Execute') as block:
            # Record groundtruth
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys)
            write_key_sequence(block, teardown_keys)
            block.print(
                '''let g:proptest_groundtruth_lline_after = line("'<")''')
            block.print(
                '''let g:proptest_groundtruth_lcol_after = col("'<")''')
            block.print(
                '''let g:proptest_groundtruth_rline_after = line("'>")''')
            block.print(
                '''let g:proptest_groundtruth_rcol_after = col("'>")''')
            block.print('let g:proptest_groundtruth_yanked = @x')
            # Record jieba
            block.print('normal! gg0')
            write_key_sequence(block, setup_keys)
            write_key_sequence(block, jieba_keys, bang=False)
            write_key_sequence(block, teardown_keys)
            block.print('''let g:proptest_jieba_lline_after = line("'<")''')
            block.print('''let g:proptest_jieba_lcol_after = col("'<")''')
            block.print('''let g:proptest_jieba_rline_after = line("'>")''')
            block.print('''let g:proptest_jieba_rcol_after = col("'>")''')
            block.print('let g:proptest_jieba_yanked = @x')
        with VaderBlock(outfile, 'Then') as block:
            gt_vars = [
                'g:proptest_groundtruth_lline_after',
                'g:proptest_groundtruth_lcol_after',
                'g:proptest_groundtruth_rline_after',
                'g:proptest_groundtruth_rcol_after',
                'g:proptest_groundtruth_yanked',
            ]
            jieba_vars = [
                'g:proptest_jieba_lline_after',
                'g:proptest_jieba_lcol_after',
                'g:proptest_jieba_rline_after',
                'g:proptest_jieba_rcol_after',
                'g:proptest_jieba_yanked',
            ]
            for gv, jv in zip(gt_vars, jieba_vars):
                block.print(f'AssertEqual {gv}, {jv}')


def write_vader_test(
    paragraph: list[str],
    mode: str,
    setup_keys: list[str] | None,
    jieba_keys: list[str] | None,
    teardown_keys: list[str] | None,
    name: Path | None = None,
) -> Path:
    if name is None:
        name = Path('cases') / (str(uuid.uuid4()) + '.vader')
    with open(name, 'w', encoding='utf-8') as outfile:
        with write_vader_hooks(outfile, mode):
            write_vader_given_block(outfile, paragraph)
            write_vader_execute_then_block(outfile, mode, setup_keys,
                                           jieba_keys, teardown_keys)
    return name


def eval_with_vim(vader_test_file: Path, unlink_on_success: bool = True):
    vim_bin = os.environ.get('VIM_BIN_NAME', 'vim')
    assert vim_bin in ('vim', 'nvim')
    try:
        subprocess.run(
            [vim_bin, '-u', 'vimrc', '-c', f'silent Vader! {vader_test_file}'],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            timeout=10)
        if unlink_on_success:
            vader_test_file.unlink()
    except subprocess.CalledProcessError:
        assert False, 'wrong result'
    except subprocess.TimeoutExpired:
        assert False, 'timeout'


@hypothesis.given(the_strategy())
@hypothesis.settings(deadline=None)
@hypothesis.example((['a'], 'n', [], ['102039494923949w'], None))
@hypothesis.example((['a'], 'xchar', ['v'], ['w'], ['"xy']))
@hypothesis.example((['a'], 'o', [], ['"xy', 'w'], None))
@hypothesis.example(([',', ',', ','], 'n', [], ['w'], None))
@hypothesis.example((['a', 'a'], 'n', [], ['b', 'w'], None))
def test_jieba_en(args):
    paragraph, mode, setup_keys, jieba_keys, teardown_keys = args
    vader_test_file = write_vader_test(paragraph, mode, setup_keys, jieba_keys,
                                       teardown_keys)
    eval_with_vim(vader_test_file)


def add_named_cases():
    cases = {}
    named_cases = Path('named_cases')
    if named_cases.is_dir():
        for vader_test_file in named_cases.iterdir():

            def _named_case():
                eval_with_vim(vader_test_file, unlink_on_success=False)

            cases[f'test_{vader_test_file.stem}'] = _named_case
    return cases


globals().update(**add_named_cases())
