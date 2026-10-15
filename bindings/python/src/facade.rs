//! Bindings for the `rustsheet` facade crate (SPEC-0001, "Python mapping").
//!
//! Values cross as `None | float | str | bool | XlError`; Excel error *values* are returned,
//! never raised. Edits run through a [`PyTransaction`] that buffers its ops and applies them as
//! one atomic `Workbook::edit` on clean exit, so an exception in the `with` body rolls back
//! trivially (nothing was applied).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, MutexGuard};

use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyInt, PyList};
use pyo3::{create_exception, IntoPyObjectExt};

use rustsheet::{
    resolve_sheet_by_name, ChangeSet, Clock, DateSystem, EditError, ErrorKind, Input, LoadOptions,
    RefError, SaveOptions, SheetChange, SheetId, Txn, Value, Workbook, WorkbookOptions, XlError,
};

create_exception!(_rustsheet, RustSheetError, PyException);
create_exception!(_rustsheet, PyRefError, RustSheetError);
create_exception!(_rustsheet, PyEditError, RustSheetError);
create_exception!(_rustsheet, PyLoadError, RustSheetError);
create_exception!(_rustsheet, PySaveError, RustSheetError);

fn ref_err(e: &RefError) -> PyErr {
    PyRefError::new_err(e.to_string())
}

fn edit_err(e: &EditError) -> PyErr {
    match e {
        EditError::Ref(r) => ref_err(r),
        other => PyEditError::new_err(other.to_string()),
    }
}

/// An Excel error value such as `#DIV/0!`. Equality is by kind (the detail is informational).
#[pyclass(name = "XlError", frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct PyXlError {
    inner: XlError,
}

#[pymethods]
impl PyXlError {
    /// Builds an error from its Excel literal, e.g. `XlError("#N/A")`.
    #[new]
    fn new(literal: &str) -> PyResult<Self> {
        let kind = ErrorKind::from_literal(literal)
            .ok_or_else(|| PyTypeError::new_err(format!("unknown error literal {literal:?}")))?;
        Ok(Self {
            inner: XlError::new(kind),
        })
    }

    /// The Excel literal, e.g. `#DIV/0!`.
    #[getter]
    fn literal(&self) -> &'static str {
        self.inner.kind.literal()
    }

    /// The `ERROR.TYPE` code, `None` for errors Excel has no code for.
    #[getter]
    fn code(&self) -> Option<u8> {
        self.inner.kind.code()
    }

    /// The engine's explanation, when it gave one.
    #[getter]
    fn detail(&self) -> Option<String> {
        self.inner.detail.as_deref().map(str::to_owned)
    }

    fn __str__(&self) -> &'static str {
        self.inner.kind.literal()
    }

    fn __repr__(&self) -> String {
        format!("XlError({:?})", self.inner.kind.literal())
    }

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        other
            .extract::<PyRef<'_, PyXlError>>()
            .is_ok_and(|o| o.inner.kind == self.inner.kind)
    }

    fn __hash__(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.inner.kind.hash(&mut h);
        h.finish()
    }
}

/// Marks a string as a formula (`Formula("=A1*2")`) instead of text parsed like Excel typing.
#[pyclass(name = "Formula", frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct PyFormula {
    #[pyo3(get)]
    text: String,
}

#[pymethods]
impl PyFormula {
    /// Wraps `text` (with its leading `=`) as a formula.
    #[new]
    fn new(text: String) -> Self {
        Self { text }
    }

    fn __repr__(&self) -> String {
        format!("Formula({:?})", self.text)
    }
}

fn value_to_py(py: Python<'_>, value: Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Empty => Ok(py.None()),
        Value::Number(n) => n.into_py_any(py),
        Value::Text(s) => (&*s).into_py_any(py),
        Value::Bool(b) => b.into_py_any(py),
        Value::Error(inner) => PyXlError { inner }.into_py_any(py),
        other => Err(RustSheetError::new_err(format!(
            "unsupported value kind: {other:?}"
        ))),
    }
}

/// `None` clears, `bool`/`int`/`float` store a number or boolean, `str` is parsed as if typed
/// into Excel, `Formula` forces a formula and `XlError` stores an error value.
fn py_to_input(obj: &Bound<'_, PyAny>) -> PyResult<Input> {
    if obj.is_none() {
        Ok(Input::Clear)
    } else if let Ok(b) = obj.cast::<PyBool>() {
        Ok(Input::from(b.is_true()))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Input::Parse(s))
    } else if let Ok(n) = obj.extract::<f64>() {
        Ok(Input::from(n))
    } else if let Ok(f) = obj.extract::<PyRef<'_, PyFormula>>() {
        Ok(Input::Formula(f.text.clone()))
    } else if let Ok(e) = obj.extract::<PyRef<'_, PyXlError>>() {
        Ok(Input::Value(Value::Error(e.inner.clone())))
    } else {
        Err(PyTypeError::new_err(
            "expected None, bool, int, float, str, Formula or XlError",
        ))
    }
}

