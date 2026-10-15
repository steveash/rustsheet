use std::sync::Arc;

use rustsheet_engine::expressions::token::Error as EngineError;
use rustsheet_engine::types::{Cell, FormulaValue, SpillValue};
use rustsheet_engine::Text;

/// The kind of an Excel error value (`#DIV/0!`, `#N/A`, ...).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// `#NULL!`
    Null,
    /// `#DIV/0!`
    Div0,
    /// `#VALUE!`
    Value,
    /// `#REF!`
    Ref,
    /// `#NAME?`
    Name,
    /// `#NUM!`
    Num,
    /// `#N/A`
    NA,
    /// `#SPILL!`
    Spill,
    /// `#CALC!`
    Calc,
    /// `#CIRC!` (engine-specific)
    Circ,
    /// `#N/IMPL!` (engine-specific: feature not implemented)
    NotImplemented,
    /// `#ERROR!` (engine-specific: the formula could not be processed). Not Excel's `#UNKNOWN!`.
    Unknown,
    /// `#GETTING_DATA` (saved to xlsx as `#N/A`, as Excel does)
    GettingData,
    /// `#CONNECT!`
    Connect,
    /// `#BLOCKED!`
    Blocked,
    /// `#FIELD!`
    Field,
    /// `#BUSY!`: only a stored constant; its ERROR.TYPE code is 0 (SPEC-0003, "Stored `#BUSY!`")
    Busy,
}

impl ErrorKind {
    /// The kind for an engine error. Exhaustive on purpose: a new engine error must be mapped.
    pub(crate) fn from_engine(error: &EngineError) -> Self {
        match error {
            EngineError::NULL => Self::Null,
            EngineError::DIV => Self::Div0,
            EngineError::VALUE => Self::Value,
            EngineError::REF => Self::Ref,
            EngineError::NAME => Self::Name,
            EngineError::NUM => Self::Num,
            EngineError::NA => Self::NA,
            EngineError::SPILL => Self::Spill,
            EngineError::CALC => Self::Calc,
            EngineError::CIRC => Self::Circ,
            EngineError::NIMPL => Self::NotImplemented,
            EngineError::ERROR => Self::Unknown,
            EngineError::GETTINGDATA => Self::GettingData,
            EngineError::CONNECT => Self::Connect,
            EngineError::BLOCKED => Self::Blocked,
            EngineError::FIELD => Self::Field,
            EngineError::BUSY => Self::Busy,
        }
    }

    /// The engine error for this kind (inverse of [`ErrorKind::from_engine`]).
    pub(crate) fn to_engine(self) -> EngineError {
        match self {
            Self::Null => EngineError::NULL,
            Self::Div0 => EngineError::DIV,
            Self::Value => EngineError::VALUE,
            Self::Ref => EngineError::REF,
            Self::Name => EngineError::NAME,
            Self::Num => EngineError::NUM,
            Self::NA => EngineError::NA,
            Self::Spill => EngineError::SPILL,
            Self::Calc => EngineError::CALC,
            Self::Circ => EngineError::CIRC,
            Self::NotImplemented => EngineError::NIMPL,
            Self::Unknown => EngineError::ERROR,
            Self::GettingData => EngineError::GETTINGDATA,
            Self::Connect => EngineError::CONNECT,
            Self::Blocked => EngineError::BLOCKED,
            Self::Field => EngineError::FIELD,
            Self::Busy => EngineError::BUSY,
        }
    }

