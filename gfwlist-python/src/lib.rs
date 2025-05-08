use gfwlist::{BuildError, GfwList};
use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "gfwlist")]
fn pygfwlist(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGfwList>()?;

    // Register the custom exceptions
    m.add("GfwListSyntaxError", py.get_type::<GfwListSyntaxError>())?;
    m.add("GfwListBuildError", py.get_type::<GfwListBuildError>())?;
    m.add("GfwListUrlError", py.get_type::<GfwListUrlError>())?;

    // Add module docstring
    m.add("__doc__", "A GFW list parser and matcher.")?;

    Ok(())
}

// Custom Python exceptions
create_exception!(pygfwlist, GfwListSyntaxError, PyValueError);
create_exception!(pygfwlist, GfwListBuildError, PyRuntimeError);
create_exception!(pygfwlist, GfwListUrlError, PyValueError);

/// A Python wrapper around the Rust GfwList implementation
#[pyclass(name = "GfwList")]
struct PyGfwList {
    inner: GfwList,
}

#[pymethods]
impl PyGfwList {
    /// Create a new GfwList instance from rules text
    ///
    /// Args:
    ///     rules_text (str): The text content of the GFW list rules
    ///
    /// Returns:
    ///     GfwList: A new GfwList instance
    ///
    /// Raises:
    ///     GfwListSyntaxError: If there's a syntax error in the rules
    ///     GfwListBuildError: If there's an error building the pattern matching engine
    #[new]
    fn new(rules_text: &str) -> PyResult<Self> {
        match GfwList::from(rules_text) {
            Ok(gfw_list) => Ok(PyGfwList { inner: gfw_list }),
            Err(err) => match err {
                BuildError::Syntax(rule, _) => {
                    Err(GfwListSyntaxError::new_err(format!("Invalid rule syntax: {}", rule)))
                }
                BuildError::AhoCorasick(err) => Err(GfwListBuildError::new_err(format!(
                    "Failed to build pattern matcher: {}",
                    err
                ))),
            },
        }
    }

    /// Test if a URL matches any rule in the GfwList
    ///
    /// Args:
    ///     url (str): The URL to test
    ///
    /// Returns:
    ///     bool: True if the URL should be blocked, False otherwise
    ///
    /// Raises:
    ///     GfwListUrlError: If the URL is invalid or cannot be parsed
    fn test(&self, url: &str) -> PyResult<bool> {
        match self.inner.test(url) {
            Ok(result) => Ok(result),
            Err(err) => Err(GfwListUrlError::new_err(format!("Invalid URL: {}", err))),
        }
    }

    /// Return the string representation of the GfwList
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("GfwList(rules_count={})", self.inner.len()))
    }

    /// Return the string representation of the GfwList
    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    /// Return the number of rules in the GfwList
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }
}