fn changes_to_py<'py>(
    py: Python<'py>,
    mut names: Vec<(SheetId, String)>,
    changes: &ChangeSet,
) -> PyResult<Bound<'py, PyDict>> {
    for s in &changes.sheets {
        if let SheetChange::Deleted { id, name } = s {
            names.push((*id, name.clone()));
        }
    }
    let a1 = |cell: rustsheet::CellRef| match name_of(&names, cell.sheet) {
        Some(name) => cell.to_a1_in(name),
        None => format!("#{}!{cell}", cell.sheet.get()),
    };

    let inputs = PyList::empty(py);
    for c in &changes.inputs {
        let d = PyDict::new(py);
        d.set_item("ref", a1(c.cell))?;
        d.set_item("before", &c.before)?;
        d.set_item("after", &c.after)?;
        inputs.append(d)?;
    }
    let values = PyList::empty(py);
    for c in &changes.values {
        let d = PyDict::new(py);
        d.set_item("ref", a1(c.cell))?;
        d.set_item("before", value_to_py(py, c.before.clone())?)?;
        d.set_item("after", value_to_py(py, c.after.clone())?)?;
        values.append(d)?;
    }
    let sheets = PyList::empty(py);
    for s in &changes.sheets {
        let d = PyDict::new(py);
        match s {
            SheetChange::Added(id) => {
                d.set_item("kind", "added")?;
                d.set_item("name", name_of(&names, *id))?;
            }
            SheetChange::Renamed { from, to, .. } => {
                d.set_item("kind", "renamed")?;
                d.set_item("from", from)?;
                d.set_item("to", to)?;
            }
            SheetChange::Deleted { name, .. } => {
                d.set_item("kind", "deleted")?;
                d.set_item("name", name)?;
            }
            _ => continue,
        }
        sheets.append(d)?;
    }
    let out = PyDict::new(py);
    out.set_item("inputs", inputs)?;
    out.set_item("values", values)?;
    out.set_item("sheets", sheets)?;
    Ok(out)
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A poisoned lock only means an earlier call panicked; the guarded state is still usable
    // (edits roll back on failure, `last_changes` is replaced wholesale).
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Converts the Python `clock` argument: `None` is the system clock, an `int` is a fixed instant
/// in ms since the Unix epoch (SPEC-0008). A `bool` or any other type is a `TypeError`; an
/// instant outside Excel's calendar is a `ValueError`.
fn py_to_clock(obj: &Bound<'_, PyAny>) -> PyResult<Clock> {
    if obj.is_none() {
        return Ok(Clock::system());
    }
    if obj.is_instance_of::<PyBool>() || !obj.is_instance_of::<PyInt>() {
        return Err(PyTypeError::new_err(
            "clock must be an int (ms since the Unix epoch) or None",
        ));
    }
    let ms = obj.extract::<i64>().map_err(|_| {
        PyValueError::new_err("clock instant is outside 1900-01-01..=9999-12-31 UTC")
    })?;
    Clock::fixed_unix_millis(ms).map_err(|e| PyValueError::new_err(e.to_string()))
}

fn load_options(recalc: bool, clock: Clock) -> LoadOptions {
    LoadOptions {
        recalc,
        workbook: WorkbookOptions {
            clock,
            ..WorkbookOptions::default()
        },
        ..LoadOptions::default()
    }
}

/// A workbook on the typed facade API.
#[pyclass(name = "Workbook")]
pub struct PyWorkbook {
    wb: Mutex<Workbook>,
    last_changes: Mutex<Option<Py<PyDict>>>,
}

impl PyWorkbook {
    fn wrap(wb: Workbook) -> Self {
        Self {
            wb: Mutex::new(wb),
            last_changes: Mutex::new(None),
        }
    }

    /// Runs `f` on the workbook. The mutex is taken with the GIL released: a thread that holds
    /// the mutex inside `detach` needs the GIL back to finish, so blocking on the mutex while
    /// holding the GIL would deadlock.
    fn read<T: Send>(&self, py: Python<'_>, f: impl FnOnce(&Workbook) -> T + Send) -> T {
        py.detach(|| f(&lock(&self.wb)))
    }

