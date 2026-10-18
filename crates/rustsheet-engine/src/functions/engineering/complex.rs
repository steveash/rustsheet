use std::fmt;

use crate::functions::lift::{num_error, value_error};
use crate::functions::util::aggregate_args::{
    AggregatePolicy, AggregateWalk, ArgReading, ErrorRule, ItemSource,
};
use crate::limits::too_many_visits;
use crate::reference_operators::RefArg;
use crate::{
    calc_result::CalcResult,
    expressions::{
        lexer::util::get_tokens,
        parser::Node,
        token::{Error, OpSum, TokenType},
        types::CellReferenceIndex,
    },
    functions::Function,
    model::Model,
    number_format::to_precision,
};

/// This implements all functions with complex arguments in the standard
/// NOTE: If performance is ever needed we should have a new entry in `CalcResult`,
/// So this functions will return `CalcResult::Complex(x,y`, Suffix)
/// and not having to parse it over and over again.

#[derive(Clone, Copy, PartialEq, Debug)]
enum Suffix {
    I,
    J,
}

impl fmt::Display for Suffix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Suffix::I => write!(f, "i"),
            Suffix::J => write!(f, "j"),
        }
    }
}

struct Complex {
    x: f64,
    y: f64,
    suffix: Suffix,
}

/// Scientific text with Excel's signed exponent (`3E+200`, `1.2E-16`); Rust prints `3E200`.
fn scientific(v: f64) -> String {
    let text = format!("{v:E}");
    match text.split_once('E') {
        Some((mantissa, exponent)) if !exponent.starts_with('-') => {
            format!("{mantissa}E+{exponent}")
        }
        _ => text,
    }
}

/// Whether a part prints in scientific notation. Tiny parts do (`<= 9e-20`, probes/eng-impower);
/// large ones from 1e21 on, like Excel's number-to-text (UNVERIFIED threshold; `1E+200`,
/// `-3E+200-4E+200i`: probes/eng-imdiv A19, A24).
fn is_scientific(v: f64) -> bool {
    v.abs() <= 9e-20 || v.abs() >= 1e21
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = to_precision(self.x, 15);
        let y = to_precision(self.y, 15);
        let suffix = &self.suffix;
        // it is a bit weird what Excel does but it seems it uses general notation for
        // numbers > 1e-20 and scientific notation for the rest
        let y_str = if is_scientific(y) {
            scientific(y)
        } else if y == 1.0 {
            "".to_string()
        } else if y == -1.0 {
            "-".to_string()
        } else {
            format!("{y}")
        };
        let x_str = if is_scientific(x) {
            scientific(x)
        } else {
            format!("{x}")
        };
        if y == 0.0 && x == 0.0 {
            write!(f, "0")
        } else if y == 0.0 {
            write!(f, "{x_str}")
        } else if x == 0.0 {
            write!(f, "{y_str}{suffix}")
        } else if y > 0.0 {
            write!(f, "{x_str}+{y_str}{suffix}")
        } else {
            write!(f, "{x_str}{y_str}{suffix}")
        }
    }
}

