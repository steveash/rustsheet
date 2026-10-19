//! AMORLINC and AMORDEGRC: the French linear and degressive depreciation of one period. They are
//! Analysis `ToolPak` functions, so every argument lifts over an array and a multi-cell range is
//! `#VALUE!` in it, with no intersection (`ArgKind::LiftArrays`, SPEC-0011; probes/fin-argument-modes
//! H19-H35, probes/fin-remaining-later-argument-ranges H11-H34).
//!
//! Excel walks the periods one by one in doubles: `AMORLINC(2400,39679,39813,300,875000,0.000001,1)`
//! is 0.0015212859222804571, the remainder after 874,999 subtractions of 0.0024, not the closed
//! form's 0.0015213114757168815 (probes/fin-amor-period-bound A6), and a period of 1E15 at a rate
//! of 1E-300 did not finish within the excel-graph timeout (docs/excel-semantics/financial.md
//! "AMORLINC and AMORDEGRC over many periods"). The kernels walk the same doubles in bounded time:
//! AMORLINC exactly, by jumping over the subtractions that are exact (`linear_charge`), and
//! AMORDEGRC exactly up to `DEGRESSIVE_LOOP_PERIODS` periods and past them in a closed form that
//! drifts from the walk by about 1e-13 relative (`DegressiveWalk::closed_form`, a DEVIATION). Moved out
//! of `financial/financial_depreciation.rs` (T-1841).
//!
//! Port of the depreciation calculations as implemented in the `ExcelFinancialFunctions` library
//! (depreciation.fs), see <https://github.com/fsprojects/ExcelFinancialFunctions>.

use chrono::{Datelike, NaiveDate};

use super::accrint::acc_days_between;
use super::number_or_error;
use crate::functions::date_and_time::is_leap_year;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    formatter::dates::from_excel_date,
    functions::lift::num_error,
    functions::Function,
    model::Model,
};

/// The most AMORDEGRC periods walked one by one. Up to here the kernel is main's walk
/// (probes/fin-amor-period-bound B6-B14, up to 10^6 periods, are Excel's values); past it, where the
/// walk has not reached a period whose charge is fixed, the decay is a closed form, which can move
/// the rounded charge once it is about 1E12 or more (fin-amor-period-bound-c). The bound is also the
/// kernel's work cap (`AMORDEGRC_WEIGHT` in functions/lift/kinds/financial.rs).
const DEGRESSIVE_LOOP_PERIODS: u64 = 1 << 20;

/// The most single subtractions `linear_charge` makes. A walk takes a few per binade it crosses
/// and stops once the charge no longer moves the value, at most 616 over every charge from 2^-1074
/// to 2^1023 against values from 1E-300 to `f64::MAX` (`linear_walks_stay_below_the_step_cap`).
/// The cap keeps the loop bounded by construction, and `AMORLINC_WEIGHT` in
/// functions/lift/kinds/financial.rs is priced from it; a walk that reached it would be `#NUM!`,
/// never a wrong value.
const LINEAR_STEP_CAP: u32 = 1 << 10;

/// Which depreciation the kernel computes.
#[derive(Clone, Copy)]
enum Amortization {
    Linear,
    Degressive,
}

