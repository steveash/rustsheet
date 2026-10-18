use super::locale_tag::{self, LocaleTag};
use crate::number_format::{parse_excel_number, parse_finite_number};

pub struct Lexer {
    position: usize,
    len: usize,
    chars: Vec<char>,
    error_message: String,
    error_position: usize,
    /// The locale of a `[$€-40C]` currency tag, returned by the call after its symbol.
    pending_locale: Option<LocaleTag>,
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Color(i32),              // [Red] or [Color 23]
    Condition(Compare, f64), // [<=100] (Comparator, number)
    Currency(char),          // [$€] ($ currency symbol)
    Literal(char), // €, £, ¥, |, $, (, ), /, :, +, -, ^, ', {, }, <, =, !, ~, >, space, digits 1-9 and literal letters
    Escaped(char), // \X
    Spacer(char),  // *X
    Ghost(char),   // _X
    Text(String),  // "Text"
    Separator,     // ;
    Raw,           // @
    Percent,       // %
    Comma,         // ,
    Period,        // .
    Sharp,         // #
    Zero,          // 0
    QuestionMark,  // ?
    Scientific,    // E+
    ScientificMinus, // E-
    General,       // General
    // Dates and time
    Day,            // d
    DayPadded,      // dd
    DayNameShort,   // ddd
    DayName,        // dddd+
    Month,          // m (or minute)
    MonthPadded,    // mm (or minute padded)
    MonthNameShort, // mmm
    MonthName,      // mmmm or mmmmmm+
    MonthLetter,    // mmmmm
    YearShort,      // y or yy
    Year,           // yyy+
    /// `e` (false) or `ee`+ (true, two digits at least): the era year, the four-digit year in a
    /// Gregorian calendar.
    EraYear(bool),
    BuddhistYearShort,   // b or bb
    BuddhistYear,        // bbb+
    Era(usize),          // g+ with its length (empty in en-US)
    Calendar(char),      // B1 (Gregorian) or B2 (Hijri)
    Hour,                // h
    HourPadded,          // hh
    Second,              // s
    SecondPadded,        // ss
    ElapsedHour,         // [h]
    ElapsedMinute,       // [m]
    ElapsedSecond,       // [s]
    ElapsedHourPadded,   // [hh]
    ElapsedMinutePadded, // [mm]
    ElapsedSecondPadded, // [ss]
    AMPM,                // AM/PM
    /// `A/P`: the letters printed for the morning and for the afternoon.
    AP(char, char),
    /// `[$-409]`: a locale with no currency symbol, which prints nothing.
    Locale(LocaleTag),
    ILLEGAL,
    EOF,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Compare {
    Equal,
    LessThan,
    GreaterThan,
    LessOrEqualThan,
    GreaterOrEqualThan,
}

impl Token {
    pub fn is_digit(&self) -> bool {
        (self == &Token::Zero) || (self == &Token::Sharp) || (self == &Token::QuestionMark)
    }

    pub fn is_date(&self) -> bool {
        self == &Token::Day
            || self == &Token::DayPadded
            || self == &Token::DayNameShort
            || self == &Token::DayName
            || self == &Token::MonthName
            || self == &Token::MonthNameShort
            || self == &Token::Month
            || self == &Token::MonthPadded
            || self == &Token::MonthLetter
            || self == &Token::YearShort
            || self == &Token::Year
            || matches!(self, Token::EraYear(_))
            || self == &Token::BuddhistYearShort
            || self == &Token::BuddhistYear
    }
}

impl Lexer {
    pub fn new(format: &str) -> Lexer {
        let chars: Vec<char> = format.chars().collect();
        let len = chars.len();
        Lexer {
            chars,
            position: 0,
            len,
            error_message: "".to_string(),
            error_position: 0,
            pending_locale: None,
        }
    }

    fn peek_char(&self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        let position = self.position;
        if position < self.len {
            self.position = position + 1;
            Some(self.chars[position])
        } else {
            None
        }
    }

