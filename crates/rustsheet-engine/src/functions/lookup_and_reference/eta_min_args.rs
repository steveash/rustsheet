//! Minimum argument counts of the functions that GROUPBY and PIVOTBY can be given by name, for
//! the names whose minimum is not 1 (`eta_min_args.txt`, one `NAME MIN` per line). It is a
//! hand-maintained copy of `eval/catalog/excel-functions.tsv` (column 3): the engine crate cannot
//! read the catalog at run time, so the `table_matches_catalog` test in `group_functions.rs`
//! reads it on purpose and fails on any drift.

const TABLE: &str = include_str!("eta_min_args.txt");

/// The catalog's minimum argument count of the function `name`, 1 when it is not listed.
pub(super) fn min_args(name: &str) -> u8 {
    entries()
        .find_map(|(known, min)| (known == name).then_some(min))
        .unwrap_or(1)
}

/// Every `(name, minimum)` of the table, in file order.
pub(super) fn entries() -> impl Iterator<Item = (&'static str, u8)> {
    TABLE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let (name, min) = line.split_once(' ')?;
            Some((name, min.parse().ok()?))
        })
}
