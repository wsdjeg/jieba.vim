/// Replace space with '·', and append '␊' as newline.
pub fn display_buffer(buffer: &[String]) -> String {
    let mut out = String::new();
    for line in buffer {
        out.push_str(&line.replace(' ', "·"));
        out.push('␊');
        out.push('\n');
    }
    out
}

pub fn to_vim_col(col: usize) -> usize {
    col + 1
}