    fn set_error(&mut self, error: &str) {
        self.error_message = error.to_string();
        self.error_position = self.position;
        self.position = self.len;
    }

    fn consume_string(&mut self) -> Option<String> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        while position < len {
            let x = self.chars[position];
            position += 1;
            // Excel: `""` inside a literal is not an escaped quote but the end of one literal and
            // the start of the next, so `0 """Millions"""` shows `42 Millions`
            // (probes/format-doubled-quotes C1-C11; docs/excel-semantics/number-formats.md).
            if x != '"' {
                chars.push(x);
            } else {
                self.position = position;
                return Some(chars);
            }
        }
        None
    }

    fn consume_number(&mut self) -> Option<f64> {
        let mut position = self.position;
        let len = self.len;
        let mut chars = "".to_string();
        // first optional '-' sign
        if position < len && self.chars[position] == '-' {
            chars.push('-');
            position += 1;
        }
        // numbers before the '.'
        while position < len {
            let x = self.chars[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        if position < len && self.chars[position] == '.' {
            // numbers after the'.'
            chars.push('.');
            position += 1;
            while position < len {
                let x = self.chars[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
        if position + 1 < len && self.chars[position].eq_ignore_ascii_case(&'e') {
            // exponential side
            let x = self.chars[position + 1];
            if x == '-' || x == '+' || x.is_ascii_digit() {
                chars.push('e');
                chars.push(x);
                position += 2;
                while position < len {
                    let x = self.chars[position];
                    if x.is_ascii_digit() {
                        chars.push(x);
                    } else {
                        break;
                    }
                    position += 1;
                }
            }
        }
        self.position = position;
        // A condition number has to be one Excel reads from text at all: `[>1e-310]` and
        // `[>0e309]` make the format invalid like `[>1e400]` (probes/text-number-tiny-paths
        // C18, C19), but the value keeps every digit.
        parse_finite_number(&chars).filter(|_| parse_excel_number(&chars).is_some())
    }

    fn consume_condition(&mut self) -> Option<(Compare, f64)> {
        let cmp;
        match self.read_next_char() {
            Some('<') => {
                if let Some('=') = self.peek_char() {
                    self.read_next_char();
                    cmp = Compare::LessOrEqualThan;
                } else {
                    cmp = Compare::LessThan;
                }
            }
            Some('>') => {
                if let Some('=') = self.peek_char() {
                    self.read_next_char();
                    cmp = Compare::GreaterOrEqualThan;
                } else {
                    cmp = Compare::GreaterThan;
                }
            }
            Some('=') => {
                cmp = Compare::Equal;
            }
            _ => {
                return None;
            }
        }

        let start = self.position;
        let value = match self.consume_number() {
            Some(v) => v,
            // Without a number, as in `[>]`, the condition compares with 0
            None if self.position == start => 0.0,
            // Something that should be a number and is not one (`[>1e400]`, too large to hold)
            // is no condition: TEXT(5,"[>1e400]0") is #VALUE! (probes/text-number-words J1)
            None => return None,
        };
        if self.read_next_char() == Some(']') {
            return Some((cmp, value));
        }
        None
    }

    // Consume elapsed time like [h], [hh], [m], [mm], [s], [ss]
    fn consume_elapsed_time(&mut self) -> Option<Token> {
        // We are called right after '['
        let start_pos = self.position;

        // First char must be one of: h, m, s
        let unit = match self.read_next_char() {
            Some(c) if ['h', 'H', 'm', 'M', 's', 'S'].contains(&c) => c.to_ascii_lowercase(),
            _ => {
                // Not elapsed time → rewind
                self.position = start_pos;
                return None;
            }
        };

        // Count how many times the same unit repeats (so we detect h vs hh, etc.)
        let mut count = 1;
        while let Some(ch) = self.peek_char() {
            if ch.to_ascii_lowercase() == unit {
                count += 1;
                self.read_next_char();
            } else {
                break;
            }
        }

        // Must be closed by ']'
        if self.read_next_char() != Some(']') {
            // Not a well-formed [h], [hh], etc. → rewind
            self.position = start_pos;
            return None;
        }

        // Map (unit, count) to a token
        let token = match (unit, count) {
            ('h', 1) => Token::ElapsedHour,
            ('h', 2) => Token::ElapsedHourPadded,
            ('m', 1) => Token::ElapsedMinute,
            ('m', 2) => Token::ElapsedMinutePadded,
            ('s', 1) => Token::ElapsedSecond,
            ('s', 2) => Token::ElapsedSecondPadded,
            // Anything else like [hhh], [mmm], [sss] → not considered elapsed time
            _ => {
                self.position = start_pos;
                return None;
            }
        };

        Some(token)
    }

    fn consume_color(&mut self) -> Option<i32> {
        let colors = [
            "black", "white", "red", "green", "blue", "yellow", "magenta",
        ];
        let mut chars = "".to_string();
        while let Some(ch) = self.read_next_char() {
            if ch == ']' {
                let lc = chars.to_lowercase();
                if let Some(index) = colors.iter().position(|&x| x == lc) {
                    return Some(index as i32);
                }
                if !chars.starts_with("Color") {
                    return None;
                }
                if let Ok(index) = chars[5..].trim().parse::<i32>() {
                    if index < 57 && index > 0 {
                        return Some(index);
                    } else {
                        return None;
                    }
                }
                return None;
            } else {
                chars.push(ch);
            }
        }
        None
    }

    /// Consumes the letters equal (ignoring case) to `letter` that follow one already read and
    /// returns how many there are in all.
    fn consume_run(&mut self, letter: char) -> usize {
        let mut count = 1;
        while let Some(c) = self.peek_char() {
            if c.eq_ignore_ascii_case(&letter) {
                count += 1;
                self.read_next_char();
            } else {
                break;
            }
        }
        count
    }

    /// Whether the characters from the current position start with `text`, ignoring ASCII case.
    fn looks_at(&self, text: &str) -> bool {
        let mut position = self.position;
        for expected in text.chars() {
            match self.chars.get(position) {
                Some(c) if c.eq_ignore_ascii_case(&expected) => position += 1,
                _ => return false,
            }
        }
        true
    }

    pub fn peek_token(&mut self) -> Token {
        let (position, pending) = (self.position, self.pending_locale);
        let token = self.next_token();
        (self.position, self.pending_locale) = (position, pending);
        token
    }

    pub fn next_token(&mut self) -> Token {
        if let Some(locale) = self.pending_locale.take() {
            return Token::Locale(locale);
        }
        let ch = self.read_next_char();
        match ch {
            Some(x) => match x {
                '$' | '€' | '£' | '¥' | '|' | '(' | ')' | '/' | ':' | '+' | '-' | '^' | '\''
                | '{' | '}' | '<' | '=' | '!' | '~' | '>' | ' ' => Token::Literal(x),
                '?' => Token::QuestionMark,
                ';' => Token::Separator,
                '#' => Token::Sharp,
                ',' => Token::Comma,
                '.' => Token::Period,
                '0' => Token::Zero,
                '@' => Token::Raw,
                '%' => Token::Percent,
                '[' => {
                    if let Some(c) = self.peek_char() {
                        if c == '<' || c == '>' || c == '=' {
                            // Condition
                            if let Some((cmp, value)) = self.consume_condition() {
                                Token::Condition(cmp, value)
                            } else {
                                self.set_error("Failed to parse condition");
                                Token::ILLEGAL
                            }
                        } else if c == '$' {
                            // currency: [$<symbol>] or [$<symbol>-<locale>]
                            // IronCalc ignores the currency/locale modifiers for now
                            // They are of two types:
                            //  * BCP-47 locale tag (like [$€-fr-FR])
                            //  * hex LCID (old format)
                            //      Like [$$-409] (=1033 for en-US) or
                            //           [$$-40C] (=1036 for fr-FR)
                            self.read_next_char();
                            // Excel: `[$-409]` has a locale and no symbol and prints nothing
                            // (probes/text-ap-locale-bracket A11-A20; number-formats.md); the
                            // locale it names applies to date names (`locale_tag`, T-1716).
                            if self.peek_char() == Some('-') {
                                self.read_next_char();
                                let mut tag = String::new();
                                while let Some(c) = self.read_next_char() {
                                    if c == ']' {
                                        return Token::Locale(locale_tag::resolve(&tag));
                                    }
                                    tag.push(c);
                                }
                                self.set_error("Failed to parse locale");
                                return Token::ILLEGAL;
                            }
                            if let Some(currency) = self.read_next_char() {
                                // The locale part of `[$€-40C]` is read as a `[$-40C]` tag that
                                // follows the symbol (probes/text-currency-locale-date).
                                if Some('-') == self.peek_char() {
                                    self.read_next_char();
                                    let mut tag = String::new();
                                    while let Some(c) = self.peek_char() {
                                        if c == ']' {
                                            break;
                                        }
                                        tag.push(c);
                                        self.read_next_char();
                                    }
                                    self.pending_locale = Some(locale_tag::resolve(&tag));
                                }
                                if self.read_next_char() == Some(']') {
                                    return Token::Currency(currency);
                                }
                            }
                            self.set_error("Failed to parse currency");
                            Token::ILLEGAL
                        } else {
                            // try bracketed (elapsed) time
                            if let Some(token) = self.consume_elapsed_time() {
                                return token;
                            }
                            // Color
                            if let Some(index) = self.consume_color() {
                                return Token::Color(index);
                            }
                            self.set_error("Failed to parse color");
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '_' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Ghost(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '*' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Spacer(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '\\' => {
                    if let Some(y) = self.read_next_char() {
                        Token::Escaped(y)
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                '"' => {
                    if let Some(s) = self.consume_string() {
                        Token::Text(s)
                    } else {
                        self.set_error("Did not find end of text string");
                        Token::ILLEGAL
                    }
                }
                'E' => {
                    if let Some(s) = self.read_next_char() {
                        if s == '+' {
                            Token::Scientific
                        } else if s == '-' {
                            Token::ScientificMinus
                        } else {
                            self.set_error(&format!("Unexpected char: {s}. Expected + or -"));
                            Token::ILLEGAL
                        }
                    } else {
                        self.set_error("Unexpected end of input");
                        Token::ILLEGAL
                    }
                }
                'd' | 'D' => {
                    let d = self.consume_run('d');
                    match d {
                        1 => Token::Day,
                        2 => Token::DayPadded,
                        3 => Token::DayNameShort,
                        _ => Token::DayName,
                    }
                }
                'm' | 'M' => {
                    let m = self.consume_run('m');
                    match m {
                        1 => Token::Month,       // (or minute)
                        2 => Token::MonthPadded, // (or minute padded)
                        3 => Token::MonthNameShort,
                        4 => Token::MonthName,
                        5 => Token::MonthLetter,
                        _ => Token::MonthName,
                    }
                }
                'y' | 'Y' => {
                    let y = self.consume_run('y');
                    if y == 1 || y == 2 {
                        Token::YearShort
                    } else {
                        Token::Year
                    }
                }
                'h' | 'H' => {
                    let h = self.consume_run('h');
                    if h == 1 {
                        Token::Hour
                    } else if h == 2 {
                        Token::HourPadded
                    } else {
                        self.set_error("Unexpected character after 'h'");
                        Token::ILLEGAL
                    }
                }
                's' | 'S' => {
                    let s = self.consume_run('s');
                    if s == 1 {
                        Token::Second
                    } else if s == 2 {
                        Token::SecondPadded
                    } else {
                        self.set_error("Unexpected character after 's'");
                        Token::ILLEGAL
                    }
                }
                'A' | 'a' => {
                    if self.looks_at("m/pm") {
                        self.position += 4;
                        Token::AMPM
                    } else if self.looks_at("/p") {
                        // Excel: `A/P` prints the first letter in the morning and the second
                        // in the afternoon, each as typed (`h A/p` is "6 A" and "6 p";
                        // probes/text-ap-locale-bracket A1-A6, A28).
                        self.read_next_char();
                        let pm = self.read_next_char().unwrap_or('p');
                        Token::AP(x, pm)
                    } else {
                        // Excel: an unquoted `a` that is not AM/PM prints as itself
                        // (probes/text-format-unquoted-letters A1, A27).
                        Token::Literal(x)
                    }
                }
                // Excel: `b`/`bb` is the two-digit Buddhist-era year and `bbb`+ the four-digit one,
                // `B1`/`B2` a calendar prefix that prints nothing, and a lone `B` closing the code is
                // invalid (`TEXT(100,"ABC")` = "A43C", `"B"` and `"dB"` are #VALUE!;
                // probes/text-format-bare-literals A40-A50, probes/text-format-unquoted-letters).
                'b' | 'B' => {
                    if let Some(c @ ('1' | '2')) = self.peek_char() {
                        self.read_next_char();
                        return Token::Calendar(c);
                    }
                    let count = self.consume_run('b');
                    if x == 'B' && count == 1 && matches!(self.peek_char(), None | Some(';')) {
                        self.set_error("Unexpected end of input after 'B'");
                        return Token::ILLEGAL;
                    }
                    if count <= 2 {
                        Token::BuddhistYearShort
                    } else {
                        Token::BuddhistYear
                    }
                }
                // Excel: a lowercase `e` (any run) is the era year, which is the four-digit year in
                // the Gregorian calendar (`TEXT(100,"e")` = "1900", `"e+"` = "1900+";
                // probes/text-format-bare-literals A51, A52, A63); under Japanese eras `e` is "5" and
                // `ee` "05" (probes/text-locale-calendars A13).
                'e' => Token::EraYear(self.consume_run('e') > 1),
                'g' | 'G' => {
                    if self
                        .chars
                        .get(self.position..)
                        .is_some_and(|rest| rest.starts_with(&['e', 'n', 'e', 'r', 'a', 'l']))
                    {
                        self.position += 6;
                        return Token::General;
                    }
                    // Excel: any other run of `g` is the era, empty for the Gregorian calendar
                    // (`TEXT(100,"g")` = "", `TEXT(1,"dG")` = "1"; probes/text-format-bare-literals
                    // A53-A56, probes/text-format-unquoted-letters).
                    Token::Era(self.consume_run('g'))
                }
                // Excel: `n`/`N` are invalid (cause unknown, not an en-US date token;
                // probes/text-format-unquoted-letters A14, A40).
                'n' | 'N' => {
                    self.set_error("Unexpected character: n");
                    Token::ILLEGAL
                }
                // Excel prints every other letter (the format-token letters have their own arms
                // above) and the digits 1-9 as they are: `TEXT(1,"0x")` = "1x", `TEXT(PI(),"ü")` =
                // "ü", `TEXT(5,"2")` = "2" (probes/text-format-unquoted-letters,
                // probes/text-format-bare-literals A1-A7).
                _ if x.is_alphabetic() || ('1'..='9').contains(&x) => Token::Literal(x),
                _ => {
                    self.set_error(&format!("Unexpected character: {x}"));
                    Token::ILLEGAL
                }
            },
            None => Token::EOF,
        }
    }
}

pub fn is_likely_date_number_format(format: &str) -> bool {
    let mut lexer = Lexer::new(format);
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            return false;
        }
        if token.is_date() {
            return true;
        }
    }
}
