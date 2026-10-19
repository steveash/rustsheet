use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

use super::number_or_error;
use crate::functions::lift::{div_error, num_error};

// ---------------------------------------------------------------------------
// VDB, DB and DDB (AMORDEGRC and AMORLINC are in `amortization.rs`)
//
// Port of the depreciation calculations as implemented in the
// `ExcelFinancialFunctions` library (depreciation.fs).
// See https://github.com/fsprojects/ExcelFinancialFunctions
// ---------------------------------------------------------------------------

// `totalDepr`: cumulative depreciation between period 0 and `period`.
//
// Walks the declining-balance schedule one (integer) period at a time,
// optionally switching to straight line when that yields a larger amount,
// and prorates the contribution of the period that contains the fractional
// part of `period`.
fn vdb_total_depr(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    factor: f64,
    straight_line: bool,
) -> f64 {
    let frac = period - period.trunc();
    let int_period = period.trunc() as i64;
    let int_life = life.trunc() as i64;

    let ddb_unclipped = |tot: f64| (cost - tot) * (factor / life);
    let ddb_formula = |tot: f64| ddb_unclipped(tot).min(cost - salvage - tot);
    let sln_formula = |tot: f64, a_period: f64| (cost - tot - salvage) / (life - a_period);

    // Salvage above cost never switches: VDB(2400,3000,10,0,1) is -600 (the declining-balance
    // charge `cost - salvage`), not the straight-line -60 (probes/fin-vdb-modes A17).
    let switches = salvage <= cost;
    // A fractional period prorates the unclipped charge, then the total stops at `cost - salvage`:
    // VDB(2400,300,10,9.5,10) is 0 (probes/fin-vdb-modes A9).
    let cap = cost - salvage;

    let mut tot_depr = 0.0;
    let mut per = 0.0_f64;
    loop {
        let ddb_depr = ddb_formula(tot_depr);
        let sln_depr = sln_formula(tot_depr, per);
        let is_sln = straight_line && switches && ddb_depr < sln_depr;
        let depr = if is_sln { sln_depr } else { ddb_depr };
        let new_total = tot_depr + depr;

        if int_period == 0 {
            return new_total * frac;
        } else if per.trunc() as i64 == int_period - 1 {
            let ddb_next = ddb_formula(new_total);
            let sln_next = sln_formula(new_total, per + 1.0);
            let is_sln_next = straight_line && switches && ddb_next < sln_next;
            let depr_next = if is_sln_next {
                if int_period == int_life {
                    0.0
                } else {
                    sln_next
                }
            } else {
                // The prorated part takes the unclipped charge; the cap below clips the total.
                ddb_unclipped(new_total)
            };
            let prorated = new_total + depr_next * frac;
            return if switches && prorated > cap {
                cap
            } else {
                prorated
            };
        } else {
            tot_depr = new_total;
            per += 1.0;
        }
    }
}

fn vdb_depr_between(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    straight_line: bool,
) -> f64 {
    vdb_total_depr(cost, salvage, life, end_period, factor, straight_line)
        - vdb_total_depr(cost, salvage, life, start_period, factor, straight_line)
}

// `db`: fixed-declining-balance depreciation for a single `period`.
pub(super) fn db_depreciation(cost: f64, salvage: f64, life: f64, period: f64, month: f64) -> f64 {
    // The depreciation rate, rounded to three decimal places.
    let rate = ((1.0 - (salvage / cost).powf(1.0 / life)) * 1000.0).round() / 1000.0;
    let int_period = period.trunc() as i64;
    let int_life = life.trunc() as i64;

    // First period (also covers a fractional period below 1).
    let mut accumulated = cost * rate * month / 12.0;
    if int_period <= 1 {
        return accumulated;
    }
    // Accumulate the declining balance through the period before the one asked.
    for _ in 2..int_period {
        accumulated += (cost - accumulated) * rate;
    }
    if int_period == int_life + 1 {
        (cost - accumulated) * rate * (12.0 - month) / 12.0
    } else {
        (cost - accumulated) * rate
    }
}

// `ddb`: double-declining-balance depreciation for a single `period`.
// (see LibreOffice's `ScGetDDB`)
pub(super) fn ddb_depreciation(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    factor: f64,
) -> f64 {
    // Excel special-cases a fractional period in (0, 1), treating it as period 1
    // (`period` is already guaranteed > 0 by the caller's validation).
    let period = if period < 1.0 { 1.0 } else { period };

    let mut rate = factor / life;
    let old_value = if rate >= 1.0 {
        rate = 1.0;
        if period == 1.0 {
            cost
        } else {
            0.0
        }
    } else {
        cost * (1.0 - rate).powf(period - 1.0)
    };
    let new_value = cost * (1.0 - rate).powf(period);

    let ddb = if new_value < salvage {
        old_value - salvage
    } else {
        old_value - new_value
    };
    ddb.max(0.0)
}

