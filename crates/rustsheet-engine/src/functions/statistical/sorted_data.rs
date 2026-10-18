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

/// The numbers of a data argument read once, sorted on the first element that asks for them. A
/// kernel checks its own arguments (and `is_empty`) before it asks, so a bad scalar `k` never
/// sorts a column (`=LARGE(A:A,0)`).
pub(crate) struct SortedNumbers {
    numbers: Vec<f64>,
    order: DataOrder,
    sorts: u32,
}

impl SortedNumbers {
    /// Whether the data holds no number; does not sort.
    pub(crate) fn is_empty(&self) -> bool {
        self.numbers.is_empty()
    }

    /// How many numbers the data holds; does not sort.
    pub(crate) fn len(&self) -> usize {
        self.numbers.len()
    }

    /// The numbers in the kernel's order, sorted on the first call only.
    pub(crate) fn sorted(&mut self) -> &[f64] {
        if self.sorts == 0 {
            self.sorts = 1;
            // A stable sort, as each function sorted before (cells hold no NaN).
            match self.order {
                DataOrder::Ascending => self
                    .numbers
                    .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Equal)),
                DataOrder::Descending => self
                    .numbers
                    .sort_by(|a, b| b.partial_cmp(a).unwrap_or(Equal)),
            }
        }
        &self.numbers
    }
}

/// The data argument as the kernel sees it: the numbers, or the read's error, which comes first
/// in every kernel's check order and is every element's result (`LARGE(B1:B3,{1;2})` with B2 =
/// `#N/A` has 2 rows and element 2 is `#N/A`, probes/stat-whole-lift W12-W13). The error is
/// kept with an empty message, so copying it per element allocates nothing.
pub(crate) struct SortedData {
    numbers: Result<SortedNumbers, CalcResult>,
}

impl SortedData {
    /// How many numbers were read (0 for a read error).
    fn len(&self) -> u64 {
        self.numbers.as_ref().map_or(0, |numbers| {
            u64::try_from(numbers.numbers.len()).unwrap_or(u64::MAX)
        })
    }

    /// How many times the numbers were sorted: at most once per call.
    #[cfg(test)]
    pub(crate) fn sorts(&self) -> u32 {
        self.numbers.as_ref().map_or(0, |numbers| numbers.sorts)
    }
}

/// A kernel over the data's numbers and the argument values of one element (slot 0, the data
/// argument, is `EmptyArg`). It takes [`SortedNumbers::sorted`] only once its own checks pass.
pub(crate) type SortedKernel<'k> =
    dyn FnMut(&Model, &mut SortedNumbers, &[CalcResult]) -> CalcResult + 'k;

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
        let numbers = match (data.read)(self, node, cell) {
            Ok(numbers) => Ok(SortedNumbers {
                numbers,
                order: data.order,
                sorts: 0,
            }),
            Err(CalcResult::Error { error, origin, .. }) => Err(CalcResult::Error {
                error,
                origin,
                message: String::new(),
            }),
            // A reader's non-error `Err` (RANK's array of errors; none here) is kept as it is.
            Err(other) => Err(other),
        };
        Ok(SortedData { numbers })
    }

    /// The elements of [`Self::lift_with_sorted_data`] over data already read: the data's error,
    /// or the kernel over the numbers. The data argument comes first in every kernel's check
    /// order (statistics.md "Array arguments given a scalar, a single cell or an error": LARGE
    /// of an error is that error).
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
            &mut |model, values| match &mut data.numbers {
                Ok(numbers) => kernel(model, numbers, values),
                Err(error) => error.clone(),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "a test fails loudly on an unexpected shape"
    )]

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
            &mut |_, data, _| {
                calls += 1;
                CalcResult::Number(data.sorted().first().copied().unwrap_or(f64::NAN))
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

    /// A bad scalar k is refused before the data is sorted: `=LARGE(A:A,0)` never sorts.
    #[test]
    fn a_bad_k_does_not_sort_the_data() {
        let mut model = new_empty_model();
        for row in 1..=8 {
            model._set(&format!("A{row}"), &row.to_string());
        }
        model.evaluate();
        let context = CellReferenceRC {
            sheet: "Sheet1".to_string(),
            row: AT.row,
            column: AT.column,
        };
        let data_arg = DataArg {
            read: |model, node, cell| {
                model
                    .numbers_from_reference_arg(node, cell)
                    .map(|numbers| numbers.into_iter().flatten().collect())
            },
            order: DataOrder::Descending,
            cost: LiftCost::Prepared,
        };
        for formula in ["LARGE(A1:A8,0)", "LARGE(A1:A8,\"x\")", "LARGE(A1:A8,1/0)"] {
            let Node::FunctionKind { args, .. } = model.parser.parse(formula, &context) else {
                panic!("not a call");
            };
            let mut data = model
                .read_sorted_data(&Function::Large, &args, &data_arg, AT)
                .ok()
                .unwrap();
            let result = model.lift_over_sorted(
                Function::Large,
                &args,
                &mut data,
                data_arg.cost,
                AT,
                &mut |model, data, values| model.large_of(data, values, AT),
            );
            assert!(matches!(result, CalcResult::Error { .. }), "{formula}");
            assert_eq!(data.sorts(), 0, "{formula}");
        }
    }

    /// A data read error still comes before a bad k, and it carries no message.
    #[test]
    fn a_data_error_wins_over_a_bad_k() {
        let mut model = new_empty_model();
        for (cell, value) in [("B1", "3"), ("B2", "=NA()"), ("B3", "5")] {
            model._set(cell, value);
        }
        let cases = [
            ("C1", "=LARGE(B1:B3,0)"),
            ("C2", "=SMALL(B1:B3,\"x\")"),
            ("C3", "=LARGE(B1:B3,1/0)"),
            ("C4", "=PERCENTILE(B1:B3,5)"),
            ("C5", "=PERCENTRANK(B1:B3,1/0)"),
            ("C6", "=TRIMMEAN(B1:B3,2)"),
            ("C7", "=INDEX(QUARTILE.EXC(B1:B3,{9;9}),2)"),
        ];
        for (cell, formula) in cases {
            model._set(cell, formula);
        }
        model.evaluate();
        for (cell, formula) in cases {
            assert_eq!(model._get_text(cell), "#N/A", "{formula}");
        }
        assert_eq!(model._get_error_message("C1"), "");
    }
}
