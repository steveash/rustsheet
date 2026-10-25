//! Excel's argument counts of every catalogued function (`arg_counts.txt`, one `NAME MIN MAX` per
//! line): the minimum of `eval/catalog/excel-functions.tsv`, and the most of Microsoft's syntax
//! line (255 when it repeats or does not parse), a non-repeating `eval/catalog/functions.yaml`
//! signature and a cited override. The engine crate cannot read the catalog at run time, so the
//! table is a hand-maintained copy that `tools/tests/test_catalog_arity.py` and
//! `table_matches_catalog` below check against it, failing on any drift (T-2632).

const TABLE: &str = include_str!("arg_counts.txt");

/// The `(minimum, maximum)` argument counts of the function `name` (its English name, `BETA.DIST`),
/// `None` when the catalog does not list it.
fn counts(name: &str) -> Option<(u8, u8)> {
    entries().find_map(|(known, min, max)| (known == name).then_some((min, max)))
}

/// The catalog's minimum argument count of the function `name`, 1 when it is not listed.
pub(crate) fn min_args(name: &str) -> u8 {
    counts(name).map_or(1, |(min, _)| min)
}

/// Whether Excel takes a call of `name` with `count` arguments, as far as the counts tell; a
/// function the catalog does not list takes any count.
pub(crate) fn takes_count(name: &str, count: usize) -> bool {
    counts(name).is_none_or(|(min, max)| (usize::from(min)..=usize::from(max)).contains(&count))
}

/// Every `(name, minimum, maximum)` of the table, in file order.
fn entries() -> impl Iterator<Item = (&'static str, u8, u8)> {
    TABLE
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let mut columns = line.split(' ');
            let name = columns.next()?;
            let min = columns.next()?.parse().ok()?;
            let max = columns.next()?.parse().ok()?;
            Some((name, min, max))
        })
}

#[cfg(test)]
mod tests {
    use super::{entries, min_args, takes_count};

    /// Every catalog function is listed once, with the catalog's minimum (the maximum is checked by
    /// `tools/tests/test_catalog_arity.py`, which reads `functions.yaml`).
    #[test]
    fn table_matches_catalog() {
        let catalog = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../eval/catalog/excel-functions.tsv"
        ));
        let mut expected: Vec<(&str, u8)> = catalog
            .lines()
            .filter(|line| !line.starts_with('#'))
            .skip(1)
            .filter_map(|line| {
                let mut columns = line.split('\t');
                let name = columns.next()?;
                let min: u8 = columns.nth(1)?.parse().ok()?;
                Some((name, min))
            })
            .collect();
        expected.sort_unstable();
        let mut actual: Vec<(&str, u8)> = entries().map(|(name, min, _)| (name, min)).collect();
        actual.sort_unstable();
        assert_eq!(actual, expected);
        assert!(entries().all(|(_, min, max)| min <= max));
    }

    #[test]
    fn counts_of_listed_and_unlisted_names() {
        assert_eq!(min_args("NOW"), 0);
        assert_eq!(min_args("XLOOKUP"), 3);
        assert_eq!(min_args("NOT.A.FUNCTION"), 1);
        assert!(!takes_count("SUM", 0));
        assert!(takes_count("SUM", 255));
        assert!(!takes_count("XLOOKUP", 7));
        assert!(takes_count("XLOOKUP", 6));
        assert!(!takes_count("ABS", 2));
        assert!(takes_count("NOT.A.FUNCTION", 300));
    }
}
