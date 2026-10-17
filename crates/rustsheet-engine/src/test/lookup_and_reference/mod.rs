mod stack_bound;
mod test_approximate_search;
mod test_fn_address_areas;
mod test_fn_choosecols_chooserows;
mod test_fn_expand;
mod test_fn_hlookup;
mod test_fn_hstack_vstack;
mod test_fn_index;
mod test_fn_lookup;
mod test_fn_match;
mod test_fn_rows_columns;
mod test_fn_trimrange;
mod test_fn_vlookup;
mod test_fn_wrapcols_wraprows;
mod test_fn_xmatch;
mod test_xlookup_lift;
mod test_xlookup_wide;

use crate::test::util::new_empty_model;

/// Evaluates each formula at `at` in a fresh model holding `data`, once as is
/// and once with `filler` set too: a cell that stretches the used area to the
/// sheet's last row or column without lying in any searched vector. The lookup
/// functions evaluate whole-column and whole-row search vectors only up to the
/// used area (T-0119); that must never change a result, so both runs must
/// agree. Returns the results.
fn results_independent_of_used_area(
    data: &[(&str, &str)],
    at: &str,
    filler: &str,
    formulas: &[&str],
) -> Vec<String> {
    let evaluate = |formula: &str, fill: bool| {
        let mut model = new_empty_model();
        for (cell, value) in data {
            model._set(cell, value);
        }
        if fill {
            model._set(filler, "filler");
        }
        model._set(at, formula);
        model.evaluate();
        model._get_text(at)
    };
    formulas
        .iter()
        .map(|formula| {
            let clipped = evaluate(formula, false);
            assert_eq!(clipped, evaluate(formula, true), "{formula}");
            clipped
        })
        .collect()
}
