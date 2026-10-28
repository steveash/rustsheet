//! The Saka (calendar digit 10) and Hebrew (calendar digit 8) dates of a `[$-...]` locale tag, and
//! the calendar each date token of a tagged section reads. Evidence: probes/t1923-calendar-digits
//! A85:A213, t2779-hebrew-saka-names, t2779-hebrew-saka-more (excel-graph, 2026-10-26/27,
//! T-2779); number-formats.md "Calendars of a locale tag".

use chrono::{Datelike, Duration, NaiveDate};

use super::{
    date_names::hijri_ymd,
    dates::excel_serial_to_ymd,
    locale_tag::{japanese_era, Calendar},
    lunar_calendar::{lunar_ymd, LAST_SERIAL},
    parser::{DatePart, TextToken},
};
use crate::{functions::date_and_time::is_leap_year, locale::excel_date_names::DateNames};

/// Days from 1899-12-30 to the real date of 1900-system `serial_1900`, for the calendars that
/// convert real dates: serial 0 reads as serial 1 (1900-01-01), serials 1 to 59 sit one day later
/// than their number because serial 60 is the fictitious 1900-02-29, which has no date (`None`,
/// `#VALUE!`), as do serials outside `0..=2958465` (Hijri: probes/t1613-hijri-b2; Saka and
/// Hebrew: t1923-calendar-digits A205:A210, `[$-100409]` and `[$-80409]` of 0 and 1 are
/// "1821-10-11" and "5660-05-01", of 60 `#VALUE!`).
pub(crate) fn real_day(serial_1900: i64) -> Option<i64> {
    match serial_1900 {
        0 => Some(2),
        1..=59 => Some(serial_1900 + 1),
        61..=LAST_SERIAL => Some(serial_1900),
        _ => None,
    }
}

/// The `(year, month, day)` the date tokens of a section print for `serial_1900`: Hijri after a
/// `B2` prefix or under calendar digit 6, the tag's calendar otherwise, Gregorian with no tag.
pub(crate) fn calendar_ymd(
    calendar: Option<Calendar>,
    hijri: bool,
    serial_1900: i64,
) -> Option<(i32, u32, u32)> {
    if hijri {
        return hijri_ymd(serial_1900);
    }
    match calendar {
        Some(Calendar::Hijri) => hijri_ymd(serial_1900),
        Some(Calendar::Lunar(table)) if serial_1900 <= LAST_SERIAL => lunar_ymd(serial_1900, table),
        Some(Calendar::Saka) => saka_ymd(serial_1900),
        Some(Calendar::Hebrew(_)) => hebrew_ymd(serial_1900),
        _ => excel_serial_to_ymd(serial_1900),
    }
}

/// The real date of `serial_1900` ([`real_day`]).
fn real_date(serial_1900: i64) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(1899, 12, 30)?
        .checked_add_signed(Duration::days(real_day(serial_1900)?))
}

/// The Saka date of `serial_1900`: the year starts on 1 Chaitra, 22 March or 21 March in a
/// Gregorian leap year, and is the Gregorian year less 78. Chaitra has 30 days, 31 in a leap year,
/// the next five months 31 and the last six 30 (`[$-100409]yyyy-mm-dd` of 2024-03-21 is
/// "1946-01-01", of 2023-06-13 "1945-03-23"; 9999-12-31 is "9921-10-10").
fn saka_ymd(serial_1900: i64) -> Option<(i32, u32, u32)> {
    let date = real_date(serial_1900)?;
    let new_year =
        |year: i32| NaiveDate::from_ymd_opt(year, 3, if is_leap_year(year) { 21 } else { 22 });
    let mut year = date.year();
    if date < new_year(year)? {
        year -= 1;
    }
    let mut rest = (date - new_year(year)?).num_days();
    let mut month = 1;
    loop {
        let length = match month {
            1 if is_leap_year(year) => 31,
            1 | 7.. => 30,
            _ => 31,
        };
        if rest < length || month == 12 {
            break;
        }
        rest -= length;
        month += 1;
    }
    Some((year - 78, month, u32::try_from(rest + 1).ok()?))
}

