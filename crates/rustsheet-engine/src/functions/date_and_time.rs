use crate::tz::Tz;
use chrono::Datelike;

mod holidays;
mod month_shift;
mod networkdays;
pub(crate) mod time_text;
mod weekend;
pub(crate) mod weeknum;
mod workday;
mod yearfrac;
use weekend::WeekendPattern;

use crate::formatter::date_clock::{parse_date_clock_serial, SECONDS_PER_DAY_F64};

pub(crate) fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

/// The length of `month` on the calendar Excel displays, where February 1900 has the fictitious
/// 29th (serial 60): DAYS360 counts 1900-02-29 as February's last day (`DAYS360(60,1)` = -59) and
/// DATEDIF "MD" borrows 29 days from it (`DATEDIF(59,61,"MD")` = 2; probes/dates-early-1900-displayed,
/// dates.md "Serials below 61 in DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC").
fn displayed_month_length(year: i32, month: u32) -> u32 {
    match month {
        2 if year == 1900 || is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// DATEDIF's whole months from `start` to `end` (displayed `(year, month, day)`): the month
/// difference, one less when the end's day is before the start's.
fn datedif_months(start: (i32, u32, u32), end: (i32, u32, u32)) -> i32 {
    let months = (end.0 - start.0) * 12 + (end.1 as i32 - start.1 as i32);
    if end.2 < start.2 {
        months - 1
    } else {
        months
    }
}

/// DATEDIF "MD": the end's day minus the start's, plus the length of the month before the end's
/// month when that is negative (gen-m6 excel-graph, dates.md "DATEDIF "MD" and "YD" day counts",
/// T-2264); February 1900 has 29 days (`DATEDIF(59,61,"MD")` = 2). The sum can stay negative.
fn datedif_month_days(start: (i32, u32, u32), end: (i32, u32, u32)) -> i64 {
    let days = i64::from(end.2) - i64::from(start.2);
    if days >= 0 {
        return days;
    }
    let (year, month) = if end.1 == 1 {
        (end.0 - 1, 12)
    } else {
        (end.0, end.1 - 1)
    };
    days + i64::from(displayed_month_length(year, month))
}

// The date serial of a raw argument (DAYS360, HOUR/MINUTE/SECOND's bound), rounding the
// time-of-day to the nearest second first: Excel rolls 46000.999999 up to 46001.
fn days360_serial(raw: f64) -> i64 {
    ((raw * SECONDS_PER_DAY_F64).round() / SECONDS_PER_DAY_F64).floor() as i64
}

// Days d1→d2 via the US (NASD) 30/360 convention, as used by the worksheet
// DAYS360 function. Note this differs from the bond 30/360 (`days_30_360_us`):
// DAYS360 only promotes the *start* date when it is the last day of February,
// and never adjusts the end date for February (it touches the end date only
// when its day-of-month is literally 31).
fn days360_us(start: (i32, u32, u32), end: (i32, u32, u32)) -> f64 {
    let (y1, m1, day1) = start;
    let (y2, m2, day2) = end;
    let mut day1 = day1 as i32;
    let mut day2 = day2 as i32;

    if day1 == 31 || (m1 == 2 && day1 == displayed_month_length(y1, 2) as i32) {
        day1 = 30;
    }
    if day2 == 31 && day1 == 30 {
        day2 = 30;
    }
    (360 * (y2 - y1) + 30 * (m2 as i32 - m1 as i32) + (day2 - day1)) as f64
}

// Days d1→d2 via the European 30/360 convention used by DAYS360 (method = TRUE):
// any day-of-month equal to 31 becomes 30, with no February handling.
fn days360_eu(start: (i32, u32, u32), end: (i32, u32, u32)) -> f64 {
    let (y1, m1, day1) = start;
    let (y2, m2, day2) = end;
    let day1 = (day1 as i32).min(30);
    let day2 = (day2 as i32).min(30);
    (360 * (y2 - y1) + 30 * (m2 as i32 - m1 as i32) + (day2 - day1)) as f64
}

/// Maps a date to the WEEKDAY number for the given `return_type`.
/// Returns `Error::VALUE`/`Error::NUM` for an invalid `return_type`.
fn weekday_number(date: chrono::NaiveDate, return_type: i32) -> Result<f64, Error> {
    let weekday = date.weekday();
    let num = match return_type {
        1 => weekday.num_days_from_sunday() + 1,
        2 => weekday.number_from_monday(),
        3 => (weekday.number_from_monday() - 1) % 7, // 0-based Monday start
        11..=17 => {
            let start = (return_type - 11) as u32; // 0 = Monday, 6 = Sunday
            let zero_based = weekday.number_from_monday() - 1; // 0..6, Monday = 0
            ((zero_based + 7 - start) % 7) + 1
        }
        _ => return Err(Error::NUM), // return_type 0 too (probes/weekday-serial-zero A1)
    };
    Ok(num as f64)
}

// Port of `isFeb29BetweenConsecutiveYears` from ExcelFinancialFunctions. Only
// called for dates at most one year apart (`y1 == y2` or `y2 == y1 + 1`).
fn is_feb29_between_consecutive_years(start: chrono::NaiveDate, end: chrono::NaiveDate) -> bool {
    let (y1, m1) = (start.year(), start.month());
    let (y2, m2) = (end.year(), end.month());
    if y1 == y2 {
        is_leap_year(y1) && m1 <= 2 && m2 > 2
    } else if y2 == y1 + 1 {
        if is_leap_year(y1) {
            m1 <= 2
        } else if is_leap_year(y2) {
            m2 > 2
        } else {
            false
        }
    } else {
        false
    }
}

/// One TIME argument: [`near_integer_whole`], then `#NUM!` outside -32768..=32767.
fn time_argument(value: f64) -> Result<f64, Error> {
    let whole = near_integer_whole(value);
    if (-32768.0..=32767.0).contains(&whole) {
        Ok(whole)
    } else {
        Err(Error::NUM)
    }
}

/// The hour, minute and second of a serial's time of day, as HOUR, MINUTE and SECOND take it
/// apart. Half a second is added to the serial, then its fraction of a day is split by floors,
/// through hours (x24) and minutes (x60), each step in double precision. So the time rounds to the
/// nearest second and carries into the minute, the hour and past midnight (23:59:59.6 is hour 0),
/// and an exact half goes whichever way the serial's binary noise takes it: `SECOND(0.5+2.5/86400)`
/// is 2, `SECOND(0.5+3.5/86400)` 4. Excel: docs/excel-semantics/dates.md "HOUR, MINUTE and SECOND
/// round to the nearest second" (probes/second-rounding, all 40 cells; T-1408).
fn time_of_day_parts(v: f64) -> [f64; 3] {
    let hours = (v + 0.5 / SECONDS_PER_DAY_F64).rem_euclid(1.0) * 24.0;
    let hour = hours.floor();
    let minutes = (hours - hour) * 60.0;
    let minute = minutes.floor();
    [hour, minute, ((minutes - minute) * 60.0).floor()]
}

use crate::cast::{calc_result_to_string, number_text_too_long};
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::date_text::parse_date;
use crate::formatter::dates::date_month_and_day;
use crate::formatter::dates::permissive_date_to_serial_number;
use crate::functions::lift::{num_error, number_result, value_error};
use crate::functions::util::near_integer::near_integer_whole;
use crate::functions::Function;
use crate::number_format::parse_excel_number;
use crate::types::DateSystem;
use crate::{
    calc_result::{CalcResult, Range},
    expressions::parser::{ArrayNode, Node},
    expressions::token::Error,
    formatter::dates::excel_serial_to_ymd,
    model::Model,
};

/// The error kind of a failed argument evaluation. A non-error result is #VALUE! on purpose: the
/// callers only pass the `Err` of a coercion, which is always an error.
fn error_kind(result: &CalcResult) -> Error {
    match result {
        CalcResult::Error { error, .. } => error.clone(),
        _ => Error::VALUE,
    }
}

/// The error of `result` with no message: a holidays error, which every lifted element of WORKDAY
/// or NETWORKDAYS returns, is then cloned without allocating.
fn bare_error(result: &CalcResult, cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(error_kind(result), cell, String::new())
}

/// Saturday-Sunday weekend (pattern 1), Monday first.
const DEFAULT_WEEKEND: [bool; 7] = WeekendPattern::SatSun.to_mask();

impl Model {
    /// DATEVALUE's reading of date text (also date arguments read as text, e.g. DATEDIF): the
    /// date of [`parse_date`] text, or of "date clock" text the day its clock time lands on
    /// (probes/date-text-grammar B53: `DATEVALUE("1/2/2020 25:00")` is 43833, 3 Jan), with
    /// year-less text in [`Model::current_year`]. Excel: see docs/excel-semantics/dates.md.
    pub(crate) fn parse_datevalue_text(&self, value: &str) -> Option<i32> {
        let (text, year) = (value.trim(), &|| self.current_year());
        let serial = if let Ok((serial, _)) = parse_date(text, self.locale, year) {
            serial
        } else {
            let serial = parse_date_clock_serial(text, self.locale, year)?.floor();
            if !(f64::from(MINIMUM_DATE_SERIAL_NUMBER)..=f64::from(MAXIMUM_DATE_SERIAL_NUMBER))
                .contains(&serial)
            {
                return None;
            }
            serial as i32
        };
        // The parsers count from the 1900 epoch; a 1904 workbook counts from 1904-01-01
        // (`DATEVALUE("1904-01-02")` is 1, probes/date-system-1904-negatives A12).
        let serial = self
            .workbook
            .settings
            .date_system
            .serial_from_1900_serial(i64::from(serial))?;
        i32::try_from(serial).ok()
    }

    /// DAY, MONTH and YEAR: `part` of the serial's date in the workbook's date system (YEAR(0) is
    /// 1904 in a 1904 workbook), on the SPEC-0011 driver (`LIFT_1`).
    fn date_part(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        part: fn((i32, u32, u32)) -> f64,
    ) -> CalcResult {
        let date_system = self.workbook.settings.date_system;
        self.numbers_fn(kind, args, [0.0], cell, |[serial]| {
            date_system
                .to_1900_serial(serial.floor() as i64)
                .and_then(excel_serial_to_ymd)
                .map(part)
                .ok_or(Error::NUM)
        })
    }

    pub(crate) fn fn_day(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.date_part(Function::Day, args, cell, |(_, _, day)| f64::from(day))
    }

    pub(crate) fn fn_month(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.date_part(Function::Month, args, cell, |(_, month, _)| {
            f64::from(month)
        })
    }

    pub(crate) fn fn_year(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.date_part(Function::Year, args, cell, |(year, _, _)| f64::from(year))
    }

    // year, month, day
    pub(crate) fn fn_date(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let date_system = self.workbook.settings.date_system;
        self.numbers_fn(Function::Date, args, [0.0; 3], cell, |[year, month, day]| {
            let [year, month, day] = [year, month, day].map(near_integer_whole);
            let mut year = year as i32;
            if year < 0 {
                return Err(Error::NUM);
            }
            let Some((month, day)) = date_month_and_day(month, day) else {
                return Err(Error::NUM);
            };
            // Excel: a year in 0..=1899 is an offset from 1900: DATE(100, 1, 1) is 2000-01-01
            if year < 1900 {
                year += 1900;
            }
            // In a 1904 workbook the serial counts from 1904-01-01 (`DATE(1904,1,1)` is 0).
            // An earlier day is #NUM!, `DATE(1903,12,31)` and `DATE(0,1,1)` included
            // (probes/date-system-1904-readers A29, A31).
            permissive_date_to_serial_number(day, month, year)
                .ok()
                .and_then(|serial| date_system.serial_from_1900_serial(i64::from(serial)))
                .map(|serial| serial as f64)
                .ok_or(Error::NUM)
        })
    }

    /// Walk a scalar / range / array value and invoke the provided closure with every
    /// numeric date serial that is encountered, in reading order. The serials are not checked
    /// against the date range here: the closure does that, so the first bad holiday decides
    /// whether its error is a type's `#VALUE!` or a range's `#NUM!`.
    fn collect_serial_numbers<F>(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
        mut handle: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(i64) -> Result<(), CalcResult>,
    {
        match value {
            CalcResult::Number(v) => handle(holiday_serial(v))?,
            CalcResult::Range { left, right } => {
                let areas = [Range { left, right }];
                self.for_each_cell_in_areas(&areas, cell, |model, value| match value {
                    CalcResult::Number(v) => handle(holiday_serial(v)),
                    CalcResult::EmptyCell => Ok(()),
                    // Excel: a cell holding number text is a holiday (dates.md "WORKDAY and
                    // NETWORKDAYS holidays: blank and number-text cells").
                    CalcResult::String(text) => handle(model.holiday_text_serial(&text, cell)?),
                    _ => Err(invalid_holiday(cell)),
                })?;
            }
            CalcResult::Array(array) => {
                for row in array {
                    for value in row {
                        match value {
                            ArrayNode::Number(num) => handle(holiday_serial(num))?,
                            // A blank element (HSTACK of a blank cell) is no holiday; skipped, as
                            // serial 0 gives the same results (dates.md "WORKDAY and NETWORKDAYS
                            // holidays: blank and number-text cells", T-2395).
                            ArrayNode::Empty => {}
                            // Number text in an array is a holiday too, as in a cell:
                            // `WORKDAY.INTL(26624,-19,5,{"1.5"})` is 26599, `{""}` `#VALUE!`
                            // (probes/t2360-array-lambda-holidays W4, W15, T-2360).
                            ArrayNode::String(text) => {
                                handle(self.holiday_text_serial(&text, cell)?)?;
                            }
                            ArrayNode::Error(error) => {
                                return Err(CalcResult::Error {
                                    error,
                                    origin: cell,
                                    message: "Error in array".to_string(),
                                });
                            }
                            ArrayNode::Boolean(_) | ArrayNode::Lambda(_) | ArrayNode::Nested(_) => {
                                return Err(invalid_holiday(cell))
                            }
                        }
                    }
                }
            }
            e @ CalcResult::Error { .. } => return Err(e),
            // A one-cell reference to a blank cell is no holiday.
            CalcResult::EmptyCell => {}
            CalcResult::String(text) => handle(self.holiday_text_serial(&text, cell)?)?,
            _ => return Err(invalid_holiday(cell)),
        }
        Ok(())
    }

    /// The serial of a holiday cell holding number text; other text is `#VALUE!`.
    fn holiday_text_serial(&self, text: &str, cell: CellReferenceIndex) -> Result<i64, CalcResult> {
        match self.cast_number(text) {
            Some(number) => Ok(holiday_serial(number)),
            None => Err(invalid_holiday(cell)),
        }
    }

    /// The weekday-faithful date of a WORKDAY or NETWORKDAYS date or holiday, serial 0 included
    /// (dates.md "WORKDAY and NETWORKDAYS from serial 0"); `#NUM!` outside the date range.
    fn excel_date(
        &self,
        serial: i64,
        cell: CellReferenceIndex,
    ) -> Result<chrono::NaiveDate, CalcResult> {
        match self
            .workbook
            .settings
            .date_system
            .date_of_serial_from_zero(serial)
        {
            Some(date) => Ok(date),
            None => Err(out_of_range_date(cell)),
        }
    }

    /// The weekend mask (Monday first) of one weekend element of WORKDAY.INTL or NETWORKDAYS.INTL.
    /// The argument lifts over a range and an array (`ArgKind::Lift`; probes/date-argument-modes
    /// F41:F42, F63:F64). An error-valued weekend is not passed on but becomes `error_as`, the
    /// caller's own: `#NUM!` for WORKDAY.INTL, `#VALUE!` for NETWORKDAYS.INTL, whatever the
    /// error, and a 3-D reference, which the lift reads as `#REF!`, is one too
    /// (probes/date-text-weekend-3d P1-P30, Q1-Q30; dates.md "An error-valued weekend").
    fn weekend_mask(
        value: &CalcResult,
        error_as: Error,
        cell: CellReferenceIndex,
    ) -> Result<[bool; 7], CalcResult> {
        // A number weekend truncates (3.9 is 3), a boolean is 1 or 0, and a blank cell is 0, which
        // is `#NUM!` like FALSE; only an omitted or empty argument is the default weekend
        // (probes/weekend-number-coercion; dates.md "weekend number coercion").
        let invalid = |error| CalcResult::new_error(error, cell, "Invalid weekend".to_string());
        let code = match value {
            CalcResult::Number(n) => n.trunc() as i32,
            CalcResult::Boolean(b) => i32::from(*b),
            CalcResult::EmptyCell => 0,
            CalcResult::EmptyArg => return Ok(DEFAULT_WEEKEND),
            CalcResult::String(s) => {
                if s.len() != 7 || !s.chars().all(|c| c == '0' || c == '1') {
                    return Err(invalid(Error::VALUE));
                }
                let mut weekend = [false; 7];
                for (day, ch) in weekend.iter_mut().zip(s.chars()) {
                    *day = ch == '1';
                }
                return Ok(weekend);
            }
            CalcResult::Error { .. } => return Err(invalid(error_as)),
            CalcResult::Range { .. } | CalcResult::Array(_) | CalcResult::Lambda(_) => {
                return Err(invalid(Error::VALUE));
            }
        };
        WeekendPattern::try_from(code)
            .map(WeekendPattern::to_mask)
            .map_err(|_| invalid(Error::NUM))
    }

    /// The serial of the model clock's now in the optional timezone argument of TODAY / NOW (the
    /// model's own timezone without one); `None` when that instant has no date there.
    fn now_in_timezone_arg(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Option<f64>, CalcResult> {
        let now = self.now_unix_millis();
        let Some(arg0) = args.first() else {
            return Ok(crate::tz::excel_serial_for_now(&self.tz, now));
        };
        let tz_str = self.get_string(arg0, cell)?;
        let tz = Tz::parse(&tz_str).map_err(|_| {
            CalcResult::new_error(Error::VALUE, cell, format!("Invalid timezone: {tz_str}"))
        })?;
        Ok(crate::tz::excel_serial_for_now(&tz, now))
    }

    // Extends the standard by adding an optional timezone parameter
    pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        let serial = match self.now_in_timezone_arg(args, cell) {
            Ok(serial) => serial,
            Err(e) => return e,
        };
        match serial {
            Some(serial) => CalcResult::Number(serial.floor()),
            None => CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Invalid date".to_string(),
            },
        }
    }

    // Extends the standard by adding an optional timezone parameter
    pub(crate) fn fn_now(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() > 1 {
            return CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Wrong number of arguments".to_string(),
            };
        }
        let serial = match self.now_in_timezone_arg(args, cell) {
            Ok(serial) => serial,
            Err(e) => return e,
        };
        match serial {
            Some(serial) => CalcResult::Number(serial),
            None => CalcResult::Error {
                error: Error::ERROR,
                origin: cell,
                message: "Invalid date".to_string(),
            },
        }
    }

    pub(crate) fn fn_time(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Lifts over a range or array in every argument (probes/date-argument-modes F43:F46).
        let kind = Function::Time;
        self.numbers_fn(kind, args, [0.0; 3], cell, |[hour, minute, second]| {
            let [hour, minute, second] = [
                time_argument(hour)?,
                time_argument(minute)?,
                time_argument(second)?,
            ];
            let total_seconds = hour * 3600.0 + minute * 60.0 + second;
            if total_seconds < 0.0 {
                return Err(Error::NUM);
            }
            // Excel divides, then drops whole days (TIME_HOUR_MINUTE_SECOND.xlsx D3/D14).
            Ok((total_seconds / SECONDS_PER_DAY_F64) % 1.0)
        })
    }

    /// HOUR, MINUTE and SECOND: `part` of a day-fraction value, on the SPEC-0011 driver
    /// (`LIFT_1`; probes/date-argument-modes F27:F32). `#NUM!` from 2958466 once rounded to the
    /// second (probes/time-parts-rounding-window).
    fn time_part(
        &mut self,
        kind: Function,
        args: &[Node],
        cell: CellReferenceIndex,
        part: fn(f64) -> f64,
    ) -> CalcResult {
        self.numbers_fn(kind, args, [0.0], cell, |[value]| {
            let last = i64::from(MAXIMUM_DATE_SERIAL_NUMBER);
            if !(value >= 0.0 && days360_serial(value) <= last) {
                return Err(Error::NUM);
            }
            Ok(part(value))
        })
    }

    // HOUR, MINUTE and SECOND share one rounding (`time_of_day_parts`): MINUTE(0.520833333) is
    // 30 (12:29:59.99997 is 12:30:00), HOUR(TIME(10,59,59)+0.6/86400) is 11.
    pub(crate) fn fn_hour(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.time_part(Function::Hour, args, cell, |v| time_of_day_parts(v)[0])
    }

    pub(crate) fn fn_minute(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.time_part(Function::Minute, args, cell, |v| time_of_day_parts(v)[1])
    }

    pub(crate) fn fn_second(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.time_part(Function::Second, args, cell, |v| time_of_day_parts(v)[2])
    }

    pub(crate) fn fn_timevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Lifts over a range or array like the text functions (probes/date-argument-modes F47:F48).
        self.lift_text(Function::Timevalue, args, cell, &mut |model, values| {
            let [text] = values else {
                return Err(Error::ERROR);
            };
            let text = calc_result_to_string(text)?;
            model
                .parse_time_text(&text)
                .map(ArrayNode::Number)
                .ok_or(Error::VALUE)
        })
    }

    pub(crate) fn fn_datevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Lifts over a range or array like the text functions (probes/date-argument-modes F9:F10).
        self.lift_text(
            Function::Datevalue,
            args,
            cell,
            &mut |model, values| match values {
                [CalcResult::String(s)] => model
                    .parse_datevalue_text(s)
                    .map(|serial| ArrayNode::Number(f64::from(serial)))
                    .ok_or(Error::VALUE),
                [CalcResult::Error { error, .. }] => Err(error.clone()),
                [_] => Err(Error::VALUE),
                _ => Err(Error::ERROR),
            },
        )
    }

    pub(crate) fn fn_datedif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Every argument lifts over a range or array: the dates (probes/date-argument-modes F5:F8)
        // and the unit (probes/date-math-lift-driver-edges G1-G5). Errors come in Excel's order: the
        // dates' own (start first), the unit's, then #NUM! of a reversed pair or an out-of-calendar
        // date, then a bad unit (probes/datedif-days360-error-order, excel-graph, T-1430).
        let date_system = self.workbook.settings.date_system;
        self.lift_call(Function::Datedif, args, cell, &mut |model, values| {
            let [start, end, unit] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let dates = model
                .datedif_date(start, cell)
                .and_then(|start| Ok((start, model.datedif_date(end, cell)?)));
            let (start, end) = match dates {
                Ok(dates) => dates,
                Err(error) => return error,
            };
            // The unit is borrowed: a lifted DATEDIF allocates nothing per element for it.
            let converted;
            let unit = match unit {
                CalcResult::String(text) => Ok(&**text),
                CalcResult::Error { error, .. } => Err(error.clone()),
                other => match Self::cast_string_ref(other, cell) {
                    Ok(text) => {
                        converted = text;
                        Ok(converted.as_str())
                    }
                    Err(error) => Err(error_kind(&error)),
                },
            };
            number_result(Self::datedif_value(date_system, start, end, unit), cell)
        })
    }

    /// A DATEDIF date argument's number: date text reads as DATEVALUE reads it (see
    /// [`Model::datedif_date_text`]), anything else as a kernel number.
    fn datedif_date(
        &self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        match value {
            CalcResult::String(s) => match self.datedif_date_text(s) {
                Ok(serial) => Ok(serial),
                Err(error) => Err(CalcResult::new_error(error, cell, String::new())),
            },
            other => self.kernel_number(other, cell),
        }
    }

    /// A DATEDIF date given as text: date text reads as DATEVALUE reads it, so a 1904 workbook
    /// counts it from 1904-01-01 (T-1330); plain number text is that number
    /// (probes/date-argument-modes F89); any other text is #VALUE! (the behaviour before lifting).
    fn datedif_date_text(&self, s: &str) -> Result<f64, Error> {
        if let Some(serial) = self.parse_datevalue_text(s) {
            return Ok(f64::from(serial));
        }
        match parse_excel_number(s.trim()) {
            Some(n) if !number_text_too_long(s) => Ok(n),
            _ => Err(Error::VALUE),
        }
    }

    /// DATEDIF of two date serials for `unit`, in any letter case. The dates are read on the
    /// calendar Excel displays: serial 0 is 1900-01-00 and serial 60 the fictitious 1900-02-29
    /// (`DATEDIF(0.5,60,"D")` = 60, `DATEDIF(60.5,45291,"MD")` = 2; gen-m6, probes/dates-early-1900-displayed,
    /// dates.md "Serials below 61 in DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC").
    fn datedif_value(
        date_system: DateSystem,
        start_serial: f64,
        end_serial: f64,
        unit: Result<&str, Error>,
    ) -> Result<f64, Error> {
        let (start_serial, end_serial) = (start_serial.floor() as i64, end_serial.floor() as i64);
        // A unit error beats a reversed pair and an out-of-calendar date: `DATEDIF(10,5,NA())` and
        // `DATEDIF(3000000,5,NA())` are #N/A (probes/datedif-days360-error-order A1, A16; dates.md
        // "DATEDIF argument errors before date checks").
        let unit = unit?;
        if end_serial < start_serial {
            return Err(Error::NUM);
        }
        let start_1900 = date_system.to_1900_serial(start_serial).ok_or(Error::NUM)?;
        let start = excel_serial_to_ymd(start_1900).ok_or(Error::NUM)?;
        let end = date_system
            .to_1900_serial(end_serial)
            .and_then(excel_serial_to_ymd)
            .ok_or(Error::NUM)?;
        // The units are ASCII, so an ASCII case-blind match is the upper-casing it replaces.
        let unit = ["Y", "M", "D", "YM", "YD", "MD"]
            .into_iter()
            .find(|known| known.eq_ignore_ascii_case(unit))
            .unwrap_or("");
        match unit {
            "Y" => Ok(f64::from(datedif_months(start, end).div_euclid(12))),
            "M" => Ok(f64::from(datedif_months(start, end))),
            "D" => Ok((end_serial - start_serial) as f64),
            "YM" => Ok(f64::from(datedif_months(start, end).rem_euclid(12))),
            "YD" => {
                // The start moved on by the "YM" months in its own year (DATE's day overflow, so
                // 31 January + 1 month is 3 March), plus the "MD" days: fitted to 39 excel-graph
                // cells, among them `DATEDIF(29872,35271,"YD")` = 285, `DATEDIF(1521,1888,"YD")` = 1
                // and `DATEDIF(DATE(2024,1,31),DATE(2025,3,1),"YD")` = 29 (dates.md "DATEDIF "MD" and
                // "YD" day counts", T-2264, T-2263).
                let (year, month, day) = start;
                let months = datedif_months(start, end).rem_euclid(12);
                let moved =
                    permissive_date_to_serial_number(day as i32, month as i32 + months, year)
                        .map_err(|_| Error::NUM)?;
                Ok((i64::from(moved) - start_1900 + datedif_month_days(start, end)) as f64)
            }
            "MD" => Ok(datedif_month_days(start, end) as f64),
            // An unknown unit is #NUM! for any in-calendar pair, text, number or boolean alike
            // (gen-m6/03333cc38ec06a14 A318, 4ca39e0876796fa5 A172; dates.md "DATEDIF and DAYS360
            // error order with a bad unit or method", T-2288).
            _ => Err(Error::NUM),
        }
    }

    pub(crate) fn fn_days(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // Lifts over a range or array in both arguments (probes/date-argument-modes F11:F14).
        self.numbers_fn(Function::Days, args, [0.0; 2], cell, |[end, start]| {
            let (end_serial, start_serial) = (end.floor() as i64, start.floor() as i64);
            // Excel accepts any serial in 0..=MAXIMUM (a 0 serial is 1900-01-00) and
            // reports #NUM! otherwise. DAYS works directly on serials, so the dates
            // themselves are never materialised.
            let range = 0..=MAXIMUM_DATE_SERIAL_NUMBER as i64;
            if !range.contains(&start_serial) || !range.contains(&end_serial) {
                return Err(Error::NUM);
            }
            Ok((end_serial - start_serial) as f64)
        })
    }

    pub(crate) fn fn_days360(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        // The dates and the method lift over a range or array (probes/date-argument-modes F15:F18,
        // probes/date-math-lift-driver-edges G6-G10). The European-method flag follows Excel's
        // logical coercion: numbers are truthy when non-zero, and only the text "TRUE"/"FALSE" is
        // accepted. The dates' own errors come first, then the method's, then the dates' #NUM!
        // (probes/datedif-days360-error-order C1-C19, excel-graph; dates.md "DATEDIF and DAYS360
        // error order", T-1430).
        let date_system = self.workbook.settings.date_system;
        self.lift_call(Function::Days360, args, cell, &mut |model, values| {
            let dates = model
                .kernel_number_or(values.first(), 0.0, cell)
                .and_then(|start| Ok((start, model.kernel_number_or(values.get(1), 0.0, cell)?)))
                .and_then(|dates| {
                    let method = match values.get(2) {
                        // Text is TRUE or FALSE in any case, else #VALUE! with no message (SPEC-0011
                        // kernels); read borrowed, so a lifted text method allocates nothing.
                        Some(CalcResult::String(s)) if s.eq_ignore_ascii_case("true") => Ok(true),
                        Some(CalcResult::String(s)) if s.eq_ignore_ascii_case("false") => Ok(false),
                        Some(CalcResult::String(_)) => Err(value_error(cell)),
                        Some(m) => Self::cast_bool_ref(m, cell),
                        None => Ok(false),
                    };
                    Ok((dates, method?))
                });
            let ((start, end), european) = match dates {
                Ok(read) => read,
                Err(error) => return error,
            };
            // A negative date is #NUM! even when rounding to the second would make it 0:
            // `DAYS360(100,-1E-15)` (probes/dates-early-1900-displayed A29-A35), while
            // `DAYS360(1E-15,100)` = 99.
            let ymd = |serial: f64| {
                if serial < 0.0 {
                    return Err(Error::NUM);
                }
                date_system
                    .to_1900_serial(days360_serial(serial))
                    .and_then(excel_serial_to_ymd)
                    .ok_or(Error::NUM)
            };
            let days = ymd(start).and_then(|start| {
                let end = ymd(end)?;
                Ok(if european {
                    days360_eu(start, end)
                } else {
                    days360_us(start, end)
                })
            });
            number_result(days, cell)
        })
    }

    pub(crate) fn fn_weekday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let ds = self.workbook.settings.date_system;
        // The date and the return type lift (probes/date-argument-modes F51:F52).
        let kind = Function::Weekday;
        self.numbers_fn(kind, args, [0.0, 1.0], cell, |[serial, return_type]| {
            let s = serial.floor();
            let date = ds.date_of_serial_from_zero(s as i64);
            weekday_number(date.ok_or(Error::NUM)?, return_type as i32)
        })
    }

    pub(crate) fn fn_isoweeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let date_system = self.workbook.settings.date_system;
        let result = self.numbers_fn(Function::Isoweeknum, args, [0.0], cell, |[serial]| {
            let serial = date_system.to_1900_serial(serial.floor() as i64);
            weeknum::iso_week(serial.ok_or(Error::NUM)?)
        });
        // ISOWEEKNUM of a valid serial keeps a Monday start in WEEKNUM's stale return_type state
        // (probes/weeknum-stale-reset-isoweeknum, SPEC-0017 rule 2).
        if matches!(result, CalcResult::Number(_)) {
            self.evaluation.order_state.weeknum_start = Some(weeknum::WeeknumStart::Monday);
        }
        result
    }
}