fn parse_complex_number(s: &str) -> Result<(f64, f64, Suffix), String> {
    // Check for i, j, -i, -j
    let (sign, s) = match s.strip_prefix('-') {
        Some(r) => (-1.0, r),
        None => (1.0, s),
    };
    match s {
        "i" => return Ok((0.0, sign * 1.0, Suffix::I)),
        "j" => return Ok((0.0, sign * 1.0, Suffix::J)),
        _ => {
            // Let it go
        }
    };

    // TODO: This is an overuse
    let tokens = get_tokens(s);

    // There has to be 1, 2 3, or 4 tokens
    // number
    // number suffix
    // number1+suffix
    // number1+number2 suffix

    match tokens.len() {
        1 => {
            // Real number
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            // i is the default
            Ok((sign * number1, 0.0, Suffix::I))
        }
        2 => {
            // number2 i
            let number2 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[1].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            Ok((0.0, sign * number2, suffix))
        }
        3 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[2].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            let number2 = if matches!(operation, OpSum::Minus) {
                -1.0
            } else {
                1.0
            };
            Ok((sign * number1, number2, suffix))
        }
        4 => {
            let number1 = match tokens[0].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let operation = match &tokens[1].token {
                TokenType::Addition(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let mut number2 = match tokens[2].token {
                TokenType::Number(f) => f,
                _ => return Err(format!("Not a complex number: {s}")),
            };
            let suffix = match &tokens[3].token {
                TokenType::Ident(w) => match w.as_str() {
                    "i" => Suffix::I,
                    "j" => Suffix::J,
                    _ => return Err(format!("Not a complex number: {s}")),
                },
                _ => {
                    return Err(format!("Not a complex number: {s}"));
                }
            };
            if matches!(operation, OpSum::Minus) {
                number2 = -number2;
            }
            Ok((sign * number1, number2, suffix))
        }
        _ => Err(format!("Not a complex number: {s}")),
    }
}

/// A parsed complex number: the real part, the imaginary part and the suffix.
type ComplexParts = (f64, f64, Suffix);

/// One complex function of a parsed number (IMABS … IMTAN).
type ComplexFn = fn(ComplexParts, CellReferenceIndex) -> CalcResult;

/// The text result of a complex function.
fn complex_result(x: f64, y: f64, suffix: Suffix) -> CalcResult {
    CalcResult::String(Complex { x, y, suffix }.to_string().into())
}

/// How a complex argument reads empty text: IMCOS, IMCOSH, IMCOT, IMCSC, IMCSCH, IMSUM and
/// IMPRODUCT parse it (`#NUM!`), the other functions read it as 0. `Zero` is our older behavior,
/// not Excel's rule for IMSQRT (probably `#NUM!`, T-1385).
#[derive(Clone, Copy, PartialEq, Eq)]
enum EmptyText {
    Zero,
    Parse,
    /// IMSUB, IMDIV and IMPOWER: a blank cell is 0 (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx
    /// Sheet2 G7:G10), empty text is `#NUM!` (probes/eng-imsub A12, A13).
    ZeroIfBlank,
}

/// How IMSUM and IMPRODUCT combine their numbers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ComplexFold {
    Sum,
    Product,
}

/// IMSUM's and IMPRODUCT's walk: the first bad item in argument order is the result, whether it
/// is an error cell, a boolean (`#VALUE!`) or non-complex text (`#NUM!`): `IMSUM(M5,M4)` is
/// `#NUM!`, `IMSUM(M4,M5)` `#VALUE!`, `IMSUM(M7,M5)` `#DIV/0!` (probes/eng-imsum-improduct-whole
/// B4:B9, B27). Blank cells are kept, so a cached blank of a linked workbook is a blank too.
const COMPLEX_FOLD_POLICY: AggregatePolicy = AggregatePolicy {
    reading: ArgReading::PickedKeepingBlanks,
    errors: ErrorRule::First,
    skip_empty_args: false,
};

/// The cells of the rectangle `rows` x `columns`, 0 when either is not positive.
fn rectangle_cells(rows: i32, columns: i32) -> u64 {
    let rows = u64::try_from(rows).unwrap_or(0);
    let columns = u64::try_from(columns).unwrap_or(0);
    rows.saturating_mul(columns)
}

impl Model {
    /// The complex number of one element value: its text, parsed. Non-complex text is `#NUM!`,
    /// and a boolean, typed or in a cell, is `#VALUE!` (`IMABS(TRUE)`, `IMSUB(M4,"1")`:
    /// probes/eng-argument-modes A89, probes/eng-imsum-improduct-whole C1:C15).
    fn complex_of(
        value: &CalcResult,
        empty: EmptyText,
        cell: CellReferenceIndex,
    ) -> Result<ComplexParts, CalcResult> {
        if matches!(value, CalcResult::Boolean(_)) {
            return Err(value_error(cell));
        }
        let blank = matches!(value, CalcResult::EmptyCell | CalcResult::EmptyArg);
        let value = Self::cast_string_ref(value, cell)?;
        if value.is_empty()
            && (empty == EmptyText::Zero || (empty == EmptyText::ZeroIfBlank && blank))
        {
            return Ok((0.0, 0.0, Suffix::I));
        }
        parse_complex_number(&value).map_err(|_| num_error(cell))
    }

    /// A one-argument complex function, lifted: its argument lifts over an array and refuses a
    /// multi-cell range (`ArgKind::LiftArrays`, probes/eng-argument-modes B27:B51, E27:E51).
    fn lift_complex(
        &mut self,
        kind: Function,
        args: &[Node],
        empty: EmptyText,
        op: ComplexFn,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(kind, args, cell, &mut |_, values| {
            let [value] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            match Self::complex_of(value, empty, cell) {
                Ok(z) => op(z, cell),
                Err(error) => error,
            }
        })
    }