/// The day number (0001-01-01 being 1) of 1 Tishrei of Hebrew year 1, the arithmetic calendar's
/// epoch.
const HEBREW_EPOCH: i64 = -1_373_427;
/// The Gregorian years of Excel's Hebrew year table. A date's Hebrew month and day come from the
/// table entry of its Gregorian year: the Hebrew date of 1 January and the length of the Hebrew year
/// it falls in, then the next year's entry from 1 Tishrei on. Each entry agrees with the arithmetic
/// calendar, but past 2238 the table wraps: Gregorian year `y` reads the entry of
/// 1585 + (y - 1585) mod 654. So Cheshvan 5999 has 29 days (2238-11-08 is "5999-03-01", the
/// arithmetic 5999-02-30) and 2239-01-01 is "5999-05-01", as 1585-01-01 is 5345-05-01; this matches
/// all 512 cells of probes/t2954-hebrew-drift and every later date of t2779-hebrew-saka-more.
const HEBREW_TABLE_FIRST_YEAR: i32 = 1585;
/// The last Gregorian year of Excel's Hebrew year table ([`HEBREW_TABLE_FIRST_YEAR`]).
const HEBREW_TABLE_LAST_YEAR: i32 = 2238;
/// The Hebrew year of 1 January of a Gregorian year is the Gregorian year plus 3760.
const HEBREW_YEAR_OF_JANUARY: i32 = 3760;
/// The last Hebrew year whose months have names: from 1 Tishrei 8263 (4502-09-30) a month name
/// token under a locale (`[$-80409]mmm`, `mmmm`, `mmmmm`) is `#VALUE!` while the numbers and the
/// weekday names still print (`[$-80409]yyyy-mm-dd|ddd` of 4502-09-30 is "8263-01-01|Shabat");
/// with no locale the Gregorian names by month number still print (`[$-80000]mmm` is "Jan";
/// probes/t2954-hebrew-value-limit).
pub(crate) const HEBREW_LAST_NAMED_YEAR: i32 = 8262;

/// Days from the epoch to 1 Tishrei of `year` before the postponements of the year's length: the
/// molad of Tishrei, a day later when it falls on a Sunday, Wednesday or Friday.
fn hebrew_elapsed_days(year: i64) -> i64 {
    let months = (235 * year - 234).div_euclid(19);
    let parts = 12_084 + 13_753 * months;
    let day = 29 * months + parts.div_euclid(25_920);
    if (3 * (day + 1)).rem_euclid(7) < 3 {
        day + 1
    } else {
        day
    }
}

/// The day number of 1 Tishrei of `year`, postponed so that no year has 356 days and no year
/// after a leap year 382.
fn hebrew_new_year(year: i64) -> i64 {
    let (before, this, after) = (
        hebrew_elapsed_days(year - 1),
        hebrew_elapsed_days(year),
        hebrew_elapsed_days(year + 1),
    );
    let delay = if after - this == 356 {
        2
    } else if this - before == 382 {
        1
    } else {
        0
    };
    HEBREW_EPOCH + this + delay
}

/// The days of month `month` (from Tishrei, 1) of a Hebrew year of `length` days: Cheshvan has 30
/// days in a 355- or 385-day year, Kislev 29 in a 353- or 383-day one; Tevet has 29 and a leap
/// year's Adar I 30; the other months alternate from there.
fn hebrew_month_length(month: u32, length: i64) -> i64 {
    match (month, length > 380) {
        (2, _) => 29 + i64::from(length % 10 == 5),
        (3, _) => 30 - i64::from(length % 10 == 3),
        (1 | 5, _) | (6, true) => 30,
        (4, _) => 29,
        (m, true) => 30 - i64::from(m % 2),
        (m, false) => 29 + i64::from(m % 2),
    }
}

