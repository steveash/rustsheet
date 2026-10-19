//! SPEC-0011 Test plan, the metamorphic test (T-1894): every lifting-table entry lifts like its
//! scalar call. With one argument written `{v;v}`, both spilled elements equal the scalar call's
//! value, errors included (an array-valued entry's: its top-left value). `WholeByCaller` arguments
//! stay as written.

use super::super::kinds::{ArgKind, ArgKinds, CallResult};
use crate::functions::Function;
use crate::language::get_default_language;
use crate::test::util::new_empty_model;

/// Sample argument values, by position: small counts, fractions, and dates with rates, so most
/// kernels reach a value and the others an error (which must lift the same way).
/// The table's longest call, ODDFPRICE's, has nine arguments.
const SAMPLES: [[&str; 9]; 3] = [
    ["3", "2", "1", "4", "1", "0", "1", "2", "0"],
    ["0.5", "0.25", "0.75", "0.1", "2", "1", "0", "1", "0"],
    [
        "45000", "45400", "44900", "45100", "0.05", "0.06", "100", "2", "0",
    ],
];

/// Whether the argument at `index` is checked: every one that lifts. A `Whole` argument is read
/// as written and handed to every element (the D-functions' database), and a `WholeByCaller` or
/// `ReferenceByCaller` argument is the function's own reader's.
fn lifts(kinds: &ArgKinds, index: usize) -> bool {
    !matches!(
        kinds.kind(index),
        None | Some(ArgKind::Whole | ArgKind::WholeByCaller | ArgKind::ReferenceByCaller)
    )
}

/// The mismatches of `function`: for each valid argument count, sample set and lifted position,
/// the call text and what each `{v;v}` element gave where the scalar call gave something else.
/// `checked` collects the positions checked.
fn mismatches(function: &Function, kinds: &ArgKinds, checked: &mut Vec<usize>) -> Vec<String> {
    let name = function.to_localized_name(get_default_language());
    // A repeated parameter is checked up to one repeat past the listed ones.
    let last = kinds.max.min(kinds.params.len() + 1).min(SAMPLES[0].len());
    let mut calls = Vec::new();
    for count in kinds.required.max(1)..=last {
        for sample in &SAMPLES {
            let scalar = &sample[..count];
            for index in (0..count).filter(|&index| lifts(kinds, index)) {
                checked.push(index);
                let mut args: Vec<String> = scalar.iter().map(|v| (*v).to_string()).collect();
                args[index] = format!("{{{v};{v}}}", v = scalar[index]);
                // An array-valued entry's element is its scalar call's top-left value
                // (SPEC-0011 "Array-valued functions" rule 2).
                let scalar_call = format!("{name}({})", scalar.join(","));
                let scalar_call = if kinds.result == CallResult::Array {
                    format!("=INDEX({scalar_call},1,1)")
                } else {
                    format!("={scalar_call}")
                };
                calls.push((scalar_call, format!("={name}({})", args.join(","))));
            }
        }
    }
    let mut model = new_empty_model();
    for (row, (scalar, lifted)) in (1..).step_by(3).zip(&calls) {
        model._set(&format!("A{row}"), lifted);
        model._set(&format!("C{row}"), scalar);
    }
    model.evaluate();
    (1..)
        .step_by(3)
        .zip(&calls)
        .filter_map(|(row, (_, lifted))| {
            // Values, not display text: only the anchor takes the units' number format.
            let value = |cell: String| model.get_cell_value_by_ref(&format!("Sheet1!{cell}"));
            let want = value(format!("C{row}"));
            let got = [value(format!("A{row}")), value(format!("A{}", row + 1))];
            (got[0] != want || got[1] != want)
                .then(|| format!("{lifted}: {got:?}, scalar {want:?}"))
        })
        .collect()
}

#[test]
fn every_lifting_table_entry_lifts_like_its_scalar_call() {
    let mut found = Vec::new();
    for function in Function::into_iter() {
        let Some(kinds) = function.arg_kinds() else {
            continue;
        };
        let mut checked = Vec::new();
        found.extend(mismatches(&function, kinds, &mut checked));
        // Every lifted parameter of the entry is checked.
        for index in (0..kinds.max.min(kinds.params.len())).filter(|&i| lifts(kinds, i)) {
            assert!(
                checked.contains(&index),
                "{function:?} argument {index} is not checked"
            );
        }
    }
    assert!(
        found.is_empty(),
        "{} mismatches:\n{}",
        found.len(),
        found.join("\n")
    );
}