impl Model {
    // VDB(cost, salvage, life, start_period, end_period, [factor], [no_switch])
    //
    // Variable declining balance depreciation between `start_period` and
    // `end_period`. By default (no_switch = FALSE) the calculation switches to
    // straight-line depreciation when that is more advantageous; passing
    // no_switch = TRUE keeps it on declining balance throughout.
    pub(crate) fn fn_vdb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let arg_count = args.len();
        if !(5..=7).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        let cost = match self.get_number(&args[0], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let salvage = match self.get_number(&args[1], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let life = match self.get_number(&args[2], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let start_period = match self.get_number(&args[3], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let end_period = match self.get_number(&args[4], cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let factor = if arg_count > 5 {
            match self.get_number(&args[5], cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            2.0
        };
        // no_switch defaults to FALSE => switch to straight line.
        let no_switch = if arg_count > 6 {
            match self.get_number(&args[6], cell) {
                Ok(f) => f != 0.0,
                Err(s) => return s,
            }
        } else {
            false
        };
        if cost < 0.0
            || salvage < 0.0
            || life <= 0.0
            || factor <= 0.0
            || start_period < 0.0
            || start_period > life
            || end_period > life
            || start_period > end_period
            || end_period <= 0.0
        {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        // When switching to straight line, life, start and end cannot all coincide.
        if !no_switch && life == start_period && start_period == end_period {
            return CalcResult::new_error(Error::NUM, cell, "invalid parameters".to_string());
        }
        let straight_line = !no_switch;
        let result = vdb_depr_between(
            cost,
            salvage,
            life,
            start_period,
            end_period,
            factor,
            straight_line,
        );
        CalcResult::Number(result)
    }
}

// SLN, SYD and DDB are core worksheet functions in closed form: every argument lifts over an
// array and a multi-cell range (`ArgKind::Lift`, SPEC-0011; probes/fin-argument-modes H97, H304,
// H309, probes/fin-later-argument-ranges K19-K23). Moved here from `financial/mod.rs` (T-1372).
// DB and VDB loop over the periods, so they stay off the driver until their loops are bounded.
impl Model {
    /// `SLN(cost, salvage, life)`: `(cost - salvage)/life`.
    pub(crate) fn fn_sln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Sln, args, cell, &mut |model, values| {
            let [cost, salvage, life] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let read = || -> Result<f64, CalcResult> {
                let cost = model.cast_number_ref(cost, cell)?;
                let salvage = model.cast_number_ref(salvage, cell)?;
                let life = model.cast_number_ref(life, cell)?;
                if life == 0.0 {
                    return Err(div_error(cell));
                }
                Ok((cost - salvage) / life)
            };
            number_or_error(read())
        })
    }

    /// `SYD(cost, salvage, life, per)`: `((cost - salvage)*(life - per + 1)*2)/(life*(life + 1))`.
    pub(crate) fn fn_syd(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Syd, args, cell, &mut |model, values| {
            let [cost, salvage, life, per] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let read = || -> Result<f64, CalcResult> {
                let cost = model.cast_number_ref(cost, cell)?;
                let salvage = model.cast_number_ref(salvage, cell)?;
                let life = model.cast_number_ref(life, cell)?;
                let per = model.cast_number_ref(per, cell)?;
                if life == 0.0 || per > life || per <= 0.0 {
                    return Err(num_error(cell));
                }
                Ok(((cost - salvage) * (life - per + 1.0) * 2.0) / (life * (life + 1.0)))
            };
            number_or_error(read())
        })
    }

    /// `DDB(cost, salvage, life, period, [factor])`; `factor` is 2 when omitted (the
    /// double-declining balance), and a boolean factor is `#VALUE!`.
    pub(crate) fn fn_ddb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Ddb, args, cell, &mut |model, values| {
            let [cost, salvage, life, period, rest @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let read = || -> Result<f64, CalcResult> {
                let cost = model.cast_number_ref(cost, cell)?;
                let salvage = model.cast_number_ref(salvage, cell)?;
                let life = model.cast_number_ref(life, cell)?;
                let period = model.cast_number_ref(period, cell)?;
                let factor = model.cast_number_no_bools_or(rest.first(), 2.0, cell)?;
                if period > life || cost < 0.0 || salvage < 0.0 || period <= 0.0 || factor <= 0.0 {
                    return Err(num_error(cell));
                }
                Ok(ddb_depreciation(cost, salvage, life, period, factor))
            };
            number_or_error(read())
        })
    }
}
