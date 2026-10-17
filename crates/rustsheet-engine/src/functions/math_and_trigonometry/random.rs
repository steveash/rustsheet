use crate::cast::NumberOrArray;
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::limits::check_array_size;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

#[cfg(not(target_arch = "wasm32"))]
fn random() -> f64 {
    rand::random()
}

#[cfg(target_arch = "wasm32")]
fn random() -> f64 {
    use js_sys::Math;
    Math::random()
}

/// One RANDBETWEEN draw: the raw bounds are compared, a span that overflows a double is `#NUM!`
/// (probes/randbetween-huge-span A1:A2) and an empty `CEILING(bottom)..FLOOR(top)` gives
/// `CEILING(bottom)`.
fn randbetween_draw(bottom: f64, top: f64) -> Result<f64, Error> {
    if bottom > top || !(top - bottom).is_finite() {
        return Err(Error::NUM);
    }
    let low = bottom.ceil();
    let high = top.floor();
    if high < low {
        return Ok(low);
    }
    Ok((low + random() * (high - low + 1.0)).floor().min(high))
}

impl Model {
    pub(crate) fn fn_rand(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(random())
    }

    /// `=RANDBETWEEN(bottom, top)`. Excel compares the raw bounds, draws a whole number from
    /// `CEILING(bottom)` to `FLOOR(top)` (`CEILING(bottom)` when that range is empty), rejects
    /// booleans and treats an omitted bound (`RANDBETWEEN(,5)`, `(5,)`, `(,)`)
    /// as `#N/A` (docs/excel-semantics/numbers.md
    /// "RANDBETWEEN and RANDARRAY argument checks", probes/volatile-invariant).
    pub(crate) fn fn_randbetween(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let first = self.evaluate_node_in_context(&args[0], cell);
        let second = self.evaluate_node_in_context(&args[1], cell);
        if matches!(first, CalcResult::EmptyArg) || matches!(second, CalcResult::EmptyArg) {
            return CalcResult::new_error(Error::NA, cell, "bottom or top omitted".to_string());
        }
        // Excel: a multi-cell range bound is `#VALUE!` (no lift, no implicit intersection:
        // probes/randbetween-lift-operands A7:A11, B2:B4, T-1316); a single-cell reference is a
        // scalar.
        for bound in [&first, &second] {
            if matches!(bound, CalcResult::Range { left, right } if left != right) {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "RANDBETWEEN bound is a range".to_string(),
                );
            }
        }
        // An array bound lifts, one draw per element (probes/volatile-invariant F1:F9); a boolean
        // element is `#VALUE!` like a scalar boolean (probes/randbetween-lift-operands A1:A3, A12).
        if [&first, &second]
            .iter()
            .any(|v| matches!(v, CalcResult::Array(_)))
        {
            // Excel reads the top first: when both bounds are errors the top's wins, per element
            // (probes/randbetween-bound-errors A6:A20), so the operands go in as [top, bottom].
            let operands = [second, first]
                .into_iter()
                .map(|v| match v {
                    CalcResult::Array(rows) => self.number_or_array(
                        CalcResult::Array(
                            rows.into_iter()
                                .map(|row| {
                                    row.into_iter()
                                        .map(|node| match node {
                                            ArrayNode::Boolean(_) => ArrayNode::Error(Error::VALUE),
                                            other => other,
                                        })
                                        .collect()
                                })
                                .collect(),
                        ),
                        cell,
                    ),
                    scalar => self
                        .cast_to_number_no_bools(scalar, cell)
                        .map(NumberOrArray::Number),
                })
                .collect();
            return self.lift_number_operands(operands, cell, "Invalid arguments", |[t, b]| {
                randbetween_draw(b, t)
            });
        }
        let top = match self.cast_to_number_no_bools(second, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        let bottom = match self.cast_to_number_no_bools(first, cell) {
            Ok(f) => f,
            Err(e) => return e,
        };
        match randbetween_draw(bottom, top) {
            Ok(n) => CalcResult::Number(n),
            Err(e) => CalcResult::new_error(e, cell, format!("RANDBETWEEN({bottom},{top})")),
        }
    }

