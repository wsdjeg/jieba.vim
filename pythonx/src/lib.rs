mod wrappers;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn jieba_vim_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<wrappers::WordMotionWrapper>()?;

    Ok(())
}