impl Model {
    /// `AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])`: a
    /// (possibly prorated) first period, then a constant `rate * cost` until the cost less the
    /// salvage is used up.
    pub(crate) fn fn_amorlinc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Amorlinc, args, cell, &mut |model, values| {
            number_or_error(model.amortization_of(values, Amortization::Linear, cell))
        })
    }

    /// `AMORDEGRC(cost, date_purchased, first_period, salvage, period, rate, [basis])`: like
    /// AMORLINC, but the rate is scaled by a coefficient of the asset life, it applies to what is
    /// left, and each period's charge is rounded to a whole number.
    pub(crate) fn fn_amordegrc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Amordegrc, args, cell, &mut |model, values| {
            number_or_error(model.amortization_of(values, Amortization::Degressive, cell))
        })
    }

    /// Reads every argument in order (the basis refuses a boolean), then checks them as main did.
    /// The checks are written as the valid ranges, so a NaN argument fails them too.
    fn amortization_of(
        &self,
        values: &[CalcResult],
        kind: Amortization,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [cost, purchased, first_period, salvage, period, rate, rest @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let cost = self.cast_number_ref(cost, cell)?;
        let purchased = self.financial_date(self.cast_number_ref(purchased, cell)?);
        let first_period = self.financial_date(self.cast_number_ref(first_period, cell)?);
        let salvage = self.cast_number_ref(salvage, cell)?;
        let period = self.cast_number_ref(period, cell)?;
        let rate = self.cast_number_ref(rate, cell)?;
        let basis = self
            .cast_number_no_bools_or(rest.first(), 0.0, cell)?
            .trunc() as i32;
        let valid = cost >= 0.0
            && salvage >= 0.0
            && salvage < cost
            && period >= 0.0
            && rate > 0.0
            && purchased < first_period
            && matches!(basis, 0 | 1 | 3 | 4);
        if !valid {
            return Err(num_error(cell));
        }
        let asset = Asset {
            cost,
            purchased: from_excel_date(purchased).map_err(|_| num_error(cell))?,
            first_period: from_excel_date(first_period).map_err(|_| num_error(cell))?,
            salvage,
            rate,
            basis,
        };
        match kind {
            Amortization::Linear => asset.linear(period),
            Amortization::Degressive => {
                // The asset life must not fall in [0, 3] or [4, 5].
                let life = 1.0 / rate;
                if (0.0..=3.0).contains(&life) || (4.0..=5.0).contains(&life) {
                    return Err(num_error(cell));
                }
                asset.degressive(period, DEGRESSIVE_LOOP_PERIODS)
            }
        }
        .map_err(|_| num_error(cell))
    }
}

/// Validated arguments: `0 <= salvage < cost`, a positive rate, the purchase before the first
/// period, a basis of 0, 1, 3 or 4.
struct Asset {
    cost: f64,
    purchased: NaiveDate,
    first_period: NaiveDate,
    salvage: f64,
    rate: f64,
    basis: i32,
}

/// The number of whole periods the walk takes for `period`, saturating: past `u64::MAX` (and
/// past 2^53, where main's `counted += 1.0` stopped moving and never finished) every walk has
/// long reached a period whose charge is fixed.
fn whole_periods(period: f64) -> u64 {
    // A float-to-int `as` saturates; `period >= 0` is checked.
    period.trunc() as u64
}

impl Asset {
    /// AMORLINC: `findDepr` charges `rate * cost` each period, clamped to what is still
    /// available. The period count's bound is `ceil(1/rate)`, which a tiny rate makes huge.
    fn linear(&self, period: f64) -> Result<f64, String> {
        let asset_life = (1.0 / self.rate).ceil();
        // A positive condition, so a NaN life (never reached: the rate is positive) is 0 too.
        let within_life = period <= asset_life;
        if !within_life {
            return Ok(0.0);
        }
        let (first_depr, _) = self.first_depreciation(self.rate, asset_life)?;
        if period == 0.0 {
            return Ok(first_depr);
        }
        let charge = self.rate * self.cost;
        let available = self.cost - self.salvage - first_depr;
        let depr = linear_charge(charge, available, whole_periods(period));
        // NaN only past `LINEAR_STEP_CAP`, which no double reaches (tests): `#NUM!` then.
        if depr.is_nan() {
            return Err(String::new());
        }
        Ok(depr)
    }

    /// AMORDEGRC: the rate times a coefficient of the life, applied to the remaining cost; two
    /// periods before the end it charges half of what is left and then the rest. `loop_periods`
    /// is `DEGRESSIVE_LOOP_PERIODS` (smaller in tests, to check the closed form against the walk).
    fn degressive(&self, period: f64, loop_periods: u64) -> Result<f64, String> {
        let ass_life = (1.0 / self.rate).ceil();
        let within_life = period <= ass_life;
        if !within_life {
            return Ok(0.0);
        }
        let depr_rate = self.rate * amor_depr_coeff(ass_life);
        let (first_depr_linc, asset_life) = self.first_depreciation(depr_rate, ass_life)?;
        let first_depr = amor_round(first_depr_linc);
        if period == 0.0 {
            return Ok(first_depr);
        }
        let walk = DegressiveWalk {
            remain: self.cost - first_depr,
            salvage: self.salvage,
            rate: depr_rate,
            half_period: half_period(asset_life),
        };
        Ok(amor_round(walk.charge(whole_periods(period), loop_periods)))
    }

