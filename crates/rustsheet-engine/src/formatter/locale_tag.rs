//! The `[$-...]` locale tag of a number format: what Excel shows for date names and `AM/PM` under
//! it, with names from [`crate::locale::excel_date_names`]. Evidence: probes/text-locale-date-names,
//! probes/text-locale-name-tables, probes/text-locale-am-neutral (excel-graph, 2026-10-18);
//! docs/excel-semantics/number-formats.md "Locale tags localize date names".

pub use crate::locale::excel_date_names::DateNames;
use crate::locale::excel_date_names::{EN_US, LOCALES};

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
}

/// A resolved locale tag: the names of months and days, the day-half strings and the calendar.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Localized {
    /// Month and day names, which some calendar digits override (`[$-90409]mmmm` is "mars").
    pub names: &'static DateNames,
    /// `AM/PM` and `A/P` always follow the LCID, whatever the calendar (`[$-30409]h AM/PM` is
    /// "6 PM", `[$-30411]h AM/PM` "6 午後").
    pub day_half: &'static DateNames,
    pub calendar: Calendar,
}

impl Localized {
    fn plain(names: &'static DateNames) -> Self {
        Localized {
            names,
            day_half: names,
            calendar: Calendar::Gregorian,
        }
    }
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
    /// (TODO(T-1875)): it prints nothing and changes nothing (`TEXT(0.75,"[$-FFFF]h A/P")` keeps
    /// "6 P").
    Silent,
}

/// The system long date format of en-US Windows (`[$-F800]`).
pub const SYSTEM_LONG_DATE: &str = "dddd, mmmm d, yyyy";
/// The system long time format of en-US Windows (`[$-F400]`).
pub const SYSTEM_LONG_TIME: &str = "h:mm:ss AM/PM";

/// Resolves the text between `[$-` and `]`.
///
/// A hex LCID may carry a calendar in the digits above the low four (see [`resolve_lcid`]). An LCID with no table falls back to its language
/// (`[$-80C]` is French, `[$-C]` too); 400 and 800, the user and system defaults, are en-US.
/// A BCP-47 tag matches case-insensitively, then by its language (`[$-fr-XX]` is French).
pub fn resolve(tag: &str) -> LocaleTag {
    if tag.eq_ignore_ascii_case("x-sysdate") {
        return LocaleTag::SystemDate;
    }
    if tag.eq_ignore_ascii_case("x-systime") {
        return LocaleTag::SystemTime;
    }
    // A tag that reads as hex is an LCID first, but one with no locale may still be a language
    // (`[$-de]` is German, not LCID 0xDE; `[$-C]` is French).
    let lcid = match u32::from_str_radix(tag, 16) {
        Ok(value) if tag.len() <= 8 && tag.chars().all(|c| c.is_ascii_hexdigit()) => {
            resolve_lcid(value)
        }
        _ => LocaleTag::Silent,
    };
    if lcid == LocaleTag::Silent {
        resolve_bcp47(tag)
    } else {
        lcid
    }
}

/// An LCID with its calendar digits (bits 16-23). Excel: 0 is the locale's own calendar (Japanese
/// eras for Japanese), 1 Gregorian, 2 and D Gregorian with en-US names, 3 Japanese eras (Japanese
/// names only for a Japanese LCID), 4 Taiwan (zh-TW names), 5 Korean (Korean names only for a
/// Korean LCID), 7 Thai (Thai names), 9 Gregorian with French names, and 15 and above are ignored
/// (`[$-FF040C]mmmm` is "mars"). Day halves follow the LCID throughout. Calendars 6 (Hijri,
/// TODO(T-1613)), E-13 (lunar, TODO(T-1922)), 8 (Hebrew), A-C (Arabic Gregorian), 10 (Saka) and 14
/// (Korean native names; TODO(T-1923)) are not modelled and stay [`LocaleTag::Silent`], as do
/// numeral-system digits (bits 24-31; TODO(T-1925)).
fn resolve_lcid(value: u32) -> LocaleTag {
    match value {
        0xF800 => return LocaleTag::SystemDate,
        0xF400 => return LocaleTag::SystemTime,
        _ => {}
    }
    if value >> 24 != 0 {
        return LocaleTag::Silent;
    }
    let lcid = (value & 0xFFFF) as u16;
    let names = if lcid == 0x400 || lcid == 0x800 {
        EN_US
    } else if let Some(names) = table(lcid).or_else(|| by_language(lcid)) {
        names
    } else {
        return LocaleTag::Silent;
    };
    let japanese = lcid & 0x3FF == 0x11;
    let localized = |names, calendar| {
        LocaleTag::Names(Localized {
            calendar,
            ..Localized::plain(names)
        })
    };
    let with_names = |calendar_names: &'static DateNames, calendar| {
        LocaleTag::Names(Localized {
            names: calendar_names,
            day_half: names,
            calendar,
        })
    };
    match value >> 16 {
        0 | 0x15.. if japanese => localized(names, Calendar::Japanese { explicit: false }),
        0 | 1 | 0x15.. => localized(names, Calendar::Gregorian),
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
        _ => LocaleTag::Silent,
    }
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

/// A BCP-47 tag is the LCID it names, with that locale's own calendar (`[$-ja-JP]g` is "R").
fn resolve_bcp47(tag: &str) -> LocaleTag {
    if let Some((id, _, _)) = LOCALES.iter().find(|(_, t, _)| t.eq_ignore_ascii_case(tag)) {
        return resolve_lcid(u32::from(*id));
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
        .map_or(LocaleTag::Silent, |(id, _, _)| resolve_lcid(u32::from(*id)))
}

/// A Japanese era: the serial of its first day (1900 date system), its first Gregorian year and
/// what `g`, `gg` and `ggg` print.
struct Era {
    first_serial: i64,
    first_year: i32,
    names: [&'static str; 3],
}

/// The eras Excel uses from 1900 on, latest first. Excel: Taisho starts at serial 4595
/// (1912-07-30), Showa at 9856, Heisei at 32516 and Reiwa at 43586 (2019-05-01); the year before is
/// the last of the previous era (`[$-411]g|e` of 43585 is "H|31"; probes/text-locale-calendars
/// A1-A14, text-locale-calendars-detail A10, A41).
static ERAS: [Era; 5] = [
    Era {
        first_serial: 43586,
        first_year: 2019,
        names: ["R", "令", "令和"],
    },
    Era {
        first_serial: 32516,
        first_year: 1989,
        names: ["H", "平", "平成"],
    },
    Era {
        first_serial: 9856,
        first_year: 1926,
        names: ["S", "昭", "昭和"],
    },
    Era {
        first_serial: 4595,
        first_year: 1912,
        names: ["T", "大", "大正"],
    },
    Era {
        first_serial: i64::MIN,
        first_year: 1868,
        names: ["M", "明", "明治"],
    },
];

/// The Japanese era of 1900-system `serial`, whose Gregorian year is `year`: what a `g` run of
/// `length` prints (`ggg` and longer print the full name) and the year in the era.
pub fn japanese_era(serial: i64, year: i32, length: usize) -> (&'static str, i32) {
    let era = ERAS
        .iter()
        .find(|era| serial >= era.first_serial)
        .unwrap_or(&ERAS[ERAS.len() - 1]);
    (era.names[length.clamp(1, 3) - 1], year - era.first_year + 1)
}
