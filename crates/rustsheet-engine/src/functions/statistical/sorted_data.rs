//! The sorted data argument of LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN
//! (SPEC-0011, T-1772). Their scalar arguments lift through `lift_call` over arrays and
//! multi-cell ranges (probes/stat-argument-modes, probes/stat-whole-lift; statistics.md
//! "Statistical function argument modes"). The data argument is `ArgKind::WholeByCaller`: each
//! function reads it with its own reader, which looks at the argument's node (a typed scalar
//! against a reference, a union, an external range), once per call before the driver runs. The
//! kernel sorts the numbers on the first element that reaches them and keeps them for the rest,
//! so the call is charged `LiftCost::Prepared(numbers)`: `LARGE(A1:A1000000,SEQUENCE(200))`
//! sorts once.

use std::cmp::Ordering::Equal;

use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::LiftCost;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// Which way a kernel wants its numbers.
#[derive(Clone, Copy)]
pub(crate) enum DataOrder {
    /// Smallest first (SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK*, TRIMMEAN).
    Ascending,
    /// Largest first (LARGE).
    Descending,
}

/// How a function reads and prices its data argument (argument 0).
pub(crate) struct DataArg {
    /// The function's own reader of the data argument's numbers.
    pub(crate) read: fn(&mut Model, &Node, CellReferenceIndex) -> Result<Vec<f64>, CalcResult>,
    pub(crate) order: DataOrder,
    /// What a call costs for that many numbers: `LiftCost::Prepared` when an element is O(1)
    /// once sorted, `LiftCost::Cells` when every element walks the numbers (TRIMMEAN's mean).
    pub(crate) cost: fn(u64) -> LiftCost<'static>,
}

/// The data argument as the kernel sees it: the numbers read once, sorted on first use, or the
/// read's error, which every element returns (`LARGE(B1:B3,{1;2})` with B2 = `#N/A` is two
/// `#N/A`, probes/stat-whole-lift W12-W13).
pub(crate) struct SortedData {
    numbers: Result<Vec<f64>, CalcResult>,
    order: DataOrder,
    sorts: u32,
}

impl SortedData {
    /// How many numbers were read (0 for a read error).
    fn len(&self) -> u64 {
        self.numbers.as_ref().map_or(0, |numbers| {
            u64::try_from(numbers.len()).unwrap_or(u64::MAX)
        })
    }

    /// The numbers in the kernel's order, sorted on the first call only; or the read's error.
    fn sorted(&mut self) -> Result<&[f64], &CalcResult> {
        let numbers = self.numbers.as_mut().map_err(|error| &*error)?;
        if self.sorts == 0 {
            self.sorts = 1;
            // A stable sort, as each function sorted before (cells hold no NaN).
            match self.order {
                DataOrder::Ascending => numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)),
                DataOrder::Descending => numbers.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Equal)),
            }
        }
        Ok(numbers)
    }

    /// How many times the numbers were sorted: at most once per call.
    #[cfg(test)]
    pub(crate) fn sorts(&self) -> u32 {
        self.sorts
    }
}

/// A kernel over the sorted numbers and the argument values of one element (slot 0, the data
/// argument, is `EmptyArg`).
pub(crate) type SortedKernel<'k> = dyn FnMut(&Model, &[f64], &[CalcResult]) -> CalcResult + 'k;

impl Model {
    /// Runs `kind` (data, then scalar arguments) by its table entry: a wrong count is the count
    /// error before anything is read; the data argument is read once by `data.read`, then the
    /// kernel runs once, or once per element of the lifted arguments.
    pub(crate) fn lift_with_sorted_data(
        &mut self,
        kind: Function,
        args: &[Node],
        data: &DataArg,
        cell: CellReferenceIndex,
        kernel: &mut SortedKernel<'_>,
    ) -> CalcResult {
        match self.read_sorted_data(&kind, args, data, cell) {
            Ok(mut sorted) => {
                self.lift_over_sorted(kind, args, &mut sorted, data.cost, cell, kernel)
            }
            Err(error) => error,
        }
    }

