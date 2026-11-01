//! The base conversions BIN2DEC … OCT2HEX. Every argument lifts over an array and refuses a
//! multi-cell range (`ArgKind::LiftArrays`, SPEC-0011; probes/eng-argument-modes E5:E7, E15:E17,
//! E24:E26, E52:E54, probes/scalar-lift-shapes H9, probes/eng-later-argument-lift H4:H11): each
//! kernel is the scalar function of one element, and builds its own errors with empty messages.

use crate::functions::lift::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::token::Error,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

/// Every base-conversion input and result has at most this many digits; the top half of each
/// 10-digit span is the two's-complement negatives (`BIN2DEC("1000000000")` is -512,
/// `HEX2DEC("8000000000")` -549755813888; probes/base-conv-boundaries).
const BASE_DIGITS: usize = 10;

/// The radixes of the three non-decimal bases.
#[derive(Clone, Copy)]
enum Radix {
    Bin = 2,
    Oct = 8,
    Hex = 16,
}

impl Radix {
    /// `radix^10`: the count of 10-digit values (1024 for binary).
    fn span(self) -> i64 {
        (self as i64).pow(BASE_DIGITS as u32)
    }

    /// The first value whose 10-digit form has the sign bit set (512 for binary).
    fn half(self) -> i64 {
        self.span() / 2
    }

    /// `value` (0 or more) in this radix, upper-case, without padding.
    fn digits(self, value: i64) -> String {
        match self {
            Radix::Bin => format!("{value:b}"),
            Radix::Oct => format!("{value:o}"),
            Radix::Hex => format!("{value:X}"),
        }
    }
}

