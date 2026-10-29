//! The `[$-...]` locale tag of a number format: what Excel shows for date names and `AM/PM` under
//! it, with names from [`crate::locale::excel_date_names`]. Evidence: probes/text-locale-date-names,
//! probes/text-locale-name-tables, probes/text-locale-am-neutral (excel-graph, 2026-10-18);
//! docs/excel-semantics/number-formats.md "Locale tags localize date names".

use chrono::NaiveDate;

use super::dates::convert_to_serial_number;
use crate::locale::calendar_date_names::{
    HEBREW_LATIN, HEBREW_LATIN_ADARS, HEBREW_SCRIPT, HEBREW_SCRIPT_ADARS, SAKA, SAKA_HINDI,
};
pub use crate::locale::excel_date_names::DateNames;
use crate::locale::excel_date_names::{
    ARABIC, ARABIC_LEVANT, EN_US, HIJRI_ARABIC, HIJRI_ENGLISH, HIJRI_FRENCH, HIJRI_TURKISH,
    LOCALES, LUNAR_JA, LUNAR_KO, LUNAR_ZH,
};

/// The calendar a locale tag selects: what `yy`, `yyyy`, `b`, `e` and `g` print.
/// Evidence: probes/text-locale-calendars, text-locale-calendars-detail, text-locale-japanese-year
/// (excel-graph, 2026-10-18, T-1859); number-formats.md "Calendars of a locale tag".
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Calendar {
    /// Gregorian years; `g` prints nothing and `e` is the four-digit year.
    Gregorian,
    /// Japanese eras: `g`/`gg`/`ggg` the era name, `e`/`ee` the era year. `explicit` (calendar
    /// digit 3) makes `yy`/`yyyy` the era year too; a Japanese locale's own calendar does that only
    /// after a `g` in the section (`[$-411]yy|g|yy` is "23|R|05").
    Japanese { explicit: bool },
    /// Calendar digit 4 (Taiwan): Gregorian, but `yy` prints four digits.
    Taiwan,
    /// Calendar digit 5: `yy`/`yyyy` count from 2333 BC; `b` and `e` stay Gregorian.
    Korean,
    /// Calendar digit 7: `yy`/`yyyy` and `b` count from 543 BC; `e` stays Gregorian.
    Thai,
    /// Calendar digits E, F, 11, 12 and 13: the lunar year, month and day for every date token, `b`
    /// and `e` included (`[$-110409]yyyy|m|d|e|bbbb` of 2024-01-09 is "2023|12|28|2023|2023";
    /// probes/text-locale-calendars A113, text-locale-lunar-tokens A75).
    Lunar(LunarTable),
    /// Calendar digit 6, or a `B2` prefix before the tag: Excel's Hijri date
    /// (`date_names::hijri_ymd`) for every date token; `yy`, `yyyy`, `e`, `b` and `bbbb` print
    /// the Hijri year and `g` nothing (`[$-60409]yy|yyyy|e|b|bbbb` of 2023-03-15 is
    /// "44|1444|1444|44|1444"; probes/t2729-hijri-locale-tag A203).
    Hijri,
    /// Calendar digit 10: the Saka year, month and day for every date token, `b` and `e`
    /// included (`[$-100409]yyyy-mm-dd|yy|e|b` of 2023-03-15 is "1944-12-24|44|1944|44";
    /// `calendar_dates::saka_ymd`).
    Saka,
    /// Calendar digit 8: the Hebrew year, month and day for every date token
    /// (`[$-80409]yyyy-mm-dd|e|b` of 2023-03-15 is "5783-06-22|5783|83";
    /// `calendar_dates::hebrew_ymd`), with the names of a leap year's months 6 and 7, `None`
    /// under an LCID with no locale, whose month names go by number.
    Hebrew(Option<&'static [&'static str; 2]>),
}

