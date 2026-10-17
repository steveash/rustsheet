//! The Bessel functions and ERF, ERFC and their .PRECISE forms. Every argument lifts over an
//! array and refuses a multi-cell range (`ArgKind::LiftArrays`, SPEC-0011;
//! probes/eng-argument-modes E1:E4, E19:E22, probes/scalar-lift-shapes H8,
//! probes/eng-later-argument-lift H1:H3, H19). The kernels build their own errors with empty
//! messages.

use crate::arithmetic::flush_subnormal;
use crate::functions::statistical::special::{erf, erfc};

use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

use super::transcendental::{bessel_i, bessel_k, excel_bessel_j, excel_bessel_y};
// https://root.cern/doc/v610/TMath_8cxx_source.html

// Notice that the parameters for Bessel functions in Excel and here have inverted order
// EXCEL_BESSEL(x, n) => bessel(n, x)

/// One Bessel function of the order `n` and the argument `x`.
type BesselFn = fn(i32, f64) -> f64;

impl Model {
    /// `BESSELI(x, n)`.
    pub(crate) fn fn_besseli(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Besseli, args, cell, &mut |model, values| {
            model.bessel_of(values, bessel_i, cell)
        })
    }

    /// `BESSELJ(x, n)`.
    pub(crate) fn fn_besselj(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Besselj, args, cell, &mut |model, values| {
            model.bessel_of(values, excel_bessel_j, cell)
        })
    }

    /// `BESSELK(x, n)`.
    pub(crate) fn fn_besselk(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Besselk, args, cell, &mut |model, values| {
            model.bessel_of(values, bessel_k, cell)
        })
    }

    /// `BESSELY(x, n)`.
    pub(crate) fn fn_bessely(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bessely, args, cell, &mut |model, values| {
            model.bessel_of(values, excel_bessel_y, cell)
        })
    }

    /// One Bessel element: `x`, then the order `n`, truncated; a negative order (Excel: `#NUM!`)
    /// and a result that is not finite are `#NUM!`. The order is capped inside `bessel`
    /// (`transcendental::MAX_ORDER`), which bounds the work of one call.
    fn bessel_of(
        &self,
        values: &[CalcResult],
        bessel: BesselFn,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let [x, n] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let x = match self.cast_number_no_bools_ref(x, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let n = match self.cast_number_no_bools_ref(n, cell) {
            Ok(f) => f.trunc() as i32,
            Err(s) => return s,
        };
        if n < 0 {
            return CalcResult::new_error(Error::NUM, cell, String::new());
        }
        let result = bessel(n, x);
        if result.is_infinite() || result.is_nan() {
            return CalcResult::new_error(Error::NUM, cell, String::new());
        }
        CalcResult::Number(result)
    }

    /// `ERF(lower_limit, [upper_limit])`.
    pub(crate) fn fn_erf(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Erf, args, cell, &mut |model, values| {
            let (lower, upper) = match values {
                [lower] => (lower, None),
                [lower, upper] => (lower, Some(upper)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            let x = match model.cast_number_no_bools_ref(lower, cell) {
                Ok(f) => f,
                Err(s) => return s,
            };
            match upper.map(|upper| model.cast_number_no_bools_ref(upper, cell)) {
                Some(Ok(y)) => CalcResult::Number(erf(y) - erf(x)),
                Some(Err(s)) => s,
                None => CalcResult::Number(erf(x)),
            }
        })
    }

    /// `ERF.PRECISE(x)`.
    pub(crate) fn fn_erfprecise(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ErfPrecise, args, cell, &mut |model, values| {
            model.unary_number_of(values, erf, cell)
        })
    }

    /// `ERFC(x)`.
    pub(crate) fn fn_erfc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Erfc, args, cell, &mut |model, values| {
            model.unary_number_of(values, |x| flush_subnormal(erfc(x)), cell)
        })
    }

    /// `ERFC.PRECISE(x)`.
    pub(crate) fn fn_erfcprecise(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::ErfcPrecise, args, cell, &mut |model, values| {
            model.unary_number_of(values, |x| flush_subnormal(erfc(x)), cell)
        })
    }

    /// `f` of the one number in `values`, a boolean being `#VALUE!`.
    fn unary_number_of(
        &self,
        values: &[CalcResult],
        f: fn(f64) -> f64,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let [x] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        match self.cast_number_no_bools_ref(x, cell) {
            Ok(x) => CalcResult::Number(f(x)),
            Err(s) => s,
        }
    }
}
