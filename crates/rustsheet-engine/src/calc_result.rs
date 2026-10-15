use std::cmp::Ordering;

use crate::expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex};
use crate::number_format::cmp_excel_precision;
use crate::text::Text;

#[derive(Clone)]
pub struct Range {
    pub left: CellReferenceIndex,
    pub right: CellReferenceIndex,
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
    pub fn new_error(error: Error, origin: CellReferenceIndex, message: String) -> CalcResult {
        CalcResult::Error {
            error,
            origin,
            message,
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
