use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{num_error, value_error};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// The longest text ARABIC reads, counted before any trimming: 255 characters work, 256 are
/// `#VALUE!` (numbers.md "ARABIC reads any Roman-letter string"; probes/t3648-arabic-numeral-shapes
/// A81:A87, A104:A106, A117).
const ARABIC_MAX_LEN: usize = 255;

/// Why ARABIC refuses a (trimmed, non-empty) text.
#[derive(Debug, PartialEq, Eq)]
enum ArabicError {
    /// A lone minus sign: `#NUM!` (probes/t3648-arabic-numeral-shapes A66, A109, A110).
    LoneMinus,
    /// Any character other than the ASCII letters IVXLCDM in either case: `#VALUE!`.
    NotANumeral,
}

/// Reads a Roman numeral the way Excel's ARABIC does (numbers.md "ARABIC reads any Roman-letter
/// string"; probes/t3648-arabic-numeral-shapes): an optional leading `-`, then one or more ASCII
/// letters I, V, X, L, C, D, M in either case, with no grammar check at all. Scanning from the
/// right, a letter worth less than the largest letter seen so far is subtracted, any other letter
/// is added, so `IIX` is 8, `IXI` is 10, `MMMMM` is 5000 and every ROMAN form reads back.
fn from_roman(s: &str) -> Result<i32, ArabicError> {
    let (sign, digits) = match s.strip_prefix('-') {
        Some("") => return Err(ArabicError::LoneMinus),
        Some(rest) => (-1, rest),
        None => (1, s),
    };
    // At most 255 letters of 1000 each, so an i32 cannot overflow.
    let mut total = 0i32;
    let mut largest = 0i32;
    for c in digits.chars().rev() {
        let v = match c.to_ascii_uppercase() {
            'I' => 1,
            'V' => 5,
            'X' => 10,
            'L' => 50,
            'C' => 100,
            'D' => 500,
            'M' => 1000,
            _ => return Err(ArabicError::NotANumeral),
        };
        if v < largest {
            total -= v;
        } else {
            largest = v;
            total += v;
        }
    }
    Ok(sign * total)
}

/// Classic Roman (strict) encoder used as a base for all forms.
fn to_roman(mut n: u32) -> Result<String, String> {
    if !(1..=3999).contains(&n) {
        return Err("value out of range (must be 1..=3999)".into());
    }

    const MAP: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut out = String::with_capacity(15);
    for &(val, sym) in MAP {
        while n >= val {
            out.push_str(sym);
            n -= val;
        }
        if n == 0 {
            break;
        }
    }
    Ok(out)
}

/// Excel/Google Sheets compatible ROMAN(number, [form]) encoder.
/// `form`: 0..=4 (0=Classic, 4=Simplified).
fn to_roman_with_form(n: u32, form: i32) -> Result<String, String> {
    let mut s = to_roman(n)?;
    if form == 0 {
        return Ok(s);
    }
    if !(0..=4).contains(&form) {
        return Err("form must be between 0 and 4".into());
    }

    // Base rules (apply for all f >= 1)
    let base_rules: &[(&str, &str)] = &[
        // C(D|M)XC -> L$1XL
        ("CDXC", "LDXL"),
        ("CMXC", "LMXL"),
        // C(D|M)L -> L$1
        ("CDL", "LD"),
        ("CML", "LM"),
        // X(L|C)IX -> V$1IV
        ("XLIX", "VLIV"),
        ("XCIX", "VCIV"),
        // X(L|C)V -> V$1
        ("XLV", "VL"),
        ("XCV", "VC"),
    ];

    // Level 2 extra rules
    let lvl2_rules: &[(&str, &str)] = &[
        // V(L|C)IV -> I$1
        ("VLIV", "IL"),
        ("VCIV", "IC"),
        // L(D|M)XL -> X$1
        ("LDXL", "XD"),
        ("LMXL", "XM"),
        // L(D|M)VL -> X$1V
        ("LDVL", "XDV"),
        ("LMVL", "XMV"),
        // L(D|M)IL -> X$1IX
        ("LDIL", "XDIX"),
        ("LMIL", "XMIX"),
    ];

    // Level 3 extra rules
    let lvl3_rules: &[(&str, &str)] = &[
        // X(D|M)V -> V$1
        ("XDV", "VD"),
        ("XMV", "VM"),
        // X(D|M)IX -> V$1IV
        ("XDIX", "VDIV"),
        ("XMIX", "VMIV"),
    ];

    // Level 4 extra rules
    let lvl4_rules: &[(&str, &str)] = &[
        // V(D|M)IV -> I$1
        ("VDIV", "ID"),
        ("VMIV", "IM"),
    ];

    // Helper to apply a batch of (from -> to) globally, in order.
    fn apply_rules(mut t: String, rules: &[(&str, &str)]) -> String {
        for (from, to) in rules {
            if t.contains(from) {
                t = t.replace(from, to);
            }
        }
        t
    }

    s = apply_rules(s, base_rules);
    if form >= 2 {
        s = apply_rules(s, lvl2_rules);
    }
    if form >= 3 {
        s = apply_rules(s, lvl3_rules);
    }
    if form >= 4 {
        s = apply_rules(s, lvl4_rules);
    }
    Ok(s)
}