    /// Two complex arguments of one element (IMDIV, IMSUB), read left to right; empty text and
    /// different suffixes are `#NUM!` (engineering.md "T-1625 probes").
    fn complex_pair_of(
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<(ComplexParts, ComplexParts), CalcResult> {
        let [first, second] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        let first = Self::complex_of(first, EmptyText::ZeroIfBlank, cell)?;
        let second = Self::complex_of(second, EmptyText::ZeroIfBlank, cell)?;
        if first.2 != second.2 {
            return Err(num_error(cell));
        }
        Ok((first, second))
    }

    /// `COMPLEX(real_num, i_num, [suffix])`: every argument lifts over an array and refuses a
    /// multi-cell range (probes/eng-argument-modes B13:E13, probes/scalar-lift-open-a Y11,
    /// probes/eng-later-argument-lift H16).
    pub(crate) fn fn_complex(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Complex, args, cell, &mut |model, values| {
            let (x, y, suffix) = match values {
                [x, y] => (x, y, None),
                [x, y, suffix] => (x, y, Some(suffix)),
                _ => return CalcResult::new_args_number_error(cell),
            };
            // A boolean is `#VALUE!`: `COMPLEX(TRUE,1)` (probes/eng-imsum-improduct-whole C12).
            let x = match model.cast_number_no_bools_ref(x, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let y = match model.cast_number_no_bools_ref(y, cell) {
                Ok(s) => s,
                Err(s) => return s,
            };
            let suffix = match suffix.map(|suffix| Self::cast_string_ref(suffix, cell)) {
                None => Suffix::I,
                Some(Ok(s)) if s == "i" || s.is_empty() => Suffix::I,
                Some(Ok(s)) if s == "j" => Suffix::J,
                Some(Ok(_)) => return value_error(cell),
                Some(Err(s)) => return s,
            };
            complex_result(x, y, suffix)
        })
    }

    pub(crate) fn fn_imabs(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imabs, args, EmptyText::Zero, imabs, cell)
    }