    /// `firstDeprLinc`: the depreciation of the (possibly partial) first period and the
    /// resulting asset life, `(first_depreciation, asset_life)`.
    fn first_depreciation(&self, rate: f64, ass_life: f64) -> Result<(f64, f64), String> {
        let days_in_yr = amor_days_in_year(self.purchased, self.basis);
        let purchased = amor_fix_29_february(self.purchased, self.basis);
        let first_period = amor_fix_29_february(self.first_period, self.basis);
        let first_len = acc_days_between(purchased, first_period, true, self.basis)?;
        let first_depr_temp = first_len / days_in_yr * rate * self.cost;
        let first_depr = if first_depr_temp == 0.0 {
            self.cost * rate
        } else {
            first_depr_temp
        };
        let asset_life = if first_depr_temp == 0.0 {
            ass_life
        } else {
            ass_life + 1.0
        };
        let avail_depr = self.cost - self.salvage;
        if first_depr > avail_depr {
            Ok((avail_depr, asset_life))
        } else {
            Ok((first_depr, asset_life))
        }
    }
}

/// The lowest value of `x`'s binade, `2^exponent`, or 0 for a subnormal (whose grid is uniform
/// down to 0). Below `2 * floor` every double is a multiple of the same ulp.
fn binade_floor(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & 0x7FF0_0000_0000_0000)
}

/// AMORLINC's charge of whole period `periods` (the walk of `findDepr`, main's loop):
/// `available` loses `charge` each period, and the period whose charge exceeds what is left
/// charges that remainder; every later period charges 0.
///
/// The walk is exact in doubles and bounded. While `available - charge` stays in `available`'s
/// binade, the subtraction rounds `charge` to the binade's grid the same way every time (a tie
/// rounds to even, and from an even value the same way again), so once two steps in one binade
/// were equal, the next ones are that step exactly, and `jump` of them is one multiplication
/// (exact: a multiple of the grid below the binade's top). Each binade then takes a few single
/// steps, and a step that does not move the value (`charge` below half its ulp) charges `charge`
/// for ever. NaN if the walk reaches `LINEAR_STEP_CAP` single steps before it ends.
fn linear_charge(charge: f64, available: f64, periods: u64) -> f64 {
    linear_walk(charge, available, periods).0
}

/// [`linear_charge`] and the single steps it took.
fn linear_walk(charge: f64, available: f64, periods: u64) -> (f64, u32) {
    let mut avail = available;
    let mut done: u64 = 0;
    // The last single step and its binade, when it stayed inside one binade.
    let mut last: Option<(f64, f64)> = None;
    let mut steps: u32 = 0;
    while done < periods && steps < LINEAR_STEP_CAP {
        // A positive condition, so a NaN (never reached) ends the walk like a clamp.
        let fits = charge <= avail;
        if !fits {
            let depr = if done + 1 == periods { avail } else { 0.0 };
            return (depr, steps);
        }
        steps += 1;
        let next = avail - charge;
        if next == avail {
            return (charge, steps);
        }
        done += 1;
        let floor = binade_floor(next);
        // In one binade, `next >= avail / 2`, so `avail - next` is exact (Sterbenz).
        let in_binade = binade_floor(avail) == floor && next > floor;
        let step = avail - next;
        avail = next;
        if !in_binade {
            last = None;
            continue;
        }
        if last == Some((step, floor)) {
            // Steps keep this size while `avail - charge >= floor`. The quotient is below 2^53
            // and off by a few units at most, so 8 steps short of it are all inside the binade.
            let room = ((avail - floor - charge) / step).floor() - 8.0;
            if room >= 1.0 {
                let jump = (room as u64).min(periods - done);
                avail -= jump as f64 * step;
                done += jump;
            }
        }
        last = Some((step, floor));
    }
    if done >= periods {
        (charge, steps)
    } else {
        // `LINEAR_STEP_CAP` single steps and the walk has not ended: no value rather than a wrong one.
        (f64::NAN, steps)
    }
}