/// Which month table a lunar calendar digit uses: F and 11 follow China; E, 12 and 13 one that
/// starts some months a day later (`[$-E0409]m` of 2023-05-19 is "4", `[$-110409]m` "5";
/// probes/text-locale-lunar-months-1900 columns A-F).
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LunarTable {
    /// Calendar digits F and 11.
    Chinese,
    /// Calendar digits E, 12 and 13.
    Eastern,
}

/// A resolved locale tag: the names of months and days, the day-half strings and the calendar.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Localized {
    /// Month and day names, which some calendar digits override (`[$-90409]mmmm` is "mars").
    pub names: &'static DateNames,
    /// `AM/PM` and `A/P` always follow the LCID, whatever the calendar (`[$-30409]h AM/PM` is
    /// "6 PM", `[$-30411]h AM/PM` "6 午後"). `None` under an LCID with no locale, whose calendar
    /// digit still applies: `A/P` stays as written (`[$-E08DE]h A/P` is "6 P";
    /// probes/t1924-locale-tag-extended-hex B13).
    pub day_half: Option<&'static DateNames>,
    pub calendar: Calendar,
    /// A Japanese LCID: after a `g`, `yy` and `yyyy` print the whole Gregorian year in its
    /// non-Japanese calendars (`[$-10411]g|yy` is "|2023", `[$-70411]g|yyyy` "|2023", but
    /// `[$-70409]g|yyyy` "|2566"; probes/text-locale-era-gregorian).
    pub japanese_lcid: bool,
    /// A `B2` before a calendar-3, -5 or -7 tag: every year token converts once more than it
    /// would without the `B2` ([`japanese_era`]; `calendar_dates::year_token_text`). Before a
    /// calendar-4 tag the years are a garbled Minguo year instead (`calendar_dates::taiwan_after_b2`).
    pub year_twice: bool,
    /// A `B2` before a calendar-1, -3 or -5 tag: the names every name token prints from the
    /// section's first `mmm` on, in place of [`Localized::names`] (`date_names::NameTokens`).
    pub names_after_mmm: Option<&'static DateNames>,
}

/// What a `[$-...]` tag (with no currency symbol) does to its section.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LocaleTag {
    /// A locale with probed names: date names and `AM/PM` follow it, and `A/P` prints its `AM/PM`
    /// strings (`TEXT(0.75,"[$-409]h A/P")` is "6 PM").
    Names(Localized),
    /// `[$-F800]` or `[$-x-sysdate]`: the section is the system long date, whatever else it holds.
    SystemDate,
    /// `[$-F400]` or `[$-x-systime]`: the section is the system long time.
    SystemTime,
    /// A tag Excel does not resolve (`[$-zzz]`, `[$-]`, `[$-FFFF]`) or one with no table here
    /// (TODO(T-1875)), or a calendar digit over an LCID with no locale that is not modelled
    /// (`without_locale`): it prints nothing and changes nothing (`TEXT(0.75,"[$-FFFF]h A/P")`
    /// keeps "6 P").
    Silent,
}

/// The system long date format of en-US Windows (`[$-F800]`).
pub const SYSTEM_LONG_DATE: &str = "dddd, mmmm d, yyyy";
/// The system long time format of en-US Windows (`[$-F400]`).
pub const SYSTEM_LONG_TIME: &str = "h:mm:ss AM/PM";
/// The tag value of the system long date (`[$-F800]`).
const SYSTEM_DATE_LCID: u32 = 0xF800;
/// The tag value of the system long time (`[$-F400]`).
const SYSTEM_TIME_LCID: u32 = 0xF400;

/// Resolves the text between `[$-` and `]`.
///
/// A hex LCID may carry a calendar in the digits above the low four (see `resolve_lcid`). An LCID
/// with no table falls back to its language (`[$-80C]` is French, `[$-C]` too); 400 and 800, the
/// user and system defaults, are en-US.
/// A BCP-47 tag matches case-insensitively, then by its language (`[$-fr-XX]` is French).
/// Any other tag is read as extended hex ([`extended_hex`], [`resolve_extended`]): `[$-fr_FR]` is
/// the Saka calendar over an LCID with no locale.
/// [`resolve_after_hijri_prefix`] resolves a tag that follows a `B2` prefix in its section.
pub fn resolve(tag: &str) -> LocaleTag {
    resolve_with(tag, false)
}