    /// The en-US literal as stored in xlsx files and accepted in formulas.
    ///
    /// A static mirror of the engine's literal table; `tests::literals_match_engine` keeps the
    /// two in sync.
    #[must_use]
    pub fn literal(self) -> &'static str {
        match self {
            Self::Null => "#NULL!",
            Self::Div0 => "#DIV/0!",
            Self::Value => "#VALUE!",
            Self::Ref => "#REF!",
            Self::Name => "#NAME?",
            Self::Num => "#NUM!",
            Self::NA => "#N/A",
            Self::Spill => "#SPILL!",
            Self::Calc => "#CALC!",
            Self::Circ => "#CIRC!",
            Self::NotImplemented => "#N/IMPL!",
            Self::Unknown => "#ERROR!",
            Self::GettingData => "#GETTING_DATA",
            Self::Connect => "#CONNECT!",
            Self::Blocked => "#BLOCKED!",
            Self::Field => "#FIELD!",
            Self::Busy => "#BUSY!",
        }
    }

    /// The literal Excel stores for this error in a workbook cell: `#N/A` for
    /// [`ErrorKind::GettingData`], [`ErrorKind::literal`] otherwise (SPEC-0003, "Stored form").
    /// `tests::stored_literals_match_engine` keeps this in step with the engine's
    /// `Error::stored_literal`.
    #[must_use]
    pub fn stored_literal(self) -> &'static str {
        match self {
            Self::GettingData => Self::NA.literal(),
            _ => self.literal(),
        }
    }

    /// Parses an en-US error literal, case-insensitively (not oracle-verified for Excel).
    ///
    /// Uses the engine's stored-name table, so it accepts every kind, including `#BLOCKED!`,
    /// `#CONNECT!`, `#FIELD!` and `#BUSY!`, which are not formula literals (SPEC-0003).
    #[must_use]
    pub fn from_literal(literal: &str) -> Option<Self> {
        rustsheet_engine::expressions::token::get_error_by_english_name(&literal.to_uppercase())
            .as_ref()
            .map(Self::from_engine)
    }

    /// The number `ERROR.TYPE` returns for this error.
    ///
    /// Always `Some` today; `Option` leaves room for kinds without a code. [`ErrorKind::Busy`]
    /// has one, and it is `Some(0)`.
    #[must_use]
    pub fn code(self) -> Option<u8> {
        Some(self.to_engine().error_type_code())
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.literal())
    }
}

/// An Excel error *value* (a cell result), never a Rust `Err`.
#[derive(Clone, Debug, PartialEq)]
pub struct XlError {
    /// Which error.
    pub kind: ErrorKind,
    /// The engine's explanation (`FormulaValue::Error.message`), when it gave one.
    pub detail: Option<Arc<str>>,
}

impl XlError {
    /// An error with no detail message.
    #[must_use]
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, detail: None }
    }

    /// An error with the engine's detail message.
    #[must_use]
    pub fn with_detail(kind: ErrorKind, detail: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            detail: Some(detail.into()),
        }
    }
}

impl From<ErrorKind> for XlError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl std::fmt::Display for XlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

/// A cell's value as Excel would show it.
///
/// `PartialEq` is the derived one, so `Number` compares as `f64` (`NaN != NaN`, `0.0 == -0.0`).
/// Bitwise comparison for change detection is `ChangeSet`'s job (T-0103), not this type's.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    /// A blank cell.
    Empty,
    /// A number (dates are serial numbers).
    Number(f64),
    /// Text.
    Text(Arc<str>),
    /// A boolean.
    Bool(bool),
    /// An error value.
    Error(XlError),
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::Text(s.into())
    }
}

impl From<XlError> for Value {
    fn from(e: XlError) -> Self {
        Self::Error(e)
    }
}

impl Value {
    /// The value of an engine cell. `shared_strings` is the workbook's shared-string table.
    ///
    /// See [`CellView::of`] for how engine cells map to values.
    pub(crate) fn from_cell(cell: &Cell, shared_strings: &[String]) -> Self {
        CellView::of(cell, shared_strings).to_value()
    }
}

/// The engine text of a formula or spill cell that holds a lone surrogate, if it does. Shared
/// strings are `String`s and cannot hold one.
pub(crate) fn lone_text(cell: &Cell) -> Option<&Text> {
    let (Cell::CellFormula {
        v: FormulaValue::Text(text),
        ..
    }
    | Cell::ArrayFormula {
        v: FormulaValue::Text(text),
        ..
    }
    | Cell::SpillCell {
        v: SpillValue::Text(text),
        ..
    }) = cell
    else {
        return None;
    };
    text.has_lone_surrogate().then_some(text)
}