/// The period that charges half of what is left: `asset_life - counted` is 2 when `counted`,
/// main's period counter, is one ahead of the period. `None` past 2^53, where main's counter
/// stopped short of it.
fn half_period(asset_life: f64) -> Option<u64> {
    const EXACT_INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;
    let period = asset_life - 3.0;
    if !(1.0..EXACT_INTEGER_LIMIT).contains(&period) {
        return None;
    }
    let period = period as u64;
    amor_are_equal(asset_life - (period + 1) as f64, 2.0).then_some(period)
}

/// AMORDEGRC's walk after the first period.
struct DegressiveWalk {
    remain: f64,
    salvage: f64,
    rate: f64,
    half_period: Option<u64>,
}

impl DegressiveWalk {
    /// The (unrounded) charge of whole period `periods`, main's loop: each period charges
    /// `rate * remain`, the half period half of `remain` (and the rate becomes 1), and once
    /// `remain` is below the salvage nothing.
    ///
    /// The walk stops early where the rest is known, bit for bit main's value: below the salvage
    /// every later period charges 0; before the half period a charge that rounds to 0 stays 0
    /// (`remain` only falls); a step that does not move `remain` repeats until the half period.
    /// Past `loop_periods` single periods it finishes in closed form ([`Self::closed_form`]).
    fn charge(mut self, periods: u64, loop_periods: u64) -> f64 {
        let half_period = self.half_period;
        let before_half = |period: u64| half_period.is_none_or(|half| period < half);
        let ends_before_half = before_half(periods);
        let mut depr = 0.0;
        let mut period: u64 = 0;
        while period < periods {
            period += 1;
            let half = half_period == Some(period);
            if before_half(period) {
                if self.remain < self.salvage {
                    return 0.0;
                }
                // `amor_round` is 0 only below 0.5, so the cheap comparison guards it.
                let charge = self.rate * self.remain;
                if ends_before_half && charge < 0.5 && amor_round(charge) == 0.0 {
                    return 0.0;
                }
                if period > loop_periods {
                    return self.closed_form(period - 1, periods);
                }
            }
            let depr_temp = if half {
                self.rate = 1.0;
                self.remain * 0.5
            } else {
                self.rate * self.remain
            };
            depr = if self.remain < self.salvage {
                0.0
            } else {
                depr_temp
            };
            let remain = self.remain - depr;
            if remain == self.remain {
                // A fixed point: every period charges `depr` until the half period, or for ever
                // after it (the rate is 1 there, so only a zero `remain` stays put).
                match half_period {
                    Some(half) if period < half && half <= periods => period = half - 1,
                    _ => return depr,
                }
            }
            self.remain = remain;
        }
        depr
    }

    /// The charge of period `periods` from the state after period `done` (before the half
    /// period): `remain` decays by `(1 - rate)` a period, `remain * exp(n * log1p(-rate))`.
    /// Main's walk adds a rounding per period, so the two drift apart by up to about 1e-13
    /// relative over 10^6 periods: equal once rounded for charges below about 1E11, but a larger
    /// charge can be a unit off (a review replay: 3.1E11; about 1E12 or more often is):
    /// `AMORDEGRC(1E20,39679,39813,300,1100000,1E-7,1)` is 18989305437101, Excel and main
    /// 18989305437102 (financial.md "AMORDEGRC past 2^20 periods", a DEVIATION).
    fn closed_form(&self, done: u64, periods: u64) -> f64 {
        let last = self
            .half_period
            .filter(|&half| half <= periods)
            .unwrap_or(periods);
        let decay = (last - 1 - done) as f64 * (-self.rate).ln_1p();
        let remain = self.remain * decay.exp();
        if remain < self.salvage {
            return 0.0;
        }
        if self.half_period != Some(last) {
            return self.rate * remain;
        }
        if periods == last {
            return remain * 0.5;
        }
        let rest = remain - remain * 0.5;
        if periods == last + 1 && rest >= self.salvage {
            rest
        } else {
            0.0
        }
    }
}