    /// Applies `ops` as one atomic edit with the GIL released and retains the change set.
    fn run_edit(&self, py: Python<'_>, ops: Vec<Op>) -> PyResult<()> {
        let (changes, names) = py
            .detach(|| {
                let mut wb = lock(&self.wb);
                let sheets = sheet_names(&wb);
                let ((), changes) = wb.edit(|txn| apply(txn, ops, sheets))?;
                Ok((changes, sheet_names(&wb)))
            })
            .map_err(|e: EditError| edit_err(&e))?;
        let d = changes_to_py(py, names, &changes)?;
        *lock(&self.last_changes) = Some(d.unbind());
        Ok(())
    }
}

/// The name of `id` in a `(id, name)` list.
fn name_of(names: &[(SheetId, String)], id: SheetId) -> Option<&String> {
    names.iter().find(|(i, _)| *i == id).map(|(_, n)| n)
}

fn sheet_names(wb: &Workbook) -> Vec<(SheetId, String)> {
    wb.sheets().into_iter().map(|s| (s.id, s.name)).collect()
}

#[pymethods]
impl PyWorkbook {
    /// Creates an empty workbook with one sheet; `date_system` is `"1900"` (default) or `"1904"`.
    #[new]
    #[pyo3(signature = (date_system="1900", clock=None))]
    fn py_new(date_system: &str, clock: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let clock = clock.map_or(Ok(Clock::system()), py_to_clock)?;
        let date_system = match date_system {
            "1900" => DateSystem::Excel1900,
            "1904" => DateSystem::Excel1904,
            other => {
                return Err(PyTypeError::new_err(format!(
                    "date_system must be \"1900\" or \"1904\", not {other:?}"
                )))
            }
        };
        let wb = Workbook::new(&WorkbookOptions {
            date_system,
            clock,
            ..WorkbookOptions::default()
        })
        .map_err(|e| PyLoadError::new_err(e.to_string()))?;
        Ok(Self::wrap(wb))
    }

