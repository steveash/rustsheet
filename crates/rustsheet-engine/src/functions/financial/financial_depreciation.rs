use crate::{
    arithmetic::power,
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

/// VDB's depreciation between `start_period` and `end_period`. Excel sums the charges of the
/// periods in the window rather than subtracting two running totals, and its two modes build the
/// charges differently (docs/excel-semantics/financial.md "VDB charge order", T-1975, T-2185):
/// - `no_switch` TRUE: [`vdb_no_switch_between`];
/// - switching to straight line: [`vdb_switching_between`].
fn vdb_depr_between(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    straight_line: bool,
) -> f64 {
    if straight_line {
        vdb_switching_between(cost, salvage, life, start_period, end_period, factor)
    } else {
        vdb_no_switch_between(cost, salvage, life, start_period, end_period, factor)
    }
}

/// The share of period `period` (which runs from `period - 1` to `period`) inside the window
/// `[start, end]`, applied to `charge`; a whole period leaves the charge unrounded.
fn prorated(charge: f64, period: f64, start: f64, end: f64) -> f64 {
    let share = end.min(period) - start.max(period - 1.0);
    if share == 1.0 {
        charge
    } else {
        charge * share
    }
}

/// VDB with `no_switch` TRUE: the sum, in doubles from the first period on, of each period's
/// charge `min(b·rate, b - salvage)` (at least 0) with the book value `b = cost·(1-rate)^(i-1)`
/// raised directly (the operator `^`'s square-and-multiply), the partial periods prorated.
/// `VDB(2400,300,10,7,10,2,TRUE)` is 203.3164800000004, where the iterated book value gives
/// 203.31648000000007 (probes/fin-vdb-modes A7, fin-vdb-schedule A16-A30, A55-A78).
fn vdb_no_switch_between(
    cost: f64,
    salvage: f64,
    life: f64,
    start: f64,
    end: f64,
    factor: f64,
) -> f64 {
    let rate = (factor / life).min(1.0);
    let first = start.floor() as u32 + 1;
    let last = end.ceil() as u32;
    let mut total = 0.0;
    for period in first..=last {
        let elapsed = f64::from(period - 1);
        let book = if rate == 1.0 {
            if period == 1 {
                cost
            } else {
                0.0
            }
        } else {
            // 0 < 1-rate < 1 and `elapsed` is a whole number, so `^` cannot fail; 0 is its
            // underflow limit.
            cost * power(1.0 - rate, elapsed).unwrap_or(0.0)
        };
        let charge = (book * rate).min(book - salvage).max(0.0);
        total += prorated(charge, f64::from(period), start, end);
    }
    total
}

/// VDB switching to straight line. Excel lays the schedule's periods on the fraction of `start`:
/// the first `start - floor(start)` of a period charges that share of the first declining-balance
/// charge `min(cost·rate, cost - salvage)` (at least 0), and whole periods follow from there, so a
/// fractional start sees the same schedule wherever it lies past the switch:
/// `VDB(2400,100,10,6.5,7.5)` and `VDB(2400,100,10,8.5,9.5)` are both 133.20886857142858,
/// `(566.23104 - 100)/3.5` (probes/fin-vdb-fractional-start A12:A13). Each period's book value
/// `b` steps down by its charge `min(b·rate, b - salvage)`, `rate = factor/life`; from the first
/// period whose straight-line charge `(b - salvage)/(life - t)` (`t` the period's start) is
/// larger than the unclipped `b·rate`, every later period charges that same amount. Comparing with
/// the unclipped charge keeps a clipped last piece declining: `VDB(2400,300,10,9.25,10)` is
/// 4.512314880000062, `0.75·(b - salvage)`, while `VDB(1000,5,10,9.25,10,4)` switches and charges
/// all of `b - salvage` (fin-vdb-fractional-start A44, fin-vdb-fractional-start-edges A1). The
/// window's declining-balance charges (and a prorated last period) are summed in doubles and the
/// whole straight-line periods are added last as one product: `VDB(1234.56,78.9,12,4,9,1.75)` is
/// 379.8854487417499, one ulp below the period-by-period sum (probes/fin-vdb-schedule A1-A15,
/// A31-A54; fin-vdb-modes A6, A9, A13). Salvage above cost never switches and keeps the negative
/// charge: `VDB(2400,3000,10,0,1)` is -600 (fin-vdb-modes A17), and so is
/// `VDB(2400,3000,10,0.5,1.5)` (fin-vdb-fractional-start-edges A7).
fn vdb_switching_between(
    cost: f64,
    salvage: f64,
    life: f64,
    start: f64,
    end: f64,
    factor: f64,
) -> f64 {
    let switches = salvage <= cost;
    let rate = factor / life;
    let whole = start.floor();
    let fraction = start - whole;
    let mut book = cost;
    if fraction > 0.0 {
        book -= fraction * (cost * rate).min(cost - salvage).max(0.0);
    }
    // Offsets of each period from `start`, so the window's shares and the straight-line
    // denominators are measured from `start` itself, not from the rounded fraction
    // (fin-vdb-fractional-start: 153 of 161 cells to the bit this way, 147 the other way).
    let remaining_life = life - start;
    let window = end - start;
    let mut straight_line: Option<f64> = None;
    let mut declining = 0.0;
    let mut straight_periods = 0.0;
    // `end` is at most MAX_DEPRECIATION_PERIODS, so the count fits. The schedule stops at the
    // first period starting at or past `end`: `8.3 - 7.3` is a hair above 1, and a period counted
    // from it would add a sliver (fin-vdb-fractional-start A43).
    for index in 0..=end.ceil() as u32 {
        if fraction + f64::from(index) >= end {
            break;
        }
        let offset = f64::from(index) - whole;
        let charge = match straight_line {
            Some(charge) => charge,
            None => {
                let unclipped = book * rate;
                let ddb = unclipped.min(book - salvage);
                let sln = (book - salvage) / (remaining_life - offset);
                if switches && sln > unclipped {
                    straight_line = Some(sln);
                    sln
                } else {
                    ddb
                }
            }
        };
        if offset >= 0.0 {
            let share = window.min(offset + 1.0) - offset;
            if straight_line.is_some() && share == 1.0 {
                straight_periods += 1.0;
            } else if share == 1.0 {
                declining += charge;
            } else {
                declining += charge * share;
            }
        }
        book -= charge;
    }
    match straight_line {
        Some(charge) if straight_periods > 0.0 => declining + straight_periods * charge,
        _ => declining,
    }
}

/// The most periods one DB or VDB call walks (`period`, `end_period`): their schedules step one
/// period at a time, so a larger one is `#NUM!`; Excel's result there is unprobed (DEVIATION,
/// decision 0014; docs/excel-semantics/financial.md "DB and VDB period cap", T-1820).
/// `DB(1,0,1E15,1E15)` ran for hours before the cap. The lifting table charges each element by
/// its period up to it.
pub(crate) const MAX_DEPRECIATION_PERIODS: u32 = 100_000;

/// DB's largest life and period, `i32::MAX`: Excel reads both as 32-bit integers
/// (probes/fin-syd-db-domain A6:A7, C7:C8).
const DB_INTEGER_MAX: f64 = 2_147_483_647.0;

/// The smallest DB life whose last year, life+1, is period 0: `u32::MAX`, as if Excel held the
/// life in an unsigned 32-bit integer that saturates and life+1 wrapped (probes/fin-syd-db-domain
/// H3:H8, L1:L6; 4294967294 and 4E9 are `#NUM!`).
const DB_WRAPPED_LIFE: f64 = 4_294_967_295.0;

/// Whether `period` is past [`MAX_DEPRECIATION_PERIODS`].
fn past_period_cap(period: f64) -> bool {
    period.trunc() > f64::from(MAX_DEPRECIATION_PERIODS)
}

// `db`: fixed-declining-balance depreciation for a single `period`, in Excel's operation order
// (docs/excel-semantics/financial.md "DB and DDB operation order", T-2033): the first period is
// `cost * rate * (month/12)`, then the remaining balance steps down by `balance * rate` and the
// last period charges `balance * (rate * (12 - month) / 12)`. `DB(100,10,4,1,2)` is
// 7.299999999999999 and `DB(23,10,4,5,2)` 1.8687313963962309 (ironcalc-calc DB_DDB.xlsx G20, G42).
fn db_depreciation(cost: f64, salvage: f64, life: f64, period: f64, month: f64) -> f64 {
    let rate = db_rate(cost, salvage, life);
    let int_period = period.trunc() as i64;
    let int_life = life.trunc() as i64;

    // First period (also covers a fractional period below 1).
    let first = cost * rate * (month / 12.0);
    if int_period <= 1 {
        return first;
    }
    // Step the remaining balance down through the period before the one asked.
    let mut balance = cost - first;
    for _ in 2..int_period {
        balance -= balance * rate;
    }
    // `int_period - 1`, not `int_life + 1`: `int_period` is in 2..=MAX_DEPRECIATION_PERIODS here,
    // while a huge `life` saturates `int_life` to `i64::MAX` (`DB(6912,0,1E+300,2,6)`, T-2570).
    if int_period - 1 == int_life {
        balance * (rate * (12.0 - month) / 12.0)
    } else {
        balance * rate
    }
}

/// DB's depreciation rate, rounded to three decimal places.
fn db_rate(cost: f64, salvage: f64, life: f64) -> f64 {
    ((1.0 - (salvage / cost).powf(1.0 / life)) * 1000.0).round() / 1000.0
}

/// DB's partial last year straight after the first: what a period below 1 is when `life` is at
/// least [`DB_WRAPPED_LIFE`] and `month` is not 12 (`DB(100,0,4294967296,0.5,8)` is
/// 11.111111111111114, `DB(100,0,1E+10,0.5,6)` 25; probes/fin-syd-db-domain L7, L8).
fn db_wrapped_last_year(cost: f64, salvage: f64, life: f64, month: f64) -> f64 {
    let rate = db_rate(cost, salvage, life);
    (cost - cost * rate * (month / 12.0)) * (rate * (12.0 - month) / 12.0)
}

// `ddb`: double-declining-balance depreciation for a single `period`
// (docs/excel-semantics/financial.md "DB and DDB operation order", T-2033).
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
        // Excel raises the book value with its own `^` (square-and-multiply for a whole period),
        // not libm `pow`: `DDB(9695,282,20,8,1)` is 338.5190042814453, where `pow` gives ...452
        // (gen-m9 6376e27be7a05871 Calc!A141, T-2033). 0 < 1-rate < 1, so `^` cannot fail; 0 is
        // its underflow limit.
        cost * power(1.0 - rate, period - 1.0).unwrap_or(0.0)
    };
    // Excel charges `min(old * rate, old - salvage)`, not the difference of two book values:
    // `DDB(10000,1000,6,2,2)` is 2222.222222222222 (probes/fin-argument-modes H95, T-2033).
    let ddb = (old_value * rate).min(old_value - salvage);
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
                // no_switch defaults to FALSE => switch to straight line. It reads as a boolean:
                // "TRUE" and "FALSE" in any case are taken and other text, numeric text too, is
                // `#VALUE!`, before the range checks (probes/t2492-vdb-ddb-arguments A1:A9, A27).
                let no_switch = rest
                    .get(1)
                    .map_or(Ok(false), |value| Self::cast_bool_ref(value, cell))?;
                // A factor of 0 is taken (no declining balance: straight line, or 0 with
                // no_switch), and so is an empty window at period 0 (probes/t2492-vdb-ddb-arguments
                // A13:A19).
                if cost < 0.0
                    || salvage < 0.0
                    || life <= 0.0
                    || factor < 0.0
                    || start_period < 0.0
                    || start_period > life
                    || end_period > life
                    || start_period > end_period
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
                // A boolean month reads as 1 or 0, from a cell too (`DB(15779,385,37,<FALSE>,<FALSE>)`
                // is `#NUM!`, probes/t2475-stat-subexpressions A165), as DDB's factor does.
                let month = model.cast_number_or(rest.first(), 12.0, cell)?.trunc();
                // Excel reads life and period as 32-bit integers: a period past i32::MAX is period 1
                // (month 12 too, probe E8), a life past it is #NUM! except the u32::MAX "wrapped
                // last year", and a negative salvage is #NUM! even at cost 0 (financial.md "DB and
                // SYD argument domains", probes/fin-syd-db-domain).
                let huge_period = period > DB_INTEGER_MAX;
                let period = if huge_period { 1.0 } else { period };
                let wrapped_last_year = life >= DB_WRAPPED_LIFE && month != 12.0 && period < 1.0;
                let huge_life = life > DB_INTEGER_MAX && !huge_period && !wrapped_last_year;
                if month == 12.0 && period > life
                    // The partial last year is period life+1 by its whole part
                    // (`DB(0.01,488,32,33.5,10)` is period 33's value, probes/fin-syd-db-domain C2).
                    || (period.trunc() > life + 1.0)
                    || month <= 0.0
                    || month > 12.0
                    || period <= 0.0
                    || cost < 0.0
                    || salvage < 0.0
                    || huge_life
                {
                    return Err(num_error(cell));
                }
                if cost == 0.0 {
                    return Ok(0.0);
                }
                if wrapped_last_year {
                    return Ok(db_wrapped_last_year(cost, salvage, life, month));
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
                if life == 0.0 || per > life || per <= 0.0 || salvage < 0.0 {
                    return Err(num_error(cell));
                }
                // `life*(life+1)` past the double range is #DIV/0! (`SYD(100,10,1.4E+154,1)`),
                // a numerator past it #NUM! (`SYD(1E+300,-1E+300,10,1)` by its salvage;
                // probes/fin-syd-db-domain F16, F20; financial.md "DB and SYD argument domains").
                let years = life * (life + 1.0);
                if !years.is_finite() {
                    return Err(div_error(cell));
                }
                Ok(((cost - salvage) * (life - per + 1.0) * 2.0) / years)
            };
            number_or_error(read())
        })
    }

    /// `DDB(cost, salvage, life, period, [factor])`; `factor` is 2 when omitted (the
    /// double-declining balance). A boolean factor is 1 or 0 and a numeric text its number:
    /// `DDB(1000,100,10,1,TRUE)` is 100, with FALSE `#NUM!`, with "3" 300 (probes/t2492-vdb-ddb-
    /// arguments A32:A38).
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
                let factor = model.cast_number_or(rest.first(), 2.0, cell)?;
                if period > life || cost < 0.0 || salvage < 0.0 || period <= 0.0 || factor <= 0.0 {
                    return Err(num_error(cell));
                }
                Ok(ddb_depreciation(cost, salvage, life, period, factor))
            };
            number_or_error(read())
        })
    }
}