/// A cell's value borrowed from the engine, so unchanged cells can be compared without
/// building an owned [`Value`] (no `Arc<str>` per text cell).
#[derive(Copy, Clone, Debug)]
pub(crate) enum CellView<'a> {
    Empty,
    Number(f64),
    Bool(bool),
    Text(&'a str),
    Error(ErrorKind, Option<&'a str>),
}

impl<'a> CellView<'a> {
    /// The view of an engine cell.
    ///
    /// Exhaustive on purpose (no wildcard arms): a new engine `Cell`, `FormulaValue` or
    /// `SpillValue` variant must be mapped here. A shared-string index outside the table reads as
    /// `Empty`, as the eval adapter does. A formula that is still `Unevaluated` reads as the
    /// engine's own `#ERROR!`, matching `Model::get_cell_value_by_index`.
    pub(crate) fn of(cell: &'a Cell, shared_strings: &'a [String]) -> Self {
        match cell {
            Cell::EmptyCell { .. } => Self::Empty,
            Cell::BooleanCell { v, .. } => Self::Bool(*v),
            Cell::NumberCell { v, .. } => Self::Number(*v),
            Cell::ErrorCell { ei, .. } => Self::Error(ErrorKind::from_engine(ei), None),
            Cell::SharedString { si, .. } => usize::try_from(*si)
                .ok()
                .and_then(|i| shared_strings.get(i))
                .map_or(Self::Empty, |s| Self::Text(s.as_str())),
            Cell::CellFormula { v, .. } | Cell::ArrayFormula { v, .. } => Self::of_formula(v),
            Cell::SpillCell { v, .. } => match v {
                SpillValue::Boolean(b) => Self::Bool(*b),
                SpillValue::Number(n) => Self::Number(*n),
                SpillValue::Text(s) => Self::Text(s),
                SpillValue::Error(ei) => Self::Error(ErrorKind::from_engine(ei), None),
            },
        }
    }

    fn of_formula(value: &'a FormulaValue) -> Self {
        match value {
            // Message mirrors rustsheet-engine `model.rs` ("Unevaluated formula").
            FormulaValue::Unevaluated => {
                Self::Error(ErrorKind::Unknown, Some("Unevaluated formula"))
            }
            FormulaValue::Boolean(b) => Self::Bool(*b),
            FormulaValue::Number(n) => Self::Number(*n),
            FormulaValue::Text(s) => Self::Text(s),
            FormulaValue::Error { ei, m, .. } => Self::Error(
                ErrorKind::from_engine(ei),
                (!m.is_empty()).then_some(m.as_str()),
            ),
        }
    }

    /// The owned value.
    pub(crate) fn to_value(self) -> Value {
        match self {
            Self::Empty => Value::Empty,
            Self::Number(n) => Value::Number(n),
            Self::Bool(b) => Value::Bool(b),
            Self::Text(s) => Value::Text(s.into()),
            Self::Error(kind, detail) => Value::Error(XlError {
                kind,
                detail: detail.map(Into::into),
            }),
        }
    }
}

/// The text is not an en-US Excel error literal (see [`ErrorKind::from_literal`]).
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("`{0}` is not an Excel error literal")]
pub struct ParseErrorKindError(pub String);

impl std::str::FromStr for ErrorKind {
    type Err = ParseErrorKindError;

    /// Parses an error literal exactly as [`ErrorKind::from_literal`] does.
    fn from_str(literal: &str) -> Result<Self, Self::Err> {
        Self::from_literal(literal).ok_or_else(|| ParseErrorKindError(literal.to_owned()))
    }
}

/// `rows * cols` does not fit in memory-addressable size, or does not match the value count.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArrayShapeError {
    /// `rows * cols` overflows `usize`.
    #[error("array dimensions {rows} x {cols} overflow")]
    Overflow {
        /// Requested rows.
        rows: usize,
        /// Requested columns.
        cols: usize,
    },
    /// The number of values is not `rows * cols`.
    #[error("array of {rows} x {cols} needs {expected} values, got {actual}")]
    LengthMismatch {
        /// Requested rows.
        rows: usize,
        /// Requested columns.
        cols: usize,
        /// `rows * cols`.
        expected: usize,
        /// Values supplied.
        actual: usize,
    },
}

/// A dense, row-major rectangle of values (the result of reading a range).
#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    rows: usize,
    cols: usize,
    values: Vec<Value>,
}

impl Array {
    /// Builds an array from row-major `values`.
    ///
    /// # Errors
    /// [`ArrayShapeError::Overflow`] if `rows * cols` overflows (checked before anything is
    /// allocated or compared), [`ArrayShapeError::LengthMismatch`] if `values.len()` differs.
    pub fn new(rows: usize, cols: usize, values: Vec<Value>) -> Result<Self, ArrayShapeError> {
        let expected = rows
            .checked_mul(cols)
            .ok_or(ArrayShapeError::Overflow { rows, cols })?;
        if values.len() != expected {
            return Err(ArrayShapeError::LengthMismatch {
                rows,
                cols,
                expected,
                actual: values.len(),
            });
        }
        Ok(Self { rows, cols, values })
    }