/// [`resolve`] for a tag after a `B2` prefix in its section: a hex or BCP-47 tag in its locale's
/// own calendar (digit 0) takes the Hijri calendar, digit 6, with that LCID's Hijri names
/// (`B2[$-40C]mmmm` is "Chaaban", `B2[$-0]mmmm` "شعبان"). Any other calendar digit wins over the
/// prefix (`B2[$-10409]yyyy` is "2023", `B2[$-110409]yyyy|m` of 2024-01-09 "2023|12"). An
/// extended-hex tag, and so `[$-zzz]`, is not resolved this way: `B2[$-zzz]` is a bare `B2`
/// (probes/t2729-hijri-locale-tag A243-A264, t2729-hijri-locale-tag-names A239-A277;
/// text-locale-lunar-tokens A79). Calendar digits 15 and above are unprobed here.
/// Under calendars 3, 5 and 7 the `B2` converts every year token once more
/// ([`Localized::year_twice`]; `B2[$-70409]yyyy|e` of 2023-03-15 is "3109|2566",
/// `B2[$-50409]yyyy|e` "6689|4356"; probes/t2932-b2-calendar-twice). Under calendar 4 it prints
/// a garbled Minguo year ("01"; `calendar_dates::taiwan_after_b2`).
/// Under calendar 1 the names from the first `mmm` on are the ar-SA Gregorian ones, and under
/// calendars 3 and 5 the en-US ones, whatever the LCID: `B2[$-10411]mmmm|ddd|mmm|mmmm|dddd` of
/// 2023-03-15 is "3月|水|مارس|مارس|الثلاثاء" and `B2[$-30411]dddd|mmm|mmmm` "水曜日|Mar|March"
/// ([`Localized::names_after_mmm`]; probes/t3101-b2-calendar-names, t3029-b2-calendar-weekday
/// A73-A84). The other digits keep the tag's names.
pub fn resolve_after_hijri_prefix(tag: &str) -> LocaleTag {
    match resolve_with(tag, true) {
        LocaleTag::Names(localized) => LocaleTag::Names(Localized {
            year_twice: matches!(
                localized.calendar,
                Calendar::Japanese { explicit: true }
                    | Calendar::Korean
                    | Calendar::Taiwan
                    | Calendar::Thai
            ),
            names_after_mmm: match hex_value(tag).map(|value| value >> 16) {
                Some(1) => Some(ARABIC),
                Some(3 | 5) => Some(EN_US),
                _ => None,
            },
            ..localized
        }),
        other => other,
    }
}

/// [`resolve`], with digit 0 read as 6 when `hijri_prefix`.
fn resolve_with(tag: &str, hijri_prefix: bool) -> LocaleTag {
    if tag.eq_ignore_ascii_case("x-sysdate") {
        return LocaleTag::SystemDate;
    }
    if tag.eq_ignore_ascii_case("x-systime") {
        return LocaleTag::SystemTime;
    }
    // A tag that reads as hex is an LCID first, but one with no locale may still be a language
    // (`[$-de]` is German, not LCID 0xDE; `[$-C]` is French).
    let hex = hex_value(tag);
    if let Some(value) = hex {
        if resolve_lcid(value) != LocaleTag::Silent {
            return resolve_lcid(hijri_default(value, hijri_prefix));
        }
    }
    match resolve_bcp47(tag, hijri_prefix) {
        LocaleTag::Silent => {}
        bcp47 => return bcp47,
    }
    // Hijri over an LCID with no locale has the Arabic names (`B2[$-0]` and `B2[$-FFFF]` print
    // "شعبان"; probes/t2729-hijri-locale-tag A202, A251).
    match hex.map(|value| resolve_lcid(hijri_default(value, hijri_prefix))) {
        Some(LocaleTag::Silent) | None => {
            extended_hex(tag).map_or(LocaleTag::Silent, resolve_extended)
        }
        Some(hijri) => hijri,
    }
}

