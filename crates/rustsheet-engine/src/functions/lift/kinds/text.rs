//! The lifting table's text entries (SPEC-0011 "The argument-kind table"): the element-wise text
//! functions, LEFT to VALUETOTEXT, and DATEVALUE and TIMEVALUE (T-1775).

use super::ArgKind::{Lift, WholeByCaller};
use super::KernelWork::Constant;
use super::{
    ArgKinds, CallResult, LIFT_1, LIFT_1_OF_2, LIFT_1_OF_3, LIFT_2, LIFT_2_OF_3, LIFT_3,
    LIFT_3_OF_4, LIFT_4,
};
use crate::functions::Function;

/// `CONCATENATE(text1, [text2], ...)`: up to Excel's 255 arguments, every one lifting.
const CONCATENATE: ArgKinds = ArgKinds {
    required: 1,
    params: &[],
    repeat: &[Lift],
    max: 255,
    work: Constant,
    result: CallResult::Value,
};

/// `TEXTJOIN(delimiter, ignore_empty, text1, [text2], ...)`: `ignore_empty` lifts over an array
/// and a range (`TEXTJOIN(",",{TRUE;FALSE},"a","","b")` is 2x1, `TEXTJOIN(",",C1:D1,..)` 1x2,
/// probes/textjoin-ignore-empty-lift; T-2318). The delimiter and the texts are read once by the
/// function itself (`Model::fn_textjoin`), which builds at most two joins, one per value of
/// `ignore_empty`, so an element costs O(1). The 255-argument cap is unprobed (Excel's
/// documentation says 252 texts).
const TEXTJOIN: ArgKinds = ArgKinds {
    required: 3,
    params: &[WholeByCaller, Lift],
    repeat: &[WholeByCaller],
    max: 255,
    work: Constant,
    result: CallResult::Value,
};

/// This family's part of the lifting table (`Function::arg_kinds`).
pub(super) fn arg_kinds(function: &Function) -> Option<&'static ArgKinds> {
    match function {
        // Every argument lifts over a range and an array, on the range's own rows too, and the
        // results broadcast as the operators' do (probes/text-argument-modes; docs/excel-semantics/
        // arrays.md "Errors broadcast through `&`, comparisons, POWER and the element-wise
        // functions", probes/error-precedence-text). DATEVALUE and TIMEVALUE lift like them
        // (probes/date-argument-modes F9:F10, F47:F48), and so do CODE, UNICHAR, UNICODE (T-2090), BAHTTEXT and
        // FILTERXML (probes/text-argument-modes F5:F6, text-bahttext-form4, -form5,
        // filterxml-array-forms). Each element is O(1) or linear in its texts, which the
        // 32,767-unit cell text limit bounds, and the driver's 64 MiB text budget bounds the call; FIND and SEARCH scan their
        // texts once per element, REPT and SUBSTITUTE check their result's length before they
        // build it.
        Function::Len
        | Function::Lenb
        | Function::Value
        | Function::Code
        | Function::Unichar
        | Function::Unicode
        | Function::Bahttext
        | Function::Datevalue
        | Function::Timevalue => Some(&LIFT_1),
        Function::Left
        | Function::Leftb
        | Function::Right
        | Function::Rightb
        | Function::Valuetotext
        | Function::Dollar => Some(&LIFT_1_OF_2),
        // Every argument of DOLLAR, FIXED and NUMBERVALUE lifts over a range and an array, and
        // mixed shapes broadcast with #N/A fill (probes/t2198-m5-lift-argument-kinds rows 20-35).
        Function::Fixed | Function::Numbervalue => Some(&LIFT_1_OF_3),
        Function::Rept | Function::Exact | Function::Text | Function::Filterxml => Some(&LIFT_2),
        Function::Find | Function::Findb | Function::Search | Function::Searchb => {
            Some(&LIFT_2_OF_3)
        }
        Function::Mid | Function::Midb => Some(&LIFT_3),
        Function::Substitute => Some(&LIFT_3_OF_4),
        Function::Replace | Function::Replaceb => Some(&LIFT_4),
        Function::Concatenate => Some(&CONCATENATE),
        Function::Textjoin => Some(&TEXTJOIN),
        _ => None,
    }
}