    /// The read phase of [`Self::lift_with_sorted_data`]: the count check, then argument 0 read
    /// by `data.read`. Only the count error is an `Err`; a read error is kept for the kernel.
    pub(crate) fn read_sorted_data(
        &mut self,
        kind: &Function,
        args: &[Node],
        data: &DataArg,
        cell: CellReferenceIndex,
    ) -> Result<SortedData, CalcResult> {
        let accepts = kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()));
        let (true, Some(node)) = (accepts, args.first()) else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        Ok(SortedData {
            numbers: (data.read)(self, node, cell),
            order: data.order,
            sorts: 0,
        })
    }

    /// The elements of [`Self::lift_with_sorted_data`] over data already read: the data's error,
    /// or the kernel over the sorted numbers. The data argument comes first in every kernel's
    /// check order (statistics.md "Array arguments given a scalar, a single cell or an error":
    /// LARGE of an error is that error).
    pub(crate) fn lift_over_sorted(
        &mut self,
        kind: Function,
        args: &[Node],
        data: &mut SortedData,
        cost: fn(u64) -> LiftCost<'static>,
        cell: CellReferenceIndex,
        kernel: &mut SortedKernel<'_>,
    ) -> CalcResult {
        let cost = cost(data.len());
        self.lift_call_costed(
            kind,
            args,
            cost,
            cell,
            &mut |model, values| match data.sorted() {
                Ok(sorted) => kernel(model, sorted, values),
                Err(error) => error.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::panic)]

    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{DataArg, DataOrder};
    use crate::calc_result::CalcResult;
    use crate::expressions::parser::Node;
    use crate::expressions::types::{CellReferenceIndex, CellReferenceRC};
    use crate::functions::lift::LiftCost;
    use crate::functions::Function;
    use crate::model::Model;
    use crate::test::util::new_empty_model;

    /// Where the call runs: Sheet1!Z1.
    const AT: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 26,
    };

    /// The data reads of [`counting_reader`].
    static READS: AtomicUsize = AtomicUsize::new(0);

    /// LARGE's reader, counting its calls.
    fn counting_reader(
        model: &mut Model,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<f64>, CalcResult> {
        READS.fetch_add(1, Ordering::SeqCst);
        model
            .numbers_from_reference_arg(node, cell)
            .map(|numbers| numbers.into_iter().flatten().collect())
    }

    /// The data argument is read once and sorted once per call, however many elements run.
    #[test]
    fn the_data_is_read_and_prepared_once_per_call() {
        let mut model = new_empty_model();
        for (row, value) in [1, 2, 4, 3, 7, 5, 8, 6].iter().enumerate() {
            model._set(&format!("A{}", row + 1), &value.to_string());
        }
        model.evaluate();
        let context = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: AT.row,
            column: AT.column,
        };
        let Node::FunctionKind { args, .. } =
            model.parser.parse("LARGE(A1:A8,{1;2;3;4})", &context)
        else {
            panic!("not a call");
        };
        let data_arg = DataArg {
            read: counting_reader,
            order: DataOrder::Descending,
            cost: LiftCost::Prepared,
        };
        let mut data = model
            .read_sorted_data(&Function::Large, &args, &data_arg, AT)
            .ok()
            .unwrap();
        let mut calls = 0;
        let result = model.lift_over_sorted(
            Function::Large,
            &args,
            &mut data,
            data_arg.cost,
            AT,
            &mut |_, sorted, _| {
                calls += 1;
                CalcResult::Number(sorted.first().copied().unwrap_or(f64::NAN))
            },
        );
        let CalcResult::Array(rows) = result else {
            panic!("not an array");
        };
        assert_eq!(rows.len(), 4);
        assert_eq!(calls, 4);
        assert_eq!(READS.load(Ordering::SeqCst), 1);
        assert_eq!(data.sorts(), 1);
        // A wrong count is refused before the data is read.
        let Node::FunctionKind { args, .. } = model.parser.parse("LARGE(A1:A8)", &context) else {
            panic!("not a call");
        };
        assert!(model
            .read_sorted_data(&Function::Large, &args, &data_arg, AT)
            .is_err());
        assert_eq!(READS.load(Ordering::SeqCst), 1);
    }
}