/// `tag` read as a hex LCID with its calendar digits, if it is one.
fn hex_value(tag: &str) -> Option<u32> {
    u32::from_str_radix(tag, 16)
        .ok()
        .filter(|_| tag.len() <= 8 && tag.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The value Excel reads from a tag that is neither hex nor BCP-47, as if it were hex with every
/// character a digit: a letter counts from 10 in either case (`g` 16, `r` 27, `z` 35), any other
/// character below `a` counts from `0` (`_` 47, `-` -3), and one above `z` from `a` (`{` 36, `~`
/// 39).
/// So `[$-fr_FR]` is 0x10E00B, the Saka calendar over an LCID with no locale, `[$-g]` is Italian
/// (0x10), `[$-3gC]` French (0x40C), `[$-0_]` Macedonian (0x2F) and `[$-~]` Lithuanian (0x27)
/// (probes/t1924-locale-tag-extended-hex, text-locale-calendars A135-A145). Of the other characters
/// below `a` only `_` is probed; `-` as -3 and the wrap at 32 bits are inferred. Tags longer than
/// eight characters and non-ASCII tags are unprobed and resolve to nothing.
fn extended_hex(tag: &str) -> Option<u32> {
    if tag.is_empty() || tag.len() > 8 || !tag.is_ascii() {
        return None;
    }
    Some(tag.bytes().fold(0u32, |value, c| {
        let digit = match c {
            b'a'.. => i32::from(c) - i32::from(b'a') + 10,
            b'A'..=b'Z' => i32::from(c) - i32::from(b'A') + 10,
            _ => i32::from(c) - i32::from(b'0'),
        };
        value.wrapping_mul(16).wrapping_add_signed(digit)
    }))
}

/// `value` with calendar digit 0 read as 6 (Hijri) after a `B2` prefix; the system tags keep
/// theirs.
fn hijri_default(value: u32, hijri_prefix: bool) -> u32 {
    if hijri_prefix && value >> 16 == 0 && value != SYSTEM_DATE_LCID && value != SYSTEM_TIME_LCID {
        value | 0x6_0000
    } else {
        value
    }
}

/// An LCID with its calendar digits (bits 16-23, hex below). Excel: 0 is the locale's own calendar
/// (Japanese eras for Japanese), 1 Gregorian, 2 and D Gregorian with en-US names, 3 Japanese eras
/// (Japanese names only for a Japanese LCID), 4 Taiwan (zh-TW names), 5 Korean (Korean names only
/// for a Korean LCID), 7 Thai (Thai names), 9 Gregorian with French names, and 15 and above are
/// ignored (`[$-FF040C]mmmm` is "mars"). E, F, 11, 12 and 13 are lunar: E with lunar month names
/// and Japanese weekdays (Korean names under a Korean LCID), F and 11 with zh-CN weekdays, 12 and
/// 13 with en-US names. A is Gregorian with Levant Arabic month names, B and C Gregorian with
/// the ar-SA names, 14 Gregorian with the Korean native month names (probes/t1923-calendar-digits).
/// 6 is Hijri, with the LCID's Hijri names ([`hijri_names`]), 8 Hebrew and 10 Saka
/// ([`hebrew_or_saka`]). Day halves follow the LCID throughout. Numeral-system digits (bits 24-31;
/// TODO(T-1925)) stay [`LocaleTag::Silent`].
fn resolve_lcid(value: u32) -> LocaleTag {
    match value {
        SYSTEM_DATE_LCID => return LocaleTag::SystemDate,
        SYSTEM_TIME_LCID => return LocaleTag::SystemTime,
        _ => {}
    }
    if value >> 24 != 0 {
        return LocaleTag::Silent;
    }
    let lcid = (value & 0xFFFF) as u16;
    // LCID 1 has no locale under Hebrew and Saka, though 0xC, 0xD and 0x39 have theirs
    // (`[$-80001]mmmm` of 2024-03-21 is "July", `[$-8000D]mmmm` "אדר ב"; probes/t2779-hebrew-saka-more
    // C29:C33, D29:D33). Under the other digits it is unprobed and reads as Arabic.
    if lcid == 1 && matches!(value >> 16, 8 | 0x10) {
        return without_locale(value >> 16);
    }
    let names = if lcid == 0x400 || lcid == 0x800 {
        EN_US
    } else if let Some(names) = table(lcid).or_else(|| by_language(lcid)) {
        names
    } else {
        return without_locale(value >> 16);
    };
    let japanese = lcid & 0x3FF == 0x11;
    let with_names = |calendar_names: &'static DateNames, calendar| {
        LocaleTag::Names(Localized {
            names: calendar_names,
            day_half: Some(names),
            calendar,
            japanese_lcid: japanese,
            year_twice: false,
            names_after_mmm: None,
        })
    };
    match value >> 16 {
        0 | 0x15.. if japanese => with_names(names, Calendar::Japanese { explicit: false }),
        0 | 1 | 0x15.. => with_names(names, Calendar::Gregorian),
        2 | 0xD => with_names(EN_US, Calendar::Gregorian),
        3 => with_names(
            if japanese { names } else { EN_US },
            Calendar::Japanese { explicit: true },
        ),
        4 => table(0x404).map_or(LocaleTag::Silent, |n| with_names(n, Calendar::Taiwan)),
        5 => with_names(
            if lcid & 0x3FF == 0x12 { names } else { EN_US },
            Calendar::Korean,
        ),
        7 => table(0x41E).map_or(LocaleTag::Silent, |n| with_names(n, Calendar::Thai)),
        9 => table(0x40C).map_or(LocaleTag::Silent, |n| with_names(n, Calendar::Gregorian)),
        0xE => with_names(
            if lcid & 0x3FF == 0x12 {
                &LUNAR_KO
            } else {
                &LUNAR_JA
            },
            Calendar::Lunar(LunarTable::Eastern),
        ),
        0xF | 0x11 => with_names(&LUNAR_ZH, Calendar::Lunar(LunarTable::Chinese)),
        0x12 | 0x13 => with_names(EN_US, Calendar::Lunar(LunarTable::Eastern)),
        0xA => with_names(&ARABIC_LEVANT, Calendar::Gregorian),
        0xB | 0xC => with_names(ARABIC, Calendar::Gregorian),
        0x14 => with_names(&LUNAR_KO, Calendar::Gregorian),
        6 => with_names(hijri_names(lcid), Calendar::Hijri),
        8 | 0x10 => {
            let (calendar_names, calendar) = hebrew_or_saka(value >> 16 == 8, lcid);
            with_names(calendar_names, calendar)
        }
    }
}

/// The names and calendar of digit 8 (`hebrew`) or 10 under `lcid`: Hebrew script for a Hebrew or
/// Arabic LCID (`[$-8040D]`, `[$-80801]`), Devanagari Saka names for an Arabic or Hindi one
/// (`[$-100401]`, `[$-100439]`), the Latin transliterations under every other LCID with a locale
/// (probes/t2779-hebrew-saka-names E1:E25, t2779-hebrew-saka-more C1:D33). bn-IN (0x445) also
/// prints the Latin Saka names, but it has no table here and so no locale (`without_locale`).
fn hebrew_or_saka(hebrew: bool, lcid: u16) -> (&'static DateNames, Calendar) {
    match (hebrew, lcid & 0x3FF) {
        (true, 0x0D | 0x01) => (&HEBREW_SCRIPT, Calendar::Hebrew(Some(&HEBREW_SCRIPT_ADARS))),
        (true, _) => (&HEBREW_LATIN, Calendar::Hebrew(Some(&HEBREW_LATIN_ADARS))),
        (false, 0x01 | 0x39) => (&SAKA_HINDI, Calendar::Saka),
        (false, _) => (&SAKA, Calendar::Saka),
    }
}

/// The Hijri names Excel shows under `lcid` (calendar digit 6): Arabic for an Arabic LCID, French
/// and Turkish transliterations for those languages, English ones for every other probed LCID
/// (`[$-60411]` and `[$-60804]` print "Sha'ban"; probes/t2729-hijri-locale-tag A169-A202,
/// t2729-hijri-locale-tag-names A1-A82).
fn hijri_names(lcid: u16) -> &'static DateNames {
    match lcid & 0x3FF {
        0x01 => &HIJRI_ARABIC,
        0x0C => &HIJRI_FRENCH,
        0x1F => &HIJRI_TURKISH,
        _ => &HIJRI_ENGLISH,
    }
}

