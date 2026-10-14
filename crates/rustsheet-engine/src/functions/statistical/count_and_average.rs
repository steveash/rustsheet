use crate::reference_operators::RefArg;
use std::cmp::Ordering;

use crate::arithmetic::{extended_sum, flush_subnormal};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// How AVERAGEA, MAXA and MINA read a text element of an array value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ArrayText {
    /// AVERAGEA counts it as 0.
    Zero,
    /// MAXA and MINA skip it.
    Skipped,
}

impl Model {
    pub(crate) fn for_each_value<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        // An error in a referenced cell ranks below a direct scalar error whichever comes first;
        // the leftmost reference error wins (docs/excel-semantics/syntax.md, "Aggregate error
        // precedence"; probes/aggregate-error-precedence C25-C31).
        let mut reference_error = None;
        for arg in args {
            // References (also via IF/INDEX) arrive as a Range; the scalar
            // arms below only see values.
            match self.evaluate_aggregate_arg(arg, cell) {
                RefArg::Value(CalcResult::Number(value)) => {
                    f(value);
                }
                // An empty argument (`AVERAGE(5,)`) counts as 0
                // (probes/average-value-args F2, F6, F9, F15, F17).
                RefArg::Value(CalcResult::EmptyArg) => f(0.0),
                RefArg::Value(CalcResult::Boolean(value))
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) =>
                {
                    f(if value { 1.0 } else { 0.0 });
                }
                RefArg::Value(CalcResult::String(value))
                    if !matches!(arg.ungrouped(), Node::ReferenceKind { .. }) =>
                {
                    f(self.cast_aggregate_arg_to_number(CalcResult::String(value), cell)?);
                }
                RefArg::Value(CalcResult::Array(array)) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    f(value);
                                }
                                // Booleans inside an array are ignored, unlike direct
                                // arguments (probes/average-value-args F1, F5, F8, F10).
                                ArrayNode::Error(error) => {
                                    return Err(CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    });
                                }
                                _ => {
                                    // ignore non-numeric
                                }
                            }
                        }
                    }
                }
                RefArg::Ranges(areas) => {
                    if reference_error.is_none() {
                        if let Err(error) = self.for_each_number_in_areas(&areas, cell, &mut f) {
                            reference_error = Some(error);
                        }
                    }
                }
                RefArg::Value(error @ CalcResult::Error { .. }) => return Err(error),
                // Everything else is ignored
                RefArg::Value(_) => {}
            }
        }

        reference_error.map_or(Ok(()), Err)
    }

    /// `for_each_value` that skips empty arguments (`DEVSQ(1, )`) instead of counting them as 0.
    /// DEVSQ, AVEDEV and SUMSQ ignored them before they moved onto `for_each_value` and there is
    /// no Excel evidence either way yet. TODO(T-0236): probe empty arguments and fold this into `for_each_value`.
    pub(crate) fn for_each_value_skip_empty<F>(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        for arg in args {
            if !matches!(arg, Node::EmptyArgKind) {
                self.for_each_value(std::slice::from_ref(arg), cell, &mut f)?;
            }
        }
        Ok(())
    }

    /// The values AVERAGEA, MAXA and MINA read from `args`: a reference's cells and a direct
    /// argument coerce text to 0 and booleans to 1/0 (coercion.md "MAXA / MINA"), while an array
    /// value's elements follow `array_text` (statistics.md "AVERAGEA, MAXA and MINA over an array").
    fn for_each_value_a<F>(
        &mut self,
        args: &[Node],
        array_text: ArrayText,
        cell: CellReferenceIndex,
        mut f: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(f64),
    {
        for arg in args {
            // A cached range of a linked workbook is a reference, not an array: its text counts 0
            // and its booleans 1/0 (`AVERAGEA([1]Uses!$A$1:$B$3)` is 3.2;
            // fixtures/external-link-cached-ranges Refs!B53).
            if let Some(cells) = self.external_reference_cells(arg, cell) {
                for value in cells? {
                    match value {
                        ArrayNode::Number(value) => f(value),
                        ArrayNode::Boolean(b) => f(if b { 1.0 } else { 0.0 }),
                        ArrayNode::String(_) => f(0.0),
                        ArrayNode::Error(error) => {
                            return Err(CalcResult::new_error(
                                error,
                                cell,
                                "Error in external reference".to_string(),
                            ))
                        }
                        ArrayNode::Empty => {}
                    }
                }
                continue;
            }
            // An XLOOKUP or INDEX that picks a cell is that cell: text there counts 0
            // (`AVERAGEA(XLOOKUP(1,A1:A2,B1:B2),5,7)` over "x" is 4;
            // probes/subtotal-reference-operands-d H22).
            match self.evaluate_aggregate_arg(arg, cell) {
                RefArg::Value(CalcResult::Number(value)) => {
                    f(value);
                }
                // A single-cell reference reads like a one-cell range: text counts as 0 and
                // a boolean as 1/0 (coercion.md "MAXA / MINA", poi EverythingTests J888, F912).
                RefArg::Value(CalcResult::Boolean(value)) => {
                    f(if value { 1.0 } else { 0.0 });
                }
                RefArg::Value(CalcResult::String(_))
                    if matches!(arg.ungrouped(), Node::ReferenceKind { .. }) =>
                {
                    f(0.0);
                }
                RefArg::Value(CalcResult::String(value)) => {
                    if let Some(error) = Self::aggregate_text_too_long(&value, cell) {
                        return Err(error);
                    }
                    if let Some(parsed) = self.cast_number(&value) {
                        f(parsed);
                    } else {
                        return Err(CalcResult::new_error(
                            Error::VALUE,
                            cell,
                            "Argument cannot be cast into number".to_string(),
                        ));
                    }
                }
                RefArg::Value(CalcResult::Array(array)) => {
                    for row in array {
                        for value in row {
                            match value {
                                ArrayNode::Number(value) => {
                                    f(value);
                                }
                                // An array element is not a cell: booleans are skipped, and
                                // text, numeric text included, is 0 for AVERAGEA and skipped by
                                // MAXA and MINA (`AVERAGEA({1,"a",TRUE})` is 0.5, `MAXA({-1,"a"})`
                                // -1; probes/averagea-array-constant-elements).
                                ArrayNode::Boolean(_) => {}
                                ArrayNode::String(_) => {
                                    if array_text == ArrayText::Zero {
                                        f(0.0);
                                    }
                                }
                                ArrayNode::Error(error) => {
                                    return Err(CalcResult::Error {
                                        error,
                                        origin: cell,
                                        message: "Error in array".to_string(),
                                    });
                                }
                                ArrayNode::Empty => {}
                            }
                        }
                    }
                }
                RefArg::Ranges(areas) => {
                    self.for_each_cell_in_areas(&areas, cell, |_, value| match value {
                        CalcResult::Number(value) => {
                            f(value);
                            Ok(())
                        }
                        CalcResult::Boolean(b) => {
                            f(if b { 1.0 } else { 0.0 });
                            Ok(())
                        }
                        CalcResult::String(_) => {
                            f(0.0);
                            Ok(())
                        }
                        CalcResult::Range { .. } => Err(CalcResult::new_error(
                            Error::ERROR,
                            cell,
                            "Unexpected Range".to_string(),
                        )),
                        _ => Ok(()),
                    })?;
                }
                RefArg::Value(error @ CalcResult::Error { .. }) => return Err(error),
                // Everything else is ignored
                RefArg::Value(_) => {}
            }
        }

        Ok(())
    }

    pub(crate) fn fn_average(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        if let Err(e) = self.for_each_value(args, cell, |f| {
            count += 1.0;
            sum = extended_sum(sum, f);
        }) {
            return e;
        }

        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::finite_number_or_num(flush_subnormal(sum / count), cell)
    }

    pub(crate) fn fn_averagea(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut count = 0.0;
        let mut sum = 0.0;
        // Coerced as MAXA coerces, a cached range of a linked workbook included
        // (`AVERAGEA([1]Uses!$A$1:$B$3)` is 3.2; fixtures/external-link-cached-ranges Refs!B53).
        if let Err(error) = self.for_each_value_a(args, ArrayText::Zero, cell, |value| {
            count += 1.0;
            sum += value;
        }) {
            return error;
        }
        if count == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }
        CalcResult::Number(flush_subnormal(sum / count))
    }

    pub(crate) fn fn_avedev(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        let mut sum = 0.0;
        let mut count: u64 = 0;

        #[inline]
        fn accumulate(values: &mut Vec<f64>, sum: &mut f64, count: &mut u64, value: f64) {
            values.push(value);
            *sum += value;
            *count += 1;
        }

        if let Err(error) = self.for_each_value_skip_empty(args, cell, |value| {
            accumulate(&mut values, &mut sum, &mut count, value);
        }) {
            return error;
        }

        if count == 0 {
            // Excel: #NUM!, not #DIV/0! (docs/excel-semantics/coercion.md, poi EverythingTests!F176).
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "AVEDEV with no numeric data".to_string(),
            );
        }

        let n = count as f64;
        let mean = sum / n;

        let mut sum_abs_dev = 0.0;
        for v in &values {
            sum_abs_dev += (v - mean).abs();
        }

        CalcResult::Number(sum_abs_dev / n)
    }

    pub(crate) fn fn_median(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for MEDIAN".to_string(),
            };
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let n = values.len();
        let median = if n % 2 == 1 {
            // odd
            values[n / 2]
        } else {
            // even: average of the two middle values
            let a = values[(n / 2) - 1];
            let b = values[n / 2];
            // Halve first when the sum would overflow: Excel gives 9E+307 for two 9E+307 values.
            let sum = a + b;
            if sum.is_finite() {
                sum / 2.0
            } else {
                a / 2.0 + b / 2.0
            }
        };

        CalcResult::Number(median)
    }

    pub(crate) fn fn_harmean(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        if values.is_empty() {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        // Excel HARMEAN: all values must be > 0
        if values.iter().any(|&v| v <= 0.0) {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "HARMEAN requires strictly positive values".to_string(),
            };
        }

        let n = values.len() as f64;
        let sum_recip: f64 = values.iter().map(|v| 1.0 / v).sum();

        if sum_recip == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Division by Zero".to_string(),
            };
        }

        CalcResult::Number(n / sum_recip)
    }

    pub(crate) fn fn_mina(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut mina: Option<f64> = None;
        if let Err(e) = self.for_each_value_a(args, ArrayText::Skipped, cell, |f| {
            if let Some(m) = mina {
                mina = Some(m.min(f));
            } else {
                mina = Some(f);
            }
        }) {
            return e;
        }
        if let Some(mina) = mina {
            CalcResult::Number(mina)
        } else {
            CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No numeric values for MINA".to_string(),
            }
        }
    }

    pub(crate) fn fn_maxa(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }
        let mut maxa: Option<f64> = None;
        if let Err(e) = self.for_each_value_a(args, ArrayText::Skipped, cell, |f| {
            if let Some(m) = maxa {
                maxa = Some(m.max(f));
            } else {
                maxa = Some(f);
            }
        }) {
            return e;
        }
        if let Some(maxa) = maxa {
            CalcResult::Number(maxa)
        } else {
            CalcResult::Error {
                error: Error::VALUE,
                origin: cell,
                message: "No numeric values for MAXA".to_string(),
            }
        }
    }

    pub(crate) fn fn_skew(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Sample skewness (Excel SKEW)
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 3 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "SKEW requires at least 3 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in SKEW".to_string(),
            };
        }

        let s = (m2 / (n_f - 1.0)).sqrt();

        let mut sum_cubed = 0.0;
        for &x in &values {
            let z = (x - mean) / s;
            sum_cubed += z * z * z;
        }

        let skew = (n_f / ((n_f - 1.0) * (n_f - 2.0))) * sum_cubed;
        CalcResult::Number(skew)
    }

    pub(crate) fn fn_skew_p(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Population skewness (Excel SKEW.P)
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 2 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "SKEW.P requires at least 2 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in SKEW.P".to_string(),
            };
        }

        let sigma = (m2 / n_f).sqrt();

        let mut sum_cubed = 0.0;
        for &x in &values {
            let z = (x - mean) / sigma;
            sum_cubed += z * z * z;
        }

        let skew_p = sum_cubed / n_f;
        CalcResult::Number(skew_p)
    }

    pub(crate) fn fn_kurt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.is_empty() {
            return CalcResult::new_args_number_error(cell);
        }

        let mut values: Vec<f64> = Vec::new();
        if let Err(e) = self.for_each_value(args, cell, |f| values.push(f)) {
            return e;
        }

        let n = values.len();
        if n < 4 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "KURT requires at least 4 data points".to_string(),
            };
        }

        let n_f = n as f64;
        let mean = values.iter().sum::<f64>() / n_f;

        let mut m2 = 0.0;
        for &x in &values {
            let d = x - mean;
            m2 += d * d;
        }

        if m2 == 0.0 {
            return CalcResult::Error {
                error: Error::DIV,
                origin: cell,
                message: "Zero variance in KURT".to_string(),
            };
        }

        let s = (m2 / (n_f - 1.0)).sqrt();

        let mut sum_fourth = 0.0;
        for &x in &values {
            let z = (x - mean) / s;
            sum_fourth += z * z * z * z;
        }

        let term1 = (n_f * (n_f + 1.0)) / ((n_f - 1.0) * (n_f - 2.0) * (n_f - 3.0)) * sum_fourth;
        let term2 = 3.0 * (n_f - 1.0) * (n_f - 1.0) / ((n_f - 2.0) * (n_f - 3.0));

        let kurt = term1 - term2;
        CalcResult::Number(kurt)
    }

    /// The `k` of LARGE/SMALL: booleans and numeric text coerce (TRUE is 1) and anything below 1
    /// is `#NUM!` before rounding. A fractional k then picks the `ceil(k)`-th largest but the
    /// `floor(k)`-th smallest, and `#NUM!` when `ceil(k)` exceeds the count, so the callers
    /// round it themselves. Evidence: probes/large-small-k (excel-graph),
    /// docs/excel-semantics/statistics.md.
    fn large_small_k(&mut self, node: &Node, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let k = self.get_number(node, cell)?;
        if k < 1.0 {
            return Err(CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K must be >= 1".to_string(),
            });
        }
        Ok(k)
    }

    pub(crate) fn fn_large(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let values = match self.numbers_from_reference_arg(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let k = match self.large_small_k(&args[1], cell) {
            Ok(k) => k,
            Err(e) => return e,
        };
        let mut numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        if numeric_values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for LARGE".to_string(),
            };
        }
        numeric_values.sort_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
        let k_usize = k.ceil() as usize;
        if k_usize > numeric_values.len() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K is larger than the number of data points".to_string(),
            };
        }
        CalcResult::Number(numeric_values[k_usize - 1])
    }

    pub(crate) fn fn_small(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }

        let values = match self.numbers_from_reference_arg(&args[0], cell) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let k = match self.large_small_k(&args[1], cell) {
            Ok(k) => k,
            Err(e) => return e,
        };

        let mut numeric_values: Vec<f64> = values.into_iter().flatten().collect();
        if numeric_values.is_empty() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "No numeric values for SMALL".to_string(),
            };
        }

        // For SMALL, sort ascending
        numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        // Excel takes the floor for SMALL but still refuses a k whose ceiling is out of range.
        let k_usize = k.floor() as usize;
        if k.ceil() as usize > numeric_values.len() {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "K is larger than the number of data points".to_string(),
            };
        }

        CalcResult::Number(numeric_values[k_usize - 1])
    }
}
