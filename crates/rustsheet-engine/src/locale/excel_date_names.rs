//! Excel's own month, day and day-half names per locale, which a `[$-...]` number-format tag
//! selects (`crate::formatter::locale_tag`). They are Excel's locale data, not CLDR (`[$-407]mmm` of
//! March is "Mrz" where CLDR and `locales.bin` have "Mär"): every string is a probed value from
//! probes/text-locale-date-names, probes/text-locale-name-tables and probes/text-locale-am-neutral
//! (excel-graph, 2026-10-18; regional variants and more languages from probes/text-locale-regional-lcids), except the morning "AM" of locales whose afternoon is "PM", which
//! is probed for fr and de only.

use super::regional_date_names::{AR, DE_AT, EL, ES_AR, ES_MX, HE, HI, SK, TH, UK, ZH_SG};

/// The date names and day-half strings of one locale.
#[derive(PartialEq, Eq, Debug)]
pub struct DateNames {
    /// `mmm`, January first.
    pub months_short: [&'static str; 12],
    /// `mmmm`, January first; `mmmmm` is its first character.
    pub months: [&'static str; 12],
    /// `ddd`, Sunday first.
    pub days_short: [&'static str; 7],
    /// `dddd`, Sunday first.
    pub days: [&'static str; 7],
    /// What `AM/PM` and `A/P` print before noon.
    pub am: &'static str,
    /// What `AM/PM` and `A/P` print from noon.
    pub pm: &'static str,
}

static EN: DateNames = DateNames {
    months_short: [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    months: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    days_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    days: [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ],
    am: "AM",
    pm: "PM",
};

static FR: DateNames = DateNames {
    months_short: [
        "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.",
        "déc.",
    ],
    months: [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ],
    days_short: ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."],
    days: [
        "dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi",
    ],
    am: "AM",
    pm: "PM",
};

static DE: DateNames = DateNames {
    months_short: [
        "Jan", "Feb", "Mrz", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
    ],
    months: [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ],
    days_short: ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"],
    days: [
        "Sonntag",
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
    ],
    am: "AM",
    pm: "PM",
};

static DE_CH: DateNames = DateNames {
    months_short: [
        "Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez",
    ],
    months: [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ],
    days_short: ["So.", "Mo.", "Di.", "Mi.", "Do.", "Fr.", "Sa."],
    days: [
        "Sonntag",
        "Montag",
        "Dienstag",
        "Mittwoch",
        "Donnerstag",
        "Freitag",
        "Samstag",
    ],
    am: "AM",
    pm: "PM",
};

static IT: DateNames = DateNames {
    months_short: [
        "gen", "feb", "mar", "apr", "mag", "giu", "lug", "ago", "set", "ott", "nov", "dic",
    ],
    months: [
        "gennaio",
        "febbraio",
        "marzo",
        "aprile",
        "maggio",
        "giugno",
        "luglio",
        "agosto",
        "settembre",
        "ottobre",
        "novembre",
        "dicembre",
    ],
    days_short: ["dom", "lun", "mar", "mer", "gio", "ven", "sab"],
    days: [
        "domenica",
        "lunedì",
        "martedì",
        "mercoledì",
        "giovedì",
        "venerdì",
        "sabato",
    ],
    am: "AM",
    pm: "PM",
};

static ES: DateNames = DateNames {
    months_short: [
        "ene.", "feb.", "mar.", "abr.", "may.", "jun.", "jul.", "ago.", "sep.", "oct.", "nov.",
        "dic.",
    ],
    months: [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ],
    days_short: ["do.", "lu.", "ma.", "mi.", "ju.", "vi.", "sá."],
    days: [
        "domingo",
        "lunes",
        "martes",
        "miércoles",
        "jueves",
        "viernes",
        "sábado",
    ],
    am: "AM",
    pm: "PM",
};

static JA: DateNames = DateNames {
    months_short: [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    ],
    months: [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    days_short: ["日", "月", "火", "水", "木", "金", "土"],
    days: [
        "日曜日",
        "月曜日",
        "火曜日",
        "水曜日",
        "木曜日",
        "金曜日",
        "土曜日",
    ],
    am: "午前",
    pm: "午後",
};

static ZH_TW: DateNames = DateNames {
    months_short: [
        "一月",
        "二月",
        "三月",
        "四月",
        "五月",
        "六月",
        "七月",
        "八月",
        "九月",
        "十月",
        "十一月",
        "十二月",
    ],
    months: [
        "一月",
        "二月",
        "三月",
        "四月",
        "五月",
        "六月",
        "七月",
        "八月",
        "九月",
        "十月",
        "十一月",
        "十二月",
    ],
    days_short: ["週日", "週一", "週二", "週三", "週四", "週五", "週六"],
    days: [
        "星期日",
        "星期一",
        "星期二",
        "星期三",
        "星期四",
        "星期五",
        "星期六",
    ],
    am: "上午",
    pm: "下午",
};

static ZH_CN: DateNames = DateNames {
    months_short: [
        "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
    ],
    months: [
        "一月",
        "二月",
        "三月",
        "四月",
        "五月",
        "六月",
        "七月",
        "八月",
        "九月",
        "十月",
        "十一月",
        "十二月",
    ],
    days_short: ["周日", "周一", "周二", "周三", "周四", "周五", "周六"],
    days: [
        "星期日",
        "星期一",
        "星期二",
        "星期三",
        "星期四",
        "星期五",
        "星期六",
    ],
    am: "上午",
    pm: "下午",
};

static RU: DateNames = DateNames {
    months_short: [
        "янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек",
    ],
    months: [
        "Январь",
        "Февраль",
        "Март",
        "Апрель",
        "Май",
        "Июнь",
        "Июль",
        "Август",
        "Сентябрь",
        "Октябрь",
        "Ноябрь",
        "Декабрь",
    ],
    days_short: ["Вс", "Пн", "Вт", "Ср", "Чт", "Пт", "Сб"],
    days: [
        "воскресенье",
        "понедельник",
        "вторник",
        "среда",
        "четверг",
        "пятница",
        "суббота",
    ],
    am: "AM",
    pm: "PM",
};

static PT: DateNames = DateNames {
    months_short: [
        "jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez",
    ],
    months: [
        "janeiro",
        "fevereiro",
        "março",
        "abril",
        "maio",
        "junho",
        "julho",
        "agosto",
        "setembro",
        "outubro",
        "novembro",
        "dezembro",
    ],
    days_short: ["dom", "seg", "ter", "qua", "qui", "sex", "sáb"],
    days: [
        "domingo",
        "segunda-feira",
        "terça-feira",
        "quarta-feira",
        "quinta-feira",
        "sexta-feira",
        "sábado",
    ],
    am: "AM",
    pm: "PM",
};

static NL: DateNames = DateNames {
    months_short: [
        "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ],
    months: [
        "januari",
        "februari",
        "maart",
        "april",
        "mei",
        "juni",
        "juli",
        "augustus",
        "september",
        "oktober",
        "november",
        "december",
    ],
    days_short: ["zo", "ma", "di", "wo", "do", "vr", "za"],
    days: [
        "zondag",
        "maandag",
        "dinsdag",
        "woensdag",
        "donderdag",
        "vrijdag",
        "zaterdag",
    ],
    am: "AM",
    pm: "PM",
};

static KO: DateNames = DateNames {
    months_short: [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
    ],
    months: [
        "1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월",
    ],
    days_short: ["일", "월", "화", "수", "목", "금", "토"],
    days: [
        "일요일",
        "월요일",
        "화요일",
        "수요일",
        "목요일",
        "금요일",
        "토요일",
    ],
    am: "오전",
    pm: "오후",
};

static PL: DateNames = DateNames {
    months_short: [
        "sty", "lut", "mar", "kwi", "maj", "cze", "lip", "sie", "wrz", "paź", "lis", "gru",
    ],
    months: [
        "styczeń",
        "luty",
        "marzec",
        "kwiecień",
        "maj",
        "czerwiec",
        "lipiec",
        "sierpień",
        "wrzesień",
        "październik",
        "listopad",
        "grudzień",
    ],
    days_short: ["niedz.", "pon.", "wt.", "śr.", "czw.", "pt.", "sob."],
    days: [
        "niedziela",
        "poniedziałek",
        "wtorek",
        "środa",
        "czwartek",
        "piątek",
        "sobota",
    ],
    am: "AM",
    pm: "PM",
};

static DA: DateNames = DateNames {
    months_short: [
        "jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ],
    months: [
        "januar",
        "februar",
        "marts",
        "april",
        "maj",
        "juni",
        "juli",
        "august",
        "september",
        "oktober",
        "november",
        "december",
    ],
    days_short: ["sø", "ma", "ti", "on", "to", "fr", "lø"],
    days: [
        "søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag",
    ],
    am: "AM",
    pm: "PM",
};

static SV: DateNames = DateNames {
    months_short: [
        "jan", "feb", "mar", "apr", "maj", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ],
    months: [
        "januari",
        "februari",
        "mars",
        "april",
        "maj",
        "juni",
        "juli",
        "augusti",
        "september",
        "oktober",
        "november",
        "december",
    ],
    days_short: ["sön", "mån", "tis", "ons", "tor", "fre", "lör"],
    days: [
        "söndag", "måndag", "tisdag", "onsdag", "torsdag", "fredag", "lördag",
    ],
    am: "AM",
    pm: "PM",
};

static NB: DateNames = DateNames {
    months_short: [
        "jan", "feb", "mar", "apr", "mai", "jun", "jul", "aug", "sep", "okt", "nov", "des",
    ],
    months: [
        "januar",
        "februar",
        "mars",
        "april",
        "mai",
        "juni",
        "juli",
        "august",
        "september",
        "oktober",
        "november",
        "desember",
    ],
    days_short: ["søn", "man", "tir", "ons", "tor", "fre", "lør"],
    days: [
        "søndag", "mandag", "tirsdag", "onsdag", "torsdag", "fredag", "lørdag",
    ],
    am: "a.m.",
    pm: "p.m.",
};

static FI: DateNames = DateNames {
    months_short: [
        "tammi", "helmi", "maalis", "huhti", "touko", "kesä", "heinä", "elo", "syys", "loka",
        "marras", "joulu",
    ],
    months: [
        "tammikuu",
        "helmikuu",
        "maaliskuu",
        "huhtikuu",
        "toukokuu",
        "kesäkuu",
        "heinäkuu",
        "elokuu",
        "syyskuu",
        "lokakuu",
        "marraskuu",
        "joulukuu",
    ],
    days_short: ["su", "ma", "ti", "ke", "to", "pe", "la"],
    days: [
        "sunnuntai",
        "maanantai",
        "tiistai",
        "keskiviikko",
        "torstai",
        "perjantai",
        "lauantai",
    ],
    am: "ap.",
    pm: "ip.",
};

static CS: DateNames = DateNames {
    months_short: [
        "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII",
    ],
    months: [
        "leden",
        "únor",
        "březen",
        "duben",
        "květen",
        "červen",
        "červenec",
        "srpen",
        "září",
        "říjen",
        "listopad",
        "prosinec",
    ],
    days_short: ["ne", "po", "út", "st", "čt", "pá", "so"],
    days: [
        "neděle",
        "pondělí",
        "úterý",
        "středa",
        "čtvrtek",
        "pátek",
        "sobota",
    ],
    am: "dop.",
    pm: "odp.",
};

static HU: DateNames = DateNames {
    months_short: [
        "jan.", "febr.", "márc.", "ápr.", "máj.", "jún.", "júl.", "aug.", "szept.", "okt.", "nov.",
        "dec.",
    ],
    months: [
        "január",
        "február",
        "március",
        "április",
        "május",
        "június",
        "július",
        "augusztus",
        "szeptember",
        "október",
        "november",
        "december",
    ],
    days_short: ["V", "H", "K", "Sze", "Cs", "P", "Szo"],
    days: [
        "vasárnap",
        "hétfő",
        "kedd",
        "szerda",
        "csütörtök",
        "péntek",
        "szombat",
    ],
    am: "de.",
    pm: "du.",
};

static TR: DateNames = DateNames {
    months_short: [
        "Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara",
    ],
    months: [
        "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim",
        "Kasım", "Aralık",
    ],
    days_short: ["Paz", "Pzt", "Sal", "Çar", "Per", "Cum", "Cmt"],
    days: [
        "Pazar",
        "Pazartesi",
        "Salı",
        "Çarşamba",
        "Perşembe",
        "Cuma",
        "Cumartesi",
    ],
    am: "ÖÖ",
    pm: "ÖS",
};

/// The probed locales: LCID, BCP-47 tag and names. For a fallback by language the first entry
/// of the language wins, so the order matters: `[$-zh]` and `[$-4]` are Traditional Chinese.
pub(crate) static LOCALES: [(u16, &str, &DateNames); 37] = [
    (0x409, "en-US", &EN),
    (0x809, "en-GB", &EN),
    (0x40C, "fr-FR", &FR),
    (0xC0C, "fr-CA", &FR),
    (0x407, "de-DE", &DE),
    (0x807, "de-CH", &DE_CH),
    (0x410, "it-IT", &IT),
    (0xC0A, "es-ES", &ES),
    (0x40A, "es-ES_tradnl", &ES),
    (0x411, "ja-JP", &JA),
    (0x404, "zh-TW", &ZH_TW),
    (0x804, "zh-CN", &ZH_CN),
    (0x419, "ru-RU", &RU),
    (0x416, "pt-BR", &PT),
    (0x816, "pt-PT", &PT),
    (0x413, "nl-NL", &NL),
    (0x412, "ko-KR", &KO),
    (0x415, "pl-PL", &PL),
    (0x406, "da-DK", &DA),
    (0x41D, "sv-SE", &SV),
    (0x414, "nb-NO", &NB),
    (0x40B, "fi-FI", &FI),
    (0x405, "cs-CZ", &CS),
    (0x40E, "hu-HU", &HU),
    (0x41F, "tr-TR", &TR),
    (0xC07, "de-AT", &DE_AT),
    (0x1004, "zh-SG", &ZH_SG),
    (0xC04, "zh-HK", &ZH_TW),
    (0x80A, "es-MX", &ES_MX),
    (0x2C0A, "es-AR", &ES_AR),
    (0x408, "el-GR", &EL),
    (0x41E, "th-TH", &TH),
    (0x41B, "sk-SK", &SK),
    (0x422, "uk-UA", &UK),
    (0x401, "ar-SA", &AR),
    (0x40D, "he-IL", &HE),
    (0x439, "hi-IN", &HI),
];

/// The names of en-US, the locale of the system tags.
pub(crate) static EN_US: &DateNames = &EN;
