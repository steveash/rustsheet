//! The numeral system of a locale tag: the byte in bits 24-31 of `[$-N000409]`, which changes the
//! digits that numbers, date and clock fields print (literals, names and text sections keep
//! theirs). Evidence: probes/t1925-numeral-digits, t3532-numeral-digit-fields,
//! t3532-numeral-digit-general (excel-graph); docs/excel-semantics/number-formats.md
//! "Numeral-system digits".

/// The digits a section prints under its locale tag's numeral byte.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum Numerals {
    /// ASCII digits: no numeral byte, or 0, 1, 20-26 and every byte above 31.
    #[default]
    Ascii,
    /// Ten consecutive code points for 0-9, digit by digit (`[$-2000409]0` on 12 is "١٢").
    /// Tamil (9) and Ethiopic (17) have no zero: theirs stays ASCII `0` (`[$-9000409]#,##0.00`
    /// on 1020304.05 is "௧,0௨0,௩0௪.0௫").
    Positional { zero: u32, ascii_zero: bool },
    /// The CJK numerals of bytes 27-31.
    Cjk(&'static CjkStyle),
}

/// How a CJK numeral byte (27-31) writes numbers. A number section prints its digits one by one
/// (`[$-1B000409]0.00` on 1234567.89 is "一二三四五六七.八九"). A date or clock field from 10 to 99
/// is written with the tens sign (`[$-1B000409]dd` of the 15th is "十五", `yy` of 2023 "二十三")
/// and every other field digit by digit (`yyyy` "二〇二三", `mm` of March "〇三"). `General`, and
/// an elapsed count, spell the integer part out in units and myriads (`[$-1B000409]General` on
/// 12345 is "一万二千三百四十五", `[h]` of 1.25 "三十") and the rest digit by digit ("〇.五",
/// "一E+一一"; probes/t3532-numeral-digit-fields, t3532-numeral-digit-general).
#[derive(PartialEq, Eq, Debug)]
pub struct CjkStyle {
    /// The digits 0-9.
    digits: [char; 10],
    /// The signs of ten, hundred and thousand.
    units: [char; 3],
    /// The signs of 10^4 and 10^8.
    myriads: [char; 2],
    /// Whether a spelled-out number writes the one before a unit sign (`一十`, `壱百`); a
    /// myriad always takes it (`[$-1B000409]General` on 10000 is "一万", on 100 "百").
    one_before_unit: bool,
    /// Whether a date field writes the one before the tens sign (`[$-1C000409]dd` of the 10th is
    /// "壱拾", but `[$-1E000409]dd` is "十" where General on 10 is "一十").
    one_before_field_ten: bool,
    /// Whether a run of zeros before a nonzero digit prints one zero digit (`[$-1E000409]General`
    /// on 105 is "一百○五"); the Japanese styles drop it ("百五").
    zero_in_gap: bool,
    /// Whether date fields use the tens sign; fullwidth digits (29) print fields digit by digit
    /// (`[$-1D000409]dd` of the 15th is "１５") though General spells them ("十５").
    field_tens: bool,
}

/// Byte 27: Japanese numerals with 〇 (U+3007).
static JAPANESE: CjkStyle = CjkStyle {
    digits: ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'],
    units: ['十', '百', '千'],
    myriads: ['万', '億'],
    one_before_unit: false,
    one_before_field_ten: false,
    zero_in_gap: false,
    field_tens: true,
};

/// Byte 28: Japanese formal numerals (daiji), with 〇.
static JAPANESE_FORMAL: CjkStyle = CjkStyle {
    digits: ['〇', '壱', '弐', '参', '四', '伍', '六', '七', '八', '九'],
    units: ['拾', '百', '阡'],
    myriads: ['萬', '億'],
    one_before_unit: true,
    one_before_field_ten: true,
    zero_in_gap: false,
    field_tens: true,
};

/// Byte 29: fullwidth digits, with the Japanese unit signs in `General`.
static FULLWIDTH: CjkStyle = CjkStyle {
    digits: ['０', '１', '２', '３', '４', '５', '６', '７', '８', '９'],
    units: ['十', '百', '千'],
    myriads: ['万', '億'],
    one_before_unit: false,
    one_before_field_ten: false,
    zero_in_gap: false,
    field_tens: false,
};