/// The arithmetic calendar's Hebrew month and day of day number `day`, and the length of its year.
fn arithmetic_hebrew_date(day: i64) -> (u32, i64, i64) {
    // A first guess from the mean year of 35,975,351 / 98,496 days, then exact.
    let mut year = (day - HEBREW_EPOCH) * 98_496 / 35_975_351 + 1;
    while hebrew_new_year(year + 1) <= day {
        year += 1;
    }
    while hebrew_new_year(year) > day {
        year -= 1;
    }
    let length = hebrew_new_year(year + 1) - hebrew_new_year(year);
    let mut rest = day - hebrew_new_year(year);
    let mut month = 1;
    while month < 13 && rest >= hebrew_month_length(month, length) {
        rest -= hebrew_month_length(month, length);
        month += 1;
    }
    (month, rest + 1, length)
}

/// The entry of Excel's Hebrew year table for Gregorian `year` ([`HEBREW_TABLE_FIRST_YEAR`]): the
/// Hebrew month and day of 1 January and the length of the Hebrew year it falls in.
fn hebrew_table_entry(year: i32) -> Option<(u32, i64, i64)> {
    let year = if year <= HEBREW_TABLE_LAST_YEAR {
        year
    } else {
        let size = HEBREW_TABLE_LAST_YEAR - HEBREW_TABLE_FIRST_YEAR + 1;
        HEBREW_TABLE_FIRST_YEAR + (year - HEBREW_TABLE_FIRST_YEAR).rem_euclid(size)
    };
    let january = NaiveDate::from_ymd_opt(year, 1, 1)?;
    Some(arithmetic_hebrew_date(i64::from(
        january.num_days_from_ce(),
    )))
}

/// The Hebrew date of `serial_1900` as Excel reads it from its year table
/// ([`HEBREW_TABLE_FIRST_YEAR`]), months numbered from Tishrei (1), so a leap year's Adar I and
/// Adar II are 6 and 7 and its Elul 13 (`[$-80409]yyyy-mm-dd` of 2023-03-15 is "5783-06-22", of
/// 2024-09-20 "5784-13-17"; 9999-12-31 is "13760-04-22"), and the length of that Hebrew year.
fn hebrew_date(serial_1900: i64) -> Option<(i32, u32, u32, i64)> {
    let date = real_date(serial_1900)?;
    let (mut month, mut day, mut length) = hebrew_table_entry(date.year())?;
    let mut year = date.year() + HEBREW_YEAR_OF_JANUARY;
    day += i64::from(date.ordinal0());
    // At most a year of months, plus the new year's.
    while day > hebrew_month_length(month, length) {
        day -= hebrew_month_length(month, length);
        month += 1;
        if month > if length > 380 { 13 } else { 12 } {
            month = 1;
            year += 1;
            (_, _, length) = hebrew_table_entry(date.year() + 1)?;
        }
    }
    Some((year, month, u32::try_from(day).ok()?, length))
}

/// The `(year, month, day)` of [`hebrew_date`].
fn hebrew_ymd(serial_1900: i64) -> Option<(i32, u32, u32)> {
    hebrew_date(serial_1900).map(|(year, month, day, _)| (year, month, day))
}

/// What a month name token prints in the Hebrew calendar for the date of `serial_1900`, with the
/// leap-year Adars `adars` of the tag's names, or `None` when the token reads `names` by month
/// number as usual (and for a weekday token). With a locale every month token is the whole name (`[$-80409]mmmmm` of Adar is
/// "Adar", of a leap year's month 7 "`AdarII`"). With no locale (`adars` `None`) month 13 prints
/// nothing for `mmm` and `mmmm` and U+0001 for `mmmmm` (`[$-80000]m|mmm|mmmm|mmmmm` of 2024-09-20
/// is "13|||\u{1}"; probes/t2779-hebrew-saka-more E1, t2779-hebrew-saka-names B8:D21).
pub(crate) fn hebrew_month_name(
    token: &TextToken,
    names: &'static DateNames,
    adars: Option<&'static [&'static str; 2]>,
    serial_1900: i64,
) -> Option<&'static str> {
    if matches!(token, TextToken::DayName | TextToken::DayNameShort) {
        return None;
    }
    // Leap or not by the year table, which past 2238 differs from the arithmetic calendar's
    // (`[$-80409]mmmm` of 4502-03-01, 8262-06-24, is "AdarI"; probes/t2954-hebrew-value-limit F2).
    let (_, month, _, length) = hebrew_date(serial_1900)?;
    let Some(adars) = adars else {
        return match (month, token) {
            (13, TextToken::MonthLetter) => Some("\u{1}"),
            (13, _) => Some(""),
            _ => None,
        };
    };
    let index = match (length > 380, month) {
        (true, 6 | 7) => return Some(adars[month as usize - 6]),
        (true, 8..) => month - 2,
        _ => month - 1,
    };
    names.months.get(index as usize).copied()
}

