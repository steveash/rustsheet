//! Day and month name tokens of a date section (`ddd`, `dddd`, `mmm`, `mmmm`, `mmmmm`), and the
//! Hijri calendar of the `B2` prefix. Evidence for Hijri: probes/t1613-hijri-b2 and
//! t1613-hijri-b2-weekday (excel-graph, 2026-10-25); number-formats.md "The `B2` (Hijri) calendar".

use super::parser::TextToken;
use crate::locale::{excel_date_names::DateNames, Locale};

/// Days from 1 Muharram 1 AH to the proleptic Gregorian 1899-12-30, the day a serial from 61 on
/// counts from: the Hijri epoch Excel uses is 622-07-18, which replays all 103 probed dates. Serials
/// 1 to 59 sit one day later because Excel's serials count a fictitious 1900-02-29 (serial 60).
const HIJRI_DAYS_TO_1899_12_30: i64 = 466_580;

/// Days in a 30-year Hijri cycle: 19 years of 354 days and 11 of 355.
const HIJRI_CYCLE_DAYS: i64 = 10_631;

/// `mmm` and `mmmm` under `B2` (the two print the same), Muharram first. Two-word names are
/// joined by a no-break space, as Excel prints them.
const HIJRI_MONTHS: [&str; 12] = [
    "محرم",
    "صفر",
    "ربيع\u{a0}الأول",
    "ربيع\u{a0}الثاني",
    "جمادى\u{a0}الأولى",
    "جمادى\u{a0}الثانية",
    "رجب",
    "شعبان",
    "رمضان",
    "شوال",
    "ذو\u{a0}القعدة",
    "ذو\u{a0}الحجة",
];

/// `ddd` and `dddd` under `B2` (the two print the same), Sunday first.
const HIJRI_DAYS: [&str; 7] = [
    "الأحد",
    "الإثنين",
    "الثلاثاء",
    "الأربعاء",
    "الخميس",
    "الجمعة",
    "السبت",
];

/// Whether year `year_in_cycle` (1-based) of a 30-year cycle has 355 days. Excel's leap years are 2, 5, 7,
/// 10, 13, 15, 18, 21, 24, 26 and 29 (the variant with 15, not 16: it alone replays the probe).
fn is_hijri_leap(year_in_cycle: i64) -> bool {
    (11 * year_in_cycle + 15) % 30 < 11
}

/// The Hijri `(year, month, day)` Excel shows under `B2` for a 1900-system serial: the tabular
/// calendar above. Serial 0 shows serial 1's date ("1317-08-29") and serial 60, the fictitious
/// 1900-02-29, is `None` (`#VALUE!`), as are serials outside `0..=2958465`.
pub(crate) fn hijri_ymd(serial_1900: i64) -> Option<(i32, u32, u32)> {
    let days = match serial_1900 {
        // Serial 0 shows serial 1's date.
        0 => 1 + HIJRI_DAYS_TO_1899_12_30 + 1,
        1..=59 => serial_1900 + HIJRI_DAYS_TO_1899_12_30 + 1,
        61..=2_958_465 => serial_1900 + HIJRI_DAYS_TO_1899_12_30,
        _ => return None,
    };
    let (cycles, mut rest) = (days / HIJRI_CYCLE_DAYS, days % HIJRI_CYCLE_DAYS);
    let mut year_in_cycle = 1;
    loop {
        let length = if is_hijri_leap(year_in_cycle) {
            355
        } else {
            354
        };
        if rest < length || year_in_cycle == 30 {
            break;
        }
        rest -= length;
        year_in_cycle += 1;
    }
    let leap = is_hijri_leap(year_in_cycle);
    let mut month = 1;
    loop {
        // Odd months have 30 days, even ones 29, and the last one 30 in a leap year.
        let length = if month % 2 == 1 || (month == 12 && leap) {
            30
        } else {
            29
        };
        if rest < length || month == 12 {
            break;
        }
        rest -= length;
        month += 1;
    }
    let year = i32::try_from(30 * cycles + year_in_cycle).ok()?;
    Some((year, month, u32::try_from(rest + 1).ok()?))
}

/// What the first name token of a `B2` section showed.
#[derive(Clone, Copy)]
enum FirstName {
    Month(u32),
    Weekday(usize),
}