/// [`resolve_lcid`] for a value read by [`extended_hex`]: an LCID without a table of its own
/// falls back to its language only when it is neutral (below 0x400: `[$-g]` 0x10 is Italian,
/// `[$-0_]` 0x2F Macedonian). `[$-x_y]` is 0x2412 and prints English, not Korean
/// (text-locale-calendars A139), so an LCID with a sublanguage and no table has no locale here.
fn resolve_extended(value: u32) -> LocaleTag {
    let lcid = (value & 0xFFFF) as u16;
    if value >> 24 == 0 && lcid > 0x3FF && lcid != 0x400 && lcid != 0x800 && table(lcid).is_none() {
        return without_locale(value >> 16);
    }
    resolve_lcid(value)
}

/// Calendar digit `calendar` over an LCID with no locale (0 included): the calendar's date with
/// its own names where it has them (Thai under 7, Arabic under A, B and C) and en-US names
/// otherwise, the lunar ones included, and `A/P` as written. `[$-E08DE]mmmm|yyyy-mm-dd` of 45000
/// is "February|2023-02-24", `[$-708DE]` gives "มีนาคม|2566-03-15", `[$-CFDFC]` "مارس", and
/// `h A/P` is "6 P" under each (probes/t1924-locale-tag-extended-hex A13, A14, B13, B14;
/// text-locale-calendars A136-A138, A142). Hijri has the Arabic names (`[$-608DE]` and
/// `[$-60000]` print "شعبان|الأربعاء"; probes/t2729-hijri-locale-tag A201, A260). Hebrew and Saka
/// name their months by number in en-US (`[$-80000]mmmm` of 2024-03-21 is "July", `[$-fr_FR]`
/// of 2023-03-15 "December"; probes/t2779-hebrew-saka-names F1:F7, E12, E13). Other digits (3,
/// 4, 5, 9 and 14 are unprobed here) leave such a tag [`LocaleTag::Silent`].
fn without_locale(calendar: u32) -> LocaleTag {
    let (names, calendar) = match calendar {
        7 => match table(0x41E) {
            Some(thai) => (thai, Calendar::Thai),
            None => return LocaleTag::Silent,
        },
        0xA => (&ARABIC_LEVANT, Calendar::Gregorian),
        0xB | 0xC => (ARABIC, Calendar::Gregorian),
        0xE | 0x12 | 0x13 => (EN_US, Calendar::Lunar(LunarTable::Eastern)),
        0xF | 0x11 => (EN_US, Calendar::Lunar(LunarTable::Chinese)),
        6 => (&HIJRI_ARABIC, Calendar::Hijri),
        8 => (EN_US, Calendar::Hebrew(None)),
        0x10 => (EN_US, Calendar::Saka),
        _ => return LocaleTag::Silent,
    };
    LocaleTag::Names(Localized {
        names,
        day_half: None,
        calendar,
        japanese_lcid: false,
        year_twice: false,
        names_after_mmm: None,
    })
}