/// The Buddhist era counts from 543 BC: serial 100 (1900-04-09) is "43" under `b` and "2443" under
/// `bbbb` (probes/text-format-bare-literals A41-A46; gen-m7/6db767d720c5b55a A30).
const BUDDHIST_ERA_OFFSET: i32 = 543;

/// The Korean calendar (locale tag calendar digit 5) counts from 2333 BC: `[$-50409]yyyy` of 45000
/// is "4356" (probes/text-locale-calendars A44-A46).
const KOREAN_ERA_OFFSET: i32 = 2333;

/// What a year or era token (`yy`, `yyyy`, `b`, `bbbb`, `e`, `ee`, `g`) of the date section `part`
/// prints for 1900-system `serial_1900`, whose date in the section's calendar is `ymd`; `seen_era`
/// when a `g` came earlier in the section.
///
/// Excel: `b` is the Buddhist year with no locale tag and the Gregorian one under a tag but in the
/// Thai calendar (`[$-40C]bbbb` is "2023", `[$-7041E]bbbb` "2566"); `yy` and `yyyy` count in the
/// tag's calendar (probes/text-locale-calendars, text-locale-calendars-detail A2, A14-A23). After
/// a `B2` before a calendar-3, -5 or -7 tag (`part.year_twice`) every year token converts once
/// more: `B2[$-70409]yyyy|e|bbbb` of 2023-03-15 is "3109|2566|3109", `B2[$-50409]yyyy|e|bbbb`
/// "6689|4356|4356" (probes/t2932-b2-calendar-twice; the Japanese eras in
/// [`japanese_era`]).
pub(crate) fn year_token_text(
    part: &DatePart,
    token: &TextToken,
    seen_era: bool,
    serial_1900: i64,
    ymd: (i32, u32, u32),
) -> String {
    let twice = part.year_twice;
    if let Some(Calendar::Japanese { explicit }) = part.calendar {
        let (era_year, names) = japanese_era(serial_1900, ymd, twice);
        // Twice converted, the era year is negative and prints as an unsigned 32-bit number
        // ("4294965434"), also under `yy`.
        let era_year = era_year.cast_unsigned();
        match token {
            TextToken::Era(length) => return names[(*length).clamp(1, 3) - 1].to_string(),
            TextToken::EraYear(false) => return format!("{era_year}"),
            TextToken::EraYear(true) => return format!("{era_year:02}"),
            TextToken::YearShort | TextToken::Year if explicit || seen_era => {
                return format!("{era_year:02}");
            }
            _ => {}
        }
    }
    let year = ymd.0;
    // The one more conversion of a `B2` before the tag.
    let again = match (twice, part.calendar) {
        (true, Some(Calendar::Thai)) => BUDDHIST_ERA_OFFSET,
        (true, Some(Calendar::Korean)) => KOREAN_ERA_OFFSET,
        _ => 0,
    };
    let buddhist = matches!(
        token,
        TextToken::BuddhistYearShort | TextToken::BuddhistYear
    );
    let short = matches!(token, TextToken::YearShort | TextToken::BuddhistYearShort);
    let year = match (token, part.calendar, buddhist) {
        (TextToken::Era(_), ..) => return String::new(),
        (TextToken::EraYear(_), ..) => year + again,
        // After a `g` a Japanese LCID's other calendars print the whole Gregorian year
        // (`[$-10411]g|yy` is "|2023", `[$-70411]g|yyyy` "|2023"); other LCIDs are unaffected
        // (`[$-70409]g|yyyy` is "|2566"; probes/text-locale-japanese-year A13,
        // text-locale-era-gregorian), and a `B2` converts it once (`B2[$-70411]g|yyyy` is
        // "|2566"; probes/t2932-b2-calendar-twice A44).
        (_, Some(_), false) if seen_era && part.japanese_lcid => {
            return format!("{}", year + again)
        }
        (_, None, true) => year + BUDDHIST_ERA_OFFSET,
        (_, Some(Calendar::Thai), _) => year + BUDDHIST_ERA_OFFSET + again,
        (_, Some(Calendar::Korean), false) => year + KOREAN_ERA_OFFSET + again,
        _ => year + again,
    };
    if matches!(token, TextToken::EraYear(_)) {
        return format!("{year}");
    }
    if buddhist && short {
        // `b` prints the third and fourth digits of the year, which differ from its last two
        // past 9999: `B2[$-70409]b|yy` of 9999-12-31 (Thai 11085) is "08|85"
        // (probes/t2932-b2-calendar-twice A24).
        let digits = year.to_string();
        return digits.get(2..4).unwrap_or(&digits).to_string();
    }
    // The Taiwan calendar's `yy` is four digits (probes/text-locale-japanese-year A15).
    if short && (buddhist || part.calendar != Some(Calendar::Taiwan)) {
        format!("{:02}", year % 100)
    } else {
        format!("{year}")
    }
}

