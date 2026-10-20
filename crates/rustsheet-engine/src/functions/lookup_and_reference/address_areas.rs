use crate::external_reference::PickedReference;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult,
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

    /// ADDRESS of one element. The arguments are checked left to right, as the scalar call
    /// always did.
    fn address_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let (row_num, col_num, rest) = match values {
            [row_num, col_num, rest @ ..] => (row_num, col_num, rest),
            _ => return CalcResult::new_args_number_error(cell),
        };
        let row_num = match self.cast_number_ref(row_num, cell) {
            Ok(n) => n as i32,
            Err(e) => return e,
        };
        if row_num < 1 {
            return CalcResult::new_error(Error::VALUE, cell, "row_num must be >= 1".to_string());
        }

        let col_num = match self.cast_number_ref(col_num, cell) {
            Ok(n) => n as i32,
            Err(e) => return e,
        };
        if col_num < 1 {
            return CalcResult::new_error(Error::VALUE, cell, "col_num must be >= 1".to_string());
        }

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
            match abs_num {
                1 => format!("{sheet_prefix}${col_letter}${row_num}"),
                2 => format!("{sheet_prefix}{col_letter}${row_num}"),
                3 => format!("{sheet_prefix}${col_letter}{row_num}"),
                4 => format!("{sheet_prefix}{col_letter}{row_num}"),
                _ => {
                    // This should never happen due to the earlier check
                    return CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "invalid abs_num".to_string(),
                    );
                }
            }
        } else {
            // R1C1 style
            let row_part = if abs_num == 1 || abs_num == 2 {
                format!("R{row_num}")
            } else {
                format!("R[{row_num}]")
            };
            let col_part = if abs_num == 1 || abs_num == 3 {
                format!("C{col_num}")
            } else {
                format!("C[{col_num}]")
            };
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
            // Per element in Excel (SPEC-0014 rule 6, T-2064); the value view until then.
            RefArg::Grid(grid) => grid.to_values(),
        }
    }
}
