//! Python binding of poe2-core: values are the core's JSON shape as dicts (ADRs 0012, 0017).

use std::path::PathBuf;
use std::time::Duration;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// How long a blocked `next()` sleeps between polls (ADR 0014).
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Turns serialized core JSON into Python objects through `json.loads`, so Python sees the CLI's shape.
fn loads(py: Python<'_>, json: serde_json::Result<String>) -> PyResult<Bound<'_, PyAny>> {
    // PyO3 has no conversion from serde_json::Error.
    let json = json.map_err(|e| PyValueError::new_err(e.to_string()))?;
    py.import("json")?.call_method1("loads", (json,))
}

#[pymodule]
mod poe2_core {
    use super::*;

    /// Find Path of Exile 2's files on this machine.
    ///
    /// Returns a dict with the ranked candidate lists `installs`, `client_log`, `user_dir`,
    /// `build_planner` and `filters`; each candidate is a dict with `value`, `source`,
    /// `exists` and `note`.
    #[pyfunction]
    fn discover(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // The probes touch the filesystem and registry, so other Python threads run meanwhile.
        let found = py.detach(poe2::locate::discover);
        loads(py, serde_json::to_string(&found))
    }

    /// Parse one client log line.
    ///
    /// Returns a dict with `timestamp`, `prefix` and `event`, or `None` for a blank line.
    #[pyfunction]
    fn parse_line<'py>(py: Python<'py>, line: &str) -> PyResult<Option<Bound<'py, PyAny>>> {
        poe2::log::parse_line(line)
            .map(|l| loads(py, serde_json::to_string(&l)))
            .transpose()
    }

    /// Follow a client log, yielding each log line appended to it as a dict.
    ///
    /// `LogTail(path, from_start=False)` starts at the file's end, or at its start with
    /// `from_start=True`, and raises `OSError` if `path` can't be opened. `next_line()` never
    /// blocks; iterating blocks until a line arrives and never ends.
    #[pyclass]
    struct LogTail {
        inner: poe2::log::LogTail,
    }

    #[pymethods]
    impl LogTail {
        #[new]
        #[pyo3(signature = (path, from_start = false))]
        fn new(py: Python<'_>, path: PathBuf, from_start: bool) -> PyResult<Self> {
            let inner = py.detach(|| {
                if from_start {
                    poe2::log::LogTail::open_from_start(path)
                } else {
                    poe2::log::LogTail::open(path)
                }
            })?;
            Ok(LogTail { inner })
        }

        /// Return the next log line as a dict, or `None` once caught up; never blocks.
        fn next_line<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
            py.detach(|| self.inner.next_line())?
                .map(|l| loads(py, serde_json::to_string(&l)))
                .transpose()
        }

        fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
            slf
        }

        /// Block until a log line arrives, polling every 250 ms with the GIL released.
        fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
            loop {
                if let Some(line) = self.next_line(py)? {
                    return Ok(line);
                }
                // Raises KeyboardInterrupt on Ctrl-C.
                py.check_signals()?;
                py.detach(|| std::thread::sleep(POLL_INTERVAL));
            }
        }
    }
}
