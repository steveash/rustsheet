use crate::external_reference::PickedReference;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult,
    constants::{LAST_COLUMN, LAST_ROW},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex, utils::quote_name},
    model::Model,
    reference_operators::{not_a_reference_array, three_d_needs_one_sheet, RefArg},
};

use crate::expressions::utils::number_to_column;

/// Quotes `sheet_text` for ADDRESS. A leading `[workbook]` part does not by
/// itself force quotes: the decision is made on the text after the `]`
/// (Excel-cached poi EverythingTests!F128; docs/excel-semantics/references.md).
fn address_sheet_text(s: &str) -> String {
    if let Some(rest) = s.strip_prefix('[') {
        if let Some((book, sheet)) = rest.split_once(']') {
            if !book.contains('[') && quote_name(sheet) == sheet {
                return s.to_string();
            }
        }
    }
    quote_name(s)
}

/// ADDRESS's row or column index from `n` (an index up to `last`, or a relative R1C1 offset),
/// or `None` when it is out of range. Excel floors the number in every mode (2.9 is 2, -2.5,
/// -2.4 and -0.5 are -3, -3 and -1) and accepts an index in 1..=last, an offset in
/// -(last - 1)..=last - 1: `ADDRESS(1048577,3)` and `ADDRESS(-1048576,3,4,FALSE)` are `#VALUE!`
/// (probes/t2275-address-bounds; docs/excel-semantics/lookup.md "ADDRESS: argument errors
/// first, `sheet_text` "TRUE"/"FALSE" quoted, bounds").
fn address_index(n: f64, last: i32, relative: bool) -> Option<i32> {
    let n = n.floor();
    let (lo, hi) = if relative {
        (f64::from(1 - last), f64::from(last - 1))
    } else {
        (1.0, f64::from(last))
    };
    // In range, so the cast is exact.
    (lo..=hi).contains(&n).then_some(n as i32)
}

/// One R1C1 part: `R5`, or relative `R[-3]`, with a 0 offset written bare (`ADDRESS(0,3,3,FALSE)`
/// is `RC3`, probes/t2275-address-bounds).
fn r1c1_part(letter: char, n: i32, relative: bool) -> String {
    match (relative, n) {
        (false, n) => format!("{letter}{n}"),
        (true, 0) => letter.to_string(),
        (true, n) => format!("{letter}[{n}]"),
    }
}

impl Model {
    // ── ADDRESS ───────────────────────────────────────────────────────────────