    /// Number of rows.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// The value at zero-based (`row`, `col`), or `None` outside the array.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<&Value> {
        if row < self.rows && col < self.cols {
            // `rows * cols` was checked in `new`, so this index cannot overflow.
            self.values.get(row * self.cols + col)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ErrorKind; 17] = [
        ErrorKind::Null,
        ErrorKind::Div0,
        ErrorKind::Value,
        ErrorKind::Ref,
        ErrorKind::Name,
        ErrorKind::Num,
        ErrorKind::NA,
        ErrorKind::Spill,
        ErrorKind::Calc,
        ErrorKind::Circ,
        ErrorKind::NotImplemented,
        ErrorKind::Unknown,
        ErrorKind::GettingData,
        ErrorKind::Connect,
        ErrorKind::Blocked,
        ErrorKind::Field,
        ErrorKind::Busy,
    ];

    #[test]
    fn every_kind_round_trips_through_engine_and_literal() {
        for kind in ALL {
            assert_eq!(ErrorKind::from_engine(&kind.to_engine()), kind);
            assert_eq!(ErrorKind::from_literal(kind.literal()), Some(kind));
            assert_eq!(
                ErrorKind::from_literal(&kind.literal().to_lowercase()),
                Some(kind)
            );
        }
        assert_eq!(ErrorKind::from_literal("#BOGUS!"), None);
    }