    /// `=RANDARRAY([rows], [cols], [min], [max], [whole_number])`
    ///
    /// Returns a 2-D array of random numbers.
    ///   * rows         – number of rows (default 1)
    ///   * cols         – number of columns (default 1)
    ///   * min          – minimum value (default 0)
    ///   * max          – maximum value (default 1)
    ///   * `whole_number` – FALSE = decimal, TRUE = integer (default FALSE)
    pub(crate) fn fn_randarray(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 5 {
            return CalcResult::new_args_number_error(cell);
        }

        let rows = if !args.is_empty() {
            let v = self.evaluate_node_in_context(&args[0], cell);
            if matches!(v, CalcResult::EmptyArg) {
                1
            } else {
                match self.cast_to_number(v, cell) {
                    Ok(n) => {
                        let n = n.floor() as i64;
                        if n == 0 {
                            return CalcResult::new_error(
                                Error::CALC,
                                cell,
                                "rows must be >= 1".to_string(),
                            );
                        }
                        if n < 0 {
                            return CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "rows must be >= 1".to_string(),
                            );
                        }
                        n as usize
                    }
                    Err(e) => return e,
                }
            }
        } else {
            1
        };

        let columns = if args.len() >= 2 {
            let v = self.evaluate_node_in_context(&args[1], cell);
            if matches!(v, CalcResult::EmptyArg) {
                1
            } else {
                match self.cast_to_number(v, cell) {
                    Ok(n) => {
                        let n = n.floor() as i64;
                        if n == 0 {
                            return CalcResult::new_error(
                                Error::CALC,
                                cell,
                                "columns must be >= 1".to_string(),
                            );
                        }
                        if n < 0 {
                            return CalcResult::new_error(
                                Error::VALUE,
                                cell,
                                "columns must be >= 1".to_string(),
                            );
                        }
                        n as usize
                    }
                    Err(e) => return e,
                }
            }
        } else {
            1
        };

        if let Some((error, message)) = check_array_size(rows, columns) {
            return CalcResult::new_error(error, cell, message);
        }

        let min = if args.len() >= 3 {
            let v = self.evaluate_node_in_context(&args[2], cell);
            if matches!(v, CalcResult::EmptyArg) {
                0.0
            } else {
                match self.cast_to_number(v, cell) {
                    Ok(n) => n,
                    Err(e) => return e,
                }
            }
        } else {
            0.0
        };

        let max = if args.len() >= 4 {
            let v = self.evaluate_node_in_context(&args[3], cell);
            if matches!(v, CalcResult::EmptyArg) {
                1.0
            } else {
                match self.cast_to_number(v, cell) {
                    Ok(n) => n,
                    Err(e) => return e,
                }
            }
        } else {
            1.0
        };

        if min > max {
            return CalcResult::new_error(Error::VALUE, cell, "min must be <= max".to_string());
        }

        // get_boolean already maps EmptyArg → false, which is the correct default.
        let whole_number = if args.len() >= 5 {
            match self.get_boolean(&args[4], cell) {
                Ok(b) => b,
                Err(e) => return e,
            }
        } else {
            false
        };

        // Excel: a fractional bound with whole_number TRUE is #VALUE! (RANDARRAY(1,1,1.5,2.5,TRUE)).
        if whole_number && (min.fract() != 0.0 || max.fract() != 0.0) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "min and max must be whole numbers".to_string(),
            );
        }

        let mut result = Vec::with_capacity(rows);
        for _ in 0..rows {
            let mut row = Vec::with_capacity(columns);
            for _ in 0..columns {
                let r = random();
                let span = max - min + 1.0;
                // Excel: a span that overflows a double still draws a finite value in range
                // (probes/randbetween-huge-span B2:B3, B14:B15), so interpolate instead. Whether such a
                // value is a whole number with `whole_number` TRUE is unprobed at this scale.
                let val = if !span.is_finite() {
                    min * (1.0 - r) + max * r
                } else if whole_number {
                    min + (r * span).floor()
                } else {
                    min + r * (max - min)
                };
                row.push(ArrayNode::Number(val));
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}
