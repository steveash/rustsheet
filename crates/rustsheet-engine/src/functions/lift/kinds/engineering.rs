//! The lifting table's engineering entries (SPEC-0011 "The argument-kind table"): the Analysis
//! `ToolPak` engineering functions, BIT* and the IMSUM/IMPRODUCT fold.

use super::ArgKind::{LiftArrays, WholeByCaller};
use super::KernelWork::{ByArg, Constant};
use super::{ArgKinds, CallResult, ARRAYS_1, ATP_1_OF_2, ATP_2, ATP_2_OF_3, ATP_3, LIFT_2};
use crate::functions::engineering::transcendental::MAX_ORDER;
use crate::functions::Function;

// --- Engineering (T-1383) -------------------------------------------------------------------
/// `BESSELI|J|K|Y(x, n)`: the recurrences loop about once (BESSELK, BESSELY) or twice (BESSELI,
/// BESSELJ's Miller recurrence) per order, up to the order cap `transcendental::MAX_ORDER`
/// (100,000), each kernel's own per-call cap; n = 0 and 1 are O(1). A lifted call is charged by
/// each element's order (T-1819), `BESSEL_WEIGHT` at the cap, so it lifts any number of small
/// orders (`BESSELJ(SEQUENCE(5000)/100,0)`) but at most 2,048 elements at the cap, which run in
/// about 2 s in release (BESSELI, the slowest: 8,192 elements took 8 s; docs/excel-semantics/
/// engineering.md "BESSEL order cap").
const BESSEL: ArgKinds = ArgKinds::fixed(
    2,
    &[LiftArrays, LiftArrays],
    ByArg {
        arg: 1,
        cap: MAX_ORDER.unsigned_abs(),
        weight: BESSEL_WEIGHT,
    },
);
/// The units one Bessel element at the order cap is charged against `MAX_LIFTED_CELL_VISITS`
/// (one more for the call itself).
const BESSEL_WEIGHT: u32 = (1 << 16) - 1;
/// `IMSUM(inumber1, ...)` and IMPRODUCT: up to 255 arguments, each read whole by the function's
/// own fold (`Model::fold_complex`), so nothing lifts and the call is one value. The fold never
/// enters the lift driver, so its `KernelWork` is not charged; it caps its own reads.
const COMPLEX_FOLD: ArgKinds = ArgKinds {
    required: 1,
    params: &[],
    repeat: Some(WholeByCaller),
    max: 255,
    work: Constant,
    result: CallResult::Value,
};
// --- End engineering (T-1383) ---------------------------------------------------------------

/// This family's part of the lifting table (`Function::arg_kinds`).
pub(super) fn arg_kinds(function: &Function) -> Option<&'static ArgKinds> {
    match function {
        // --- Engineering (T-1383) ---
        // The engineering functions are Analysis ToolPak functions: every argument lifts
        // over an array and refuses a multi-cell range, also on the range's own rows
        // (docs/excel-semantics/engineering.md "Engineering function argument modes"). Each
        // argument is probed over both: the first in probes/eng-argument-modes B:E; the
        // later ones in B:D there and over a range in probes/scalar-lift-shapes H8-H10
        // (BESSELJ's n, DEC2BIN's places, DELTA), probes/scalar-lift-open-a Y8-Y15
        // (CONVERT's units, COMPLEX's suffix, IMSUB's and IMDIV's second, IMPOWER's n) and
        // probes/eng-later-argument-lift H1-H11, H16-H19 (the other orders and places,
        // COMPLEX's i_num, DELTA's number2, GESTEP's step, ERF's upper_limit).
        Function::Besseli | Function::Besselj | Function::Besselk | Function::Bessely => {
            Some(&BESSEL)
        }
        Function::Bin2dec
        | Function::Hex2dec
        | Function::Oct2dec
        | Function::ErfPrecise
        | Function::Erfc
        | Function::ErfcPrecise
        | Function::Imabs
        | Function::Imaginary
        | Function::Imargument
        | Function::Imconjugate
        | Function::Imcos
        | Function::Imcosh
        | Function::Imcot
        | Function::Imcsc
        | Function::Imcsch
        | Function::Imexp
        | Function::Imln
        | Function::Imlog10
        | Function::Imlog2
        | Function::Imreal
        | Function::Imsec
        | Function::Imsech
        | Function::Imsin
        | Function::Imsinh
        | Function::Imsqrt
        | Function::Imtan => Some(&ARRAYS_1),
        // `[places]`, `[number2]`, `[step]`, `[upper_limit]`: one required, one optional.
        Function::Bin2hex
        | Function::Bin2oct
        | Function::Dec2Bin
        | Function::Dec2hex
        | Function::Dec2oct
        | Function::Hex2bin
        | Function::Hex2oct
        | Function::Oct2bin
        | Function::Oct2hex
        | Function::Delta
        | Function::Gestep
        | Function::Erf => Some(&ATP_1_OF_2),
        Function::Imdiv | Function::Imsub | Function::Impower => Some(&ATP_2),
        // `COMPLEX(real_num, i_num, [suffix])`.
        Function::Complex => Some(&ATP_2_OF_3),
        Function::Convert => Some(&ATP_3),
        // BIT* are worksheet functions: both arguments lift over a range too
        // (probes/eng-argument-modes E8:E12, probes/scalar-lift-shapes H7,
        // probes/eng-later-argument-lift H12-H15, H20, K12-K15). The kernels are O(1).
        Function::Bitand
        | Function::Bitor
        | Function::Bitxor
        | Function::Bitlshift
        | Function::Bitrshift => Some(&LIFT_2),
        // IMSUM and IMPRODUCT fold their arguments whole: a range or an array is never
        // lifted, `ROWS(IMSUM({"1+2i";"3"}))` is 1 (probes/eng-argument-modes A60:A83,
        // B/C/E42, B/C/E50; engineering.md "IMSUM and IMPRODUCT take their arguments
        // whole", T-1384).
        Function::Imsum | Function::Improduct => Some(&COMPLEX_FOLD),
        // --- End engineering (T-1383) ---
        _ => None,
    }
}