// `daysInYear` for the AMOR* functions. Actual/Actual (basis 1) uses the
// length of the calendar year that contains `date`; Actual/365 (basis 3)
// uses 365; the 30/360 bases (0 and 4) use 360. Actual/360 (basis 2) is
// rejected before reaching here.
fn amor_days_in_year(date: NaiveDate, basis: i32) -> f64 {
    match basis {
        1 => {
            if is_leap_year(date.year()) {
                366.0
            } else {
                365.0
            }
        }
        3 => 365.0,
        _ => 360.0,
    }
}

// `fix29February`: for the actual-day bases (1 and 3) a Feb 29 (or any late
// February day) is snapped to Feb 28 so leap days don't distort the day count.
fn amor_fix_29_february(d: NaiveDate, basis: i32) -> NaiveDate {
    if (basis == 1 || basis == 3) && is_leap_year(d.year()) && d.month() == 2 && d.day() >= 28 {
        // Feb 28 always exists.
        NaiveDate::from_ymd_opt(d.year(), 2, 28).unwrap_or(d)
    } else {
        d
    }
}

// Depreciation coefficient as a function of asset life (in years).
fn amor_depr_coeff(asset_life: f64) -> f64 {
    if (3.0..=4.0).contains(&asset_life) {
        1.5
    } else if (5.0..=6.0).contains(&asset_life) {
        2.0
    } else if asset_life > 6.0 {
        2.5
    } else {
        1.0
    }
}

fn amor_are_equal(x: f64, y: f64) -> bool {
    (x - y).abs() < 0.0001
}