    pub(crate) fn fn_imaginary(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imaginary, args, EmptyText::Zero, imaginary, cell)
    }

    pub(crate) fn fn_imargument(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(
            Function::Imargument,
            args,
            EmptyText::Zero,
            imargument,
            cell,
        )
    }

    pub(crate) fn fn_imconjugate(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(
            Function::Imconjugate,
            args,
            EmptyText::Zero,
            imconjugate,
            cell,
        )
    }

    pub(crate) fn fn_imcos(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcos, args, EmptyText::Parse, imcos, cell)
    }

    pub(crate) fn fn_imcosh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcosh, args, EmptyText::Parse, imcosh, cell)
    }

    pub(crate) fn fn_imcot(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcot, args, EmptyText::Parse, imcot, cell)
    }

    pub(crate) fn fn_imcsc(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsc, args, EmptyText::Parse, imcsc, cell)
    }

    pub(crate) fn fn_imcsch(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imcsch, args, EmptyText::Parse, imcsch, cell)
    }

    /// `IMDIV(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B36:E36, probes/scalar-lift-open-a Y13).
    pub(crate) fn fn_imdiv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Imdiv, args, cell, &mut |_, values| {
            let ((x1, y1, suffix), (x2, y2, _)) = match Self::complex_pair_of(values, cell) {
                Ok(pair) => pair,
                Err(error) => return error,
            };
            if x2 == 0.0 && y2 == 0.0 {
                return num_error(cell);
            }
            // Smith's algorithm: `x2 * x2 + y2 * y2` underflows for a tiny divisor, but Excel
            // gives `IMDIV("1","1E-200")` = 1E+200 (probes/eng-imdiv A19, A24).
            let (re, im) = if x2.abs() >= y2.abs() {
                let r = y2 / x2;
                let d = x2 + y2 * r;
                ((x1 + y1 * r) / d, (y1 - x1 * r) / d)
            } else {
                let r = x2 / y2;
                let d = x2 * r + y2;
                ((x1 * r + y1) / d, (y1 * r - x1) / d)
            };
            complex_result(re, im, suffix)
        })
    }

    pub(crate) fn fn_imexp(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imexp, args, EmptyText::Zero, imexp, cell)
    }

    pub(crate) fn fn_imln(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imln, args, EmptyText::Zero, imln, cell)
    }

    pub(crate) fn fn_imlog10(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imlog10, args, EmptyText::Zero, imlog10, cell)
    }

    pub(crate) fn fn_imlog2(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imlog2, args, EmptyText::Zero, imlog2, cell)
    }

    // IMPOWER(imnumber, power): both lift over an array and refuse a multi-cell range
    // (probes/eng-argument-modes B41:E41, probes/scalar-lift-open-a Y14).
    // If $(r, \theta)$ is the polar representation the formula is:
    //  $$ x = r^n*\cos(n\dot\theta), y = r^n*\csin(n\dot\theta) $
    pub(crate) fn fn_impower(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::Impower, args, cell, &mut |model, values| {
            let [number, n] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            let (x, y, suffix) = match Self::complex_of(number, EmptyText::ZeroIfBlank, cell) {
                Ok(s) => s,
                Err(error) => return error,
            };
            let n = match model.cast_number_no_bools_ref(n, cell) {
                Ok(f) => f,
                Err(s) => return s,
            };
            if n == 0.0 && x == 0.0 && y == 0.0 {
                return num_error(cell); // probes/eng-impower A6
            }
            let r = f64::sqrt(x * x + y * y);
            let a = f64::atan2(y, x);

            let x = r.powf(n) * f64::cos(a * n);
            let y = r.powf(n) * f64::sin(a * n);

            if x.is_infinite() || y.is_infinite() || x.is_nan() || y.is_nan() {
                return num_error(cell);
            }
            complex_result(x, y, suffix)
        })
    }

    /// `IMPRODUCT(inumber1, ...)`: the product of every number its arguments hold, read whole
    /// ([`Model::fold_complex`]).
    pub(crate) fn fn_improduct(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_complex(Function::Improduct, args, ComplexFold::Product, cell)
    }

    pub(crate) fn fn_imreal(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imreal, args, EmptyText::Zero, imreal, cell)
    }

    pub(crate) fn fn_imsec(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsec, args, EmptyText::Zero, imsec, cell)
    }

    pub(crate) fn fn_imsech(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsech, args, EmptyText::Zero, imsech, cell)
    }

    pub(crate) fn fn_imsin(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsin, args, EmptyText::Zero, imsin, cell)
    }

    pub(crate) fn fn_imsinh(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsinh, args, EmptyText::Zero, imsinh, cell)
    }

    pub(crate) fn fn_imsqrt(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imsqrt, args, EmptyText::Zero, imsqrt, cell)
    }

    /// `IMSUB(inumber1, inumber2)`: both lift over an array and refuse a multi-cell range
    /// (probes/eng-argument-modes B49:E49, probes/scalar-lift-open-a Y12, Y15).
    pub(crate) fn fn_imsub(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(
            Function::Imsub,
            args,
            cell,
            &mut |_, values| match Self::complex_pair_of(values, cell) {
                Ok(((x1, y1, suffix), (x2, y2, _))) => complex_result(x1 - x2, y1 - y2, suffix),
                Err(error) => error,
            },
        )
    }

    /// `IMSUM(inumber1, ...)`: the sum of every number its arguments hold, read whole
    /// ([`Model::fold_complex`]).
    pub(crate) fn fn_imsum(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.fold_complex(Function::Imsum, args, ComplexFold::Sum, cell)
    }

    /// IMSUM and IMPRODUCT: every argument is read whole, never lifted (`ArgKind::WholeByCaller`),
    /// so the call is one value (probes/eng-argument-modes A60:A83, B/C/E42, B/C/E50;
    /// docs/excel-semantics/engineering.md "IMSUM and IMPRODUCT take their arguments whole"):
    /// - a range, a reference a function returns and an array give each of their cells, in
    ///   row-major order; a union or a 3-D reference is `#VALUE!` (`IMSUM((M1,M6))`,
    ///   probes/eng-imsum-improduct-whole B19);
    /// - a blank cell is 0, so IMPRODUCT over a range with a blank cell, also one past the used
    ///   area, is "0" (A73, B11); an empty argument is skipped (`IMPRODUCT(M1,)` is "1+2i", B18);
    /// - empty text is `#NUM!` (B13:B16, B25), and the first bad item in order is the result
    ///   ([`COMPLEX_FOLD_POLICY`]); different suffixes are `#VALUE!` (B10).
    ///
    /// The cells read are capped (directive 5): the cells a range walks (a whole column or row
    /// clipped to the used area) and the elements of an array add up, and the call is `#ERROR!`
    /// once they pass `MAX_LIFTED_CELL_VISITS`, before the range is walked. Excel has no such
    /// limit: DEVIATION (engineering.md).
    fn fold_complex(
        &mut self,
        kind: Function,
        args: &[Node],
        fold: ComplexFold,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !kind
            .arg_kinds()
            .is_some_and(|kinds| kinds.accepts(args.len()))
        {
            return CalcResult::new_args_number_error(cell);
        }
        let mut total: Option<ComplexParts> = None;
        let mut has_blank = false;
        let mut visits: u64 = 0;
        let mut walk = AggregateWalk::new(COMPLEX_FOLD_POLICY);
        for arg in args {
            let read = self.evaluate_picked_reference_arg(arg, cell);
            // The cells of the range (blank cells included), and the cells its walk visits.
            let (cells, walked) = match &read {
                RefArg::Ranges(ranges) => match &ranges[..] {
                    [range] if range.left.sheet == range.right.sheet => {
                        let (left, right) = (range.left, range.right);
                        let corner = self.used_area_corner(
                            left.sheet,
                            left.row,
                            left.column,
                            right.row,
                            right.column,
                        );
                        let (row2, column2) = match corner {
                            Ok(corner) => corner,
                            Err(message) => {
                                return CalcResult::new_error(Error::ERROR, cell, message)
                            }
                        };
                        let cells = rectangle_cells(
                            right.row - left.row + 1,
                            right.column - left.column + 1,
                        );
                        let walked =
                            rectangle_cells(row2 - left.row + 1, column2 - left.column + 1);
                        (cells, walked)
                    }
                    _ => return value_error(cell),
                },
                RefArg::Value(CalcResult::Array(array)) => {
                    let cells: usize = array.iter().map(Vec::len).sum();
                    (0, u64::try_from(cells).unwrap_or(u64::MAX))
                }
                RefArg::Value(_) => (0, 1),
            };
            visits = visits.saturating_add(walked);
            if let Some(error) = too_many_visits(visits, cell) {
                return error;
            }
            let is_reference = matches!(arg.ungrouped(), Node::ReferenceKind { .. });
            // The cells of a range the walk hands on: the others are blank.
            let mut referenced: u64 = 0;
            let walked = self.walk_aggregate_read(
                read,
                is_reference,
                cell,
                &mut walk,
                &mut |_, source, value| {
                    if source == ItemSource::ReferencedCell {
                        referenced += 1;
                    }
                    let z = match value {
                        CalcResult::EmptyArg => return Ok(()),
                        CalcResult::EmptyCell => {
                            has_blank = true;
                            return Ok(());
                        }
                        value => Self::complex_of(&value, EmptyText::Parse, cell)?,
                    };
                    total = Some(match total {
                        None => z,
                        Some((x, y, suffix)) if suffix == z.2 => match fold {
                            ComplexFold::Sum => (x + z.0, y + z.1, suffix),
                            ComplexFold::Product => (x * z.0 - y * z.1, x * z.1 + y * z.0, suffix),
                        },
                        Some(_) => return Err(value_error(cell)),
                    });
                    Ok(())
                },
            );
            if let Err(error) = walked {
                return error;
            }
            if referenced < cells {
                has_blank = true;
            }
        }
        if let Err(error) = walk.finish() {
            return error;
        }
        let (x, y, suffix) = total.unwrap_or((0.0, 0.0, Suffix::I));
        if fold == ComplexFold::Product && has_blank {
            return complex_result(0.0, 0.0, suffix);
        }
        complex_result(x, y, suffix)
    }

    pub(crate) fn fn_imtan(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_complex(Function::Imtan, args, EmptyText::Zero, imtan, cell)
    }
}