/// The `#NUM!` of a WORKDAY or NETWORKDAYS date or holiday off the calendar.
fn out_of_range_date(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::NUM,
        cell,
        "Out of range parameters for date".to_string(),
    )
}

/// The serial of a holiday number: its floor, not yet checked against the date range.
fn holiday_serial(number: f64) -> i64 {
    number.floor() as i64
}

/// The `#VALUE!` of a holiday that is not a date.
fn invalid_holiday(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Invalid holiday date".to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2004));
        assert!(!is_leap_year(2001));
    }

    #[test]
    fn test_is_feb29_between_consecutive_years() {
        // Same leap year, spanning Feb 29.
        let d1 = chrono::NaiveDate::from_ymd_opt(2020, 2, 28).unwrap();
        let d2 = chrono::NaiveDate::from_ymd_opt(2020, 3, 1).unwrap();
        assert!(is_feb29_between_consecutive_years(d1, d2));
    }

    #[test]
    fn test_is_feb29_between_consecutive_years_false() {
        // Non-leap year, no Feb 29 in range.
        let d1 = chrono::NaiveDate::from_ymd_opt(2021, 2, 28).unwrap();
        let d2 = chrono::NaiveDate::from_ymd_opt(2021, 3, 1).unwrap();
        assert!(!is_feb29_between_consecutive_years(d1, d2));

        // Leap year, but the period starts after Feb 29 (March).
        let d3 = chrono::NaiveDate::from_ymd_opt(1992, 3, 4).unwrap();
        let d4 = chrono::NaiveDate::from_ymd_opt(1993, 3, 1).unwrap();
        assert!(!is_feb29_between_consecutive_years(d3, d4));
    }
}