/// Byte 30: Chinese numerals with ○ (U+25CB).
static CHINESE: CjkStyle = CjkStyle {
    digits: ['○', '一', '二', '三', '四', '五', '六', '七', '八', '九'],
    units: ['十', '百', '千'],
    myriads: ['万', '亿'],
    one_before_unit: true,
    one_before_field_ten: false,
    zero_in_gap: true,
    field_tens: true,
};

/// Byte 31: Chinese formal (financial) numerals.
static CHINESE_FORMAL: CjkStyle = CjkStyle {
    digits: ['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖'],
    units: ['拾', '佰', '仟'],
    myriads: ['万', '亿'],
    one_before_unit: true,
    one_before_field_ten: true,
    zero_in_gap: true,
    field_tens: true,
};

/// The largest integer spelled out in units and myriads; past it (unprobed, beyond 億 groups) the
/// integer prints digit by digit.
const MAX_SPELLED: u64 = 999_999_999_999;

impl Numerals {
    /// The numeral system of byte `byte` (bits 24-31 of a locale tag value).
    pub fn from_byte(byte: u32) -> Numerals {
        let positional = |zero: u32| Numerals::Positional {
            zero,
            ascii_zero: false,
        };
        match byte {
            2 => positional(0x0660),
            3 => positional(0x06F0),
            4 => positional(0x0966),
            5 => positional(0x09E6),
            6 => positional(0x0A66),
            7 => positional(0x0AE6),
            8 => positional(0x0B66),
            9 => Numerals::Positional {
                zero: 0x0BE6,
                ascii_zero: true,
            },
            10 => positional(0x0C66),
            11 => positional(0x0CE6),
            12 => positional(0x0D66),
            13 => positional(0x0E50),
            14 => positional(0x0ED0),
            15 => positional(0x0F20),
            16 => positional(0x1040),
            // Ethiopic one is U+1369; the code point before it is no zero.
            17 => Numerals::Positional {
                zero: 0x1368,
                ascii_zero: true,
            },
            18 => positional(0x17E0),
            19 => positional(0x1810),
            27 => Numerals::Cjk(&JAPANESE),
            28 => Numerals::Cjk(&JAPANESE_FORMAL),
            29 => Numerals::Cjk(&FULLWIDTH),
            30 => Numerals::Cjk(&CHINESE),
            31 => Numerals::Cjk(&CHINESE_FORMAL),
            _ => Numerals::Ascii,
        }
    }

    /// `c` in this numeral system when it is an ASCII digit; any other character as it is.
    pub fn digit(self, c: char) -> char {
        let Some(d) = c.to_digit(10) else {
            return c;
        };
        match self {
            Numerals::Ascii => c,
            Numerals::Positional { ascii_zero, .. } if ascii_zero && d == 0 => c,
            Numerals::Positional { zero, .. } => char::from_u32(zero + d).unwrap_or(c),
            Numerals::Cjk(style) => style.digits[d as usize],
        }
    }

    /// Every ASCII digit of `text` in this numeral system, digit by digit.
    pub fn digits(self, text: &str) -> String {
        text.chars().map(|c| self.digit(c)).collect()
    }

    /// A date or clock field (`d`, `mm`, `yy`, `e`, `h`, `ss`, ...) as Excel prints it: a CJK
    /// field from 10 to 99 with the tens sign (`三十一`, `壱拾弐`), anything else digit by digit
    /// (`〇五`, `二〇二三`).
    pub fn field(self, text: &str) -> String {
        if let Numerals::Cjk(style) = self {
            if style.field_tens {
                if let Some(value) = two_digit_value(text) {
                    return style.tens(value, style.one_before_field_ten);
                }
            }
        }
        self.digits(text)
    }

    /// `General` text, or an elapsed count: a CJK style spells out the integer part
    /// (`千二百三十四.五`, `-十五`) and writes the rest digit by digit (`一E+二〇`); the other
    /// systems map every digit.
    pub fn number(self, text: &str) -> String {
        let Numerals::Cjk(style) = self else {
            return self.digits(text);
        };
        let sign_len = text.len() - text.trim_start_matches('-').len();
        let (sign, rest) = text.split_at(sign_len);
        let integer_len = rest.bytes().take_while(u8::is_ascii_digit).count();
        let (integer, tail) = rest.split_at(integer_len);
        let spelled = match integer.parse::<u64>() {
            // A padded count (`[hh]` under 10) keeps its zero digit by digit; unprobed.
            Ok(value)
                if value <= MAX_SPELLED && !(integer.len() > 1 && integer.starts_with('0')) =>
            {
                style.spell(value)
            }
            _ => self.digits(integer),
        };
        format!("{sign}{spelled}{}", self.digits(tail))
    }
}

