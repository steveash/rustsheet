//! The month and day names of the Saka (calendar digit 10) and Hebrew (calendar digit 8) calendars
//! of a `[$-...]` locale tag (`crate::formatter::locale_tag`). Every string is Excel's, from
//! probes/t1923-calendar-digits, t2779-hebrew-saka-names and t2779-hebrew-saka-more (excel-graph,
//! 2026-10-27, T-2779); number-formats.md "Calendars of a locale tag".

use super::excel_date_names::{DateNames, EN};
use super::regional_date_names::{HE, HI};

/// The Saka names of every LCID with a locale but an Arabic or Hindi one: Chaitra first (`mmm` the
/// same), short Sanskrit weekdays for `ddd` and long ones for `dddd`, Sunday first
/// (`[$-100409]mmmm|ddd|dddd` of 2023-03-15 is "Phalguna|Budh|Budhavara").
pub(crate) static SAKA: DateNames = DateNames {
    months_short: SAKA_MONTHS,
    months: SAKA_MONTHS,
    days_short: ["Ravi", "Som", "Mangal", "Budh", "Guru", "Sukra", "Sani"],
    days: [
        "Ravivara",
        "Somavara",
        "Mangalvara",
        "Budhavara",
        "Guruvara",
        "Sukravara",
        "Sanivara",
    ],
    ..EN
};

const SAKA_MONTHS: [&str; 12] = [
    "Chaitra",
    "Vaisakha",
    "Jyaistha",
    "Asadha",
    "Sravana",
    "Bhadra",
    "Asvina",
    "Karthika",
    "Agrahayana",
    "Pausa",
    "Magha",
    "Phalguna",
];

/// The Saka names of an Arabic or Hindi LCID (`[$-100401]`, `[$-100439]`): Devanagari months, and
/// the hi-IN `dddd` for both `ddd` and `dddd` (`[$-100439]mmmm|ddd` of 2023-03-15 is
/// "फल्गुण|बुधवार"; probes/t2779-hebrew-saka-more B1:B23, C32, C33).
pub(crate) static SAKA_HINDI: DateNames = DateNames {
    months_short: SAKA_HINDI_MONTHS,
    months: SAKA_HINDI_MONTHS,
    days_short: HI.days,
    ..HI
};

const SAKA_HINDI_MONTHS: [&str; 12] = [
    "चैत्र",
    "वैशाख",
    "ज्येष्ट",
    "आषाढ़",
    "श्रावण",
    "भाद्र",
    "आश्वीन",
    "क़ार्तीक",
    "अग्रहायन",
    "पौसा",
    "माघ",
    "फल्गुण",
];

/// The Hebrew names of every LCID with a locale but a Hebrew or Arabic one: the months of a common
/// year from Tishrei (a leap year's Adars are [`HEBREW_LATIN_ADARS`]), and `ddd` the same as `dddd`
/// (`[$-80409]mmmm|ddd` of 2023-03-15 is "Adar|Yom Revi'i"; probes/t2779-hebrew-saka-names B1:B21).
pub(crate) static HEBREW_LATIN: DateNames = DateNames {
    months_short: HEBREW_LATIN_MONTHS,
    months: HEBREW_LATIN_MONTHS,
    days_short: HEBREW_LATIN_DAYS,
    days: HEBREW_LATIN_DAYS,
    ..EN
};

const HEBREW_LATIN_MONTHS: [&str; 12] = [
    "Tishrei", "Cheshvan", "Kislev", "Tevet", "Shvat", "Adar", "Nisan", "Iyar", "Sivan", "Tamuz",
    "Av", "Elul",
];

const HEBREW_LATIN_DAYS: [&str; 7] = [
    "Yom Rishon",
    "Yom Sheni",
    "Yom Shlishi",
    "Yom Revi'i",
    "Yom Chamishi",
    "Yom Shishi",
    "Shabat",
];

/// Months 6 and 7 of a Hebrew leap year under [`HEBREW_LATIN`].
pub(crate) static HEBREW_LATIN_ADARS: [&str; 2] = ["AdarI", "AdarII"];

/// The Hebrew names of a Hebrew or Arabic LCID (`[$-8040D]`, `[$-80401]`): Hebrew-script months,
/// `ddd` the weekday's letter and `dddd` the he-IL name (`[$-8040D]mmmm|ddd|dddd` of 2023-03-15 is
/// "אדר|ד|יום רביעי"; probes/t2779-hebrew-saka-names C1:D21, t1923-calendar-digits A211).
pub(crate) static HEBREW_SCRIPT: DateNames = DateNames {
    months_short: HEBREW_SCRIPT_MONTHS,
    months: HEBREW_SCRIPT_MONTHS,
    days_short: ["א", "ב", "ג", "ד", "ה", "ו", "ש"],
    ..HE
};

const HEBREW_SCRIPT_MONTHS: [&str; 12] = [
    "תשרי", "חשון", "כסלו", "טבת", "שבט", "אדר", "ניסן", "אייר", "סיון", "תמוז", "אב", "אלול",
];

/// Months 6 and 7 of a Hebrew leap year under [`HEBREW_SCRIPT`], with a plain space
/// (probes/t1923-calendar-digits A212).
pub(crate) static HEBREW_SCRIPT_ADARS: [&str; 2] = ["אדר א", "אדר ב"];
