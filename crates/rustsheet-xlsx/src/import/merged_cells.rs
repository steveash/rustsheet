//! A sheet's merged ranges on import: keeping the first of any that intersect, in time close to
//! linear in their number (T-3454), and clearing their covered cells, in time proportional to the
//! cells and ranges the sheet holds, not to the area the ranges name (T-3442). Both are directive 5:
//! a few MB of `<mergeCell>`s are about 10^5 ranges, each able to name the whole sheet.

use std::collections::BTreeMap;

use rustsheet_engine::{
    types::{Cell, MergedCell, SheetData},
    LAST_COLUMN,
};

/// The merged ranges kept so far, which never intersect, indexed so that whether a new range
/// intersects one of them takes O(log^2) time instead of a check against each.
///
/// A segment tree over the columns `1..=LAST_COLUMN`, of depth 15, holding row spans: a range is
/// filed under the canonical nodes its columns decompose into (`whole`), and those nodes and their
/// ancestors note its rows (`within`). Two column spans overlap exactly when a canonical node of one
/// is at or under a canonical node of the other, so a range `q` hits a kept range when the rows
/// noted `within` one of `q`'s canonical nodes, or filed `whole` at one of their ancestors, overlap
/// `q`'s rows. Filing a range touches at most about 4 x 15 nodes.
#[derive(Default)]
pub(super) struct KeptRanges {
    /// The tree's nodes by heap number: the root is 1, the children of `n` are `2n` and `2n + 1`.
    /// Allocated on the first range kept: `TREE_NODES` empty nodes, about 1.5 MB.
    nodes: Vec<ColumnNode>,
}

/// The heap numbers of the column tree are below this: `LAST_COLUMN` is 2^14, so the leaves are
/// numbered `2^14..2^15`.
const TREE_NODES: usize = 1 << 15;

/// The row spans one node of [`KeptRanges`] holds.
#[derive(Default)]
struct ColumnNode {
    /// The rows of the kept ranges whose columns cover all of this node's columns and decompose
    /// into this node.
    whole: RowSpans,
    /// The rows of the kept ranges with a canonical node at or under this one.
    within: RowSpans,
}

/// A union of row spans, merged into disjoint ones keyed by first row: whether a span overlaps
/// the union is one lookup, and a merge removes the spans it absorbs, so filing is amortized
/// O(log n).
#[derive(Default)]
struct RowSpans(BTreeMap<i32, i32>);

impl RowSpans {
    /// Whether some row of `first..=last` is in the union. The union's spans are disjoint, so the
    /// last one starting at or before `last` reaches furthest.
    fn overlaps(&self, first: i32, last: i32) -> bool {
        self.0
            .range(..=last)
            .next_back()
            .is_some_and(|(_, &end)| end >= first)
    }

    /// Adds `first..=last` to the union, merging the spans it overlaps.
    fn insert(&mut self, mut first: i32, mut last: i32) {
        // Already in one span: the common case near the root, where the spans merge early.
        if self
            .0
            .range(..=first)
            .next_back()
            .is_some_and(|(_, &end)| end >= last)
        {
            return;
        }
        while let Some((&start, &end)) = self.0.range(..=last).next_back() {
            if end < first {
                break;
            }
            self.0.remove(&start);
            first = first.min(start);
            last = last.max(end);
        }
        self.0.insert(first, last);
    }
}

impl KeptRanges {
    /// Keeps `range` unless it intersects a range kept before; returns whether it was kept.
    pub(super) fn keep(&mut self, range: &MergedCell) -> bool {
        let (first_row, last_row) = (range.row, range.last_row());
        let mut hit = false;
        for_each_node(range.column, range.last_column(), |index, canonical| {
            if let Some(node) = self.nodes.get(index) {
                let rows = if canonical { &node.within } else { &node.whole };
                hit = hit || rows.overlaps(first_row, last_row);
            }
        });
        if hit {
            return false;
        }
        if self.nodes.is_empty() {
            self.nodes.resize_with(TREE_NODES, ColumnNode::default);
        }
        for_each_node(range.column, range.last_column(), |index, canonical| {
            if let Some(node) = self.nodes.get_mut(index) {
                node.within.insert(first_row, last_row);
                if canonical {
                    node.whole.insert(first_row, last_row);
                }
            }
        });
        true
    }
}

