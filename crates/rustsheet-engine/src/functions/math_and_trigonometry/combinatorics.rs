use crate::functions::lift::num_error;
use crate::functions::Function;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error,
    expressions::types::CellReferenceIndex, model::Model,
};

/// COMBIN's `number`, and COMBINA's `number + number_chosen - 1`, at or above which Excel gives
/// `#NUM!`, whatever the coefficient: 2^31-1 (probes/t2199-text-length-and-combin-cuts A20-A37,
/// B20-B34; numbers.md "COMBINA is COMBIN(n+k-1, k)", "Bound").
const COMBIN_MAX_NUMBER: f64 = crate::limits::COUNT_LIMIT_EXCLUSIVE;

impl Model {
    pub(crate) fn fn_fact(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.number_fn(Function::Fact, args, cell, |x: f64| {
            let x = x.floor();
            if x < 0.0 {
                return Err(Error::NUM);
            }
            descending_product(x, 1.0)
        })
    }
    /// `FACTDOUBLE(number)`, an Analysis `ToolPak` function: it lifts over an array and refuses a
    /// multi-cell range (probes/math-atp-array-lift C3-C4, scalar-lift-open-a Y21), and a boolean is
    /// `#VALUE!`, typed, in a cell or as an array element (probes/math-atp-array-lift-b E1-E3, E8;
    /// docs/excel-semantics/numbers.md "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes").
    pub(crate) fn fn_factdouble(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Factdouble, args, cell, &mut |model, values| {
            let [number] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let x = match model.cast_number_no_bools_ref(number, cell) {
                Ok(x) => x.floor(),
                Err(error) => return error,
            };
            if x < -1.0 {
                return num_error(cell);
            }
            if x < 0.0 {
                return CalcResult::Number(1.0);
            }
            match descending_product(x, 2.0) {
                Ok(product) => CalcResult::Number(product),
                Err(_) => num_error(cell),
            }
        })
    }

    /// COMBIN; both arguments lift over a range or an array, as COMBINA's do:
    /// `SUM(COMBIN(18,FILTER(..)))` and `SUM(COMBIN(D1:D3,1))` add the elements' results
    /// (probes/t2579-array-value-edges B60-B68, excel-graph; T-2579).
    pub(crate) fn fn_combin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Combin, args, cell, &mut |model, values| {
            let [n, k] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let n = match model.kernel_number(n, cell) {
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            let k = match model.kernel_number(k, cell) {
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            Self::combin_of(n, k, cell)
        })
    }

    /// COMBIN on a floored `n` and `k` (see `fn_combin`).
    fn combin_of(n: f64, k: f64, cell: CellReferenceIndex) -> CalcResult {
        if n < 0.0 || k < 0.0 || k > n || n >= COMBIN_MAX_NUMBER {
            return num_error(cell);
        }
        match combin(n, k) {
            Ok(result) => CalcResult::Number(result),
            Err(error) => CalcResult::Error {
                error,
                origin: cell,
                message: "Result is too large".to_string(),
            },
        }
    }

    pub(crate) fn fn_combina(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Combina, args, cell, &mut |model, values| {
            let [n, k] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            // `number` truncates toward zero before its sign check, and number_chosen floors, so
            // one in (-1, 0) is #NUM! (`COMBINA(7,1-1.005)`, `COMBINA(24,-0.0001)`; gen-m5
            // c688ca172fe533dc A118, gen-m6 44194b2fefa30de2 A222), except with a number of 0,
            // where it truncates too: `COMBINA(-0.5,-0.0001)` is 1 (probes/t2475-stat-subexpressions
            // P!A35; statistics.md "Statistical argument-domain errors and extreme values (T-2621)").
            let n = match model.kernel_number(n, cell) {
                Ok(f) => f.trunc(),
                Err(error) => return error,
            };
            let k = match model.kernel_number(k, cell) {
                Ok(f) if n == 0.0 => f.trunc(),
                Ok(f) => f.floor(),
                Err(error) => return error,
            };
            Self::combina_of(n, k, cell)
        })
    }

    /// COMBINA on an integral `n` and `k` (see `fn_combina`).
    fn combina_of(n: f64, k: f64, cell: CellReferenceIndex) -> CalcResult {
        if n < 0.0 || k < 0.0 || (n == 0.0 && k > 0.0) {
            return num_error(cell);
        }
        // COMBINA(n, k) is Excel's COMBIN(n + k - 1, k), rounding included: `COMBINA(20,19)` is
        // 35345263799.999985 (numbers.md "COMBINA is COMBIN(n+k-1, k)"), and so is its bound:
        // `COMBINA(2^31-2,1)` is 2147483646, `COMBINA(2^31-2,2)` and `COMBINA(2^31-1,1)` are #NUM!.
        // That also caps `combin`'s loop below 2^31 steps (min(k, n - 1)), and it stops at the first
        // infinity, within about 1,030 (the lifting table's weight, `lift/kinds/general.rs`).
        if n + k - 1.0 >= COMBIN_MAX_NUMBER {
            return num_error(cell);
        }
        match combin(n + k - 1.0, k) {
            Ok(result) => CalcResult::Number(result),
            Err(error) => CalcResult::Error {
                error,
                origin: cell,
                message: "Result is too large".to_string(),
            },
        }
    }
}

/// COMBIN(n, k) for integers `0 <= k <= n`, rounded as Excel rounds it. With `j = min(k, n-k)`,
/// Excel multiplies the factors `(n-j+i)/(i+1)` for i = 1..j-1, each a rounded quotient, into one
/// double running product and then multiplies by `n`: `COMBIN(10,4)` is 209.99999999999997, not 210.
/// This matches all 841 numeric cells of probes/combin-algorithm-sweep bit for bit (numbers.md
/// "COMBIN is inexact"). Every factor is at least 1, so the product only grows: it overflows (`#NUM!`,
/// as `COMBIN(1030,515)` is in Excel) within a few hundred
/// factors whenever `j` is large, which bounds the work (directive 5).
fn combin(n: f64, k: f64) -> Result<f64, Error> {
    let j = k.min(n - k);
    if j < 1.0 {
        return Ok(1.0);
    }
    let mut acc = 1.0;
    let mut i = 1.0;
    while i < j {
        acc *= (n - j + i) / (i + 1.0);
        if !acc.is_finite() {
            return Err(Error::NUM);
        }
        i += 1.0;
    }
    let result = acc * n;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(Error::NUM)
    }
}

/// `n * (n - step) * (n - 2 step) * ...` down to 1 or 2, multiplied from `n` down, which is the
/// order Excel's FACT (`step` 1) and FACTDOUBLE (`step` 2) round in: `FACT(170)` is
/// 7.257415615308004e306 where the ascending product gives ...7994e306 (probes/fact-product-order-sweep,
/// 420 cells bit-exact; docs/excel-semantics/numbers.md "FACT and FACTDOUBLE"). `#NUM!` once the
/// product overflows (`FACT(171)`, `FACTDOUBLE(301)`); the first factors are the largest, so a huge
/// `n` overflows within a few steps and the loop stays bounded (directive 5).
fn descending_product(n: f64, step: f64) -> Result<f64, Error> {
    let mut acc = 1.0;
    let mut k = n;
    while k > 1.0 {
        acc *= k;
        if !acc.is_finite() {
            return Err(Error::NUM);
        }
        k -= step;
    }
    Ok(acc)
}