impl Model {
    /// The number and the truncated optional `places` of one conversion element.
    ///
    /// `places` is read and range-checked (truncated, then 1 to 10) before `number` is looked
    /// at: Excel reads the second argument first, so its error wins
    /// (probes/eng-first-vs-second-error) and so does an invalid value: `DEC2HEX(<#N/A>,0)`,
    /// `HEX2OCT(<#VALUE!>,0.5)` and `BIN2OCT("ABC",-1)` are `#NUM!` (gen-m9 T-2258 triage; T-2545).
    fn number_and_places<'a>(
        &self,
        values: &'a [CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(&'a CalcResult, Option<i32>), CalcResult> {
        let (number, places) = match values {
            [number] => (number, None),
            [number, places] => (number, Some(places)),
            _ => return Err(CalcResult::new_args_number_error(cell)),
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(e)) => return Err(e),
            None => None,
        };
        if places.is_some_and(|p| !(1..=10).contains(&p)) {
            return Err(num_error(cell));
        }
        Ok((number, places))
    }

    /// The digit text of a BIN/OCT/HEX `number`. Excel reads a blank cell and empty text as 0 in every
    /// base conversion (probes/eng-base-conv-blank, docs/excel-semantics/engineering.md).
    ///
    /// A boolean is `#VALUE!`, as in `BIN2DEC(TRUE)` (T-2420; engineering.md "Engineering function
    /// argument modes").
    fn base_digits(value: &CalcResult, cell: CellReferenceIndex) -> Result<String, CalcResult> {
        if matches!(value, CalcResult::Boolean(_)) {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expecting number".to_string(),
            ));
        }
        let text = Self::cast_string_ref(value, cell)?;
        Ok(if text.is_empty() {
            "0".to_owned()
        } else {
            text
        })
    }

    /// The signed value of a BIN2*, OCT2* or HEX2* `number` read in `radix`.
    ///
    /// The number's text ([`Self::base_digits`]; a number is read as its displayed text, so
    /// `BIN2DEC(1E2)` reads "100" and `HEX2DEC(1E3)` reads "1000") must be at most ten
    /// characters, every one a digit of `radix` (either case for hex), or it is `#NUM!`: an
    /// 11-digit input even with leading zeros (`BIN2DEC("00000000001")`), a sign (`"+101"`,
    /// `"-1"`), a space (`" 101"`, `"101 "`), a decimal point (`"1.0"`, `1.5`) or an exponent
    /// (`"1E2"` in BIN2*, `1E+20`) (probes/base-conv-boundaries; T-2420, T-2544, T-3651).
    /// A value in the top half of the 10-digit span is the two's-complement negative.
    fn signed_base_value(
        value: &CalcResult,
        radix: Radix,
        cell: CellReferenceIndex,
    ) -> Result<i64, CalcResult> {
        let digits = Self::base_digits(value, cell)?;
        let well_formed =
            digits.len() <= BASE_DIGITS && digits.chars().all(|c| c.is_digit(radix as u32));
        if !well_formed {
            return Err(num_error(cell));
        }
        let unsigned = i64::from_str_radix(&digits, radix as u32).map_err(|_| num_error(cell))?;
        Ok(if unsigned >= radix.half() {
            unsigned - radix.span()
        } else {
            unsigned
        })
    }

    /// `value` written in `radix` as a conversion result.
    ///
    /// Outside the radix's 10-digit two's-complement range is `#NUM!` (`DEC2BIN(512)`,
    /// `HEX2OCT("FFDFFFFFFF")`). A negative result is always ten digits and ignores a valid
    /// `places` (`HEX2OCT("FFFFFFFFFF",4)` is "7777777777", probes/eng-gen-sample-edges C1, C6;
    /// `DEC2BIN(-5,1)`, probes/base-conv-boundaries). A non-negative result is zero-padded to
    /// `places`, and `places` shorter than it is `#NUM!` (`DEC2BIN(5,2)`).
    fn write_base(
        value: i64,
        radix: Radix,
        places: Option<i32>,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(-radix.half()..radix.half()).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            return CalcResult::String(radix.digits(value + radix.span()).into());
        }
        let digits = radix.digits(value);
        match places.map(|p| usize::try_from(p).unwrap_or(0)) {
            Some(width) if width < digits.len() => num_error(cell),
            Some(width) => CalcResult::String(format!("{digits:0>width$}").into()),
            None => CalcResult::String(digits.into()),
        }
    }

    /// A `from` to `to` conversion with optional places: BIN2HEX, HEX2BIN and the rest.
    fn convert_base(
        &self,
        values: &[CalcResult],
        from: Radix,
        to: Radix,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (number, places) = match self.number_and_places(values, cell) {
            Ok(parts) => parts,
            Err(e) => return e,
        };
        match Self::signed_base_value(number, from, cell) {
            Ok(value) => Self::write_base(value, to, places, cell),
            Err(e) => e,
        }
    }

    /// A BIN2DEC, OCT2DEC or HEX2DEC conversion.
    fn base_to_decimal(values: &[CalcResult], from: Radix, cell: CellReferenceIndex) -> CalcResult {
        let [number] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        match Self::signed_base_value(number, from, cell) {
            Ok(value) => CalcResult::Number(value as f64),
            Err(e) => e,
        }
    }

    /// A DEC2BIN, DEC2OCT or DEC2HEX conversion. The number is truncated toward zero
    /// (`DEC2BIN(-1.5)` is "1111111111", `DEC2BIN(-0.5)` and `DEC2BIN(-512.9)` are "0" and
    /// "1000000000"); numeric text is read as a number (`DEC2HEX(" 5")`, `DEC2HEX("1E1")`,
    /// `DEC2HEX("$5")`; probes/base-conv-boundaries).
    fn decimal_to_base(
        &self,
        values: &[CalcResult],
        to: Radix,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let (number, places) = match self.number_and_places(values, cell) {
            Ok(parts) => parts,
            Err(e) => return e,
        };
        match self.dec_digits(number, cell) {
            // A saturating cast: anything past i64 is far outside every range anyway.
            Ok(f) => Self::write_base(f.trunc() as i64, to, places, cell),
            Err(e) => e,
        }
    }

    /// The `number` of a DEC2* call as a number. Unlike BIN2*, empty text (typed or a ref to
    /// `=""`) is `#VALUE!`, as is other non-numeric text; a blank cell is 0
    /// (probes/eng-dec2base-empty-text, excel-graph; engineering.md "DEC2BIN, DEC2HEX and DEC2OCT
    /// reject empty text", T-3498).
    fn dec_digits(&self, value: &CalcResult, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
        self.cast_number_no_bools_ref(value, cell)
    }

    // BIN2DEC(number)
    pub(crate) fn fn_bin2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2dec, args, cell, &mut |_model, values| {
            Self::base_to_decimal(values, Radix::Bin, cell)
        })
    }

    // BIN2HEX(number, [places])
    pub(crate) fn fn_bin2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2hex, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Bin, Radix::Hex, cell)
        })
    }

    // BIN2OCT(number, [places])
    pub(crate) fn fn_bin2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2oct, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Bin, Radix::Oct, cell)
        })
    }

    // DEC2BIN(number, [places])
    pub(crate) fn fn_dec2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2Bin, args, cell, &mut |model, values| {
            model.decimal_to_base(values, Radix::Bin, cell)
        })
    }

    // DEC2HEX(number, [places])
    pub(crate) fn fn_dec2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2hex, args, cell, &mut |model, values| {
            model.decimal_to_base(values, Radix::Hex, cell)
        })
    }

    // DEC2OCT(number, [places])
    pub(crate) fn fn_dec2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2oct, args, cell, &mut |model, values| {
            model.decimal_to_base(values, Radix::Oct, cell)
        })
    }

    // HEX2BIN(number, [places])
    pub(crate) fn fn_hex2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2bin, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Hex, Radix::Bin, cell)
        })
    }

    // HEX2DEC(number)
    pub(crate) fn fn_hex2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2dec, args, cell, &mut |_model, values| {
            Self::base_to_decimal(values, Radix::Hex, cell)
        })
    }

    // HEX2OCT(number, [places])
    pub(crate) fn fn_hex2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2oct, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Hex, Radix::Oct, cell)
        })
    }

    // OCT2BIN(number, [places])
    pub(crate) fn fn_oct2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2bin, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Oct, Radix::Bin, cell)
        })
    }

    // OCT2DEC(number)
    pub(crate) fn fn_oct2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2dec, args, cell, &mut |_model, values| {
            Self::base_to_decimal(values, Radix::Oct, cell)
        })
    }

    // OCT2HEX(number, [places])
    pub(crate) fn fn_oct2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2hex, args, cell, &mut |model, values| {
            model.convert_base(values, Radix::Oct, Radix::Hex, cell)
        })
    }
}