// The one-argument complex functions, of a parsed number.

fn imabs((x, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(f64::sqrt(x * x + y * y))
}

fn imaginary((_, y, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(y)
}

fn imargument((x, y, _): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if x == 0.0 && y == 0.0 {
        return CalcResult::new_error(Error::DIV, cell, String::new());
    }
    CalcResult::Number(f64::atan2(y, x))
}

fn imconjugate((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x, -y, suffix)
}

fn imcos((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.cos() * y.cosh(), -x.sin() * y.sinh(), suffix)
}

fn imcosh((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.cosh() * y.cos(), x.sinh() * y.sin(), suffix)
}

fn imcot((x, y, suffix): ComplexParts, cell: CellReferenceIndex) -> CalcResult {
    if x == 0.0 && y != 0.0 {
        return complex_result(0.0, -1.0 / y.tanh(), suffix);
    } else if y == 0.0 {
        return complex_result(1.0 / x.tan(), 0.0, suffix);
    }

    let x_cot = 1.0 / x.tan();
    let y_coth = 1.0 / y.tanh();

    let t = x_cot * x_cot + y_coth * y_coth;
    let x = (x_cot * y_coth * y_coth - x_cot) / t;
    let y = (-x_cot * x_cot * y_coth - y_coth) / t;

    if x.is_infinite() || y.is_infinite() || x.is_nan() || y.is_nan() {
        return num_error(cell);
    }
    complex_result(x, y, suffix)
}

fn imcsc((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cos = x.cos();
    let x_sin = x.sin();

    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_sin * x_sin * y_cosh * y_cosh + x_cos * x_cos * y_sinh * y_sinh;
    complex_result(x_sin * y_cosh / t, -x_cos * y_sinh / t, suffix)
}

fn imcsch((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_sinh * x_sinh * y_cos * y_cos + x_cosh * x_cosh * y_sin * y_sin;
    complex_result(x_sinh * y_cos / t, -x_cosh * y_sin / t, suffix)
}

fn imexp((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.exp() * y.cos(), x.exp() * y.sin(), suffix)
}

fn imln((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.ln(), a, suffix)
}

fn imlog10((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.log10(), a * f64::log10(f64::exp(1.0)), suffix)
}

fn imlog2((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y);
    let a = f64::atan2(y, x);
    complex_result(r.log2(), a * f64::log2(f64::exp(1.0)), suffix)
}

fn imreal((x, _, _): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    CalcResult::Number(x)
}

fn imsec((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cos = x.cos();
    let x_sin = x.sin();

    let y_cosh = y.cosh();
    let y_sinh = y.sinh();

    let t = x_cos * x_cos * y_cosh * y_cosh + x_sin * x_sin * y_sinh * y_sinh;
    complex_result(x_cos * y_cosh / t, x_sin * y_sinh / t, suffix)
}

fn imsech((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_cosh = x.cosh();
    let x_sinh = x.sinh();

    let y_cos = y.cos();
    let y_sin = y.sin();

    let t = x_cosh * x_cosh * y_cos * y_cos + x_sinh * x_sinh * y_sin * y_sin;
    complex_result(x_cosh * y_cos / t, -x_sinh * y_sin / t, suffix)
}

fn imsin((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.sin() * y.cosh(), x.cos() * y.sinh(), suffix)
}

fn imsinh((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    complex_result(x.sinh() * y.cos(), x.cosh() * y.sin(), suffix)
}

fn imsqrt((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let r = f64::sqrt(x * x + y * y).sqrt();
    let a = f64::atan2(y, x);
    complex_result(r * f64::cos(a / 2.0), r * f64::sin(a / 2.0), suffix)
}

fn imtan((x, y, suffix): ComplexParts, _: CellReferenceIndex) -> CalcResult {
    let x_tan = x.tan();
    let y_tanh = y.tanh();

    let t = 1.0 + x_tan * x_tan * y_tanh * y_tanh;
    complex_result(
        (x_tan - x_tan * y_tanh * y_tanh) / t,
        (y_tanh + x_tan * x_tan * y_tanh) / t,
        suffix,
    )
}

#[cfg(test)]
mod tests {
    use crate::functions::engineering::complex::Suffix;

    use super::parse_complex_number as parse;

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse("1+2i"), Ok((1.0, 2.0, Suffix::I)));
        assert_eq!(parse("2i"), Ok((0.0, 2.0, Suffix::I)));
        assert_eq!(parse("7.5"), Ok((7.5, 0.0, Suffix::I)));
        assert_eq!(parse("-7.5"), Ok((-7.5, 0.0, Suffix::I)));
        assert_eq!(parse("7-5i"), Ok((7.0, -5.0, Suffix::I)));
        assert_eq!(parse("i"), Ok((0.0, 1.0, Suffix::I)));
        assert_eq!(parse("7+i"), Ok((7.0, 1.0, Suffix::I)));
        assert_eq!(parse("7-i"), Ok((7.0, -1.0, Suffix::I)));
        assert_eq!(parse("-i"), Ok((0.0, -1.0, Suffix::I)));
        assert_eq!(parse("0"), Ok((0.0, 0.0, Suffix::I)));
    }
}
