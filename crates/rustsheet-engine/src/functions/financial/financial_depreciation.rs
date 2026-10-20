use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
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
                // Only probed with salvage <= cost; salvage above cost keeps the clipped charge.
                if switches {
                    ddb_unclipped(new_total)
                } else {
                    ddb_next
                }
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

/// The most periods one DB or VDB call walks (`period`, `end_period`): their schedules step one
/// period at a time, so a larger one is `#NUM!`; Excel's result there is unprobed (DEVIATION,
/// decision 0014; docs/excel-semantics/financial.md "DB and VDB period cap", T-1820).
/// `DB(1,0,1E15,1E15)` ran for hours before the cap. The lifting table charges each element by
/// its period up to it.
pub(crate) const MAX_DEPRECIATION_PERIODS: u32 = 100_000;

/// Whether `period` is past [`MAX_DEPRECIATION_PERIODS`].
fn past_period_cap(period: f64) -> bool {
    period.trunc() > f64::from(MAX_DEPRECIATION_PERIODS)
}

// `db`: fixed-declining-balance depreciation for a single `period`.
fn db_depreciation(cost: f64, salvage: f64, life: f64, period: f64, month: f64) -> f64 {
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
    /// `VDB(cost, salvage, life, start_period, end_period, [factor], [no_switch])`: variable
    /// declining balance depreciation between `start_period` and `end_period`, switching to
    /// straight line when that is larger unless `no_switch` is TRUE. Every argument lifts over an
    /// array and a range (SPEC-0011; probes/fin-argument-modes, docs/excel-semantics/financial.md
    /// "Financial function argument modes"); each element is charged by its `end_period`, the
    /// periods its schedule walks (T-1820).
    pub(crate) fn fn_vdb(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Vdb, args, cell, &mut |model, values| {
            let [cost, salvage, life, start_period, end_period, rest @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let read = || -> Result<f64, CalcResult> {
                let cost = model.cast_number_ref(cost, cell)?;
                let salvage = model.cast_number_ref(salvage, cell)?;
                let life = model.cast_number_ref(life, cell)?;
                let start_period = model.cast_number_ref(start_period, cell)?;
                let end_period = model.cast_number_ref(end_period, cell)?;
                let factor = model.cast_number_or(rest.first(), 2.0, cell)?;
                // no_switch defaults to FALSE => switch to straight line.
                let no_switch = model.cast_number_or(rest.get(1), 0.0, cell)? != 0.0;
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
                    return Err(num_error(cell));
                }
                // When switching to straight line, life, start and end cannot all coincide.
                if !no_switch && life == start_period && start_period == end_period {
                    return Err(num_error(cell));
                }
                if past_period_cap(end_period) {
                    return Err(num_error(cell));
                }
                Ok(vdb_depr_between(
                    cost,
                    salvage,
                    life,
                    start_period,
                    end_period,
                    factor,
                    !no_switch,
                ))
            };
            number_or_error(read())
        })
    }

    /// `DB(cost, salvage, life, period, [month])`: fixed-declining-balance depreciation for one
    /// period. Every argument lifts over an array and a range (SPEC-0011; probes/fin-argument-
    /// modes); each element is charged by its `period`, the periods its schedule walks (T-1820).
    pub(crate) fn fn_db(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Db, args, cell, &mut |model, values| {
            let [cost, salvage, life, period, rest @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let read = || -> Result<f64, CalcResult> {
                let cost = model.cast_number_ref(cost, cell)?;
                let salvage = model.cast_number_ref(salvage, cell)?;
                let life = model.cast_number_ref(life, cell)?;
                let period = model.cast_number_ref(period, cell)?;
                let month = model
                    .cast_number_no_bools_or(rest.first(), 12.0, cell)?
                    .trunc();
                if month == 12.0 && period > life
                    || (period > life + 1.0)
                    || month <= 0.0
                    || month > 12.0
                    || period <= 0.0
                    || cost < 0.0
                {
                    return Err(num_error(cell));
                }
                if cost == 0.0 {
                    return Ok(0.0);
                }
                if past_period_cap(period) {
                    return Err(num_error(cell));
                }
                Ok(db_depreciation(cost, salvage, life, period, month))
            };
            number_or_error(read())
        })
    }
}

// SLN, SYD and DDB are core worksheet functions in closed form: every argument lifts over an
// array and a multi-cell range (`ArgKind::Lift`, SPEC-0011; probes/fin-argument-modes H97, H304,
// H309, probes/fin-later-argument-ranges K19-K23). Moved here from `financial/mod.rs` (T-1372).
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