    #[test]
    fn stored_literals_match_engine() {
        assert_eq!(ErrorKind::GettingData.stored_literal(), "#N/A");
        for kind in ALL {
            // The engine prints `#N/IMPL` without the `!` its parser requires.
            assert_eq!(
                kind.stored_literal().trim_end_matches('!'),
                kind.to_engine().stored_literal().trim_end_matches('!'),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn from_str_delegates_to_from_literal() {
        for kind in ALL {
            assert_eq!(kind.literal().parse::<ErrorKind>(), Ok(kind));
            assert_eq!(kind.literal().to_lowercase().parse::<ErrorKind>(), Ok(kind));
        }
        let err = "#BOGUS!".parse::<ErrorKind>().unwrap_err();
        assert!(err.to_string().contains("#BOGUS!"));
    }

    #[test]
    fn literals_match_engine() {
        for kind in ALL {
            let engine = kind.to_engine().to_string();
            // The engine prints `#N/IMPL` without the `!` its parser requires.
            assert_eq!(
                kind.literal().trim_end_matches('!'),
                engine.trim_end_matches('!'),
                "{kind:?}"
            );
            assert_eq!(
                ErrorKind::from_literal(kind.literal()),
                Some(kind),
                "{kind:?}"
            );
        }
    }

    #[test]
    fn codes_follow_error_type() {
        // Excel's ERROR.TYPE numbers; 101/102/104 are engine-specific.
        let expected = [1, 2, 3, 4, 5, 6, 7, 9, 14, 104, 102, 101, 8, 10, 11, 13, 0];
        for (kind, code) in ALL.into_iter().zip(expected) {
            assert_eq!(kind.code(), Some(code), "{kind:?}");
        }
    }

    #[test]
    fn xl_error_displays_literal() {
        let e = XlError::with_detail(ErrorKind::Div0, "Divide by zero");
        assert_eq!(e.to_string(), "#DIV/0!");
        assert_eq!(XlError::from(ErrorKind::NA), XlError::new(ErrorKind::NA));
    }

    fn from_cell(cell: &Cell) -> Value {
        Value::from_cell(cell, &["shared".to_string()])
    }

    #[test]
    fn every_engine_cell_variant_converts() {
        let s = 0;
        let formula = |v| Cell::CellFormula { f: 0, s, v };
        let array = |v| Cell::ArrayFormula {
            f: 0,
            s,
            r: (1, 1),
            kind: rustsheet_engine::types::ArrayKind::Dynamic,
            v,
        };
        let cases = [
            (Cell::EmptyCell { s }, Value::Empty),
            (Cell::BooleanCell { v: true, s }, Value::Bool(true)),
            (Cell::NumberCell { v: 1.5, s }, Value::Number(1.5)),
            (
                Cell::ErrorCell {
                    ei: EngineError::NA,
                    s,
                },
                Value::Error(XlError::new(ErrorKind::NA)),
            ),
            (Cell::SharedString { si: 0, s }, Value::from("shared")),
            (Cell::SharedString { si: 7, s }, Value::Empty),
            (Cell::SharedString { si: -1, s }, Value::Empty),
            (formula(FormulaValue::Boolean(false)), Value::Bool(false)),
            (formula(FormulaValue::Number(2.0)), Value::Number(2.0)),
            (formula(FormulaValue::Text("t".into())), Value::from("t")),
            (
                formula(FormulaValue::Unevaluated),
                Value::Error(XlError::with_detail(
                    ErrorKind::Unknown,
                    "Unevaluated formula",
                )),
            ),
            (array(FormulaValue::Number(3.0)), Value::Number(3.0)),
            (
                Cell::SpillCell {
                    s,
                    a: (1, 1),
                    v: SpillValue::Boolean(true),
                },
                Value::Bool(true),
            ),
            (
                Cell::SpillCell {
                    s,
                    a: (1, 1),
                    v: SpillValue::Number(4.0),
                },
                Value::Number(4.0),
            ),
            (
                Cell::SpillCell {
                    s,
                    a: (1, 1),
                    v: SpillValue::Text("x".into()),
                },
                Value::from("x"),
            ),
            (
                Cell::SpillCell {
                    s,
                    a: (1, 1),
                    v: SpillValue::Error(EngineError::VALUE),
                },
                Value::Error(XlError::new(ErrorKind::Value)),
            ),
        ];
        for (cell, expected) in cases {
            assert_eq!(from_cell(&cell), expected, "{cell:?}");
        }
    }

    #[test]
    fn formula_errors_keep_kind_and_message() {
        for (ei, kind, literal) in [
            (EngineError::DIV, ErrorKind::Div0, "#DIV/0!"),
            (EngineError::NA, ErrorKind::NA, "#N/A"),
            (EngineError::SPILL, ErrorKind::Spill, "#SPILL!"),
            (EngineError::CALC, ErrorKind::Calc, "#CALC!"),
        ] {
            let cell = Cell::CellFormula {
                f: 0,
                s: 0,
                v: FormulaValue::Error {
                    ei,
                    o: "Sheet1!A1".into(),
                    m: "why".into(),
                },
            };
            let Value::Error(e) = from_cell(&cell) else {
                panic!("not an error");
            };
            assert_eq!(e, XlError::with_detail(kind, "why"));
            assert_eq!(e.to_string(), literal);
        }
        let blank = Cell::CellFormula {
            f: 0,
            s: 0,
            v: FormulaValue::Error {
                ei: EngineError::NA,
                o: String::new(),
                m: String::new(),
            },
        };
        assert_eq!(from_cell(&blank), Value::Error(XlError::new(ErrorKind::NA)));
    }

    #[test]
    fn value_from_primitives_and_number_equality() {
        assert_eq!(Value::from(1.0), Value::Number(1.0));
        assert_eq!(Value::from(true), Value::Bool(true));
        assert_eq!(Value::from("a"), Value::Text("a".into()));
        assert_eq!(Value::Number(0.0), Value::Number(-0.0));
        assert_ne!(Value::Number(f64::NAN), Value::Number(f64::NAN));
    }

    #[test]
    fn array_shape_is_checked() {
        let a = Array::new(2, 2, vec![1.0.into(), 2.0.into(), 3.0.into(), 4.0.into()]).unwrap();
        assert_eq!((a.rows(), a.cols()), (2, 2));
        assert_eq!(a.get(1, 0), Some(&Value::Number(3.0)));
        assert_eq!(a.get(0, 1), Some(&Value::Number(2.0)));
        assert_eq!(a.get(2, 0), None);
        assert_eq!(a.get(0, 2), None);
        assert_eq!(
            Array::new(usize::MAX, 2, vec![]),
            Err(ArrayShapeError::Overflow {
                rows: usize::MAX,
                cols: 2
            })
        );
        assert_eq!(
            Array::new(2, 2, vec![Value::Empty]),
            Err(ArrayShapeError::LengthMismatch {
                rows: 2,
                cols: 2,
                expected: 4,
                actual: 1
            })
        );
        assert_eq!(Array::new(0, 5, vec![]).unwrap().get(0, 0), None);
    }
}
