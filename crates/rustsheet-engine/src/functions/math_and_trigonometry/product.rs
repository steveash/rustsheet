use crate::arithmetic::{extended_product, flush_subnormal};
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::aggregate_args::{AggregatePolicy, ArgReading, ErrorRule, NumberRules};
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// PRODUCT: a referenced-cell or in-array error ranks below a direct scalar error; the leftmost
/// wins (probes/aggregate-error-precedence C1-C8, probes/product-array-2d-and-scalar A8-A17).
const PRODUCT_ARGS: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::Aggregate,
    errors: ErrorRule::Ranked,
    skip_empty_args: false,
};

impl Model {
    /// PRODUCT(number1, [number2], ...)
    ///
    /// The PRODUCT function multiplies values. You can add individual values, cell references or ranges or a mix of all
    /// three.
    ///
    /// # Details
    /// Generally each argument is evaluated and the result is converted to a number. However there are some special
    /// cases:
    /// - **Args evaluating to `CalcResult::Range`**: The product of all the numbers in the range is calculated. No
    ///   conversions are performed, non-numeric values are ignored and errors are propagated.
    /// - **`Node::ReferenceKind` args**: Similar to ranges, conversions are not performed on the evaluated value of the
    ///   referenced cell and errors are propagated.
    /// - **Args evaluating to `CalcResult::Array`**: Only the numbers are multiplied; text and booleans are skipped and
    ///   the first error element is propagated
    ///   after any direct scalar error (it ranks like a referenced-cell error).
    /// - **Args evaluating to `CalcResult::Error`**: Errors are propagated.
    pub(crate) fn fn_product(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut result = 1.0;
        let mut seen_value = false;
        // An empty argument counts as seen (`PRODUCT(,)` is 0 like `PRODUCT(0)`; the walker hands
        // it on as a direct value that casts to 0).
        if let Err(error) = self.for_each_aggregate_number(
            args,
            cell,
            PRODUCT_ARGS,
            NumberRules::SUM_LIKE,
            |value| {
                seen_value = true;
                result = flush_subnormal(extended_product(result, value));
            },
        ) {
            return error;
        }
        if !seen_value {
            return CalcResult::Number(0.0);
        }
        CalcResult::finite_number_or_num(flush_subnormal(result), cell)
    }
}