    /// The workbook's date system, `"1900"` or `"1904"`.
    #[getter]
    fn date_system(&self) -> &'static str {
        match lock(&self.wb).date_system() {
            DateSystem::Excel1900 => "1900",
            DateSystem::Excel1904 => "1904",
        }
    }

    /// The fixed clock in ms since the Unix epoch (UTC), `None` for the system clock. Setting it
    /// is one edit: NOW, TODAY and year-less date text recalculate and `changes` reports them.
    #[getter]
    fn clock(&self) -> Option<i64> {
        lock(&self.wb).clock().fixed()
    }

    #[setter]
    fn set_clock(&self, py: Python<'_>, clock: &Bound<'_, PyAny>) -> PyResult<()> {
        let clock = py_to_clock(clock)?;
        self.run_edit(py, vec![Op::Clock(clock)])
    }

    /// Opens an xlsx file (the GIL is released while loading).
    #[staticmethod]
    #[pyo3(signature = (path, recalc=true, clock=None))]
    fn open(
        py: Python<'_>,
        path: &str,
        recalc: bool,
        clock: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let options = load_options(recalc, clock.map_or(Ok(Clock::system()), py_to_clock)?);
        let wb = py
            .detach(|| Workbook::open(path, &options))
            .map_err(|e| PyLoadError::new_err(e.to_string()))?;
        Ok(Self::wrap(wb))
    }

    /// Loads a workbook from xlsx bytes (the GIL is released while loading).
    #[staticmethod]
    #[pyo3(signature = (data, recalc=true, clock=None))]
    fn from_bytes(
        py: Python<'_>,
        data: &[u8],
        recalc: bool,
        clock: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let options = load_options(recalc, clock.map_or(Ok(Clock::system()), py_to_clock)?);
        let wb = py
            .detach(|| Workbook::from_xlsx_bytes(data, &options))
            .map_err(|e| PyLoadError::new_err(e.to_string()))?;
        Ok(Self::wrap(wb))
    }

    /// Saves to an xlsx file (the GIL is released while saving).
    fn save_xlsx(&self, py: Python<'_>, path: &str) -> PyResult<()> {
        py.detach(|| lock(&self.wb).save_xlsx(path, SaveOptions::default()))
            .map_err(|e| PySaveError::new_err(e.to_string()))
    }

    /// Returns the workbook as xlsx bytes.
    fn to_xlsx_bytes(&self, py: Python<'_>) -> PyResult<Vec<u8>> {
        py.detach(|| lock(&self.wb).to_xlsx_bytes(SaveOptions::default()))
            .map_err(|e| PySaveError::new_err(e.to_string()))
    }

    /// Recalculates every formula (the GIL is released).
    fn recalculate(&self, py: Python<'_>) {
        py.detach(|| lock(&self.wb).recalculate());
    }

    /// Sheet names in tab order.
    #[getter]
    fn sheet_names(&self, py: Python<'_>) -> Vec<String> {
        self.read(py, |wb| {
            sheet_names(wb).into_iter().map(|(_, n)| n).collect()
        })
    }

    /// The value of a cell (the top-left cell of a range), e.g. `wb.value("Sheet1!A1")`.
    fn value(&self, py: Python<'_>, r: &str) -> PyResult<Py<PyAny>> {
        let v = self.read(py, |wb| wb.value(r)).map_err(|e| ref_err(&e))?;
        value_to_py(py, v)
    }

    fn __getitem__(&self, py: Python<'_>, r: &str) -> PyResult<Py<PyAny>> {
        self.value(py, r)
    }

    /// The values of a range as a list of rows.
    fn values<'py>(&self, py: Python<'py>, r: &str) -> PyResult<Bound<'py, PyList>> {
        let arr = self.read(py, |wb| wb.values(r)).map_err(|e| ref_err(&e))?;
        let rows = PyList::empty(py);
        for row in 0..arr.rows() {
            let out = PyList::empty(py);
            for col in 0..arr.cols() {
                let v = arr.get(row, col).cloned().unwrap_or(Value::Empty);
                out.append(value_to_py(py, v)?)?;
            }
            rows.append(out)?;
        }
        Ok(rows)
    }

    /// The formula of a cell (with leading `=`), or `None`.
    fn formula(&self, py: Python<'_>, r: &str) -> PyResult<Option<String>> {
        Ok(self
            .read(py, |wb| wb.cell(r))
            .map_err(|e| ref_err(&e))?
            .formula)
    }

    /// The number format code of a cell, `None` for General.
    fn number_format(&self, py: Python<'_>, r: &str) -> PyResult<Option<String>> {
        Ok(self
            .read(py, |wb| wb.cell(r))
            .map_err(|e| ref_err(&e))?
            .number_format)
    }

    /// The cell as Excel displays it.
    fn formatted(&self, py: Python<'_>, r: &str) -> PyResult<String> {
        self.read(py, |wb| wb.formatted(r)).map_err(|e| ref_err(&e))
    }

    /// The used range of a sheet as an A1 string (with sheet name), `None` if empty.
    fn used_range(&self, py: Python<'_>, sheet: &str) -> PyResult<Option<String>> {
        self.read(py, |wb| {
            let id = wb.sheet_id(sheet)?;
            let range = wb.used_range(id)?;
            let name = name_of(&sheet_names(wb), id).cloned().unwrap_or_default();
            Ok(range.map(|r| r.to_a1_in(&name)))
        })
        .map_err(|e: RefError| ref_err(&e))
    }

    /// Turns the per-edit value diff on or off (off: `changes["values"]` stays empty).
    fn set_track_changes(&self, py: Python<'_>, on: bool) {
        py.detach(|| lock(&self.wb).set_track_changes(on));
    }

    /// Stores one cell as a one-op transaction; see [`PyTransaction`] for accepted inputs.
    fn set(&self, py: Python<'_>, r: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let input = py_to_input(value)?;
        self.run_edit(py, vec![Op::Set(r.to_owned(), input)])
    }

    /// Starts a transaction: `with wb.edit() as tx: tx["A1"] = 1`. Commits on clean exit,
    /// rolls back (applies nothing) when the body raises.
    fn edit(slf: Py<Self>) -> PyTransaction {
        PyTransaction {
            wb: slf,
            ops: Vec::new(),
            done: false,
        }
    }

    /// The change set of the last edit as `{"inputs": [...], "values": [...], "sheets": [...]}`;
    /// retained until the next edit (the Rust `Workbook` stores none), `None` before any edit.
    #[getter]
    fn changes(&self, py: Python<'_>) -> Option<Py<PyDict>> {
        lock(&self.last_changes).as_ref().map(|d| d.clone_ref(py))
    }
}

enum Op {
    Set(String, Input),
    AddSheet(String),
    RenameSheet(String, String),
    DeleteSheet(String),
    Clock(Clock),
}