/// The table of exactly `lcid`.
fn table(lcid: u16) -> Option<&'static DateNames> {
    LOCALES
        .iter()
        .find(|(id, _, _)| *id == lcid)
        .map(|(_, _, names)| *names)
}

/// The first table of `lcid`'s language, for an LCID with no table of its own.
fn by_language(lcid: u16) -> Option<&'static DateNames> {
    let language = lcid & 0x3FF;
    if language == 0 {
        return None;
    }
    LOCALES
        .iter()
        .find(|(id, _, _)| id & 0x3FF == language)
        .map(|(_, _, names)| *names)
}

/// A BCP-47 tag is the LCID it names, with that locale's own calendar (`[$-ja-JP]g` is "R"), or
/// Hijri after a `B2` prefix.
fn resolve_bcp47(tag: &str, hijri_prefix: bool) -> LocaleTag {
    let resolve_id = |id: u16| resolve_lcid(hijri_default(u32::from(id), hijri_prefix));
    if let Some((id, _, _)) = LOCALES.iter().find(|(_, t, _)| t.eq_ignore_ascii_case(tag)) {
        return resolve_id(*id);
    }
    let Some(language) = tag.split('-').next().filter(|l| !l.is_empty()) else {
        return LocaleTag::Silent;
    };
    LOCALES
        .iter()
        .find(|(_, t, _)| {
            t.split('-')
                .next()
                .is_some_and(|l| l.eq_ignore_ascii_case(language))
        })
        .map_or(LocaleTag::Silent, |(id, _, _)| resolve_id(*id))
}

