use std::cmp::Ordering;

use crate::expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex};
use crate::number_format::cmp_excel_precision;
use crate::text::Text;

/// Rows and columns of the range with corners `left` and `right`, in order.
pub(crate) fn range_dims(left: CellReferenceIndex, right: CellReferenceIndex) -> (usize, usize) {
    (
        usize::try_from(right.row - left.row + 1).unwrap_or(0),
        usize::try_from(right.column - left.column + 1).unwrap_or(0),
    )
}

#[derive(Clone)]
pub struct Range {
    pub left: CellReferenceIndex,
    pub right: CellReferenceIndex,
}

impl Range {
    /// The range between two corners on the sheet of `a`, its corners in order (top-left, then
    /// bottom-right) whichever way round `a` and `b` are.
    pub(crate) fn between(a: CellReferenceIndex, b: CellReferenceIndex) -> Range {
        Range {
            left: CellReferenceIndex {
                sheet: a.sheet,
                row: a.row.min(b.row),
                column: a.column.min(b.column),
            },
            right: CellReferenceIndex {
                sheet: a.sheet,
                row: a.row.max(b.row),
                column: a.column.max(b.column),
            },
        }
    }
}

#[derive(Clone)]
pub(crate) enum CalcResult {
    String(Text),
    Number(f64),
    Boolean(bool),
    Error {
        error: Error,
        origin: CellReferenceIndex,
        message: String,
    },
    Range {
        left: CellReferenceIndex,
        right: CellReferenceIndex,
    },
    EmptyCell,
    EmptyArg,
    Array(Vec<Vec<ArrayNode>>),
    Lambda(usize),
}

impl CalcResult {
    /// Whether this is more than one value: any array (a 1x1 one too) or a reference to several
    /// cells. A single-cell reference is a scalar.
    pub(crate) fn is_multi_value(&self) -> bool {
        match self {
            CalcResult::Range { left, right } => left != right,
            CalcResult::Array(_) => true,
            _ => false,
        }
    }

    /// Rows and columns of a range or array result (1x1 for anything else).
    pub(crate) fn shape(&self) -> (usize, usize) {
        match self {
            CalcResult::Range { left, right } => range_dims(*left, *right),
            CalcResult::Array(array) => (array.len(), array.first().map_or(0, Vec::len)),
            _ => (1, 1),
        }
    }

    pub fn new_error(error: Error, origin: CellReferenceIndex, message: String) -> CalcResult {
        CalcResult::Error {
            error,
            origin,
            message,
        }
    }
    /// An error with its message dropped, anything else as it is: a lifted kernel copies a
    /// data argument's error into every element, and an empty message allocates nothing.
    pub(crate) fn without_message(self) -> CalcResult {
        match self {
            CalcResult::Error { error, origin, .. } => CalcResult::Error {
                error,
                origin,
                message: String::new(),
            },
            other => other,
        }
    }

    pub fn new_args_number_error(origin: CellReferenceIndex) -> CalcResult {
        CalcResult::Error {
            error: Error::ERROR,
            origin,
            message: "Wrong number of arguments".to_string(),
        }
    }
    /// `value` as a number, or `#NUM!` when it is beyond the double range: Excel errors at once
    /// instead of carrying an infinity (probes/arith-overflow Calc!A4:D4, ERROR.TYPE 6).
    pub fn finite_number_or_num(value: f64, origin: CellReferenceIndex) -> CalcResult {
        if value.is_finite() {
            CalcResult::Number(value)
        } else {
            CalcResult::new_error(Error::NUM, origin, "Number too large".to_string())
        }
    }
    pub fn is_error(&self) -> bool {
        matches!(self, CalcResult::Error { .. })
    }

    /// Whether this is a `#CALC!`, the error of an empty array (TAKE or DROP of nothing), which SWITCH's
    /// expression and cases, IMSUM, IMPRODUCT and the holidays of WORKDAY and NETWORKDAYS turn
    /// into `#VALUE!` (docs/excel-semantics/arrays.md "An empty array as a function argument").
    pub(crate) fn is_empty_array(&self) -> bool {
        matches!(
            self,
            CalcResult::Error {
                error: Error::CALC,
                ..
            }
        )
    }

    /// An empty array's `#CALC!` as the `#VALUE!` those readers give it; any other value unchanged.
    pub(crate) fn empty_array_as_value_error(self, cell: CellReferenceIndex) -> CalcResult {
        if self.is_empty_array() {
            return CalcResult::new_error(Error::VALUE, cell, "Empty array".to_string());
        }
        self
    }