/// Buffers edits and applies them as one atomic `Workbook::edit` when the `with` block exits
/// cleanly. Like the Rust `Txn` it is write-only (no reads inside the block). Sheet ops address
/// sheets by name, and later ops may use a sheet added earlier in the same block.
#[pyclass(name = "Transaction")]
pub struct PyTransaction {
    wb: Py<PyWorkbook>,
    ops: Vec<Op>,
    done: bool,
}

impl PyTransaction {
    fn push(&mut self, op: Op) -> PyResult<()> {
        if self.done {
            return Err(PyEditError::new_err("transaction already finished"));
        }
        self.ops.push(op);
        Ok(())
    }
}

/// Position of `name` in `sheets`, resolved with the facade's own case-insensitive rule. The
/// facade's `Workbook::sheet_id` cannot be used because the transaction's ops are not applied
/// yet, so sheets added or renamed earlier in the same block exist only in this list.
fn find_sheet(sheets: &[(SheetId, String)], name: &str) -> Result<usize, EditError> {
    let pairs: Vec<(SheetId, &str)> = sheets.iter().map(|(id, n)| (*id, n.as_str())).collect();
    resolve_sheet_by_name(&pairs, Some(name))
        .and_then(|id| sheets.iter().position(|(i, _)| *i == id))
        .ok_or_else(|| EditError::Ref(RefError::UnknownSheet(name.to_owned())))
}

fn apply(
    txn: &mut Txn<'_>,
    ops: Vec<Op>,
    mut sheets: Vec<(SheetId, String)>,
) -> Result<(), EditError> {
    for op in ops {
        match op {
            Op::Set(r, input) => txn.set(r.as_str(), input)?,
            Op::Clock(clock) => txn.set_clock(clock)?,
            Op::AddSheet(name) => {
                let id = txn.add_sheet(&name)?;
                sheets.push((id, name));
            }
            Op::RenameSheet(from, to) => {
                let i = find_sheet(&sheets, &from)?;
                txn.rename_sheet(sheets[i].0, &to)?;
                sheets[i].1 = to;
            }
            Op::DeleteSheet(name) => {
                let i = find_sheet(&sheets, &name)?;
                txn.delete_sheet(sheets[i].0)?;
                sheets.remove(i);
            }
        }
    }
    Ok(())
}

#[pymethods]
impl PyTransaction {
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __exit__(
        &mut self,
        py: Python<'_>,
        exc_type: Option<&Bound<'_, PyAny>>,
        _exc: Option<&Bound<'_, PyAny>>,
        _tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        if self.done {
            return Ok(false);
        }
        self.done = true;
        let ops = std::mem::take(&mut self.ops);
        if exc_type.is_some() {
            return Ok(false);
        }
        self.wb.bind(py).borrow().run_edit(py, ops)?;
        Ok(false)
    }

    /// Stores a cell: `None` clears, numbers/bools are stored, `str` is parsed like Excel
    /// typing (`"12%"`, `"=A1*2"`, `"'text"`), `Formula` forces a formula.
    fn set(&mut self, r: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        let input = py_to_input(value)?;
        self.push(Op::Set(r.to_owned(), input))
    }

    fn __setitem__(&mut self, r: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.set(r, value)
    }

    /// Clears a cell.
    fn clear(&mut self, r: &str) -> PyResult<()> {
        self.push(Op::Set(r.to_owned(), Input::Clear))
    }

    /// Adds a sheet at the end.
    fn add_sheet(&mut self, name: &str) -> PyResult<()> {
        self.push(Op::AddSheet(name.to_owned()))
    }

    /// Renames a sheet.
    fn rename_sheet(&mut self, name: &str, new_name: &str) -> PyResult<()> {
        self.push(Op::RenameSheet(name.to_owned(), new_name.to_owned()))
    }

    /// Deletes a sheet.
    fn delete_sheet(&mut self, name: &str) -> PyResult<()> {
        self.push(Op::DeleteSheet(name.to_owned()))
    }
}

/// Registers the facade classes and exceptions on the extension module.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add_class::<PyWorkbook>()?;
    m.add_class::<PyTransaction>()?;
    m.add_class::<PyXlError>()?;
    m.add_class::<PyFormula>()?;
    m.add("RustSheetError", py.get_type::<RustSheetError>())?;
    m.add("RefError", py.get_type::<PyRefError>())?;
    m.add("EditError", py.get_type::<PyEditError>())?;
    m.add("LoadError", py.get_type::<PyLoadError>())?;
    m.add("SaveError", py.get_type::<PySaveError>())?;
    Ok(())
}