/// Writes the name tokens of one date section in order.
pub(crate) struct NameTokens {
    hijri: bool,
    first: Option<FirstName>,
}

impl NameTokens {
    /// `hijri` is whether the section is in the `B2` calendar.
    pub(crate) fn new(hijri: bool) -> Self {
        NameTokens { hijri, first: None }
    }

    /// Appends `token`'s name for `month` (1-12) and the Sunday-first `weekday`, from `names` (a
    /// locale tag's) or else the workbook `locale`.
    pub(crate) fn push(
        &mut self,
        token: &TextToken,
        text: &mut String,
        names: Option<&DateNames>,
        locale: &Locale,
        month: u32,
        weekday: usize,
    ) {
        let is_day = matches!(token, TextToken::DayName | TextToken::DayNameShort);
        if self.hijri {
            text.push_str(self.hijri_name(token, is_day, month, weekday));
            return;
        }
        let m = month as usize - 1;
        let dates = &locale.dates;
        match token {
            TextToken::DayNameShort => {
                text.push_str(
                    names.map_or(&dates.day_names_short[weekday], |n| n.days_short[weekday]),
                );
            }
            TextToken::DayName => {
                text.push_str(names.map_or(&dates.day_names[weekday], |n| n.days[weekday]));
            }
            TextToken::MonthNameShort => {
                text.push_str(names.map_or(&dates.months_short[m], |n| n.months_short[m]));
            }
            TextToken::MonthName => text.push_str(names.map_or(&dates.months[m], |n| n.months[m])),
            // Under a locale tag `mmmmm` is the first character of the month name
            // (`[$-804]mmmmm` of December is "十"; probes/text-locale-am-neutral A28).
            _ => match names.and_then(|n| n.months[m].chars().next()) {
                Some(letter) => text.push(letter),
                None => text.push_str(&dates.months_letter[m]),
            },
        }
    }

    /// Excel: in a `B2` section every name token after the first reads the first one's value. A
    /// weekday after a month name is weekday number min(month, 7), Sunday being 1
    /// (`B2mmmm|dddd` of 1444-08-23 is "شعبان|السبت"), and a month name after a weekday prints
    /// the weekday again (`B2dddd|mmmm` is "الأربعاء|الأربعاء"); probes/t1613-hijri-b2-weekday.
    fn hijri_name(
        &mut self,
        token: &TextToken,
        is_day: bool,
        month: u32,
        weekday: usize,
    ) -> &'static str {
        let first = *self.first.get_or_insert(if is_day {
            FirstName::Weekday(weekday)
        } else {
            FirstName::Month(month)
        });
        let name = match (is_day, first) {
            (true, FirstName::Month(m)) => HIJRI_DAYS[m.clamp(1, 7) as usize - 1],
            (_, FirstName::Weekday(w)) => HIJRI_DAYS[w],
            (false, FirstName::Month(m)) => HIJRI_MONTHS[m as usize - 1],
        };
        if matches!(token, TextToken::MonthLetter) {
            // `B2mmmmm` is the first letter ("ش" for Sha'ban).
            return name
                .char_indices()
                .nth(1)
                .map_or(name, |(end, _)| &name[..end]);
        }
        name
    }
}

#[cfg(test)]
mod tests {
    use super::hijri_ymd;

    #[test]
    fn hijri_dates_of_the_probe() {
        // probes/t1613-hijri-b2 A28-A41, A64, A142, A143.
        for (serial, ymd) in [
            (0, (1317, 8, 29)),
            (1, (1317, 8, 29)),
            (2, (1317, 9, 1)),
            (59, (1317, 10, 28)),
            (61, (1317, 10, 29)),
            (45000, (1444, 8, 23)),
            (45125, (1445, 1, 1)),
            (45036, (1444, 9, 30)),
            (45037, (1444, 10, 1)),
            (952_726, (4006, 3, 9)),
            (1_303_730, (4996, 9, 13)),
            (2_958_465, (9666, 4, 3)),
        ] {
            assert_eq!(hijri_ymd(serial), Some(ymd), "{serial}");
        }
        assert_eq!(hijri_ymd(60), None);
        assert_eq!(hijri_ymd(-1), None);
        assert_eq!(hijri_ymd(2_958_466), None);
    }
}
