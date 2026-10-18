//! The `[$-...]` locale tag of a number format: what Excel shows for date names and `AM/PM` under
//! it, with names from [`crate::locale::excel_date_names`]. Evidence: probes/text-locale-date-names,
//! probes/text-locale-name-tables, probes/text-locale-am-neutral (excel-graph, 2026-10-18);
//! docs/excel-semantics/number-formats.md "Locale tags localize date names".

pub use crate::locale::excel_date_names::DateNames;
use crate::locale::excel_date_names::{EN_US, LOCALES};

/// What a `[$-...]` tag (with no currency symbol) does to its section.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum LocaleTag {
    /// A locale with probed names: date names and `AM/PM` follow it, and `A/P` prints its `AM/PM`
    /// strings (`TEXT(0.75,"[$-409]h A/P")` is "6 PM").
    Names(&'static DateNames),
    /// `[$-F800]` or `[$-x-sysdate]`: the section is the system long date, whatever else it holds.
    SystemDate,
    /// `[$-F400]` or `[$-x-systime]`: the section is the system long time.
    SystemTime,
    /// A tag Excel does not resolve (`[$-zzz]`, `[$-]`, `[$-FFFF]`) or one with no table here
    /// (TODO(T-1858)): it prints nothing and changes nothing (`TEXT(0.75,"[$-FFFF]h A/P")` keeps
    /// "6 P").
    Silent,
}

/// The system long date format of en-US Windows (`[$-F800]`).
pub const SYSTEM_LONG_DATE: &str = "dddd, mmmm d, yyyy";
/// The system long time format of en-US Windows (`[$-F400]`).
pub const SYSTEM_LONG_TIME: &str = "h:mm:ss AM/PM";

/// Resolves the text between `[$-` and `]`.
///
/// A hex LCID may carry a calendar in the digits above the low four: `[$-1040C]` and `[$-0040C]`
/// are French, but other calendars (`[$-11040C]` is Chinese month names) are not modelled and stay
/// [`LocaleTag::Silent`] (TODO(T-1856)). An LCID with no table falls back to its language
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

fn resolve_lcid(value: u32) -> LocaleTag {
    match value {
        0xF800 => return LocaleTag::SystemDate,
        0xF400 => return LocaleTag::SystemTime,
        _ => {}
    }
    if value >> 16 > 1 {
        return LocaleTag::Silent;
    }
    let lcid = (value & 0xFFFF) as u16;
    if lcid == 0x400 || lcid == 0x800 {
        return LocaleTag::Names(EN_US);
    }
    if let Some((_, _, names)) = LOCALES.iter().find(|(id, _, _)| *id == lcid) {
        return LocaleTag::Names(names);
    }
    let language = lcid & 0x3FF;
    if language == 0 {
        return LocaleTag::Silent;
    }
    LOCALES
        .iter()
        .find(|(id, _, _)| id & 0x3FF == language)
        .map_or(LocaleTag::Silent, |(_, _, names)| LocaleTag::Names(names))
}

fn resolve_bcp47(tag: &str) -> LocaleTag {
    if let Some((_, _, names)) = LOCALES.iter().find(|(_, t, _)| t.eq_ignore_ascii_case(tag)) {
        return LocaleTag::Names(names);
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
        .map_or(LocaleTag::Silent, |(_, _, names)| LocaleTag::Names(names))
}