/// The value of `text` when it is one or two ASCII digits from 10 to 99.
fn two_digit_value(text: &str) -> Option<u64> {
    if text.len() > 2 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<u64>().ok().filter(|v| (10..=99).contains(v))
}

impl CjkStyle {
    /// `value` (10-99) as tens sign and digits: `十五`, `二十`, or `壱拾伍` with `one`.
    fn tens(&self, value: u64, one: bool) -> String {
        let (tens, units) = (value / 10, value % 10);
        let mut text = String::new();
        if tens > 1 || one {
            text.push(self.digits[tens as usize]);
        }
        text.push(self.units[0]);
        if units > 0 {
            text.push(self.digits[units as usize]);
        }
        text
    }

    /// `value` spelled out in groups of four digits, each followed by its myriad sign
    /// (`一万二千三百四十五`, `一億二千三百四十五万六千七百八十九`); zero is the zero digit.
    fn spell(&self, value: u64) -> String {
        if value == 0 {
            return self.digits[0].to_string();
        }
        let mut text = String::new();
        // Whether a digit has printed, and whether zeros have run since the last one.
        let mut started = false;
        let mut gap = false;
        for myriad in (0..3_u32).rev() {
            let group = value / 10_u64.pow(4 * myriad) % 10_000;
            if group == 0 {
                gap |= started;
                continue;
            }
            for place in (0..4_u32).rev() {
                let d = (group / 10_u64.pow(place) % 10) as usize;
                if d == 0 {
                    gap |= started;
                    continue;
                }
                if gap && self.zero_in_gap {
                    text.push(self.digits[0]);
                }
                gap = false;
                if place == 0 || d != 1 || self.one_before_unit {
                    text.push(self.digits[d]);
                }
                if place > 0 {
                    text.push(self.units[place as usize - 1]);
                }
                started = true;
            }
            if myriad > 0 {
                text.push(self.myriads[myriad as usize - 1]);
            }
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::Numerals;

    #[test]
    fn spells_general_as_excel() {
        // probes/t3532-numeral-digit-general rows 10-29.
        let japanese = Numerals::from_byte(27);
        let cases = [
            ("5", "五"),
            ("10", "十"),
            ("105", "百五"),
            ("1010", "千十"),
            ("10000", "一万"),
            ("100000", "十万"),
            ("123456789", "一億二千三百四十五万六千七百八十九"),
            ("1000000000", "十億"),
            ("0.5", "〇.五"),
            ("-15", "-十五"),
            ("1E+11", "一E+一一"),
            ("12.05", "十二.〇五"),
        ];
        for (text, expected) in cases {
            assert_eq!(japanese.number(text), expected, "{text}");
        }
        let chinese = Numerals::from_byte(30);
        assert_eq!(chinese.number("10"), "一十");
        assert_eq!(chinese.number("105"), "一百○五");
        assert_eq!(chinese.number("1010"), "一千○一十");
        assert_eq!(chinese.number("1000000000"), "一十亿");
        assert_eq!(Numerals::from_byte(28).number("100000"), "壱拾萬");
        assert_eq!(
            Numerals::from_byte(29).number("12345"),
            "１万２千３百４十５"
        );
        assert_eq!(Numerals::from_byte(31).number("1010"), "壹仟零壹拾");
    }

    #[test]
    fn fields_use_the_tens_sign() {
        // probes/t3532-numeral-digit-fields.
        assert_eq!(Numerals::from_byte(27).field("05"), "〇五");
        assert_eq!(Numerals::from_byte(27).field("10"), "十");
        assert_eq!(Numerals::from_byte(30).field("10"), "十");
        assert_eq!(Numerals::from_byte(28).field("31"), "参拾壱");
        assert_eq!(Numerals::from_byte(31).field("10"), "壹拾");
        assert_eq!(Numerals::from_byte(29).field("15"), "１５");
        assert_eq!(Numerals::from_byte(27).field("2023"), "二〇二三");
        assert_eq!(Numerals::from_byte(9).field("2023"), "௨0௨௩");
        assert_eq!(Numerals::from_byte(17).digits("10"), "፩0");
        assert_eq!(Numerals::from_byte(26).digits("10"), "10");
    }
}
