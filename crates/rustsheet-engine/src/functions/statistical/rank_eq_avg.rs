use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::ArrayNode, expressions::parser::Node,
    expressions::token::Error, model::Model,
};

impl Model {
    /// A bad `number` next to a non-reference array `ref`: Excel spills that error over the
    /// array's shape instead of returning one cell (docs/excel-semantics/statistics.md,
    /// "RANK of a non-reference array"; probes/rank-nonref-errors, T-1208).
    fn rank_number_error(
        &mut self,
        error: CalcResult,
        ref_arg: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let CalcResult::Error { error: kind, .. } = &error else {
            return error;
        };
        match self.numbers_from_rank_ref_arg(ref_arg, cell) {
            Err(CalcResult::Array(shape)) => CalcResult::Array(
                shape
                    .iter()
                    .map(|row| row.iter().map(|_| ArrayNode::Error(kind.clone())).collect())
                    .collect(),
            ),
            _ => error,
        }
    }
}

impl Model {
    // Helper to collect numeric values from the 2nd argument of RANK.*
    fn collect_rank_values(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        let values = self.numbers_from_rank_ref_arg(arg, cell)?;

        let numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        Ok(numeric_values)
    }

    // RANK.EQ(number, ref, [order])
    pub(crate) fn fn_rank_eq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }

        // number
        let number = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return self.rank_number_error(e, &args[1], cell),
        };

        // ref
        let mut values = match self.collect_rank_values(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        // order: default 0 (descending)
        let order = if args.len() == 2 {
            0.0
        } else {
            match self.get_number_no_bools(&args[2], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        };

        // Excel: a reference with no numeric value (or an empty one) gives #N/A like any absent
        // number, never #NUM! (docs/excel-semantics/statistics.md, "RANK/RANK.EQ/RANK.AVG: a number absent").
        values.retain(|v| !v.is_nan());

        // "better" = greater (descending) or smaller (ascending)
        let mut better = 0;
        let mut equal = 0;

        if order == 0.0 {
            // descending
            for v in &values {
                if *v > number {
                    better += 1;
                } else if *v == number {
                    equal += 1;
                }
            }
        } else {
            // ascending
            for v in &values {
                if *v < number {
                    better += 1;
                } else if *v == number {
                    equal += 1;
                }
            }
        }

        if equal == 0 {
            return CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Number not found in reference for RANK.EQ".to_string(),
            };
        }

        let rank = (better as f64) + 1.0;
        CalcResult::Number(rank)
    }

    // RANK.AVG(number, ref, [order])
    pub(crate) fn fn_rank_avg(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }

        // number
        let number = match self.get_number_no_bools(&args[0], cell) {
            Ok(f) => f,
            Err(e) => return self.rank_number_error(e, &args[1], cell),
        };

        // ref
        let mut values = match self.collect_rank_values(&args[1], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        // order: default 0 (descending)
        let order = if args.len() == 2 {
            0.0
        } else {
            match self.get_number_no_bools(&args[2], cell) {
                Ok(f) => f,
                Err(e) => return e,
            }
        };

        // Same #N/A rule for an empty value set (statistics.md, "RANK/RANK.EQ/RANK.AVG: a number absent").
        values.retain(|v| !v.is_nan());

        // > or < depending on order
        let mut better = 0;
        let mut equal = 0;

        if order == 0.0 {
            // descending
            for v in &values {
                if *v > number {
                    better += 1;
                } else if *v == number {
                    equal += 1;
                }
            }
        } else {
            // ascending
            for v in &values {
                if *v < number {
                    better += 1;
                } else if *v == number {
                    equal += 1;
                }
            }
        }

        if equal == 0 {
            return CalcResult::Error {
                error: Error::NA,
                origin: cell,
                message: "Number not found in reference for RANK.AVG".to_string(),
            };
        }

        // For ties, average of the ranks. If the equal values occupy positions
        // (better+1) ..= (better+equal), the average is:
        // better + (equal + 1) / 2
        let better_f = better as f64;
        let equal_f = equal as f64;
        let rank = better_f + (equal_f + 1.0) / 2.0;

        CalcResult::Number(rank)
    }
}