/// The Japanese eras Excel uses from 1900 on, latest first: the serial of the first day (1900 date
/// system), the first Gregorian year and what `g`, `gg` and `ggg` print. Excel: Taisho starts at
/// serial 4595 (1912-07-30), Showa at 9856, Heisei at 32516 and Reiwa at 43586 (2019-05-01); the
/// year before is the last of the previous era (`[$-411]g|e` of 43585 is "H|31";
/// probes/text-locale-calendars A1-A14, text-locale-calendars-detail A10, A41).
static ERAS: [(i64, i32, [&str; 3]); 5] = [
    (43586, 2019, ["R", "令", "令和"]),
    (32516, 1989, ["H", "平", "平成"]),
    (9856, 1926, ["S", "昭", "昭和"]),
    (4595, 1912, ["T", "大", "大正"]),
    (i64::MIN, 1868, ["M", "明", "明治"]),
];

/// The Japanese era of 1900-system `serial`: its first Gregorian year and its names.
fn era_of(serial: i64) -> (i32, &'static [&'static str; 3]) {
    let (_, first_year, names) = ERAS
        .iter()
        .find(|(first_serial, _, _)| serial >= *first_serial)
        .unwrap_or(&ERAS[ERAS.len() - 1]);
    (*first_year, names)
}

/// The Japanese era of 1900-system `serial`, whose Gregorian date is `(year, month, day)`: the
/// era year and what `g`, `gg` and `ggg` print. With `twice` (a `B2` before a calendar-3 tag)
/// Excel converts once more, reading the era year as the Gregorian year of a date with the same
/// month and day: Reiwa 5 (2023-03-15) is year 5, before Meiji, so Meiji 5 - 1868 + 1 = -1862,
/// which Excel prints as an unsigned 32-bit number, "4294965434|M"; 9999-12-31, Reiwa 7981, is
/// Reiwa 5963, and the era changes on its own day of the year read back: 3930-07-29 (Reiwa 1912)
/// is Meiji 45, 3930-07-30 Taisho 1 (probes/t2932-b2-calendar-twice A1-A16, A69-A80).
pub fn japanese_era(
    serial: i64,
    (year, month, day): (i32, u32, u32),
    twice: bool,
) -> (i32, &'static [&'static str; 3]) {
    let (first_year, names) = era_of(serial);
    let era_year = year - first_year + 1;
    if !twice {
        return (era_year, names);
    }
    // Day 0 (serial 0, "1900-01-00") and a 29 February of a common year have no date: the
    // month's 28th stands in for them, which no era boundary separates.
    let again = NaiveDate::from_ymd_opt(era_year, month, day)
        .or_else(|| NaiveDate::from_ymd_opt(era_year, month, 28))
        .map_or(i64::MIN, |date| i64::from(convert_to_serial_number(date)));
    let (first_year, names) = era_of(again);
    (era_year - first_year + 1, names)
}
