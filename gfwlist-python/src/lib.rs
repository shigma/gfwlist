use gfwlist::{BuildError, GfwList};
use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "gfwlist")]
fn pygfwlist(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGfwList>()?;

    m.add("GfwListSyntaxError", py.get_type::<GfwListSyntaxError>())?;
    m.add("GfwListBuildError", py.get_type::<GfwListBuildError>())?;
    m.add("GfwListUrlError", py.get_type::<GfwListUrlError>())?;

    m.add("__doc__", "A GFW list parser and matcher.")?;

    Ok(())
}

create_exception!(pygfwlist, GfwListSyntaxError, PyValueError);
create_exception!(pygfwlist, GfwListBuildError, PyRuntimeError);
create_exception!(pygfwlist, GfwListUrlError, PyValueError);

#[pyclass(name = "GfwList")]
struct PyGfwList {
    inner: GfwList,
}

#[pymethods]
impl PyGfwList {
    #[new]
    fn new(rules_text: &str) -> PyResult<Self> {
        match GfwList::from(rules_text) {
            Ok(gfw_list) => Ok(PyGfwList { inner: gfw_list }),
            Err(err) => match err {
                BuildError::Syntax(rule, _) => Err(GfwListSyntaxError::new_err(format!("Invalid rule syntax: {rule}"))),
                BuildError::AhoCorasick(err) => Err(GfwListBuildError::new_err(format!(
                    "Failed to build pattern matcher: {err}",
                ))),
            },
        }
    }

    fn test(&self, url: &str) -> PyResult<Option<&str>> {
        match self.inner.test(url) {
            Ok(result) => Ok(result),
            Err(err) => Err(GfwListUrlError::new_err(format!("Invalid URL: {err}"))),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("GfwList(rules_count={})", self.inner.len()))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }
}
