//! RRI and PDURATION: the growth rate and the number of periods that take `pv` to `fv`.

use crate::functions::lift::{div_error, num_error, number_or_error};
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;
use crate::{
    arithmetic::{extended_product, extended_quotient, extended_sum},
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

impl Model {
    /// `RRI(nper, pv, fv)`: Excel's `EXP(LN(fv/pv)*(1/nper))-1`, and `fv/pv-1` at nper 1.
    pub(crate) fn fn_rri(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Rri, args, cell, &mut |model, values| {
            number_or_error(model.rri_of(values, cell))
        })
    }

    fn rri_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        let [nper, pv, fv] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let nper = self.cast_number_ref(nper, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_ref(fv, cell)?;
        if nper <= 0.0 {
            return Err(num_error(cell));
        }
        // When the value is unchanged the rate is zero (this also covers the
        // pv == fv == 0 case, which Excel returns as 0 rather than an error).
        if fv == pv {
            return Ok(0.0);
        }
        if pv == 0.0 {
            // Note error is NUM not DIV/0 also below
            return Err(num_error(cell));
        }
        // Excel computes `EXP(LN(fv/pv)*(1/nper))-1` with its own `/`, LN, `*`, EXP and `-`,
        // taking the reciprocal first: `RRI(0.5,3,4)` is 0.7777777777777775 where `(4/3)^2-1` is
        // ...777 (RRI.xlsx D12). The form fits 134 of the 136 numeric points of the T-2216 sweep,
        // the other two being the fv = 0 edges below (probes-numeric/m9-fin-rri-sweep; `^` fits
        // 112, `/nper` 87), and probes/fin-rri and fin-mduration-rri-fit A37-A59. Unlike `^`, a
        // negative ratio is always #NUM!: `RRI(1,-100,200)` is #NUM! where `(200/-100)^1-1` is -3,
        // and so are odd roots (probes/fin-rri-vdb-edges A1-A16).
        let ratio = extended_quotient(fv, pv);
        if ratio < 0.0 {
            return Err(num_error(cell));
        }
        // fv = 0 is -1 although LN(0) is #NUM! (m9-fin-rri-sweep rows 131-139).
        if ratio == 0.0 {
            return Ok(-1.0);
        }
        // nper = 1 exactly skips EXP/LN: `RRI(1,826,40633)` is 48.19249394673123, `40633/826-1`,
        // where EXP(LN(r)) is one ulp higher, while nper 1 ± 1 ulp follow the EXP/LN form
        // (probes-numeric/m9-fin-rri-nper-one: 32 of 32 nper-1 rows, rows 33-34).
        let growth = if nper == 1.0 {
            ratio
        } else {
            excel_exp(extended_product(
                excel_ln(ratio),
                extended_quotient(1.0, nper),
            ))
        };
        let result = extended_sum(growth, -1.0);
        if !result.is_finite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `PDURATION(rate, pv, fv)`: `(ln(fv) - ln(pv))/ln(1+r)`.
    pub(crate) fn fn_pduration(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Pduration, args, cell, &mut |model, values| {
            number_or_error(model.pduration_of(values, cell))
        })
    }

    fn pduration_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [rate, pv, fv] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let rate = self.cast_number_ref(rate, cell)?;
        let pv = self.cast_number_ref(pv, cell)?;
        let fv = self.cast_number_ref(fv, cell)?;
        if fv <= 0.0 || pv <= 0.0 || rate <= 0.0 {
            return Err(num_error(cell));
        }
        let result = (fv.ln() - pv.ln()) / ((1.0 + rate).ln());
        if result.is_infinite() {
            return Err(div_error(cell));
        }
        if result.is_nan() {
            return Err(num_error(cell));
        }
        Ok(result)
    }
}
