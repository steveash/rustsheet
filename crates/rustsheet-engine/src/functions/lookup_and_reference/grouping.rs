//! The grouping core GROUPBY and PIVOTBY share (SPEC-0006 Design steps 2 and 4, T-1322, T-1323):
//! records grouped by a key tuple, sorted by key or by an aggregate, and laid out with the
//! subtotals of a total depth beyond 1 (T-1324). Excel: docs/excel-semantics/arrays.md "GROUPBY
//! and PIVOTBY".

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::borrow::Cow;

use crate::{
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::spill_functions::{cell_key, sort_key_cmp, CellKey},
    functions::util::collation::cmp_text,
};

/// How the groups are ordered: GROUPBY's `sort_order` (PIVOTBY's `row_sort_order` and
/// `col_sort_order` too), read by `Model::sort_order`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortOrder {
    /// By the key fields, `ascending[i]` for field `i`; fields not named ascend
    /// ([`Grouping::sort_by_keys`]).
    Keys(Vec<bool>),
    /// By the aggregate of value column `column` (0-based among the value columns), with
    /// [`Grouping::sort_by_aggregate`].
    Value { column: usize, ascending: bool },
}

/// A key as its group shows it: the blank key as the number 0 (G!A1, PIVOTBY's column keys too:
/// probes/groupby-pivotby-followup G!A821), every other key as first spelled.
pub(crate) fn shown_key(key: &ArrayNode) -> ArrayNode {
    match key {
        ArrayNode::Empty => ArrayNode::Number(0.0),
        other => other.clone(),
    }
}

/// Empty text, the filler of total rows, corners and empty PIVOTBY cells (not a blank: G!AC33).
pub(crate) fn empty_text() -> ArrayNode {
    ArrayNode::String("".into())
}

/// One group: its key tuple as first met in the source (the first spelling, `East` for `East`
/// and `east`, G!A1) and the indices of its source rows, in source order.
pub(crate) struct Group {
    pub(crate) keys: Vec<ArrayNode>,
    pub(crate) rows: Vec<usize>,
    /// The keys as they group ([`group_key`]), built once by [`Grouping::new`] so that comparing
    /// key prefixes later converts no text.
    cells: Vec<CellKey>,
}

/// Rows grouped by a key tuple: the unit GROUPBY lays out as rows and PIVOTBY as a grid.
pub(crate) struct Grouping {
    pub(crate) groups: Vec<Group>,
}

impl Grouping {
    /// Groups `rows` (indices into `keys`, one key tuple per row) like UNIQUE: text is
    /// case-insensitive and numbers compare exactly ([`group_key`]). The key is built once per row
    /// and looked up in a hash map, so grouping is O(rows x key fields). The groups are in order
    /// of first appearance until [`Grouping::sort_by_keys`]. `limit(n)` is the error `n` groups
    /// would give (the output size check, SPEC-0006 step 6); grouping stops at the first group
    /// that does not fit, so no more groups are built than the output can hold.
    pub(crate) fn new(
        keys: &[Vec<ArrayNode>],
        rows: impl IntoIterator<Item = usize>,
        limit: impl Fn(usize) -> Option<(Error, String)>,
    ) -> Result<Self, (Error, String)> {
        let mut index: HashMap<Vec<CellKey>, usize> = HashMap::new();
        let mut groups: Vec<Group> = Vec::new();
        for row in rows {
            let Some(tuple) = keys.get(row) else {
                continue;
            };
            let cells: Vec<CellKey> = tuple.iter().map(group_key).collect();
            match index.entry(cells.clone()) {
                Entry::Occupied(entry) => {
                    if let Some(group) = groups.get_mut(*entry.get()) {
                        group.rows.push(row);
                    }
                }
                Entry::Vacant(entry) => {
                    if let Some(error) = limit(groups.len() + 1) {
                        return Err(error);
                    }
                    entry.insert(groups.len());
                    groups.push(Group {
                        keys: tuple.iter().map(positive_zero).collect(),
                        rows: vec![row],
                        cells,
                    });
                }
            }
        }
        Ok(Grouping { groups })
    }