    /// `=ADDRESS(row_num, col_num, [abs_num], [a1], [sheet_text])`
    ///
    /// Returns a cell address as text.
    ///   * `abs_num`: 1=absolute $A$1 (default), 2=row-abs/col-rel A$1, 3=row-rel/col-abs $A1, 4=relative A1
    ///   * a1: TRUE=A1 style (default), FALSE=R1C1 style
    ///   * `sheet_text`: optional sheet name prefix
    ///
    /// Every argument lifts over an array and a multi-cell range (`ROWS(ADDRESS(A1:A3,1))` and
    /// `ROWS(ADDRESS({1;2;3},1))` are 3; probes/lookup-argument-modes F1:G4, gen-m7/f68fafa0ca2eba26
    /// A7; docs/excel-semantics/lookup.md "Lookup and reference function argument modes").
    pub(crate) fn fn_address(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Address, args, cell, &mut |model, values| {
            model.address_of(values, cell)
        })
    }

    /// ADDRESS of one element. The arguments are cast left to right, as the scalar call always
    /// did; row and column bounds depend on `abs_num` and `a1`, so they are checked after every cast
    /// (an argument error beats a bad row: docs/excel-semantics/lookup.md).
    fn address_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (row_num, col_num, rest) = match values {
            [row_num, col_num, rest @ ..] => (row_num, col_num, rest),
            _ => return CalcResult::new_args_number_error(cell),
        };
        let row_num = match self.cast_number_ref(row_num, cell) {
            Ok(n) => n,
            Err(e) => return e,
        };
        let col_num = match self.cast_number_ref(col_num, cell) {
            Ok(n) => n,
            Err(e) => return e,
        };

        let abs_num = match rest.first() {
            Some(value) => match self.cast_number_ref(value, cell) {
                Ok(n) => n as i32,
                Err(e) => return e,
            },
            None => 1,
        };

        if !(1..=4).contains(&abs_num) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "abs_num must be 1, 2, 3, or 4".to_string(),
            );
        }

        let use_a1 = match rest.get(1) {
            Some(value) => match Self::cast_bool_ref(value, cell) {
                Ok(b) => b,
                Err(e) => return e,
            },
            None => true,
        };

        // When `sheet_text` is present the `!` separator is always emitted, even
        // for an empty string: ADDRESS(1,1,1,TRUE,"") is "!$A$1" in Excel. A
        // `sheet_text` that is omitted entirely produces no prefix.
        let sheet_prefix = match rest.get(2) {
            Some(value) => match Self::cast_string_ref(value, cell) {
                Ok(s) => format!("{}!", address_sheet_text(&s)),
                Err(e) => return e,
            },
            None => String::new(),
        };

        // The row is relative in R1C1 style for abs_num 3 and 4, the column for 2 and 4; in A1
        // style both are written as plain numbers and letters, so both are absolute indexes.
        let row_relative = !use_a1 && (abs_num == 3 || abs_num == 4);
        let col_relative = !use_a1 && (abs_num == 2 || abs_num == 4);
        let Some(row_num) = address_index(row_num, LAST_ROW, row_relative) else {
            return CalcResult::new_error(Error::VALUE, cell, "row_num out of range".to_string());
        };
        let Some(col_num) = address_index(col_num, LAST_COLUMN, col_relative) else {
            return CalcResult::new_error(Error::VALUE, cell, "col_num out of range".to_string());
        };

        let address = if use_a1 {
            let col_letter = match number_to_column(col_num) {
                Some(s) => s,
                None => {
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "col_num out of range".to_string(),
                    )
                }
            };
            let col_mark = if abs_num == 1 || abs_num == 3 {
                "$"
            } else {
                ""
            };
            let row_mark = if abs_num == 1 || abs_num == 2 {
                "$"
            } else {
                ""
            };
            format!("{sheet_prefix}{col_mark}{col_letter}{row_mark}{row_num}")
        } else {
            let row_part = r1c1_part('R', row_num, row_relative);
            let col_part = r1c1_part('C', col_num, col_relative);
            format!("{sheet_prefix}{row_part}{col_part}")
        };

        CalcResult::String(address.into())
    }

    // ── AREAS ─────────────────────────────────────────────────────────────────

    /// `=AREAS(reference)`
    ///
    /// Returns the number of areas in a reference (SPEC-0004, probes/multiarea-areas-*). A 3-D
    /// reference is `#VALUE!` (probes/3d-ref-functions). The parser refuses an argument not
    /// written as a reference (`AREAS(1)`), as Excel does; one that evaluates to a value
    /// (`AREAS(IF(TRUE,5))`, `AREAS(LAMBDA(x,x))`) is `#VALUE!` and an error is itself
    /// (excel-graph scratch probes, docs/excel-semantics/references.md "SUBTOTAL and AREAS take
    /// references only", T-1163).
    pub(crate) fn fn_areas(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // A union of references into another workbook counts its areas (`AREAS((…!A1,…!B2))` is
        // 2, probes/external-ref-no-link-shapes A15).
        // So does one IF picks (`AREAS(IF(TRUE,(…!A1,…!B2)))` is 2,
        // probes/external-ref-no-link-ifchoose A17).
        let arg = match self.pick_reference(&args[0], cell) {
            PickedReference::External(areas) => {
                return match areas {
                    Ok(areas) => CalcResult::Number(areas.len() as f64),
                    Err(error) => error,
                };
            }
            PickedReference::Node(node) => self.evaluate_picked_reference_arg(node, cell),
            PickedReference::Value(value) => RefArg::Value(value),
        };
        if let Some(areas) = arg.single_sheet_areas() {
            return CalcResult::Number(areas.len() as f64);
        }
        match arg {
            RefArg::Value(error) if error.is_error() => error,
            // An array spills `#VALUE!` per element (probes/subtotal-reference-operands-b H4).
            RefArg::Value(CalcResult::Array(rows)) => not_a_reference_array(rows),
            RefArg::Value(_) => {
                CalcResult::new_error(Error::VALUE, cell, "AREAS takes a reference".to_string())
            }
            RefArg::Ranges(_) => three_d_needs_one_sheet(cell),
            // Reached only through TAKE or DROP of one: per-element calls ask for the grid where
            // `may_hand_on_grid` says one can come (SPEC-0014 rule 6); here, its value view.
            RefArg::Grid(grid) => grid.to_values(),
        }
    }
}