/// Visits the nodes of the column tree that `first..=last` reaches: each canonical node of its
/// decomposition (`true`) and each ancestor of one (`false`). Columns outside `1..=LAST_COLUMN`
/// reach no node. Iterative: the stack holds at most one pending sibling per level, 15 in all.
fn for_each_node(first: i32, last: i32, mut visit: impl FnMut(usize, bool)) {
    let mut stack = [(0_usize, 0_i32, 0_i32); 16];
    stack[0] = (1, 1, LAST_COLUMN);
    let mut depth = 1;
    while depth > 0 {
        depth -= 1;
        let (index, low, high) = stack[depth];
        if last < low || high < first {
            continue;
        }
        if first <= low && high <= last {
            visit(index, true);
            continue;
        }
        visit(index, false);
        let middle = low + (high - low) / 2;
        // A pushed node is only split again while its span is longer than one column, so the
        // stack never outgrows the tree's depth; `get_mut` keeps an unexpected overflow harmless.
        for child in [(2 * index + 1, middle + 1, high), (2 * index, low, middle)] {
            if let Some(slot) = stack.get_mut(depth) {
                *slot = child;
                depth += 1;
            }
        }
    }
}

/// Empties the cells that `merged_cells` cover (all but each range's top left cell), keeping their
/// styles: covered cells must not hold content (an engine invariant), and Excel leaves them empty
/// while third-party producers sometimes leave values behind.
///
/// One sweep over the stored cells, in row order, with the ranges whose rows are open kept by first
/// column: a range can name the whole sheet in a few bytes, so walking each one's area would cost
/// 2^34 lookups. `load_merge_cells` drops intersecting ranges, so the open ones never overlap, and
/// the one with the largest first column at or before a cell is the only one that can cover it.
///
/// # Preconditions
/// - The ranges never intersect, as `load_merge_cells` (worksheets.rs) guarantees by dropping any
///   that intersects one already kept. With intersecting ranges some covered cells stay as they
///   were; nothing panics.
/// - `SheetData::cells` yields the cells by row, then by column, as it documents.
pub(super) fn clear_covered_cells(sheet_data: &mut SheetData, merged_cells: &[MergedCell]) {
    let mut by_row: Vec<&MergedCell> = merged_cells.iter().collect();
    by_row.sort_by_key(|merged| merged.row);
    let mut opening = by_row.into_iter().peekable();
    let mut open: BTreeMap<i32, &MergedCell> = BTreeMap::new();
    let mut covered = Vec::new();
    for (row, column, _) in sheet_data.cells() {
        while let Some(merged) = opening.next_if(|merged| merged.row <= row) {
            open.insert(merged.column, merged);
        }
        // A range whose rows have closed is dropped when first met; each is dropped once.
        while let Some((&first, merged)) = open.range(..=column).next_back() {
            if merged.last_row() >= row {
                if column <= merged.last_column() && (row, column) != (merged.row, merged.column) {
                    covered.push((row, column));
                }
                break;
            }
            open.remove(&first);
        }
    }
    for (row, column) in covered {
        if let Some(cell) = sheet_data.cell_mut(row, column) {
            *cell = Cell::EmptyCell {
                s: cell.get_style(),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use rustsheet_engine::types::{Cell, MergedCell, SheetData};

    use super::{clear_covered_cells, KeptRanges};

    fn number(v: f64) -> Cell {
        Cell::NumberCell { v, s: 3 }
    }

    fn merged(row: i32, column: i32, width: i32, height: i32) -> MergedCell {
        MergedCell {
            row,
            column,
            width,
            height,
        }
    }

    /// Covered cells are emptied with their style kept; anchors and cells outside stay.
    #[test]
    fn covered_cells_are_emptied_and_the_rest_kept() {
        let mut data = SheetData::new();
        for (row, column) in [(1, 1), (1, 2), (2, 1), (2, 2), (3, 3), (5, 2), (6, 2)] {
            data.set_cell(row, column, number(1.0));
        }
        let ranges = [merged(1, 1, 2, 2), merged(5, 2, 1, 2), merged(3, 4, 1, 1)];
        clear_covered_cells(&mut data, &ranges);
        let empty = Cell::EmptyCell { s: 3 };
        assert_eq!(data.cell(1, 1), Some(&number(1.0)));
        assert_eq!(data.cell(1, 2), Some(&empty));
        assert_eq!(data.cell(2, 1), Some(&empty));
        assert_eq!(data.cell(2, 2), Some(&empty));
        assert_eq!(data.cell(3, 3), Some(&number(1.0)));
        assert_eq!(data.cell(5, 2), Some(&number(1.0)));
        assert_eq!(data.cell(6, 2), Some(&empty));
    }

    /// A closed range with a smaller first column does not hide an open one, and a range that
    /// starts later in the sheet is only open from its first row.
    #[test]
    fn closed_and_later_ranges_do_not_mix_up() {
        let mut data = SheetData::new();
        for (row, column) in [(1, 1), (1, 3), (2, 3), (4, 1), (4, 3), (5, 3)] {
            data.set_cell(row, column, number(2.0));
        }
        let ranges = [merged(1, 1, 4, 1), merged(4, 3, 1, 2)];
        clear_covered_cells(&mut data, &ranges);
        assert_eq!(data.cell(1, 3), Some(&Cell::EmptyCell { s: 3 }));
        assert_eq!(data.cell(2, 3), Some(&number(2.0)));
        assert_eq!(data.cell(4, 1), Some(&number(2.0)));
        assert_eq!(data.cell(4, 3), Some(&number(2.0)));
        assert_eq!(data.cell(5, 3), Some(&Cell::EmptyCell { s: 3 }));
    }

    /// The rule the index replaces: keep a range unless it intersects one kept before.
    fn kept_by_pairwise_checks(ranges: &[MergedCell]) -> Vec<MergedCell> {
        let mut kept: Vec<MergedCell> = Vec::new();
        for range in ranges {
            if !kept
                .iter()
                .any(|m| m.intersects(range.row, range.column, range.width, range.height))
            {
                kept.push(*range);
            }
        }
        kept
    }

    fn kept_by_index(ranges: &[MergedCell]) -> Vec<MergedCell> {
        let mut index = KeptRanges::default();
        ranges.iter().filter(|r| index.keep(r)).copied().collect()
    }

    /// Shapes that cross without either holding a corner of the other, whole rows and columns,
    /// shared edges and touching but disjoint ranges.
    #[test]
    fn the_index_keeps_what_pairwise_checks_keep_on_edge_shapes() {
        let ranges = [
            merged(5, 1, 16_384, 1),    // a whole row 5
            merged(1, 3, 1, 1_048_576), // a whole column C: crosses row 5
            merged(1, 3, 1, 4),         // inside column C
            merged(6, 1, 16_384, 1),    // row 6: touches row 5, disjoint
            merged(7, 100, 3, 3),       // CV7:CX9
            merged(9, 102, 2, 2),       // shares the corner CX9
            merged(10, 102, 2, 2),      // just below: disjoint
            merged(3, 101, 1, 10),      // a column through CW7:CW9
            merged(1, 16_383, 2, 4),    // the last columns
            merged(4, 16_384, 1, 1),    // single cell under it, still disjoint
            merged(1, 16_384, 1, 2),    // inside the last-column range
        ];
        assert_eq!(kept_by_index(&ranges), kept_by_pairwise_checks(&ranges));
        assert_eq!(kept_by_index(&ranges).len(), 6);
    }

    /// A xorshift generator of values below a bound, so the random tests are reproducible.
    fn pseudo_random() -> impl FnMut(u64) -> i32 {
        let mut state: u64 = 0x2545_f491_4f6c_dd1d;
        move |bound| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            i32::try_from(state % bound).unwrap_or(0)
        }
    }

    /// Pseudo-random ranges on a small grid, so that many intersect, give the same kept list.
    #[test]
    fn the_index_keeps_what_pairwise_checks_keep_on_random_ranges() {
        let mut next = pseudo_random();
        let ranges: Vec<MergedCell> = (0..4_000)
            .map(|_| merged(1 + next(200), 1 + next(300), 1 + next(12), 1 + next(12)))
            .collect();
        let kept = kept_by_pairwise_checks(&ranges);
        assert!(kept.len() > 100 && kept.len() < 3_000, "{}", kept.len());
        assert_eq!(kept_by_index(&ranges), kept);
    }

    /// Pseudo-random ranges over all the columns, some thousands of columns wide, so that the
    /// tree's upper levels hold ranges too, give the same kept list.
    #[test]
    fn the_index_keeps_what_pairwise_checks_keep_on_wide_random_ranges() {
        let mut next = pseudo_random();
        let ranges: Vec<MergedCell> = (0..4_000)
            .map(|i| {
                let width = if i % 4 == 0 {
                    1 + next(8_000)
                } else {
                    1 + next(40)
                };
                let column = 1 + next(u64::from((16_385 - width).unsigned_abs()));
                merged(1 + next(2_000), column, width, 1 + next(30))
            })
            .collect();
        let kept = kept_by_pairwise_checks(&ranges);
        assert!(kept.len() > 100 && kept.len() < 3_900, "{}", kept.len());
        assert!(kept.iter().any(|m| m.width > 1_000), "no wide range kept");
        assert_eq!(kept_by_index(&ranges), kept);
    }
}