    /// A blank cell's value as a function hands it on: the empty value, which COUNT, COUNTA and
    /// AVERAGE count as 0 while ISBLANK still sees it blank. IFERROR and IFNA give it for a blank
    /// value argument (docs/excel-semantics/logic.md "IFERROR / IFNA of a blank cell is an empty
    /// value"), VLOOKUP, HLOOKUP and LOOKUP for a blank result cell (lookup.md "VLOOKUP and
    /// HLOOKUP hand a blank result cell to their caller as 0"). Anything else is unchanged.
    pub(crate) fn blank_as_empty_value(self) -> CalcResult {
        match self {
            CalcResult::EmptyCell => CalcResult::EmptyArg,
            other => other,
        }
    }

    /// The empty value as the number 0, anything else unchanged: what IFS and SWITCH give for
    /// it (`IFS(TRUE,VLOOKUP(..))&""` is "0" when the lookup lands on a blank, while a blank
    /// reference stays blank; lookup.md "VLOOKUP and HLOOKUP hand a blank result cell to their
    /// caller as 0").
    pub(crate) fn empty_value_as_zero(self) -> CalcResult {
        match self {
            CalcResult::EmptyArg => CalcResult::Number(0.0),
            other => other,
        }
    }
}

impl Ord for CalcResult {
    // ..., -2, -1, 0, 1, 2, ..., A-Z, FALSE, TRUE, empty;
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (CalcResult::Number(value1), CalcResult::Number(value2)) => {
                cmp_excel_precision(*value1, *value2)
            }
            (CalcResult::Number(_value1), CalcResult::String(_value2)) => Ordering::Less,
            (CalcResult::Number(_value1), CalcResult::Boolean(_value2)) => Ordering::Less,
            (CalcResult::String(value1), CalcResult::String(value2)) => {
                let value1 = value1.to_uppercase();
                let value2 = value2.to_uppercase();
                value1.cmp(&value2)
            }
            (CalcResult::String(_value1), CalcResult::Boolean(_value2)) => Ordering::Less,
            (CalcResult::Boolean(value1), CalcResult::Boolean(value2)) => {
                if value1 == value2 {
                    return Ordering::Equal;
                }
                if *value1 {
                    return Ordering::Greater;
                }
                Ordering::Less
            }
            (CalcResult::EmptyCell, CalcResult::String(_value2)) => Ordering::Greater,
            (CalcResult::EmptyArg, CalcResult::String(_value2)) => Ordering::Greater,
            (CalcResult::String(_value1), CalcResult::EmptyCell) => Ordering::Less,
            (CalcResult::EmptyCell, CalcResult::Number(_value2)) => Ordering::Greater,
            (CalcResult::EmptyArg, CalcResult::Number(_value2)) => Ordering::Greater,
            (CalcResult::Number(_value1), CalcResult::EmptyCell) => Ordering::Less,
            // A blank is larger than a boolean too (probes/xmatch-text-bool-equal-runs Txt!M21, M24).
            (CalcResult::Boolean(_value1), CalcResult::EmptyCell) => Ordering::Less,
            (CalcResult::EmptyCell, CalcResult::Boolean(_value2)) => Ordering::Greater,
            // An omitted argument orders like a blank cell, `XLOOKUP(,L:L,M:M,,1,2)` of the Excel-authored
            // calc_tests/LOOKUP_AND_REFERENCE/XLOOKUP.xlsx (Errors!N1) finds the first blank.
            (CalcResult::EmptyArg, CalcResult::Boolean(_value2)) => Ordering::Greater,
            (
                CalcResult::String(_) | CalcResult::Number(_) | CalcResult::Boolean(_),
                CalcResult::EmptyArg,
            ) => Ordering::Less,
            (CalcResult::EmptyCell, CalcResult::EmptyCell) => Ordering::Equal,
            (CalcResult::EmptyCell, CalcResult::EmptyArg) => Ordering::Equal,
            (CalcResult::EmptyArg, CalcResult::EmptyCell) => Ordering::Equal,
            (CalcResult::EmptyArg, CalcResult::EmptyArg) => Ordering::Equal,
            // NOTE: Errors and Ranges are not covered
            (_, _) => Ordering::Greater,
        }
    }
}

impl PartialOrd for CalcResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CalcResult {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CalcResult {}
