use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::functions::statistical::special::normal_cdf;

use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// Z.TEST's p-value: NORM.S.DIST(-z,TRUE), not 1 - NORM.S.DIST(z,TRUE), with
/// z = fl(fl(mean - x)*fl(sqrt(n)))/sigma, every operation rounded twice through x87 extended
/// precision (`extended_*`). Each choice is Excel's: the NORM.S.DIST(-z) form matches 120 of 120
/// cells of probes-numeric/t1916-erf-wrappers column I (19 for 1 - NORM.S.DIST(z)), and this z,
/// against (mean - x)/(sigma/sqrt(n)), matches 240 of 240 there (columns I and J, 141 for the
/// plain-double `(mean - x)*sqrt(n)/sigma`) and the 30 numeric cells of t1048-ztest and t1049-ztest
/// (24 for the quotient form). docs/excel-semantics/statistics.md, T-1778 section, "Engine (T-1929)".
fn z_test_p(mean: f64, x: f64, n: f64, sigma: f64) -> f64 {
    let z = extended_quotient(
        extended_product(extended_sum(mean, -x), extended_sqrt(n)),
        sigma,
    );
    normal_cdf(-z, 0.0, 1.0)
}

/// Z.TEST's data argument read once: its numbers' count, mean and sample standard deviation.
struct ZTestData {
    count: u64,
    mean: f64,
    /// The sample standard deviation, `None` when the variance is not positive.
    stdev: Option<f64>,
}

impl Model {
    /// Z.TEST's data: the numbers of a range or an array (text and booleans skipped), or a
    /// number; anything else is `#VALUE!`.
    fn z_test_data(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<ZTestData, CalcResult> {
        let values = match self.evaluate_node_in_context(node, cell) {
            CalcResult::Range { left, right } => self.values_from_range(left, right)?,
            CalcResult::Array(array) => self
                .values_from_array(array)
                .map_err(|_| CalcResult::new_error(Error::VALUE, cell, String::new()))?,
            CalcResult::Number(v) => vec![Some(v)],
            error @ CalcResult::Error { .. } => return Err(error),
            _ => return Err(CalcResult::new_error(Error::VALUE, cell, String::new())),
        };
        let mut sum = 0.0;
        let mut count: u64 = 0;
        for x in values.iter().flatten() {
            sum += x;
            count += 1;
        }
        let n = count as f64;
        let mean = sum / n;
        let mut sumsq_dev = 0.0;
        for x in values.iter().flatten() {
            let d = x - mean;
            sumsq_dev += d * d;
        }
        let var = sumsq_dev / (n - 1.0);
        Ok(ZTestData {
            count,
            mean,
            stdev: (var > 0.0).then(|| var.sqrt()),
        })
    }

    /// `Z.TEST(array, x, [sigma])` and ZTEST: `x` and `sigma` lift over an array and a multi-cell
    /// range, the data is read once (SPEC-0011, T-1813). An element reads `x` and `sigma` as
    /// numbers, then checks the data, then `sigma`'s sign: `Z.TEST(D1:D3,"a")` with D2 = `#N/A`
    /// is `#VALUE!`, `Z.TEST({"a","b"},"x")` `#VALUE!`, `Z.TEST(D1:D3,4,0)` `#N/A`, and a lifted
    /// element with a good `x` is the data's `#N/A` (probes/stat-test-lift P45, P86;
    /// stat-test-lift-b P92, P93). `x` and `sigma` coerce
    /// numeric text and booleans, and a reference to other text is `#VALUE!` (P46-P51).
    pub(crate) fn fn_z_test(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let kind = Function::ZTest;
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let data = match args.first() {
            Some(node) => self
                .z_test_data(node, cell)
                .map_err(CalcResult::without_message),
            None => return CalcResult::new_args_number_error(cell),
        };
        self.lift_call(kind, args, cell, &mut |model, values| {
            z_test_element(model, &data, values, cell)
        })
    }
}

/// One Z.TEST element over the data read once.
fn z_test_element(
    model: &Model,
    data: &Result<ZTestData, CalcResult>,
    values: &[CalcResult],
    cell: CellReferenceIndex,
) -> CalcResult {
    let error = |error| CalcResult::new_error(error, cell, String::new());
    let x = match values.get(1).map(|x| model.kernel_number(x, cell)) {
        Some(Ok(x)) => x,
        Some(Err(error)) => return error,
        None => return CalcResult::new_args_number_error(cell),
    };
    let sigma = match values.get(2).map(|sigma| model.kernel_number(sigma, cell)) {
        Some(Ok(sigma)) => Some(sigma),
        Some(Err(error)) => return error,
        None => None,
    };
    let data = match data {
        Ok(data) => data,
        Err(error) => return error.clone(),
    };
    // Excel: an array with no numeric values is #N/A.
    if data.count == 0 {
        return error(Error::NA);
    }
    // A sigma of 0 or below is #NUM! (`Z.TEST(B1:B10,5,-1)`, probes-numeric/t1049-ztest
    // A26, t1048-ztest A26; probes/stat-test-lift P44, P52), after the data's error:
    // `Z.TEST(D1:D3,4,0)` with D2 = `#N/A` is `#N/A` (probes/stat-test-lift-b P93).
    if sigma.is_some_and(|sigma| sigma <= 0.0) {
        return error(Error::NUM);
    }
    // One numeric value is #DIV/0!, with or without sigma (`Z.TEST(D1:D1,4,1)`,
    // `Z.TEST(7,4,1)`: probes-numeric/t1049-ztest A40, A42; t1916-erf-wrappers column T).
    if data.count <= 1 {
        return error(Error::DIV);
    }
    // If sigma is omitted, the sample standard deviation STDEV(array).
    let Some(sigma) = sigma.or(data.stdev) else {
        return error(Error::DIV);
    };
    let n = data.count as f64;
    // A sigma/sqrt(n) that underflows is #DIV/0! (not probed; kept from the fork, TODO(T-1942)).
    // The guard keeps the fork's plain-double test so it fires exactly where main's did; the
    // result itself takes Excel's x87 steps (`z_test_p`), which never divide by this quotient.
    if sigma / n.sqrt() == 0.0 {
        return error(Error::DIV);
    }
    // A NaN p (an omitted sigma over data whose sum overflows) is #NUM!, not a NaN number
    // (not probed, TODO(T-1942)).
    let p = z_test_p(data.mean, x, n, sigma);
    if !p.is_finite() {
        return error(Error::NUM);
    }
    CalcResult::Number(p)
}
