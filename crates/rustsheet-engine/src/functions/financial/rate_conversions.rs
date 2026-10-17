//! EFFECT and NOMINAL (an interest rate converted between nominal and effective) and DOLLARDE and
//! DOLLARFR (a price converted between fractional and decimal dollars). They are Analysis
//! `ToolPak` functions, so both arguments lift over an array and a multi-cell range is `#VALUE!`
//! in either, with no intersection (`ArgKind::LiftArrays`, SPEC-0011): probes/fin-argument-modes,
//! probes/scalar-lift-shapes H11, H12 and -b H, I, probes/scalar-lift-open-a W3, W4, Y16-Y18,
//! probes/fin-later-argument-ranges H1 (docs/excel-semantics/financial.md "Financial function
//! argument modes"). Moved out of `financial/mod.rs` (T-1372).

use super::number_or_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// The `#NUM!` a kernel's own check gives; it allocates no message.
fn num_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NUM, cell, String::new())
}

/// The `#DIV/0!` a kernel's own check gives.
fn div_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::DIV, cell, String::new())
}

impl Model {
    /// The two numbers of EFFECT, NOMINAL, DOLLARDE and DOLLARFR, in order; a boolean is
    /// `#VALUE!`.
    fn two_numbers(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(f64, f64), CalcResult> {
        let [first, second] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        Ok((
            self.cast_number_no_bools_ref(first, cell)?,
            self.cast_number_no_bools_ref(second, cell)?,
        ))
    }

    /// `NOMINAL(effect_rate, npery)`: `n*((1+r)^(1/n) - 1)`, with `n` floored.
    pub(crate) fn fn_nominal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Nominal, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .two_numbers(values, cell)
                    .and_then(|(effect_rate, npery)| {
                        let npery = npery.floor();
                        if effect_rate <= 0.0 || npery < 1.0 {
                            return Err(num_error(cell));
                        }
                        rate_result(((1.0 + effect_rate).powf(1.0 / npery) - 1.0) * npery, cell)
                    }),
            )
        })
    }

    /// `EFFECT(nominal_rate, npery)`: `(1+r/n)^n - 1`, with `n` floored.
    pub(crate) fn fn_effect(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Effect, args, cell, &mut |model, values| {
            number_or_error(
                model
                    .two_numbers(values, cell)
                    .and_then(|(nominal_rate, npery)| {
                        let npery = npery.floor();
                        if nominal_rate <= 0.0 || npery < 1.0 {
                            return Err(num_error(cell));
                        }
                        rate_result((1.0 + nominal_rate / npery).powf(npery) - 1.0, cell)
                    }),
            )
        })
    }

    /// `DOLLARDE(fractional_dollar, fraction)`.
    pub(crate) fn fn_dollarde(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dollarde, args, cell, &mut |model, values| {
            number_or_error(model.two_numbers(values, cell).and_then(
                |(fractional_dollar, fraction)| {
                    let a_base = fraction_base(fraction, cell)?;
                    // See LibreOffice/ExcelFinancialFunctions: the fractional part is scaled
                    // by 10^ceil(log10(a_base)) and divided by the fraction base.
                    let dollar = fractional_dollar.trunc();
                    let remainder = fractional_dollar - dollar;
                    let digits = 10f64.powf(a_base.log10().ceil());
                    Ok(remainder * digits / a_base + dollar)
                },
            ))
        })
    }

    /// `DOLLARFR(decimal_dollar, fraction)`.
    pub(crate) fn fn_dollarfr(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dollarfr, args, cell, &mut |model, values| {
            number_or_error(model.two_numbers(values, cell).and_then(
                |(decimal_dollar, fraction)| {
                    let a_base = fraction_base(fraction, cell)?;
                    // Inverse of DOLLARDE: the decimal remainder is expressed in fractional
                    // units (a_base) and scaled down by 10^ceil(log10(a_base)).
                    let dollar = decimal_dollar.trunc();
                    let remainder = decimal_dollar - dollar;
                    let digits = 10f64.powf(a_base.log10().ceil());
                    Ok(remainder * a_base / digits.abs() + dollar)
                },
            ))
        })
    }
}

/// A converted rate, `#DIV/0!` when infinite and `#NUM!` when NaN.
fn rate_result(result: f64, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    if result.is_infinite() {
        return Err(div_error(cell));
    }
    if result.is_nan() {
        return Err(num_error(cell));
    }
    Ok(result)
}

/// DOLLARDE's and DOLLARFR's `fraction`, truncated: negative is `#NUM!`, below 1 `#DIV/0!`.
fn fraction_base(fraction: f64, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    if fraction < 0.0 {
        return Err(num_error(cell));
    }
    let a_base = fraction.trunc();
    if a_base < 1.0 {
        return Err(div_error(cell));
    }
    Ok(a_base)
}
