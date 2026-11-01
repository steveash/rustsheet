use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::functions::Function;
use crate::x87_exp::{excel_exp, excel_expm1};
use crate::x87_log10::excel_ln;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

impl Model {
    /// `WEIBULL.DIST(x, alpha, beta, cumulative)`, and WEIBULL: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H552-H563,
    /// probes/stat-later-argument-lift H45, H47). The kernel is closed form.
    pub(crate) fn fn_weibull_dist(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(Function::WeibullDist, args, cell, &mut |model, values| {
            number_or_error(model.weibull_dist_of(values, cell))
        })
    }

    fn weibull_dist_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [x, alpha, beta, cumulative] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // A TRUE x is 1: WEIBULL(TRUE,1,1,TRUE) = 0.6321205588285577 (probes-numeric/t1048-weibull
        // A25, t1049-weibulldist A33).
        let x = self.cast_number_ref(x, cell)?;
        // A boolean alpha or beta reads as 1 or 0 (gen-m9/d37313406b9f6e55 A380, 928d08e3e1eecb5f A303).
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta = self.cast_number_ref(beta, cell)?;
        let cumulative = Self::cast_bool_ref(cumulative, cell)?;
        if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
            return Err(num_error(cell));
        }
        // The density and the cdf at x = 0 are 0 for every shape (t1048-weibull A16, A17;
        // t1049-weibulldist A27, A28; `WEIBULL.DIST(0,1E+8,4.6E-09,TRUE)` is 0 where
        // beta^-alpha overflows, probes/t2475-stat-subexpressions P!A78).
        if x == 0.0 {
            return Ok(0.0);
        }
        // Excel reports the overflow of (x/beta)^alpha, though the cdf's limit is 1:
        // `WEIBULL(1E+15,1E+15,TRUE,1)` and `WEIBULL.DIST(7,1E+8,0.3,"TRUE")` are #NUM!
        // (probes/t2475-stat-subexpressions P!A407, probes/t2496-fin-subexpressions P!A90),
        // as are `WEIBULL.DIST(100,1E+10,0.01,1)` and `WEIBULL(1E+300,2.5,100,0.5)`
        // (gen-m9/0bbab6206b5c9a07 A340, 329b177b5d5165ad A101).
        let t = scaled_power(x, alpha, beta);
        if !t.is_finite() {
            return Err(num_error(cell));
        }
        let result = if cumulative {
            weibull_cdf(t)
        } else {
            weibull_density(x, alpha, beta, t, cell)?
        };
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}

/// t = (x/beta)^alpha as Excel evaluates it, `EXP(alpha*LN(x/beta))` with the worksheet LN and
/// EXP kernels, not a correctly rounded power. Its rounding shows in the far tail, where Excel's
/// density is less accurate than exp(-t): `WEIBULL.DIST(100,1,1,FALSE)` is
/// 3.7200759760206773E-44, not exp(-100) = 3.720075976020836E-44 (probes-numeric/m9-weibull-dist
/// A64; docs/excel-semantics/statistics.md "The far-tail density is less accurate than exp(-t)").
/// The cdf takes x/beta, not x^alpha/beta^alpha: `WEIBULL(1E+15,42,10000000001,TRUE)` is a number
/// though 10000000001^42 overflows (gen-m9/7f7ca726c3f68554 A308) and `WEIBULL(0.2,1E+15,0.2,TRUE)`
/// is 0.6321205588285577 (probes/t2436/abc2939ebd1e2365 A196).
fn scaled_power(x: f64, alpha: f64, beta: f64) -> f64 {
    let ratio = x / beta;
    // An x/beta that underflows to 0 keeps t = 0, its limit, as the engine's earlier powf did
    // (unprobed; LN(0) itself would be #NUM!).
    if ratio == 0.0 {
        return 0.0;
    }
    excel_exp(alpha * excel_ln(ratio))
}

/// The cdf 1 - exp(-t): below t = 1 as minus Excel's expm1 of -t (`excel_expm1`, Kahan's formula),
/// which keeps the small-x cells (`WEIBULL.DIST(0.1,3.7,9,TRUE)` is 5.878989740656108E-08,
/// m9-weibull-dist A159), and from there on as `1-EXP(-t)` (`WEIBULL.DIST(30,2,9,TRUE)` is
/// 0.9999850546614752, A127). probes-numeric/t2239-weibull-cdf-switch puts the switch between
/// t = 0.99868 and 1.35; the two forms agree on every probed cell in between
/// (docs/excel-semantics/statistics.md "WEIBULL.DIST's cdf switches from expm1 to 1-EXP(-t) near t = 1").
fn weibull_cdf(t: f64) -> f64 {
    if t < 1.0 {
        -excel_expm1(-t)
    } else {
        1.0 - excel_exp(-t)
    }
}

/// The density as Excel evaluates it, `alpha / beta^alpha * x^(alpha-1) * exp(-t)`, each power
/// `EXP(y*LN(b))` with the worksheet kernels. That divisor shows in the values (`WEIBULL(10,3,8,FALSE)`
/// is 0.08310360884023987 where the true 0.08310360884023976 is what `alpha/beta*(x/beta)^(alpha-1)`
/// gives; probes-numeric/t1048-weibull A8, B8) and in the errors: a `beta^alpha` that underflows to 0
/// is `#DIV/0!` (`WEIBULL.DIST(0.1,10000000000.5,0.2,FALSE)`, gen-m9/941c409532603d32 A142) and one
/// that overflows is `#NUM!`, even where the density's limit is 0 (`WEIBULL.DIST(0.3,99999,99999,"FALSE")`,
/// gen-m9/a63d1c7d6f96c56d A352; `WEIBULL(1,1E+8,1E+10,FALSE)`, 7fd0a98183c6babf A390).
fn weibull_density(
    x: f64,
    alpha: f64,
    beta: f64,
    t: f64,
    cell: CellReferenceIndex,
) -> Result<f64, CalcResult> {
    let divisor = excel_exp(alpha * excel_ln(beta));
    if divisor == 0.0 {
        return Err(div_error(cell));
    }
    if !divisor.is_finite() {
        return Err(num_error(cell));
    }
    let density = alpha / divisor * excel_exp((alpha - 1.0) * excel_ln(x)) * excel_exp(-t);
    // A subnormal density is 0, as in Excel (`(x)=0` TRUE: probes/t2693-subnormal-weibull A2;
    // docs/excel-semantics/numbers.md "Distribution densities").
    Ok(flush_subnormal(density))
}
