//! The base conversions BIN2DEC … OCT2HEX. Every argument lifts over an array and refuses a
//! multi-cell range (`ArgKind::LiftArrays`, SPEC-0011; probes/eng-argument-modes E5:E7, E15:E17,
//! E24:E26, E52:E54, probes/scalar-lift-shapes H9, probes/eng-later-argument-lift H4:H11): each
//! kernel is the scalar function of one element, and builds its own errors with empty messages.

use crate::functions::lift::num_error;
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, types::CellReferenceIndex},
    functions::Function,
    model::Model,
};

// 8_i64.pow(10);
const OCT_MAX: i64 = 1_073_741_824;
const OCT_MAX_HALF: i64 = 536_870_912;
// 16_i64.pow(10)
const HEX_MAX: i64 = 1_099_511_627_776;
const HEX_MAX_HALF: i64 = 549_755_813_888;
// Binary numbers are 10 bits and the most significant bit is the sign

fn from_binary_to_decimal(value: f64) -> Result<i64, ()> {
    let value = format!("{value}");

    let Ok(result) = i64::from_str_radix(&value, 2) else {
        return Err(());
    };
    if !(0..=1023).contains(&result) {
        // 2^10
        return Err(());
    } else if result > 511 {
        // 2^9
        return Ok(result - 1024);
    };
    Ok(result)
}

/// The number and the optional `places` of one conversion element.
/// Splits the arguments of a base conversion. Callers coerce `places` before `number`: Excel reads
/// the second argument first, so its error wins (probes/eng-first-vs-second-error).
fn number_and_places(values: &[CalcResult]) -> Option<(&CalcResult, Option<&CalcResult>)> {
    match values {
        [number] => Some((number, None)),
        [number, places] => Some((number, Some(places))),
        _ => None,
    }
}

impl Model {
    /// The digit text of a HEX/OCT `number`. Excel reads a blank cell and empty text as 0 in every
    /// base conversion (probes/eng-base-conv-blank, docs/excel-semantics/engineering.md).
    fn base_digits(value: &CalcResult, cell: CellReferenceIndex) -> Result<String, CalcResult> {
        let text = Self::cast_string_ref(value, cell)?;
        Ok(if text.is_empty() {
            "0".to_owned()
        } else {
            text
        })
    }

    /// The `number` of a BIN2* call as a number, with a blank cell or empty text read as 0 (same
    /// probe as `base_digits`).
    fn binary_digits(
        &self,
        value: &CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        if matches!(value, CalcResult::String(s) if s.is_empty()) {
            return Ok(0.0);
        }
        self.cast_number_no_bools_ref(value, cell)
    }

