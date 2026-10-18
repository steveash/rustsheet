use crate::tz::Tz;
use chrono::Datelike;
use std::collections::HashMap;

mod month_shift;
pub(crate) mod time_text;
mod weekend;
mod weeknum;
mod workday;
mod yearfrac;
use weekend::WeekendPattern;

const SECONDS_PER_DAY: i32 = 86_400;
const SECONDS_PER_DAY_F64: f64 = SECONDS_PER_DAY as f64;

pub(crate) fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0)
}

fn last_day_of_feb(year: i32) -> u32 {
    if is_leap_year(year) {
        29
    } else {
        28
    }
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

    if day1 == 31 || (m1 == 2 && day1 == last_day_of_feb(y1) as i32) {
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

// The time of day of a serial in whole seconds, after rounding it to the
// nearest millisecond (0..=86399; a time that rounds up to midnight is 0).
fn seconds_of_day(v: f64) -> i64 {
    let ms = (v.rem_euclid(1.0) * SECONDS_PER_DAY_F64 * 1000.0).round() as i64;
    (ms % (SECONDS_PER_DAY as i64 * 1000)) / 1000
}

// ---------------------------------------------------------------------------
// Helper macros to eliminate boilerplate in date/time component extraction
// functions (DAY, MONTH, YEAR, HOUR, MINUTE, SECOND).
// ---------------------------------------------------------------------------

// Generate DAY / MONTH / YEAR helpers – convert the serial number with
// `excel_serial_to_ymd` and return the component at tuple index `$part`
// (0 = year, 1 = month, 2 = day) as a number.
macro_rules! date_part_fn {
    ($name:ident, $part:tt) => {
        pub(crate) fn $name(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
            if args.len() != 1 {
                return CalcResult::new_args_number_error(cell);
            }
            // Convert the serial number (in the workbook's date system: YEAR(0) is 1904 in a
            // 1904 workbook) to its date and return the requested component. Broadcasts
            // element-wise over ranges/arrays. The date system is copied out so the closure
            // does not borrow self.
            let date_system = self.workbook.settings.date_system;
            self.apply_number_unary(&args[0], cell, |f| {
                match date_system
                    .to_1900_serial(f.floor() as i64)
                    .and_then(excel_serial_to_ymd)
                {
                    Some(ymd) => Ok(f64::from(ymd.$part)),
                    None => Err(Error::NUM),
                }
            })
        }
    };
}

// Generate HOUR / MINUTE / SECOND helpers – extract the desired component from
// a day-fraction value. We pass an extraction closure so each helper can keep
// its own formula while sharing the surrounding boilerplate.
macro_rules! time_part_fn {
    ($name:ident, $extract:expr) => {
        pub(crate) fn $name(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
            if args.len() != 1 {
                return CalcResult::new_args_number_error(cell);
            }
            // Lifts over a range or array (probes/date-argument-modes F27:F32); #NUM! from 2958466 once rounded to the second (probes/time-parts-rounding-window).
            self.lift_numbers(args, cell, "Invalid time", |[value]| {
                let last = i64::from(MAXIMUM_DATE_SERIAL_NUMBER);
                if !(value >= 0.0 && days360_serial(value) <= last) {
                    return Err(Error::NUM);
                }
                Ok(($extract)(value))
            })
        }
    };
}

use crate::cast::{array_node_to_string, number_text_too_long, NumberOrArray};
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::date_text::parse_date;
use crate::formatter::dates::date_month_and_day;
use crate::formatter::dates::permissive_date_to_serial_number;
use crate::formatter::dates::shift_months;
use crate::formatter::dates::DATE_OUT_OF_RANGE_MESSAGE;
use crate::functions::lift::num_error;
use crate::functions::text::text_arg::broadcast_text;
use crate::functions::util::array_node_to_calc_result;
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

/// Saturday-Sunday weekend (pattern 1), Monday first.
const DEFAULT_WEEKEND: [bool; 7] = WeekendPattern::SatSun.to_mask();

/// A weekend argument: one mask, or one (possibly invalid) mask per element of a lifted
/// range or array.
enum Weekend {
    One([bool; 7]),
    Many(Vec<Vec<Result<[bool; 7], CalcResult>>>),
}

impl Weekend {
    /// Runs `compute` on each mask: the value (or the error) for [`Weekend::One`], an array
    /// of them for [`Weekend::Many`].
    fn lift(&self, mut compute: impl FnMut([bool; 7]) -> Result<f64, CalcResult>) -> CalcResult {
        let into_node = |r: Result<f64, CalcResult>| match r {
            Ok(n) => ArrayNode::Number(n),
            Err(CalcResult::Error { error, .. }) => ArrayNode::Error(error),
            // Every `Err` of the weekend argument is a `CalcResult::Error`.
            Err(_) => ArrayNode::Error(Error::VALUE),
        };
        match self {
            Weekend::One(mask) => match compute(*mask) {
                Ok(n) => CalcResult::Number(n),
                Err(e) => e,
            },
            Weekend::Many(rows) => {
                // At most 128 distinct masks exist, however many cells the argument has, so each
                // is computed once: the day loop of the caller is millions of steps long.
                let mut computed: HashMap<[bool; 7], ArrayNode> = HashMap::new();
                CalcResult::Array(
                    rows.iter()
                        .map(|row| {
                            row.iter()
                                .map(|mask| match mask {
                                    Ok(mask) => computed
                                        .entry(*mask)
                                        .or_insert_with(|| into_node(compute(*mask)))
                                        .clone(),
                                    Err(e) => into_node(Err(e.clone())),
                                })
                                .collect()
                        })
                        .collect(),
                )
            }
        }
    }
}

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
            let serial = time_text::parse_date_clock_serial(text, self.locale, year)?.floor();
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

    // -----------------------------------------------------------------------
    // Auto-generated DATE part helpers (DAY / MONTH / YEAR)
    // -----------------------------------------------------------------------
    date_part_fn!(fn_day, 2);
    date_part_fn!(fn_month, 1);
    date_part_fn!(fn_year, 0);

    // year, month, day
    pub(crate) fn fn_date(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let date_system = self.workbook.settings.date_system;
        self.lift_numbers(
            args,
            cell,
            DATE_OUT_OF_RANGE_MESSAGE,
            |[year, month, day]| {
                let mut year = year.floor() as i32;
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
            },
        )
    }

    /// Walk a scalar / range / array node and invoke the provided closure with every
    /// numeric date serial that is encountered.
    fn collect_serial_numbers<F>(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        mut handle: F,
    ) -> Result<(), CalcResult>
    where
        F: FnMut(i64) -> Result<(), CalcResult>,
    {
        match self.evaluate_node_in_context(node, cell) {
            CalcResult::Number(v) => {
                let serial = v.floor() as i64;
                // Validate serial is in bounds
                self.excel_date(serial, cell)?;
                handle(serial)?;
            }
            CalcResult::Range { left, right } => {
                let areas = [Range { left, right }];
                self.for_each_cell_in_areas(&areas, cell, |model, value| match value {
                    CalcResult::Number(v) => {
                        let serial = v.floor() as i64;
                        model.excel_date(serial, cell)?;
                        handle(serial)
                    }
                    CalcResult::EmptyCell => Ok(()),
                    _ => Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid holiday date".to_string(),
                    )),
                })?;
            }
            CalcResult::Array(array) => {
                for row in array {
                    for value in row {
                        match value {
                            ArrayNode::Number(num) => {
                                let serial = num.floor() as i64;
                                self.excel_date(serial, cell)?;
                                handle(serial)?;
                            }
                            ArrayNode::Error(error) => {
                                return Err(CalcResult::Error {
                                    error,
                                    origin: cell,
                                    message: "Error in array".to_string(),
                                });
                            }
                            _ => {
                                return Err(CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Invalid holiday date".to_string(),
                                ))
                            }
                        }
                    }
                }
            }
            e @ CalcResult::Error { .. } => return Err(e),
            _ => {
                return Err(CalcResult::new_error(
                    Error::VALUE,
                    cell,
                    "Invalid holiday date".to_string(),
                ))
            }
        }
        Ok(())
    }

    fn get_array_of_dates(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<i64>, CalcResult> {
        let mut values = Vec::new();
        self.collect_serial_numbers(arg, cell, |serial| {
            values.push(serial);
            Ok(())
        })?;
        Ok(values)
    }

    /// The start and end serials of NETWORKDAYS / NETWORKDAYS.INTL (booleans are rejected).
    fn networkdays_bounds(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(i64, i64), CalcResult> {
        let start = self.get_number_no_bools(&args[0], cell)?.floor() as i64;
        let end = self.get_number_no_bools(&args[1], cell)?.floor() as i64;
        Ok((start, end))
    }

    /// The holiday serials of NETWORKDAYS / NETWORKDAYS.INTL (none without the argument).
    fn networkdays_holidays(
        &mut self,
        arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<std::collections::HashSet<i64>, CalcResult> {
        match arg {
            Some(arg) => Ok(self.get_array_of_dates(arg, cell)?.into_iter().collect()),
            None => Ok(std::collections::HashSet::new()),
        }
    }

    pub(crate) fn fn_networkdays(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (start_serial, end_serial) = match self.networkdays_bounds(args, cell) {
            Ok(bounds) => bounds,
            Err(e) => return e,
        };
        let holidays = match self.networkdays_holidays(args.get(2), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };

        let (from, to, sign) = if start_serial <= end_serial {
            (start_serial, end_serial, 1.0)
        } else {
            (end_serial, start_serial, -1.0)
        };
        let mut count = 0i64;
        for serial in from..=to {
            let date = match self.excel_date(serial, cell) {
                Ok(d) => d,
                Err(e) => return e,
            };
            let weekday = date.weekday().number_from_monday();
            let is_weekend = matches!(weekday, 6 | 7);
            if !is_weekend && !holidays.contains(&serial) {
                count += 1;
            }
        }
        CalcResult::Number(count as f64 * sign)
    }

    fn excel_date(
        &self,
        serial: i64,
        cell: CellReferenceIndex,
    ) -> Result<chrono::NaiveDate, CalcResult> {
        match self.workbook.settings.date_system.date_of_serial(serial) {
            Some(date) => Ok(date),
            None => Err(CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "Out of range parameters for date".to_string(),
            }),
        }
    }

    /// The weekend argument of WORKDAY.INTL / NETWORKDAYS.INTL. A multi-cell range or an array
    /// lifts, one mask per element (probes/date-argument-modes F41:F42, F63:F64).
    fn weekend_arg(
        &mut self,
        node: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<Weekend, CalcResult> {
        let Some(node) = node else {
            return Ok(Weekend::One(DEFAULT_WEEKEND));
        };
        let elements = match self.evaluate_node_in_context(node, cell) {
            // A 3D range (two sheets) is unprobed (T-1428): never lift it over its first sheet.
            CalcResult::Range { left, right } if left != right && left.sheet == right.sheet => {
                self.evaluate_range(left, right, cell)?
            }
            CalcResult::Array(rows) => rows,
            value => return Self::weekend_mask(value, cell).map(Weekend::One),
        };
        Ok(Weekend::Many(
            elements
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|n| Self::weekend_mask(array_node_to_calc_result(n, cell), cell))
                        .collect()
                })
                .collect(),
        ))
    }

    /// The weekend mask (Monday first) a scalar weekend argument stands for.
    fn weekend_mask(value: CalcResult, cell: CellReferenceIndex) -> Result<[bool; 7], CalcResult> {
        // Default: Saturday-Sunday weekend (pattern 1)
        let mut weekend = DEFAULT_WEEKEND;
        match value {
            CalcResult::Number(n) => {
                let code = n.trunc() as i32;
                if (n - n.trunc()).abs() > f64::EPSILON {
                    return Err(CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                weekend = match WeekendPattern::try_from(code) {
                    Ok(pattern) => pattern.to_mask(),
                    Err(_) => {
                        return Err(CalcResult::new_error(
                            Error::NUM,
                            cell,
                            "Invalid weekend".to_string(),
                        ))
                    }
                };
                Ok(weekend)
            }
            CalcResult::String(s) => {
                if s.len() != 7 {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                if !s.chars().all(|c| c == '0' || c == '1') {
                    return Err(CalcResult::new_error(
                        Error::VALUE,
                        cell,
                        "Invalid weekend".to_string(),
                    ));
                }
                weekend = [false; 7];
                for (i, ch) in s.chars().enumerate() {
                    weekend[i] = ch == '1';
                }
                Ok(weekend)
            }
            CalcResult::Boolean(_) => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Invalid weekend".to_string(),
            )),
            e @ CalcResult::Error { .. } => Err(e),
            CalcResult::EmptyCell | CalcResult::EmptyArg => Ok(weekend),
            CalcResult::Range { .. } | CalcResult::Array(_) | CalcResult::Lambda(_) => {
                Err(CalcResult::Error {
                    error: Error::VALUE,
                    origin: cell,
                    message: "Invalid weekend".to_string(),
                })
            }
        }
    }

    pub(crate) fn fn_networkdays_intl(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(2..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let (start_serial, end_serial) = match self.networkdays_bounds(args, cell) {
            Ok(bounds) => bounds,
            Err(e) => return e,
        };

        let weekend = match self.weekend_arg(args.get(2), cell) {
            Ok(w) => w,
            Err(e) => return e,
        };

        let holidays = match self.networkdays_holidays(args.get(3), cell) {
            Ok(h) => h,
            Err(e) => return e,
        };

        let (from, to, sign) = if start_serial <= end_serial {
            (start_serial, end_serial, 1.0)
        } else {
            (end_serial, start_serial, -1.0)
        };
        // The days between are counted per weekday once, so each mask of a lifted weekend
        // costs seven additions, not another scan of up to 2.9M days. Serials between two valid
        // ones are valid.
        let mut per_weekday = [0i64; 7];
        for serial in [from, to] {
            if let Err(e) = self.excel_date(serial, cell) {
                return e;
            }
        }
        for serial in from..=to {
            if holidays.contains(&serial) {
                continue;
            }
            let Some(date) = self.workbook.settings.date_system.date_of_serial(serial) else {
                continue;
            };
            per_weekday[date.weekday().number_from_monday() as usize - 1] += 1;
        }
        weekend.lift(|weekend_pattern| {
            let count: i64 = (0..7)
                .filter(|&d| !weekend_pattern[d])
                .map(|d| per_weekday[d])
                .sum();
            Ok(count as f64 * sign)
        })
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
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a range or array in every argument (probes/date-argument-modes F43:F46).
        self.lift_numbers(args, cell, "Invalid time", |[hour, minute, second]| {
            let total_seconds = hour.floor() * 3600.0 + minute.floor() * 60.0 + second.floor();
            if total_seconds < 0.0 {
                return Err(Error::NUM);
            }
            // Excel divides, then drops whole days (TIME_HOUR_MINUTE_SECOND.xlsx D3/D14).
            Ok((total_seconds / SECONDS_PER_DAY_F64) % 1.0)
        })
    }

    // -----------------------------------------------------------------------
    // Auto-generated TIME part helpers (HOUR / MINUTE / SECOND)
    // -----------------------------------------------------------------------

    time_part_fn!(fn_hour, |v: f64| (v.rem_euclid(1.0) * 24.0).floor());
    // MINUTE and SECOND round the time of day to the nearest millisecond
    // before taking it apart, as Excel does: MINUTE(0.520833333) is 30
    // (12:29:59.99997 is 12:30:00.000), SECOND(TIME(14,30,45.999)) stays 45.
    time_part_fn!(fn_minute, |v: f64| ((seconds_of_day(v) / 60) % 60) as f64);
    time_part_fn!(fn_second, |v: f64| (seconds_of_day(v) % 60) as f64);

    pub(crate) fn fn_timevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a range or array like the text functions (probes/date-argument-modes F47:F48).
        let text = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text], |i, j| {
            let text = array_node_to_string(text.elem(i, j))?;
            self.parse_time_text(&text)
                .map(ArrayNode::Number)
                .ok_or(Error::VALUE)
        })
    }

    pub(crate) fn fn_datevalue(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a range or array like the text functions (probes/date-argument-modes F9:F10).
        let text = match self.text_arg(&args[0], cell) {
            Ok(t) => t,
            Err(e) => return e,
        };
        broadcast_text(cell, &[&text], |i, j| match text.elem(i, j) {
            ArrayNode::String(s) => self
                .parse_datevalue_text(s)
                .map(|serial| ArrayNode::Number(f64::from(serial)))
                .ok_or(Error::VALUE),
            ArrayNode::Error(e) => Err(e.clone()),
            _ => Err(Error::VALUE),
        })
    }

    pub(crate) fn fn_datedif(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 3 {
            return CalcResult::new_args_number_error(cell);
        }

        // Errors keep their pre-lift order: the dates' own, #NUM! of a reversed pair, the unit's, then
        // out-of-calendar #NUM! (T-1355; UNVERIFIED, dates.md "DATEDIF and DAYS360 error order").
        let unit = self.get_string(&args[2], cell).map(|s| s.to_uppercase());
        // Both dates lift over a range or array (probes/date-argument-modes F5:F8).
        let operands = args[..2]
            .iter()
            .map(|arg| {
                let value = match self.evaluate_node_in_context(arg, cell) {
                    CalcResult::String(s) => array_node_to_calc_result(
                        &self.datedif_date_text(ArrayNode::String(s)),
                        cell,
                    ),
                    value => value,
                };
                self.number_or_array(value, cell).map(|na| match na {
                    NumberOrArray::Array(rows) => NumberOrArray::Array(
                        rows.into_iter()
                            .map(|row| row.into_iter().map(|n| self.datedif_date_text(n)).collect())
                            .collect(),
                    ),
                    number @ NumberOrArray::Number(_) => number,
                })
            })
            .collect();
        let date_system = self.workbook.settings.date_system;
        self.lift_number_operands(operands, cell, "Invalid arguments", |[start, end]| {
            Self::datedif_value(date_system, start, end, unit.as_deref().map_err(error_kind))
        })
    }

    /// A DATEDIF date given as text (other elements are kept): date text reads as DATEVALUE reads
    /// it, so a 1904 workbook counts it from 1904-01-01 (T-1330); plain number text is that number
    /// (probes/date-argument-modes F89); any other text is #VALUE! (the behaviour before lifting).
    fn datedif_date_text(&self, node: ArrayNode) -> ArrayNode {
        let ArrayNode::String(s) = node else {
            return node;
        };
        if let Some(serial) = self.parse_datevalue_text(&s) {
            return ArrayNode::Number(f64::from(serial));
        }
        match parse_excel_number(s.trim()) {
            Some(n) if !number_text_too_long(&s) => ArrayNode::Number(n),
            _ => ArrayNode::Error(Error::VALUE),
        }
    }

    /// DATEDIF of two date serials for the (upper-cased) `unit`.
    fn datedif_value(
        date_system: DateSystem,
        start_serial: f64,
        end_serial: f64,
        unit: Result<&str, Error>,
    ) -> Result<f64, Error> {
        let (start_serial, end_serial) = (start_serial.floor() as i64, end_serial.floor() as i64);
        if end_serial < start_serial {
            return Err(Error::NUM);
        }
        // A unit error beats an out-of-calendar start date (dates.md "DATEDIF argument errors before date checks").
        let unit = unit?;
        let start = date_system.date_of_serial(start_serial).ok_or(Error::NUM)?;
        let end = date_system.date_of_serial(end_serial).ok_or(Error::NUM)?;
        match unit {
            "Y" => {
                let mut years = end.year() - start.year();
                if (end.month(), end.day()) < (start.month(), start.day()) {
                    years -= 1;
                }
                Ok(f64::from(years))
            }
            "M" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                Ok(f64::from(months))
            }
            "D" => Ok((end_serial - start_serial) as f64),
            "YM" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                Ok(f64::from((months % 12).abs()))
            }
            "YD" => {
                let make_date = |y: i32, m: u32, d: u32| -> Result<chrono::NaiveDate, Error> {
                    chrono::NaiveDate::from_ymd_opt(y, m, d).ok_or(Error::NUM)
                };

                // Compute the last valid day of a given month/year.
                let make_last_day_of_month = |y: i32, m: u32| -> Result<chrono::NaiveDate, Error> {
                    let (next_y, next_m) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                    let first_next = make_date(next_y, next_m, 1)?;
                    let last_day = first_next - chrono::Duration::days(1);
                    make_date(y, m, last_day.day())
                };

                // Attempt to build the anniversary date in the end year.
                let mut start_adj =
                    match chrono::NaiveDate::from_ymd_opt(end.year(), start.month(), start.day()) {
                        Some(d) => d,
                        None => make_last_day_of_month(end.year(), start.month())?,
                    };

                // If the adjusted date is after the end date, shift one year back.
                if start_adj > end {
                    let shift_year = end.year() - 1;
                    start_adj = match chrono::NaiveDate::from_ymd_opt(
                        shift_year,
                        start.month(),
                        start.day(),
                    ) {
                        Some(d) => d,
                        None => make_last_day_of_month(shift_year, start.month())?,
                    };
                }

                Ok((end - start_adj).num_days() as f64)
            }
            "MD" => {
                let mut months =
                    (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                if end.day() < start.day() {
                    months -= 1;
                }
                // Both dates are in Excel's calendar, so the shift cannot leave chrono's range;
                // the checked form keeps a panic off this path all the same (T-1419).
                let Some(start_shifted) = shift_months(start, months) else {
                    return Err(Error::NUM);
                };
                Ok((end - start_shifted).num_days() as f64)
            }
            _ => Err(Error::VALUE),
        }
    }

    pub(crate) fn fn_days(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        // Lifts over a range or array in both arguments (probes/date-argument-modes F11:F14).
        self.lift_numbers(
            args,
            cell,
            "Out of range parameters for date",
            |[end, start]| {
                let (end_serial, start_serial) = (end.floor() as i64, start.floor() as i64);
                // Excel accepts any serial in 0..=MAXIMUM (a 0 serial is 1900-01-00) and
                // reports #NUM! otherwise. DAYS works directly on serials, so the dates
                // themselves are never materialised.
                let range = 0..=MAXIMUM_DATE_SERIAL_NUMBER as i64;
                if !range.contains(&start_serial) || !range.contains(&end_serial) {
                    return Err(Error::NUM);
                }
                Ok((end_serial - start_serial) as f64)
            },
        )
    }

    pub(crate) fn fn_days360(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(2..=3).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        // The European-method flag follows Excel's logical coercion: numbers are
        // truthy when non-zero, and only the text "TRUE"/"FALSE" is accepted.
        // The dates' own errors come first, then the method's, then the dates' #NUM!, as before the
        // lift (UNVERIFIED, dates.md "DATEDIF and DAYS360 error order", T-1430).
        let method = if args.len() == 3 {
            self.get_boolean(&args[2], cell)
        } else {
            Ok(false)
        };
        // Both dates lift over a range or array (probes/date-argument-modes F15:F18).
        let date_system = self.workbook.settings.date_system;
        self.lift_numbers(
            args,
            cell,
            "Out of range parameters for date",
            |[start, end]| {
                let method = method.as_ref().map_err(error_kind)?;
                let ymd = |serial| {
                    date_system
                        .to_1900_serial(days360_serial(serial))
                        .and_then(excel_serial_to_ymd)
                        .ok_or(Error::NUM)
                };
                let (start, end) = (ymd(start)?, ymd(end)?);
                Ok(if *method {
                    days360_eu(start, end)
                } else {
                    days360_us(start, end)
                })
            },
        )
    }

    pub(crate) fn fn_weekday(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=2).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let ds = self.workbook.settings.date_system;
        let sat = chrono::NaiveDate::from_ymd_opt(1899, 12, 30); // 1900 serial 0: weekday-serial-zero
        let date = move |serial: f64| {
            let s = serial.floor();
            let d = ds.date_of_serial(s as i64).or(sat.filter(|_| s == 0.0));
            d.ok_or(Error::NUM)
        };
        if args.len() == 1 {
            // Broadcasts element-wise over a range/array date argument.
            return self
                .apply_number_unary(&args[0], cell, |serial| weekday_number(date(serial)?, 1));
        }
        // The return type lifts too (probes/date-argument-modes F51:F52).
        self.lift_numbers(args, cell, "Invalid arguments", |[serial, return_type]| {
            weekday_number(date(serial)?, return_type as i32)
        })
    }

    pub(crate) fn fn_isoweeknum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        // Broadcasts element-wise over a range/array date argument.
        let date_system = self.workbook.settings.date_system;
        self.apply_number_unary(&args[0], cell, |serial| {
            let serial = date_system.to_1900_serial(serial.floor() as i64);
            weeknum::iso_week(serial.ok_or(Error::NUM)?)
        })
    }
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