#[cfg(test)]
mod tests {
    use super::{hebrew_ymd, real_day, saka_ymd};

    #[test]
    fn real_days_skip_the_fictitious_leap_day() {
        assert_eq!(real_day(0), Some(2));
        assert_eq!(real_day(59), Some(60));
        assert_eq!(real_day(60), None);
        assert_eq!(real_day(61), Some(61));
        assert_eq!(real_day(2_958_466), None);
    }

    #[test]
    fn saka_dates_of_the_probe() {
        // probes/t1923-calendar-digits A134, A137, A142, A144, A205, A207.
        assert_eq!(saka_ymd(45372), Some((1946, 1, 1)));
        assert_eq!(saka_ymd(45007), Some((1945, 1, 1)));
        assert_eq!(saka_ymd(61), Some((1821, 12, 10)));
        assert_eq!(saka_ymd(2_958_465), Some((9921, 10, 10)));
        assert_eq!(saka_ymd(1), Some((1821, 10, 11)));
        assert_eq!(saka_ymd(0), Some((1821, 10, 11)));
        assert_eq!(saka_ymd(60), None);
    }

    #[test]
    fn hebrew_dates_of_the_probe() {
        // probes/t1923-calendar-digits A145, A186, A199, A202, A206; t2779-hebrew-saka-more A10.
        assert_eq!(hebrew_ymd(44927), Some((5783, 4, 8)));
        assert_eq!(hebrew_ymd(45555), Some((5784, 13, 17)));
        assert_eq!(hebrew_ymd(45185), Some((5784, 1, 1)));
        assert_eq!(hebrew_ymd(61), Some((5660, 6, 30)));
        assert_eq!(hebrew_ymd(1), Some((5660, 5, 1)));
        assert_eq!(hebrew_ymd(123_758), Some((5999, 2, 23)));
        // probes/t2954-hebrew-drift A6, A7, B13, B46; t2779-hebrew-saka-more A14; t1923 A204.
        assert_eq!(hebrew_ymd(123_764), Some((5999, 2, 29)));
        assert_eq!(hebrew_ymd(123_765), Some((5999, 3, 1)));
        assert_eq!(hebrew_ymd(123_819), Some((5999, 5, 1)));
        assert_eq!(hebrew_ymd(124_085), Some((6000, 1, 1)));
        assert_eq!(hebrew_ymd(124_823), Some((6001, 13, 28)));
        assert_eq!(hebrew_ymd(2_958_465), Some((13760, 4, 22)));
        assert_eq!(hebrew_ymd(60), None);
    }
}
