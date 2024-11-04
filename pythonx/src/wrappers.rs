use jieba_rs::Jieba;
use jieba_vim_rs_core::motion::{BufferLike, WordMotion};
use jieba_vim_rs_core::token::JiebaPlaceholder;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;

struct BoundWrapper<'b, 'py, T>(&'b Bound<'py, T>);

impl<'b, 'py, T> From<&'b Bound<'py, T>> for BoundWrapper<'b, 'py, T> {
    fn from(value: &'b Bound<'py, T>) -> Self {
        Self(value)
    }
}

impl<'b, 'py> BufferLike for BoundWrapper<'b, 'py, PyAny> {
    type Error = PyErr;

    fn getline(&self, lnum: usize) -> Result<String, Self::Error> {
        Ok(self.0.get_item(lnum - 1)?.extract::<String>()?)
    }

    fn lines(&self) -> Result<usize, Self::Error> {
        Ok(self.0.len()?)
    }
}

struct JiebaWrapper(Jieba);

impl JiebaPlaceholder for JiebaWrapper {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.0.cut(sentence, true)
    }
}

struct LazyJiebaWrapper {
    path: Option<String>,
    jieba: RefCell<Option<Jieba>>,
}

impl JiebaPlaceholder for LazyJiebaWrapper {
    fn cut_hmm<'a>(&self, sentence: &'a str) -> Vec<&'a str> {
        self.jieba
            .borrow_mut()
            .get_or_insert_with(|| match &self.path {
                None => Jieba::new(),
                Some(path) => {
                    let mut reader = BufReader::new(File::open(path).unwrap());
                    Jieba::with_dict(&mut reader).unwrap()
                }
            })
            .cut(sentence, true)
    }
}

#[pyclass]
#[pyo3(name = "WordMotion")]
pub struct WordMotionWrapper {
    wm: WordMotion<JiebaWrapper>,
}

#[pymethods]
impl WordMotionWrapper {
    /// Load jieba with the default dictionary, or with custom dictionary given
    /// dictionary path.
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn from_dict(path: Option<&str>) -> PyResult<Self> {
        let jieba = match path {
            None => Jieba::new(),
            Some(path) => {
                let mut reader = BufReader::new(
                    File::open(path).map_err(|err| PyIOError::new_err(err))?,
                );
                Jieba::with_dict(&mut reader).map_err(|err| {
                    PyValueError::new_err(format!("jieba error: {}", err))
                })?
            }
        };
        Ok(Self {
            wm: WordMotion::new(JiebaWrapper(jieba)),
        })
    }

    pub fn nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: usize,
    ) -> PyResult<(usize, usize)> {
        self.wm
            .nmap_w(&BoundWrapper(buffer), cursor_pos, count, true)
    }

    #[allow(non_snake_case)]
    pub fn nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: usize,
    ) -> PyResult<(usize, usize)> {
        self.wm
            .nmap_w(&BoundWrapper(buffer), cursor_pos, count, false)
    }
}

#[pyclass]
#[pyo3(name = "LazyWordMotion")]
pub struct LazyWordMotionWrapper {
    wm: WordMotion<LazyJiebaWrapper>,
}

#[pymethods]
impl LazyWordMotionWrapper {
    #[new]
    #[pyo3(signature = (path=None))]
    pub fn from_dict(path: Option<String>) -> PyResult<Self> {
        // Check if `path` is readable beforehand.
        if let Some(path) = &path {
            File::open(path).map_err(|err| PyIOError::new_err(err))?;
        }
        Ok(Self {
            wm: WordMotion::new(LazyJiebaWrapper {
                path,
                jieba: RefCell::new(None),
            }),
        })
    }

    pub fn nmap_w(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: usize,
    ) -> PyResult<(usize, usize)> {
        self.wm
            .nmap_w(&BoundWrapper(buffer), cursor_pos, count, true)
    }

    #[allow(non_snake_case)]
    pub fn nmap_W(
        &self,
        buffer: &Bound<'_, PyAny>,
        cursor_pos: (usize, usize),
        count: usize,
    ) -> PyResult<(usize, usize)> {
        self.wm
            .nmap_w(&BoundWrapper(buffer), cursor_pos, count, false)
    }
}
