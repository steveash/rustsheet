use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{value_error, LiftCost};
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// How tied numbers rank: RANK and RANK.EQ give them the top rank, RANK.AVG their average.
#[derive(Clone, Copy)]
enum Ties {
    Top,
    Average,
}

impl Model {
    // RANK(number, ref, [order]) and RANK.EQ
    pub(crate) fn fn_rank_eq(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.rank(args, Ties::Top, cell)
    }

    // RANK.AVG(number, ref, [order])
    pub(crate) fn fn_rank_avg(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.rank(args, Ties::Average, cell)
    }

    /// The RANK family: `number` and `order` lift over an array and a multi-cell range, and a
    /// reference `ref` is read once (SPEC-0011, T-1999). An element reads `number` (numeric text
    /// and booleans coerce, also from a cell), then `order`, then takes the data's error:
    /// `RANK("x",D1:D3)` and `RANK(4,D1:D3,"x")` with D2 = `#N/A` are `#VALUE!`,
    /// `RANK({1;3},D1:D3)` is `#N/A` per element (probes/stat-test-lift P20-P27, P32, P33, P85;
    /// probes/rank-direct-boolean). A `ref` that is not a reference is an array of `#VALUE!` (or
    /// the element's own error) that broadcasts with the lifted arguments, after `number`'s error
    /// and before `order`'s (statistics.md "RANK of a non-reference array"; P28-P31,
    /// probes/rank-nonref-errors). Each element walks the numbers, so the call is charged their
    /// count per element.
    fn rank(&mut self, args: &[Node], ties: Ties, cell: CellReferenceIndex) -> CalcResult {
        // RANK and RANK.AVG share RANK.EQ's table entry (`ONE_THEN_REFERENCE_THEN_1`).
        let kind = Function::RankEq;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let Some(reference) = args.get(1) else {
            return CalcResult::new_args_number_error(cell);
        };
        let numbers = match self.numbers_from_rank_ref_arg(reference, cell) {
            // Excel: a reference with no numeric value (or an empty one) gives #N/A like any
            // absent number, never #NUM! (statistics.md, "RANK/RANK.EQ/RANK.AVG: a number absent").
            Ok(values) => Ok(values
                .into_iter()
                .flatten()
                .filter(|v| !v.is_nan())
                .collect::<Vec<f64>>()),
            Err(CalcResult::Array(grid)) => {
                // Every element of `grid` is an error, so no element reads the numbers.
                let numbers = Err(value_error(cell));
                return self.lift_call_beside(
                    kind,
                    args,
                    (1, grid),
                    LiftCost::Unit,
                    cell,
                    &mut |model, values| rank_element(model, &numbers, values, ties, cell),
                );
            }
            Err(error) => Err(error.without_message()),
        };
        let cells = numbers.as_ref().map_or(0, |numbers| {
            u64::try_from(numbers.len()).unwrap_or(u64::MAX)
        });
        self.lift_call_costed(
            kind,
            args,
            // At least one unit an element, so a data error copied per element is charged too.
            LiftCost::Cells(cells.max(1)),
            cell,
            &mut |model, values| rank_element(model, &numbers, values, ties, cell),
        )
    }
}

/// One RANK element over the numbers read once. `values[1]` is the slot of `ref`: `EmptyArg`
/// for a reference, or the error of a non-reference array's element.
fn rank_element(
    model: &Model,
    numbers: &Result<Vec<f64>, CalcResult>,
    values: &[CalcResult],
    ties: Ties,
    cell: CellReferenceIndex,
) -> CalcResult {
    let number = match values
        .first()
        .map(|number| model.kernel_number(number, cell))
    {
        Some(Ok(number)) => number,
        Some(Err(error)) => return error,
        None => return CalcResult::new_args_number_error(cell),
    };
    if let Some(error @ CalcResult::Error { .. }) = values.get(1) {
        return error.clone();
    }
    // order: default 0 (descending). Text is `#VALUE!`, numeric text too, where `number` reads
    // it; a boolean is 0 or 1 (probes/rank-direct-boolean H4, H10, H15; stat-test-lift P33).
    let order = match values.get(2) {
        Some(CalcResult::String(_)) => return value_error(cell),
        order => match model.kernel_number_or(order, 0.0, cell) {
            Ok(order) => order,
            Err(error) => return error,
        },
    };
    let numbers = match numbers {
        Ok(numbers) => numbers,
        Err(error) => return error.clone(),
    };
    // "better" = greater (descending) or smaller (ascending)
    let mut better: usize = 0;
    let mut equal: usize = 0;
    for &v in numbers {
        let is_better = if order == 0.0 { v > number } else { v < number };
        if is_better {
            better += 1;
        } else if v == number {
            equal += 1;
        }
    }
    if equal == 0 {
        return CalcResult::new_error(Error::NA, cell, String::new());
    }
    let better = better as f64;
    let rank = match ties {
        Ties::Top => better + 1.0,
        // The tied numbers hold ranks better+1 ..= better+equal; their average.
        Ties::Average => better + (equal as f64 + 1.0) / 2.0,
    };
    CalcResult::Number(rank)
}