    /// Sorts the groups by their keys, field by field, `ascending[i]` for field `i` (ascending
    /// when missing). Mixed types sort numbers < text < FALSE < TRUE (G!A193) and the blank key
    /// sorts last in both directions (G!A1, G!A145): that is `sort_key_cmp`. Numbers compare
    /// exactly, not to 15 digits: 0.3 sorts before 0.1+0.2 (probes/groupby-pivotby-followup
    /// G!A761), matching the exact `cell_key` grouping. (Errors against text, booleans and the
    /// blank key: unprobed, SPEC-0006.)
    pub(crate) fn sort_by_keys(&mut self, ascending: &[bool], cell: CellReferenceIndex) {
        self.groups.sort_by(|a, b| {
            a.keys
                .iter()
                .zip(&b.keys)
                .enumerate()
                .map(|(field, (x, y))| {
                    key_cmp(x, y, ascending.get(field).copied().unwrap_or(true), cell)
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Sorts the groups by an aggregate (GROUPBY's `sort_order` naming a value column; PIVOTBY's
    /// sort by a total), and returns the aggregates it sorted by, each group's and (when the
    /// parents sort by value) each parent's, so the caller shows them, subtotals included, instead
    /// of calling the function again: no duplicate calls, and a volatile function shows the values
    /// the order follows. `aggregate` gets source rows in source order.
    ///
    /// With several key fields the sort is hierarchical: the parents (key prefixes) are ordered
    /// by the aggregate of all their rows, and the children by their own value inside each parent
    /// (probes/groupby-pivotby-followup2 G!A261; groupby-sort-filter-args B21-B22 descending).
    /// [`ParentOrder::ByKey`] keeps the parents in key order and sorts only the last level by
    /// value, as PIVOTBY's columns do without subtotals (probes/pivotby-followup P!A1141, P!A1161);
    /// [`ParentOrder::Flat`] sorts every group by its own value, ignoring the parents, as GROUPBY's
    /// table relationship does (probes/groupby-subtotals G!A391-A481).
    /// Equal values keep the ascending key order in both directions (groupby-sort-filter-args
    /// B32-B33, B27-B28), and the blank key is not pinned last (G!J145, G!S145). Values compare like
    /// key cells: numbers < text < FALSE < TRUE < errors, errors last ascending and first
    /// descending (B34-B37; groupby-sort-filter-args2 B27-B28, B37). A parent's value is the
    /// function of all its rows, not of its children's values (AVERAGE and `100-SUM(x)` over two
    /// keys, args2 B22-B25), at every level (three keys, args2 B19-B21).
    ///
    /// Work: one `aggregate` call per group and per parent, at most groups x key fields, which the
    /// output size bounds; each level's parent rows are gathered once (O(rows log rows) per level).
    pub(crate) fn sort_by_aggregate(
        &mut self,
        ascending: bool,
        order: ParentOrder,
        cell: CellReferenceIndex,
        mut aggregate: impl FnMut(&[usize]) -> ArrayNode,
    ) -> Aggregates {
        self.sort_by_keys(&[], cell);
        let fields = match order {
            ParentOrder::Flat => 1,
            _ => self.groups.first().map_or(0, |group| group.keys.len()),
        };
        let parents_by_value = order == ParentOrder::ByValue;
        // `levels[l][g]`: the parent of group g at depth l + 1 (an index in key order of first
        // appearance, so equal parent values keep key order), with the parents' aggregates.
        let mut levels: Vec<(Vec<usize>, Vec<ArrayNode>)> = Vec::new();
        let mut previous = vec![0; self.groups.len()];
        for level in 0..fields.saturating_sub(1) {
            let mut index: HashMap<(usize, &CellKey), usize> = HashMap::new();
            let mut parent_rows: Vec<Vec<usize>> = Vec::new();
            let mut parents = Vec::with_capacity(self.groups.len());
            for (group, parent) in self.groups.iter().zip(&previous) {
                let key = group.cells.get(level).unwrap_or(&CellKey::Empty);
                let id = *index.entry((*parent, key)).or_insert_with(|| {
                    parent_rows.push(Vec::new());
                    parent_rows.len() - 1
                });
                // Parent rows are only gathered when the parents sort by their aggregate.
                if let Some(rows) = parent_rows.get_mut(id).filter(|_| parents_by_value) {
                    rows.extend_from_slice(&group.rows);
                }
                parents.push(id);
            }
            let values = parent_rows
                .iter_mut()
                .map(|rows| {
                    if !parents_by_value {
                        return ArrayNode::Empty;
                    }
                    rows.sort_unstable();
                    aggregate(rows)
                })
                .collect();
            previous.clone_from(&parents);
            levels.push((parents, values));
        }
        let leaves: Vec<ArrayNode> = self
            .groups
            .iter()
            .map(|group| aggregate(&group.rows))
            .collect();
        let mut order: Vec<usize> = (0..self.groups.len()).collect();
        // A stable sort: groups that compare equal stay in key order.
        order.sort_by(|&a, &b| {
            for (parents, values) in &levels {
                let (x, y) = (parents.get(a), parents.get(b));
                if x != y {
                    let (x, y) = (x.copied().unwrap_or(0), y.copied().unwrap_or(0));
                    if !parents_by_value {
                        return x.cmp(&y);
                    }
                    return compare_aggregates(values.get(x), values.get(y), ascending, cell)
                        .then(x.cmp(&y));
                }
            }
            compare_aggregates(leaves.get(a), leaves.get(b), ascending, cell)
        });
        let mut groups: Vec<Option<Group>> = std::mem::take(&mut self.groups)
            .into_iter()
            .map(Some)
            .collect();
        self.groups = order
            .iter()
            .filter_map(|&at| groups.get_mut(at).and_then(Option::take))
            .collect();
        let in_order = |values: &[usize]| -> Vec<usize> {
            order
                .iter()
                .map(|&at| values.get(at).copied().unwrap_or(0))
                .collect()
        };
        Aggregates {
            leaves: order
                .iter()
                .map(|&at| leaves.get(at).cloned().unwrap_or(ArrayNode::Empty))
                .collect(),
            parents: if parents_by_value {
                levels
                    .iter()
                    .map(|(parents, values)| (in_order(parents), values.clone()))
                    .collect()
            } else {
                Vec::new()
            },
        }
    }
}

/// [`key_cmp`] for two aggregates that may be missing (they never are: the indices come from
/// the same groups).
fn compare_aggregates(
    a: Option<&ArrayNode>,
    b: Option<&ArrayNode>,
    ascending: bool,
    cell: CellReferenceIndex,
) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => key_cmp(a, b, ascending, cell),
        _ => Ordering::Equal,
    }
}

/// A key cell as it groups: `cell_key`, except that -0 is 0. Excel stores a cell `=-0` as 0, so
/// 0, `=-0` and `=0` are one group (probes/groupby-pivotby-followup G!A781); the engine keeps the
/// sign of a computed -0.
fn group_key(node: &ArrayNode) -> CellKey {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => cell_key(&ArrayNode::Number(0.0)),
        other => cell_key(other),
    }
}

/// A key cell as a group shows it, -0 as 0 (see [`group_key`]).
fn positive_zero(node: &ArrayNode) -> ArrayNode {
    match node {
        ArrayNode::Number(n) if *n == 0.0 => ArrayNode::Number(0.0),
        other => other.clone(),
    }
}

/// [`sort_key_cmp`] with numbers compared exactly (see [`Grouping::sort_by_keys`]), and text
/// compared in place with the comparator `sort_key_cmp` reaches (`cmp_text`, through
/// `compare_values`) instead of converting and cloning both strings on every comparison.
fn key_cmp(a: &ArrayNode, b: &ArrayNode, ascending: bool, cell: CellReferenceIndex) -> Ordering {
    let ordering = match (a, b) {
        (ArrayNode::Number(x), ArrayNode::Number(y)) => x.total_cmp(y),
        (ArrayNode::String(x), ArrayNode::String(y)) => cmp_text(x, y),
        _ => return sort_key_cmp(a, b, ascending, cell),
    };
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

/// How [`Grouping::sort_by_aggregate`] orders the parents (key prefixes) of a value sort.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParentOrder {
    /// Each parent by the aggregate of all its rows, at every level: GROUPBY, PIVOTBY's rows, and
    /// its columns with subtotals (probes/pivotby-followup P!A941; probes/groupby-subtotals G!A961,
    /// G!A991).
    ByValue,
    /// The parents in key order: PIVOTBY's columns without subtotals (P!A1141, P!A1161).
    ByKey,
    /// No parents: every group by its own value, as GROUPBY's table relationship sorts
    /// (probes/groupby-subtotals G!A391-A481).
    Flat,
}

/// One line of a grouping laid out with subtotals: group `index`, or the subtotal of the groups
/// `start..end`, which share their first `level` keys and no others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Line {
    Group(usize),
    Subtotal {
        level: usize,
        start: usize,
        end: usize,
    },
}

/// The aggregates a value sort ordered the groups by ([`Grouping::sort_by_aggregate`]).
pub(crate) struct Aggregates {
    /// Each group's, in the sorted order.
    leaves: Vec<ArrayNode>,
    /// Per level l (the first l + 1 keys), each group's parent (in the sorted order) and each
    /// parent's aggregate; empty unless the parents sort by value.
    parents: Vec<(Vec<usize>, Vec<ArrayNode>)>,
}

impl Aggregates {
    /// The aggregate of a group or subtotal line, when the sort computed it: a subtotal of the
    /// first `level` keys is the aggregate of the parent at that level.
    pub(crate) fn of_line(&self, line: Line) -> Option<&ArrayNode> {
        match line {
            Line::Group(index) => self.leaves.get(index),
            Line::Subtotal { level, start, .. } => {
                let (parents, values) = self.parents.get(level.checked_sub(1)?)?;
                values.get(*parents.get(start)?)
            }
        }
    }
}

impl Grouping {
    /// The key cells and the source rows (in source order) of a group or subtotal line: a
    /// group's keys as shown, or a subtotal's ([`Grouping::subtotal_label_cells`]) over all its
    /// groups' rows ([`Grouping::rows_of`]). `None` for a group that does not exist.
    pub(crate) fn line_parts(
        &self,
        line: Line,
        key_columns: usize,
    ) -> Option<(Vec<ArrayNode>, Cow<'_, [usize]>)> {
        match line {
            Line::Group(index) => {
                let group = self.groups.get(index)?;
                let label = group.keys.iter().map(shown_key).collect();
                Some((label, Cow::Borrowed(&group.rows[..])))
            }
            Line::Subtotal { level, start, end } => Some((
                self.subtotal_label_cells(level, start, key_columns),
                Cow::Owned(self.rows_of(start, end)),
            )),
        }
    }

    /// The groups in their order with the subtotals of the first `levels` key fields (a total
    /// depth of ±(`levels` + 1)): after the groups they cover, the deepest first, or before them
    /// (`before`, a negative depth), the shallowest first. Depth 2 over three keys subtotals only
    /// the first, 3 both (probes/groupby-subtotals G!A1, G!A61, G!A91; G!J49, G!S49).
    ///
    /// Groups that share a key prefix are next to each other after any sort used with subtotals
    /// (the key sort is lexicographic, the value sort hierarchical), so one linear pass over the
    /// groups finds the subtotals (SPEC-0006 Design step 2); there are at most `levels` per group.
    pub(crate) fn lines(&self, levels: usize, before: bool) -> Vec<Line> {
        let count = self.groups.len();
        // `breaks[i]`: the first key field where group i differs from group i - 1, and 0 before
        // the first group and after the last, where every subtotal starts and ends.
        let breaks: Vec<usize> = (0..=count).map(|i| self.first_difference(i)).collect();
        let mut starts = vec![0; levels + 1];
        let mut lines = Vec::with_capacity(count);
        for (index, pair) in breaks.windows(2).enumerate() {
            let &[here, next] = pair else {
                continue;
            };
            for start in starts.iter_mut().skip(here + 1) {
                *start = index;
            }
            lines.push(Line::Group(index));
            for level in (next + 1..=levels).rev() {
                let start = starts.get(level).copied().unwrap_or(index);
                lines.push(Line::Subtotal {
                    level,
                    start,
                    end: index + 1,
                });
            }
        }
        if !before {
            return lines;
        }
        // Above its groups: each subtotal goes right before its first group, the shallowest
        // first.
        let (mut subtotals, groups): (Vec<Line>, Vec<Line>) = lines
            .into_iter()
            .partition(|line| matches!(line, Line::Subtotal { .. }));
        subtotals.sort_by_key(|line| match *line {
            Line::Subtotal { level, start, .. } => (start, level),
            Line::Group(index) => (index, 0),
        });
        let mut subtotals = subtotals.into_iter().peekable();
        let mut ordered = Vec::with_capacity(groups.len() + subtotals.len());
        for group in groups {
            let Line::Group(index) = group else {
                continue;
            };
            while let Some(line) = subtotals
                .next_if(|line| matches!(line, Line::Subtotal { start, .. } if *start == index))
            {
                ordered.push(line);
            }
            ordered.push(group);
        }
        ordered
    }

    /// Sorts by the keys `order` names, or ascending when it sorts by a value (which
    /// [`Grouping::sort_by_aggregate`] does later, after the output size check): sorting by keys
    /// calls no function.
    pub(crate) fn sort_by_key_order(&mut self, order: &SortOrder, cell: CellReferenceIndex) {
        match order {
            SortOrder::Keys(ascending) => self.sort_by_keys(ascending, cell),
            SortOrder::Value { .. } => self.sort_by_keys(&[], cell),
        }
    }

    /// The first key field where group `index` differs from the group before it, 0 for the
    /// first group and past the last one.
    fn first_difference(&self, index: usize) -> usize {
        let previous = index.checked_sub(1).and_then(|i| self.groups.get(i));
        let (Some(previous), Some(group)) = (previous, self.groups.get(index)) else {
            return 0;
        };
        previous
            .cells
            .iter()
            .zip(&group.cells)
            .position(|(a, b)| a != b)
            .unwrap_or(group.cells.len())
    }

    /// The source rows of the groups `start..end`, in source order: a subtotal's function gets
    /// its values in source order, as a group's does (`LAMBDA(x,ARRAYTOTEXT(x))`: East is
    /// "10, 30, 5", probes/groupby-subtotals G!A331, G!A1081).
    pub(crate) fn rows_of(&self, start: usize, end: usize) -> Vec<usize> {
        let mut rows: Vec<usize> = self
            .groups
            .get(start..end)
            .unwrap_or_default()
            .iter()
            .flat_map(|group| group.rows.iter().copied())
            .collect();
        rows.sort_unstable();
        rows
    }

    /// The key cells of the subtotal of the groups from `start` that share their first `level`
    /// keys: those keys, then `""` (G!J49; PIVOTBY's subtotal columns too, P!M97).
    pub(crate) fn subtotal_label_cells(
        &self,
        level: usize,
        start: usize,
        key_columns: usize,
    ) -> Vec<ArrayNode> {
        let keys = self
            .groups
            .get(start)
            .map_or(&[][..], |group| &group.keys[..]);
        (0..key_columns)
            .map(|field| match keys.get(field) {
                Some(key) if field < level => shown_key(key),
                _ => empty_text(),
            })
            .collect()
    }
}

/// The total depth one axis is laid out with: as given, except that a value sort over several
/// fields with the depth omitted shows subtotals at every level, as the full depth does
/// (two keys: probes/pivotby-followup P!A921 for GROUPBY, P!A201, P!A901 for PIVOTBY's rows,
/// P!A941 for its columns; three keys: probes/groupby-subtotals G!A121). An explicit depth is
/// kept (followup2 G!A261; P!A861, P!A1141), and so is an omitted one with `col_total_depth` 0,
/// although the rows then keep their key order (G!A1021).
pub(crate) fn effective_depth(depth: isize, given: bool, sort: &SortOrder, fields: usize) -> isize {
    if !given && fields > 1 && matches!(sort, SortOrder::Value { .. }) {
        isize::try_from(fields).unwrap_or(isize::MAX)
    } else {
        depth
    }
}

/// The number of subtotal levels of a total depth: none for 0 and ±1, the first |depth| - 1 key
/// fields otherwise.
pub(crate) fn subtotal_levels(depth: isize) -> usize {
    depth.unsigned_abs().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::{Grouping, Line, ParentOrder};
    use crate::expressions::{parser::ArrayNode, types::CellReferenceIndex};

    const CELL: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn text(s: &str) -> ArrayNode {
        ArrayNode::String(s.into())
    }

    /// The first key of each group and its rows, after sorting with `ascending`.
    fn sorted(keys: &[ArrayNode], ascending: bool) -> Vec<(ArrayNode, Vec<usize>)> {
        let keys: Vec<Vec<ArrayNode>> = keys.iter().map(|key| vec![key.clone()]).collect();
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        grouping.sort_by_keys(&[ascending], CELL);
        grouping
            .groups
            .into_iter()
            .map(|group| (group.keys[0].clone(), group.rows))
            .collect()
    }

    #[test]
    fn case_folded_first_spelling_and_blank_last_both_ways() {
        let keys = [
            text("West"),
            ArrayNode::Empty,
            text("east"),
            ArrayNode::Number(2.0),
            text("East"),
            ArrayNode::Boolean(true),
        ];
        let ascending = sorted(&keys, true);
        assert_eq!(
            ascending,
            [
                (ArrayNode::Number(2.0), vec![3]),
                (text("east"), vec![2, 4]),
                (text("West"), vec![0]),
                (ArrayNode::Boolean(true), vec![5]),
                (ArrayNode::Empty, vec![1]),
            ]
        );
        // Descending reverses every type but keeps the blank key last (G!A145).
        let descending = sorted(&keys, false);
        assert_eq!(
            descending
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>(),
            [
                ArrayNode::Boolean(true),
                text("West"),
                text("east"),
                ArrayNode::Number(2.0),
                ArrayNode::Empty,
            ]
        );
    }

    #[test]
    fn two_key_fields_sort_lexicographically() {
        let keys = vec![
            vec![text("b"), ArrayNode::Number(2.0)],
            vec![text("a"), ArrayNode::Number(3.0)],
            vec![text("b"), ArrayNode::Number(1.0)],
            vec![text("a"), ArrayNode::Number(3.0)],
        ];
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        grouping.sort_by_keys(&[true, false], CELL);
        let rows: Vec<Vec<usize>> = grouping.groups.into_iter().map(|g| g.rows).collect();
        assert_eq!(rows, [vec![1, 3], vec![0], vec![2]]);
    }

    #[test]
    fn value_sort_keeps_parents_with_equal_aggregates_together() {
        // Parents a and b both sum to 4: they keep key order and their children do not
        // interleave; b's children tie at 2 and keep key order (w before y).
        let keys = vec![
            vec![text("a"), text("x")],
            vec![text("b"), text("y")],
            vec![text("a"), text("z")],
            vec![text("b"), text("w")],
        ];
        let values = [1.0, 2.0, 3.0, 2.0];
        let mut grouping = Grouping::new(&keys, 0..keys.len(), |_| None)
            .unwrap_or(Grouping { groups: Vec::new() });
        let mut calls = 0;
        let sums = grouping.sort_by_aggregate(true, ParentOrder::ByValue, CELL, |rows| {
            calls += 1;
            ArrayNode::Number(rows.iter().map(|&row| values[row]).sum())
        });
        let leaves: Vec<_> = (0..4)
            .map(|i| sums.of_line(Line::Group(i)).cloned())
            .collect();
        let rows: Vec<Vec<usize>> = grouping.groups.iter().map(|g| g.rows.clone()).collect();
        assert_eq!(rows, [vec![0], vec![2], vec![3], vec![1]]);
        assert_eq!(
            leaves,
            [1.0, 3.0, 2.0, 2.0].map(|n| Some(ArrayNode::Number(n)))
        );
        // One call per group and one per parent, and the subtotal lines get the parents' sums
        // without another call.
        assert_eq!(calls, 6);
        let subtotals: Vec<_> = grouping
            .lines(1, false)
            .into_iter()
            .filter(|line| matches!(line, Line::Subtotal { .. }))
            .map(|line| sums.of_line(line).cloned())
            .collect();
        assert_eq!(
            subtotals,
            [Some(ArrayNode::Number(4.0)), Some(ArrayNode::Number(4.0))]
        );
        assert_eq!(calls, 6);
    }

    /// Three-key groups a/x/1, a/x/2, a/y/1, b/x/1, already in key order.
    fn three_keys() -> Grouping {
        let keys: Vec<Vec<ArrayNode>> = [
            ("a", "x", 1.0),
            ("a", "x", 2.0),
            ("a", "y", 1.0),
            ("b", "x", 1.0),
        ]
        .iter()
        .map(|(a, b, c)| vec![text(a), text(b), ArrayNode::Number(*c)])
        .collect();
        Grouping::new(&keys, 0..keys.len(), |_| None).unwrap_or(Grouping { groups: Vec::new() })
    }

    #[test]
    fn subtotal_lines_after_and_before_their_groups() {
        let grouping = three_keys();
        let subtotal = |level, start, end| Line::Subtotal { level, start, end };
        // Depth 2: the first level only (probes/groupby-subtotals G!A1).
        assert_eq!(
            grouping.lines(1, false),
            [
                Line::Group(0),
                Line::Group(1),
                Line::Group(2),
                subtotal(1, 0, 3),
                Line::Group(3),
                subtotal(1, 3, 4)
            ]
        );
        // Depth 3: the deepest subtotal first below the groups (G!A61) ...
        assert_eq!(
            grouping.lines(2, false),
            [
                Line::Group(0),
                Line::Group(1),
                subtotal(2, 0, 2),
                Line::Group(2),
                subtotal(2, 2, 3),
                subtotal(1, 0, 3),
                Line::Group(3),
                subtotal(2, 3, 4),
                subtotal(1, 3, 4),
            ]
        );
        // ... and the shallowest first above them (G!A91).
        assert_eq!(
            grouping.lines(2, true),
            [
                subtotal(1, 0, 3),
                subtotal(2, 0, 2),
                Line::Group(0),
                Line::Group(1),
                subtotal(2, 2, 3),
                Line::Group(2),
                subtotal(1, 3, 4),
                subtotal(2, 3, 4),
                Line::Group(3),
            ]
        );
        assert_eq!(
            grouping.lines(0, true),
            (0..4).map(Line::Group).collect::<Vec<_>>()
        );
        assert_eq!(grouping.rows_of(0, 3), [0, 1, 2]);
        assert_eq!(
            grouping.subtotal_label_cells(1, 3, 3),
            [text("b"), text(""), text("")]
        );
    }
}
