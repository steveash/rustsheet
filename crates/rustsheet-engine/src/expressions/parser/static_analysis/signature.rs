//! The argument signatures of functions: whether each argument position takes a scalar or a
//! vector (an array or a range), used by `get_function_args_signature` to decide implicit
//! intersection. Moved out of `static_analysis.rs` (T-1326) to keep it under its size lock.

#[derive(Clone)]
pub(super) enum Signature {
    Scalar,
    Vector,
    Error,
}

pub(super) fn args_signature_scalars(
    arg_count: usize,
    required_count: usize,
    optional_count: usize,
) -> Vec<Signature> {
    if arg_count >= required_count && arg_count <= required_count + optional_count {
        vec![Signature::Scalar; arg_count]
    } else {
        vec![Signature::Error; arg_count]
    }
}

/// FIELDVALUE(value, `field_name)`: `value` may be an array (the result then has its shape).
pub(super) fn args_signature_fieldvalue(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector, Signature::Scalar]
    } else {
        vec![Signature::Error; arg_count]
    }
}

/// `GETPIVOTDATA(data_field, pivot_table, [field1, item1, ...])`: every argument is taken as is, a
/// range included (it never intersects or lifts; probes/getpivotdata-evidence-floor D6, D26).
pub(super) fn args_signature_getpivotdata(arg_count: usize) -> Vec<Signature> {
    if arg_count >= 2 {
        vec![Signature::Vector; arg_count]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_lambda(arg_count: usize) -> Vec<Signature> {
    // LAMBDA([param1, param2, ...], body) — at least 1 arg (the body)
    if arg_count == 0 {
        return vec![Signature::Error; arg_count];
    }
    // All parameter names are Scalar; the final body argument is Vector.
    let mut sig = vec![Signature::Scalar; arg_count];
    sig[arg_count - 1] = Signature::Vector;
    sig
}

pub(super) fn args_signature_let(arg_count: usize) -> Vec<Signature> {
    // LET requires an odd number of args >= 3: name1, value1, [name2, value2, ...], body
    if arg_count < 3 || arg_count.is_multiple_of(2) {
        return vec![Signature::Error; arg_count];
    }
    (0..arg_count)
        .map(|i| {
            // Odd indices are value expressions; last index is the body.
            // Both should be Vector so range references are not implicitly intersected.
            // Even indices (except last) are name declarations — Scalar is fine since
            // they are never actually evaluated as range expressions.
            if i % 2 == 1 || i == arg_count - 1 {
                Signature::Vector
            } else {
                Signature::Scalar
            }
        })
        .collect()
}

pub(super) fn args_signature_arraytotext(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        1 => vec![Signature::Vector],
        2 => vec![Signature::Vector, Signature::Scalar],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_one_vector(arg_count: usize) -> Vec<Signature> {
    if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_sumif(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Scalar],
        3 => vec![Signature::Vector, Signature::Scalar, Signature::Vector],
        _ => vec![Signature::Error; arg_count],
    }
}

/// COUNTIFS (`first_criterion` 1), SUMIFS/AVERAGEIFS/MAXIFS/MINIFS (2): every criterion is a
/// scalar, so a legacy formula intersects a range criterion like COUNTIF and a dynamic one lifts
/// it (T-0278, fixtures/ifs-legacy-range-criteria, oracle excel-graph; T-0207).
pub(super) fn args_signature_ifs(arg_count: usize, first_criterion: usize) -> Vec<Signature> {
    if arg_count <= first_criterion || (arg_count - first_criterion).is_multiple_of(2) {
        return vec![Signature::Error; arg_count];
    }
    let mut signature = vec![Signature::Vector; arg_count];
    for slot in signature.iter_mut().skip(first_criterion).step_by(2) {
        *slot = Signature::Scalar;
    }
    signature
}

/// `CELL(info_type`, [reference]): a legacy formula does not intersect `reference` (C3 and C8 both
/// read A1 for `A1:A5`; probes/info-cell-contents-range D3, D8, excel-graph, T-1230).
pub(super) fn args_signature_cell(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        1 => vec![Signature::Scalar],
        2 => vec![Signature::Scalar, Signature::Vector],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_hlookup(arg_count: usize) -> Vec<Signature> {
    if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Scalar]
    } else if arg_count == 4 {
        vec![
            Signature::Vector,
            Signature::Vector,
            Signature::Scalar,
            Signature::Vector,
        ]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_index(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Scalar],
        3 => vec![Signature::Vector, Signature::Scalar, Signature::Scalar],
        4 => vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_lookup(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Vector],
        3 => vec![Signature::Vector, Signature::Vector, Signature::Vector],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_match(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Vector],
        3 => vec![Signature::Vector, Signature::Vector, Signature::Scalar],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_offset(arg_count: usize) -> Vec<Signature> {
    if arg_count == 3 {
        vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
    } else if arg_count == 4 {
        vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ]
    } else if arg_count == 5 {
        vec![
            Signature::Vector,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
            Signature::Scalar,
        ]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_row(arg_count: usize) -> Vec<Signature> {
    if arg_count == 0 {
        vec![]
    } else if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_xlookup(arg_count: usize) -> Vec<Signature> {
    if !(3..=6).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result[1] = Signature::Vector;
    result[2] = Signature::Vector;
    result
}

pub(super) fn args_signature_sort(arg_count: usize) -> Vec<Signature> {
    if !(1..=4).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_sortby(arg_count: usize) -> Vec<Signature> {
    // Valid: 2, 3, 5, 7, ... (n==2 || n==3 || n>=5 with odd count)
    if arg_count < 2 || (arg_count > 3 && arg_count.is_multiple_of(2)) {
        return vec![Signature::Error; arg_count];
    }
    // arg[0] = array (Vector), arg[1] = by_array1 (Vector),
    // optional arg[2] = sort_order1 (Scalar), then pairs (Vector, Scalar)
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    let mut i = 1;
    while i < arg_count {
        result[i] = Signature::Vector; // by_array
        i += 2; // skip the sort_order (Scalar)
    }
    result
}

pub(super) fn args_signature_unique(arg_count: usize) -> Vec<Signature> {
    if !(1..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_filter(arg_count: usize) -> Vec<Signature> {
    if !(2..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result[1] = Signature::Vector;
    result
}

pub(super) fn args_signature_sequence(arg_count: usize) -> Vec<Signature> {
    if !(1..=4).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    vec![Signature::Scalar; arg_count]
}

pub(super) fn args_signature_randarray(arg_count: usize) -> Vec<Signature> {
    if arg_count > 5 {
        return vec![Signature::Error; arg_count];
    }
    vec![Signature::Scalar; arg_count]
}

pub(super) fn args_signature_take(arg_count: usize) -> Vec<Signature> {
    if !(2..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_drop(arg_count: usize) -> Vec<Signature> {
    if !(2..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_tocol(arg_count: usize) -> Vec<Signature> {
    if !(1..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_bycol_byrow(arg_count: usize) -> Vec<Signature> {
    if arg_count != 2 {
        return vec![Signature::Error; arg_count];
    }
    vec![Signature::Vector, Signature::Scalar]
}

/// GROUPBY: every argument is a vector except `field_headers` and `total_depth`, the two integer
/// options (SPEC-0006 step 7; `sort_order` may be an array such as `{2,-1}`).
pub(super) fn args_signature_groupby(arg_count: usize) -> Vec<Signature> {
    if !(3..=8).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Vector; arg_count];
    // `arg_count` is 3 to 8 here, so the range is in bounds.
    result[3..arg_count.min(5)].fill(Signature::Scalar);
    result
}

/// PIVOTBY: every argument is a vector except `field_headers`, `row_total_depth` and
/// `col_total_depth`, the integer options (SPEC-0006 step 7; the sort orders may be arrays such as
/// `{2,-1}`). `relative_to` is a vector too, so nothing intersects it: PIVOTBY lifts an array there
/// per element itself (probes/pivotby-followup P!A681, `Model::lifted_relative_to`, T-1327).
pub(super) fn args_signature_pivotby(arg_count: usize) -> Vec<Signature> {
    if !(4..=11).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Vector; arg_count];
    for option in [4, 5, 7] {
        if let Some(signature) = result.get_mut(option) {
            *signature = Signature::Scalar;
        }
    }
    result
}

pub(super) fn args_signature_map(arg_count: usize) -> Vec<Signature> {
    if arg_count < 2 {
        return vec![Signature::Error; arg_count];
    }
    // All args except the last (lambda) are Vector; the lambda is Scalar.
    let mut result = vec![Signature::Vector; arg_count];
    result[arg_count - 1] = Signature::Scalar;
    result
}

pub(super) fn args_signature_reduce(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Scalar],
        3 => vec![Signature::Scalar, Signature::Vector, Signature::Scalar],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_scan(arg_count: usize) -> Vec<Signature> {
    match arg_count {
        2 => vec![Signature::Vector, Signature::Scalar],
        3 => vec![Signature::Scalar, Signature::Vector, Signature::Scalar],
        _ => vec![Signature::Error; arg_count],
    }
}

pub(super) fn args_signature_textsplit(arg_count: usize) -> Vec<Signature> {
    if !(2..=6).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    // col_delimiter (index 1) and row_delimiter (index 2) can be arrays
    let mut result = vec![Signature::Scalar; arg_count];
    result[1] = Signature::Vector;
    if arg_count >= 3 {
        result[2] = Signature::Vector;
    }
    result
}

pub(super) fn args_signature_textafter(arg_count: usize) -> Vec<Signature> {
    if !(2..=6).contains(&arg_count) {
        vec![Signature::Scalar; arg_count]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_textjoin(arg_count: usize) -> Vec<Signature> {
    if arg_count >= 3 {
        let mut result = vec![Signature::Vector; arg_count];
        result[0] = Signature::Scalar;
        result[1] = Signature::Scalar;
        result
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_npv(arg_count: usize) -> Vec<Signature> {
    if arg_count < 2 {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Vector; arg_count];
    result[0] = Signature::Scalar;
    result
}

pub(super) fn args_signature_irr(arg_count: usize) -> Vec<Signature> {
    if arg_count > 2 {
        vec![Signature::Error; arg_count]
    } else if arg_count == 1 {
        vec![Signature::Vector]
    } else {
        vec![Signature::Vector, Signature::Scalar]
    }
}

pub(super) fn args_signature_xirr(arg_count: usize) -> Vec<Signature> {
    if arg_count == 2 {
        vec![Signature::Vector; arg_count]
    } else if arg_count == 3 {
        vec![Signature::Vector, Signature::Vector, Signature::Scalar]
    } else {
        vec![Signature::Error; arg_count]
    }
}

pub(super) fn args_signature_mirr(arg_count: usize) -> Vec<Signature> {
    if arg_count != 3 {
        vec![Signature::Error; arg_count]
    } else {
        vec![Signature::Vector, Signature::Scalar, Signature::Scalar]
    }
}

pub(super) fn args_signature_xnpv(arg_count: usize) -> Vec<Signature> {
    if arg_count != 3 {
        vec![Signature::Error; arg_count]
    } else {
        vec![Signature::Scalar, Signature::Vector, Signature::Vector]
    }
}

// NETWORKDAYS(start_date, end_date, [holidays]), NETWORKDAYS.INTL(start_date, end_date, [weekend],
// [holidays]), WORKDAY(start_date, days, [holidays]), WORKDAY.INTL(start_date, days, [weekend],
// [holidays]): `required` scalars, then `middle` optional scalars (the weekend), then an optional
// `holidays` that takes the whole range, so a legacy formula does not implicitly intersect it
// (docs/excel-semantics/dates.md "WORKDAY holidays take the whole range").
pub(super) fn args_signature_with_holidays(
    arg_count: usize,
    required: usize,
    middle: usize,
) -> Vec<Signature> {
    let holidays_at = required + middle;
    if arg_count < required || arg_count > holidays_at + 1 {
        vec![Signature::Error; arg_count]
    } else if arg_count == holidays_at + 1 {
        let mut signature = vec![Signature::Scalar; holidays_at];
        signature.push(Signature::Vector);
        signature
    } else {
        vec![Signature::Scalar; arg_count]
    }
}

// FIXME: This is terrible duplications of efforts. We use the signature in at least three different places:
// 1. When computing the function
// 2. Checking the arguments to see if we need to insert the implicit intersection operator
// 3. Understanding the return value
//
pub(super) fn args_signature_xmatch(arg_count: usize) -> Vec<Signature> {
    if !(2..=4).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[1] = Signature::Vector;
    result
}

pub(super) fn args_signature_trimrange(arg_count: usize) -> Vec<Signature> {
    if !(1..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_address(arg_count: usize) -> Vec<Signature> {
    if !(2..=5).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    vec![Signature::Scalar; arg_count]
}

pub(super) fn args_signature_choose(arg_count: usize) -> Vec<Signature> {
    // CHOOSE(index, value1, ...): the index is a scalar; the value arguments
    // pass references through untouched (Excel: SUM(CHOOSE(2, A1:A5, B1:B5))
    // sums the chosen range). In scalar context the implicit intersection is
    // applied to the CHOOSE call itself, never to its arms.
    if arg_count < 2 {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Vector; arg_count];
    result[0] = Signature::Scalar;
    result
}

pub(super) fn args_signature_choosecols(arg_count: usize) -> Vec<Signature> {
    if arg_count < 2 {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_expand(arg_count: usize) -> Vec<Signature> {
    if !(2..=4).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}

pub(super) fn args_signature_wrapcols(arg_count: usize) -> Vec<Signature> {
    if !(2..=3).contains(&arg_count) {
        return vec![Signature::Error; arg_count];
    }
    let mut result = vec![Signature::Scalar; arg_count];
    result[0] = Signature::Vector;
    result
}