// `round excelComplaint`: pre-round to 13 significant digits (Excel's internal
// precision) then round to the nearest integer, ties away from zero.
fn amor_round(x: f64) -> f64 {
    let k = (x * 1e13).round() / 1e13;
    k.round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::util::Rng;

    /// Main's AMORLINC loop (`findDepr`) with an integer counter, the reference walk.
    fn main_linear_walk(charge: f64, available: f64, periods: u64) -> f64 {
        let mut depr = charge;
        let mut avail = available;
        for _ in 0..periods {
            if depr > avail {
                depr = avail;
            }
            let avail_temp = avail - depr;
            avail = if avail_temp < 0.0 { 0.0 } else { avail_temp };
        }
        depr
    }

    /// Main's AMORDEGRC loop after the first period, the reference walk.
    fn degressive_walk(walk: &DegressiveWalk, asset_life: f64, periods: u64) -> f64 {
        let mut depr = 0.0;
        let mut depr_rate = walk.rate;
        let mut remain = walk.remain;
        for counted in 2..=periods + 1 {
            let calc_t = asset_life - counted as f64;
            let depr_temp = if amor_are_equal(calc_t, 2.0) {
                depr_rate = 1.0;
                remain * 0.5
            } else {
                depr_rate * remain
            };
            depr = if remain < walk.salvage {
                0.0
            } else {
                depr_temp
            };
            remain -= depr;
        }
        depr
    }

    /// A double in [0, 1) from the seeded test generator.
    fn spread(rng: &mut Rng) -> f64 {
        (rng.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    #[test]
    fn linear_jumps_equal_the_walk_bit_for_bit() {
        let mut seed = Rng(0x9E37_79B9_7F4A_7C15);
        for _ in 0..400 {
            let available = 10f64.powf(spread(&mut seed) * 8.0 - 2.0);
            // 10 to about 10^6 periods of real progress, the clamp included.
            let charge = available / 10f64.powf(1.0 + spread(&mut seed) * 5.0);
            let clamp = (available / charge) as u64;
            for periods in [1, 2, clamp / 2, clamp, clamp + 1, clamp + 2, clamp + 3] {
                assert_eq!(
                    linear_charge(charge, available, periods).to_bits(),
                    main_linear_walk(charge, available, periods).to_bits(),
                    "charge {charge:e}, available {available:e}, period {periods}"
                );
            }
        }
        // Ties: a charge of half an ulp of the binade, and an odd number of half ulps.
        for charge in [0.5_f64.powi(44), 3.0 * 0.5_f64.powi(43), 2.4, 0.0024] {
            for periods in [100, 1000, 874_999, 875_000, 875_001] {
                assert_eq!(
                    linear_charge(charge, 2100.0, periods).to_bits(),
                    main_linear_walk(charge, 2100.0, periods).to_bits(),
                    "{charge:e} {periods}"
                );
            }
        }
    }

    #[test]
    fn linear_walks_stay_below_the_step_cap() {
        // The longest walks cross every binade: the largest value with the smallest charges
        // that still move it, and the subnormal charges.
        let mut most = 0;
        for available in [f64::MAX, 1e308, 2100.0, 1.0, 1e-300] {
            for exponent in -1074..=1023 {
                let charge = 2f64.powi(exponent);
                for charge in [charge, charge * 1.5, charge * 1.000_000_1] {
                    if charge.is_finite() && charge > 0.0 {
                        most = most.max(linear_walk(charge, available, u64::MAX).1);
                    }
                }
            }
        }
        assert!(most < LINEAR_STEP_CAP, "{most} single steps");
        assert_eq!(linear_charge(5e-324, f64::MAX, u64::MAX), 5e-324);
        assert_eq!(linear_charge(1.0, f64::MAX, u64::MAX), 1.0);
        assert_eq!(linear_charge(f64::MAX / 2.0, f64::MAX, u64::MAX), 0.0);
    }

    #[test]
    fn degressive_shortcuts_equal_the_walk_and_the_closed_form_stays_near_it() {
        let mut seed = Rng(0x0123_4567_89AB_CDEF);
        for _ in 0..300 {
            let rate = 10f64.powf(-1.0 - spread(&mut seed) * 4.0) * 2.5;
            // Up to 1E20, where a charge reaches 1E12 or more and the closed form's drift shows.
            let remain = 10f64.powf(spread(&mut seed) * 20.0);
            let salvage = if spread(&mut seed) < 0.3 {
                0.0
            } else {
                remain * spread(&mut seed) * 0.5
            };
            let asset_life = (1.0 / (rate / 2.5)).ceil() + 1.0;
            let walk = || DegressiveWalk {
                remain,
                salvage,
                rate,
                half_period: half_period(asset_life),
            };
            let last = (asset_life - 1.0) as u64;
            for periods in [
                1,
                2,
                7,
                last / 3,
                last / 2,
                last - 3,
                last - 2,
                last - 1,
                last,
            ] {
                let raw = degressive_walk(&walk(), asset_life, periods);
                let expected = amor_round(raw);
                // Every period walked: main's value exactly.
                assert_eq!(
                    amor_round(walk().charge(periods, u64::MAX)),
                    expected,
                    "rate {rate:e}, remain {remain:e}, salvage {salvage:e}, period {periods}"
                );
                // The closed form after 64 periods: within 1e-13 relative of the walk's charge
                // (its documented drift), so off by at most one unit more than that once rounded.
                let closed = walk().charge(periods, 64);
                let context = format!(
                    "closed form: rate {rate:e}, remain {remain:e}, salvage {salvage:e}, period {periods}"
                );
                // A charge that rounds to 0 returns 0 early, unrounded.
                if expected != 0.0 {
                    assert!(
                        (closed - raw).abs() <= raw.abs() * 1e-13,
                        "{closed:e} vs {raw:e}: {context}"
                    );
                }
                assert!(
                    (amor_round(closed) - expected).abs() <= 1.0 + raw.abs() * 1e-13,
                    "{context}"
                );
            }
        }
    }

    #[test]
    fn degressive_fixed_points_jump_to_the_half_period() {
        // rate 1E-300 on 1E300: the charge 2.5 never moves the remaining value.
        let walk = || DegressiveWalk {
            remain: 1e300,
            salvage: 300.0,
            rate: 2.5e-300,
            half_period: half_period(1e15 + 1.0),
        };
        assert_eq!(amor_round(walk().charge(1_000_000, 1 << 20)), 3.0);
        assert_eq!(
            walk().charge(1e15 as u64 - 2, 1 << 20),
            5e299,
            "the half period"
        );
        assert_eq!(walk().charge(1e15 as u64 - 1, 1 << 20), 5e299);
        assert_eq!(walk().charge(1e15 as u64, 1 << 20), 0.0);
    }
}
