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
/// The day number of 1899-12-30, from which [`real_day`] counts.
const DAY_NUMBER_1899_12_30: i64 = 693_594;
/// The last serial whose Hebrew date is modelled: 2238-11-07, 29 Cheshvan 5999. Excel's dates match
/// the arithmetic calendar at every probed date to 2238-11-01 (A10); the days to 29 Cheshvan are
/// extrapolated (a drift that starts there would not show before Kislev). Excel's dates drift from it from Kislev 5999 on (2238-12-01 is "5999-03-24",
/// not 23; 2239-09-24 is "6000-01-01", not 5999-13-24) and are `#VALUE!` from some date between
/// 4000-01-01 and 5000-01-01 to 9999-12-31 (probes/t2779-hebrew-saka-more A1:A35). Past it the
/// date is `#VALUE!` here (TODO(T-2954)).
const HEBREW_LAST_SERIAL: i64 = 123_764;

/// Whether Hebrew `year` has 13 months: years 3, 6, 8, 11, 14, 17 and 19 of the 19-year cycle.
pub(crate) fn is_hebrew_leap_year(year: i32) -> bool {
    (7 * i64::from(year) + 1).rem_euclid(19) < 7
}

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

/// The Hebrew date of `serial_1900` by the arithmetic calendar, months numbered from Tishrei (1),
/// so a leap year's Adar I and Adar II are 6 and 7 and its Elul 13 (`[$-80409]yyyy-mm-dd` of
/// 2023-03-15 is "5783-06-22", of 2024-09-20 "5784-13-17"). `None` past [`HEBREW_LAST_SERIAL`].
fn hebrew_ymd(serial_1900: i64) -> Option<(i32, u32, u32)> {
    if serial_1900 > HEBREW_LAST_SERIAL {
        return None;
    }
    let day = DAY_NUMBER_1899_12_30 + real_day(serial_1900)?;
    // 1900-01-01 is in year 5660; the loop walks at most the years to 5999.
    let mut year = 5660;
    while hebrew_new_year(year + 1) <= day {
        year += 1;
    }
    let length = hebrew_new_year(year + 1) - hebrew_new_year(year);
    let leap = is_hebrew_leap_year(i32::try_from(year).ok()?);
    let mut rest = day - hebrew_new_year(year);
    let mut month: u32 = 1;
    loop {
        // Cheshvan has 30 days in a 355- or 385-day year, Kislev 29 in a 353- or 383-day one;
        // Tevet has 29 and a leap year's Adar I 30; the other months alternate from there.
        let days = match (month, leap) {
            (2, _) => 29 + i64::from(length % 10 == 5),
            (3, _) => 30 - i64::from(length % 10 == 3),
            (1 | 5, _) | (6, true) => 30,
            (4, _) => 29,
            (m, true) => 30 - i64::from(m % 2),
            (m, false) => 29 + i64::from(m % 2),
        };
        if rest < days || month == 13 {
            break;
        }
        rest -= days;
        month += 1;
    }
    Some((
        i32::try_from(year).ok()?,
        month,
        u32::try_from(rest + 1).ok()?,
    ))
}

/// What a month name token prints in the Hebrew calendar for month `month` of `year`, with the
/// leap-year Adars `adars` of the tag's names, or `None` when the token reads `names` by month
/// number as usual (and for a weekday token). With a locale every month token is the whole name (`[$-80409]mmmmm` of Adar is
/// "Adar", of a leap year's month 7 "`AdarII`"). With no locale (`adars` `None`) month 13 prints
/// nothing for `mmm` and `mmmm` and U+0001 for `mmmmm` (`[$-80000]m|mmm|mmmm|mmmmm` of 2024-09-20
/// is "13|||\u{1}"; probes/t2779-hebrew-saka-more E1, t2779-hebrew-saka-names B8:D21).
pub(crate) fn hebrew_month_name(
    token: &TextToken,
    names: &'static DateNames,
    adars: Option<&'static [&'static str; 2]>,
    (year, month): (i32, u32),
) -> Option<&'static str> {
    if matches!(token, TextToken::DayName | TextToken::DayNameShort) {
        return None;
    }
    let Some(adars) = adars else {
        return match (month, token) {
            (13, TextToken::MonthLetter) => Some("\u{1}"),
            (13, _) => Some(""),
            _ => None,
        };
    };
    let index = match (is_hebrew_leap_year(year), month) {
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
    use super::{hebrew_ymd, real_day, saka_ymd, HEBREW_LAST_SERIAL};

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
        assert_eq!(hebrew_ymd(HEBREW_LAST_SERIAL), Some((5999, 2, 29)));
        assert_eq!(hebrew_ymd(HEBREW_LAST_SERIAL + 1), None);
        assert_eq!(hebrew_ymd(60), None);
    }
}