    // BIN2DEC(number)
    pub(crate) fn fn_bin2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2dec, args, cell, &mut |model, values| {
            model.bin2dec_of(values, cell)
        })
    }

    fn bin2dec_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [number] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let value = match self.binary_digits(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        match from_binary_to_decimal(value) {
            Ok(n) => CalcResult::Number(n as f64),
            Err(()) => num_error(cell),
        }
    }

    // BIN2HEX(number, [places])
    pub(crate) fn fn_bin2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2hex, args, cell, &mut |model, values| {
            model.bin2hex_of(values, cell)
        })
    }

    fn bin2hex_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match self.binary_digits(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let value = match from_binary_to_decimal(value) {
            Ok(n) => n,
            Err(()) => return num_error(cell),
        };
        if value < 0 {
            CalcResult::String(format!("{:0width$X}", HEX_MAX + value, width = 9).into())
        } else {
            let result = format!("{value:X}");
            if let Some(places) = places {
                if places < result.len() as i32 {
                    return num_error(cell);
                }
                return CalcResult::String(
                    format!("{:0width$X}", value, width = places as usize).into(),
                );
            }
            CalcResult::String(result.into())
        }
    }

    // BIN2OCT(number, [places])
    pub(crate) fn fn_bin2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Bin2oct, args, cell, &mut |model, values| {
            model.bin2oct_of(values, cell)
        })
    }

    fn bin2oct_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match self.binary_digits(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        let value = match from_binary_to_decimal(value) {
            Ok(n) => n,
            Err(()) => return num_error(cell),
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        if value < 0 {
            CalcResult::String(format!("{:0width$o}", OCT_MAX + value, width = 9).into())
        } else {
            let result = format!("{value:o}");
            if let Some(places) = places {
                if places < result.len() as i32 {
                    return num_error(cell);
                }
                return CalcResult::String(
                    format!("{:0width$o}", value, width = places as usize).into(),
                );
            }
            CalcResult::String(result.into())
        }
    }

    pub(crate) fn fn_dec2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2Bin, args, cell, &mut |model, values| {
            model.dec2bin_of(values, cell)
        })
    }

    fn dec2bin_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value_raw = match self.binary_digits(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = value_raw.trunc() as i64;
        if !(-512..=511).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += 1024;
        }
        let result = format!("{value:b}");
        if let Some(places) = places {
            if value_raw > 0.0 && places < result.len() as i32 {
                return num_error(cell);
            }
            let result = format!("{:0width$b}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    pub(crate) fn fn_dec2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2hex, args, cell, &mut |model, values| {
            model.dec2hex_of(values, cell)
        })
    }

    fn dec2hex_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value_raw = match self.binary_digits(number, cell) {
            Ok(f) => f.trunc(),
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = value_raw.trunc() as i64;
        if !(-HEX_MAX_HALF..=HEX_MAX_HALF - 1).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += HEX_MAX;
        }
        let result = format!("{value:X}");
        if let Some(places) = places {
            if value_raw > 0.0 && places < result.len() as i32 {
                return num_error(cell);
            }
            let result = format!("{:0width$X}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    pub(crate) fn fn_dec2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Dec2oct, args, cell, &mut |model, values| {
            model.dec2oct_of(values, cell)
        })
    }

    fn dec2oct_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value_raw = match self.binary_digits(number, cell) {
            Ok(f) => f,
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = value_raw.trunc() as i64;

        if !(-OCT_MAX_HALF..=OCT_MAX_HALF - 1).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += OCT_MAX;
        }
        let result = format!("{value:o}");
        if let Some(places) = places {
            if value_raw > 0.0 && places < result.len() as i32 {
                return num_error(cell);
            }
            let result = format!("{:0width$o}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    // HEX2BIN(number, [places])
    pub(crate) fn fn_hex2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2bin, args, cell, &mut |model, values| {
            model.hex2bin_of(values, cell)
        })
    }

    fn hex2bin_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        if value.len() > 10 {
            return num_error(cell);
        }
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = match i64::from_str_radix(&value, 16) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value >= HEX_MAX_HALF {
            value -= HEX_MAX;
        }
        if !(-512..=511).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += 1024;
        }
        let result = format!("{value:b}");
        if let Some(places) = places {
            if places <= 0 || (value > 0 && places < result.len() as i32) {
                return num_error(cell);
            }
            let result = format!("{:0width$b}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    // HEX2DEC(number)
    pub(crate) fn fn_hex2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2dec, args, cell, &mut |model, values| {
            model.hex2dec_of(values, cell)
        })
    }

    fn hex2dec_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [number] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        if value.len() > 10 {
            return num_error(cell);
        }
        let mut value = match i64::from_str_radix(&value, 16) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value >= HEX_MAX_HALF {
            value -= HEX_MAX;
        }
        CalcResult::Number(value as f64)
    }

    pub(crate) fn fn_hex2oct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Hex2oct, args, cell, &mut |model, values| {
            model.hex2oct_of(values, cell)
        })
    }

    fn hex2oct_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        if value.len() > 10 {
            return num_error(cell);
        }
        let mut value = match i64::from_str_radix(&value, 16) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value > HEX_MAX_HALF {
            value -= HEX_MAX;
        }
        if !(-OCT_MAX_HALF..=OCT_MAX_HALF - 1).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += OCT_MAX;
        }
        let result = format!("{value:o}");
        if let Some(places) = places {
            if places <= 0 || (value > 0 && places < result.len() as i32) {
                return num_error(cell);
            }
            let result = format!("{:0width$o}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    pub(crate) fn fn_oct2bin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2bin, args, cell, &mut |model, values| {
            model.oct2bin_of(values, cell)
        })
    }

    fn oct2bin_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = match i64::from_str_radix(&value, 8) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value >= OCT_MAX_HALF {
            value -= OCT_MAX;
        }
        if !(-512..=511).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += 1024;
        }
        let result = format!("{value:b}");
        if let Some(places) = places {
            if value < 512 && places < result.len() as i32 {
                return num_error(cell);
            }
            let result = format!("{:0width$b}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }

    pub(crate) fn fn_oct2dec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2dec, args, cell, &mut |model, values| {
            model.oct2dec_of(values, cell)
        })
    }

    fn oct2dec_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let [number] = values else {
            return CalcResult::new_args_number_error(cell);
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        let mut value = match i64::from_str_radix(&value, 8) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value >= OCT_MAX_HALF {
            value -= OCT_MAX;
        }
        CalcResult::Number(value as f64)
    }

    pub(crate) fn fn_oct2hex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Oct2hex, args, cell, &mut |model, values| {
            model.oct2hex_of(values, cell)
        })
    }

    fn oct2hex_of(&self, values: &[CalcResult], cell: CellReferenceIndex) -> CalcResult {
        let Some((number, places)) = number_and_places(values) else {
            return CalcResult::new_args_number_error(cell);
        };
        let places = match places.map(|p| self.cast_number_no_bools_ref(p, cell)) {
            Some(Ok(f)) => Some(f.trunc() as i32),
            Some(Err(s)) => return s,
            None => None,
        };
        let value = match Self::base_digits(number, cell) {
            Ok(s) => s,
            Err(s) => return s,
        };
        // There is not a default value for places
        // But if there is a value it needs to be positive and less than 11
        if let Some(p) = places {
            if p <= 0 || p > 10 {
                return num_error(cell);
            }
        }
        let mut value = match i64::from_str_radix(&value, 8) {
            Ok(f) => f,
            Err(_) => {
                return num_error(cell);
            }
        };
        if value < 0 {
            return num_error(cell);
        }

        if value >= OCT_MAX_HALF {
            value -= OCT_MAX;
        }

        if !(-HEX_MAX_HALF..=HEX_MAX_HALF - 1).contains(&value) {
            return num_error(cell);
        }
        if value < 0 {
            value += HEX_MAX;
        }
        let result = format!("{value:X}");
        if let Some(places) = places {
            if value < HEX_MAX_HALF && places < result.len() as i32 {
                return num_error(cell);
            }
            let result = format!("{:0width$X}", value, width = places as usize);
            return CalcResult::String(result.into());
        }
        CalcResult::String(result.into())
    }
}
