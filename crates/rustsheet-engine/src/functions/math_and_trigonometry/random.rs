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

impl Model {
    pub(crate) fn fn_rand(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        CalcResult::Number(random())
    }

    /// `=RANDBETWEEN(bottom, top)`. Excel compares the raw bounds, draws a whole number from
    /// `CEILING(bottom)` to `FLOOR(top)` (`CEILING(bottom)` when that range is empty), rejects
    /// booleans and treats `RANDBETWEEN(,)` as `#N/A` (docs/excel-semantics/numbers.md
    /// "RANDBETWEEN and RANDARRAY argument checks", probes/volatile-invariant).
    pub(crate) fn fn_randbetween(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let first = self.evaluate_node_in_context(&args[0], cell);
        let second = self.evaluate_node_in_context(&args[1], cell);
        if matches!(first, CalcResult::EmptyArg) && matches!(second, CalcResult::EmptyArg) {
            return CalcResult::new_error(Error::NA, cell, "bottom and top omitted".to_string());
        }
        let mut bounds = [0.0; 2];
        for (slot, v) in bounds.iter_mut().zip([first, second]) {
            if matches!(v, CalcResult::Boolean(_)) {
                return CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "booleans are not numbers here".to_string(),
                );
            }
            match self.cast_to_number(v, cell) {
                Ok(f) => *slot = f,
                Err(e) => return e,
            }
        }
        let [bottom, top] = bounds;
        if bottom > top {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: format!("{bottom}>{top}"),
            };
        }
        let low = bottom.ceil();
        let high = top.floor();
        if high < low {
            return CalcResult::Number(low);
        }
        CalcResult::Number((low + random() * (high - low + 1.0)).floor().min(high))
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
                let val = if whole_number {
                    let span = max - min + 1.0;
                    min + (random() * span).floor()
                } else {
                    min + random() * (max - min)
                };
                row.push(ArrayNode::Number(val));
            }
            result.push(row);
        }

        CalcResult::Array(result)
    }
}