impl Model {
    /// `ROMAN(number, [form])`: both arguments lift over arrays and ranges (T-2410). The
    /// arguments are read in order (number, then form), the first error winning over both range
    /// checks, which then run number first, form second (numbers.md "ROMAN truncates before its
    /// sign check; BASE checks the sign first").
    pub(crate) fn fn_roman(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Roman, args, cell, &mut |model, values| {
            let (number, form) = match values {
                [number] => (number, None),
                [number, form] => (number, Some(form)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            let number = match model.kernel_number(number, cell) {
                // Excel truncates before the sign check: `ROMAN(-0.5025)` is "" and not `#VALUE!`
                // (gen-m5/49bfbf67a1409563 Calc!A179; numbers.md "ROMAN truncates before its sign check; BASE checks the sign first").
                Ok(f) => f.trunc(),
                Err(error) => return error,
            };
            let form = match form {
                // TRUE is the classic form 0, FALSE the simplified form 4.
                Some(CalcResult::Boolean(b)) => {
                    if *b {
                        0
                    } else {
                        4
                    }
                }
                Some(form) => match model.kernel_number(form, cell) {
                    Ok(f) => f as i32,
                    Err(error) => return error,
                },
                None => 0,
            };
            // A form that is itself an error wins over a number out of range: `ROMAN(-5,NA())` is `#N/A`, while
            // `ROMAN(-5,"x")` stays `#VALUE!` (probes/roman-lift-forms Calc!G1, L1; numbers.md
            // "ROMAN probed (T-2423 ...)" paragraph).
            if number != 0.0 && !(1.0..=3999.0).contains(&number) {
                return value_error(cell);
            }
            if !(0..=4).contains(&form) {
                return value_error(cell);
            }
            // The form is read and checked even for 0 (`ROMAN(0,#N/A)` is `#N/A`, `ROMAN(0.2,8.09)` is
            // `#VALUE!`; gen-m5/0362d48f56108faa Calc!A39, gen-m5/1962de140dae0111 Calc!A206).
            if number == 0.0 {
                return CalcResult::String(String::new().into());
            }
            match to_roman_with_form(number as u32, form) {
                Ok(s) => CalcResult::String(s.into()),
                Err(_) => value_error(cell),
            }
        })
    }

    /// `ARABIC(text)`: the text lifts over arrays and ranges (probes/t2198-m5-lift-argument-kinds
    /// row 36; docs/excel-semantics/arrays.md). A blank cell (or its empty value) is 0, as `""`
    /// is, and spaces, tabs, line feeds and carriage returns either side of the numeral are
    /// ignored, but not inside it nor CHAR(160); any other non-text (`0`, `FALSE`) is `#VALUE!`
    /// (numbers.md "ARABIC of a blank cell, and padded numerals";
    /// probes/t2200-blank-and-padded-m5-args A47:A70). A text longer than 255 characters before
    /// trimming is `#VALUE!`; the rest is read by [`from_roman`].
    pub(crate) fn fn_arabic(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Arabic, args, cell, &mut |_, values| {
            let roman_numeral = match values {
                [CalcResult::String(s)] if s.chars().count() > ARABIC_MAX_LEN => {
                    return value_error(cell)
                }
                [CalcResult::String(s)] => s.trim_matches([' ', '\t', '\n', '\r']),
                // A blank cell, or the empty value a lookup hands on for one (lookup.md "VLOOKUP
                // and HLOOKUP hand a blank result cell"; gen-m5/6a511beadb56b932 Calc!A106).
                [CalcResult::EmptyCell | CalcResult::EmptyArg] => "",
                [error @ CalcResult::Error { .. }] => return error.clone(),
                [_] => return value_error(cell),
                _ => return CalcResult::new_args_number_error(cell),
            };
            if roman_numeral.is_empty() {
                return CalcResult::Number(0.0);
            }
            match from_roman(roman_numeral) {
                Ok(value) => CalcResult::Number(f64::from(value)),
                Err(ArabicError::LoneMinus) => num_error(cell),
                Err(ArabicError::NotANumeral) => value_error(cell),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{from_roman, ArabicError};

    #[test]
    fn reads_any_letter_string_right_to_left() {
        // Excel cells: probes/t3648-arabic-numeral-shapes column A.
        for (text, value) in [
            ("xiv", 14),
            ("IIX", 8),
            ("IXI", 10),
            ("IVX", 4),
            ("MIM", 1999),
            ("DCM", 400),
            ("MMMMM", 5000),
            ("LDVLIV", 499),
            ("-IIX", -8),
            ("-xiv", -14),
        ] {
            assert_eq!(from_roman(text), Ok(value), "{text}");
        }
        assert_eq!(from_roman("-"), Err(ArabicError::LoneMinus));
        for text in [
            "N", "+XIV", "--X", "- XIV", "-0", "12", "X-", "\u{2160}", "\u{131}",
        ] {
            assert_eq!(from_roman(text), Err(ArabicError::NotANumeral), "{text}");
        }
    }
}
