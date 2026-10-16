//! PIVOTBY's `relative_to`: the relative set each grid cell gives a two-argument function such as
//! PERCENTOF (SPEC-0006 step 6, T-1327). Excel: docs/excel-semantics/arrays.md "GROUPBY and
//! PIVOTBY", evidence probes/groupby-pivotby P!Y25-A73 and probes/groupby-pivotby-functions
//! P!A1-A841 (excel-graph, 2026-10-16), where `LAMBDA(x,y,SUM(y))` shows each cell's set.

use std::collections::{BTreeSet, HashMap};

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{parser::ArrayNode, token::Error, types::CellReferenceIndex},
    functions::util::array_node_to_calc_result,
    model::Model,
};

use super::grouping::{empty_text, value_error, Axis, Grouping, Line, Relative, SetId};

/// Which records of an axis a cell's relative set keeps: all, those of the cell's own row or
/// column item (a group, a subtotal or the Total), or those of that item's parent, its key prefix
/// one field shorter (for a single field, and for a Total, all).
#[derive(Clone, Copy, PartialEq)]
enum Select {
    All,
    Item,
    Parent,
}

/// The `relative_to` argument: omitted or 0 the column total, 1 the row total, 2 the grand total,
/// 3 the parent column total, 4 the parent row total. As (rows, columns) selections: 0 is all
/// rows of the cell's column, 1 the cell's row in all columns, 3 the cell's row under the parent
/// of its column, 4 the parent of its row in its column (P!Y25, P!A49, P!M49, P!Y49, P!A73; two
/// fields per axis: probes/groupby-pivotby-functions P!A101-A321; with subtotals P!A341-A441).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct RelativeTo {
    rows: Select,
    columns: Select,
}

impl RelativeTo {
    /// Omitted: the column total.
    pub(crate) fn column_total() -> Self {
        RelativeTo {
            rows: Select::All,
            columns: Select::Item,
        }
    }

    /// The relative set of a group's value in a value sort: the set of its Total cell (the
    /// Total column for rows, the Total row for columns), given its `rows`, the group one level
    /// up, `up` (`None` at the top level), and all `records`: its own rows, all records, or the
    /// group one level up, all records at the top. With `relative_to` 4 over two row fields the
    /// Region parents tie at the grand total and keep their key order (probes/groupby-pivotby-
    /// functions3 P!A91; 1: P!A121); with 3 over two column fields the years tie at their
    /// product's total (Apple 2023 before Apple 2024 descending, both 64), while 0 and 4 sort them
    /// by their own records (49 before 15), 1 and 2 tie at the grand total, with and without
    /// column subtotals and over one column field (probes/groupby-pivotby-functions4 P!A1-A201).
    pub(crate) fn of_sort<'a>(
        self,
        axis: Axis,
        rows: &'a [usize],
        up: Option<Relative<'a>>,
        records: &'a [usize],
    ) -> Relative<'a> {
        let all = Relative {
            id: Some(SetId::All),
            rows: records,
        };
        let select = match axis {
            Axis::Rows => self.rows,
            Axis::Columns => self.columns,
        };
        match select {
            Select::All => all,
            Select::Parent => up.unwrap_or(all),
            Select::Item => Relative { id: None, rows },
        }
    }
}

impl Model {
    /// One `relative_to` element: 0 to 4, text and booleans read as numbers (`"1"` and TRUE are 1:
    /// probes/groupby-pivotby-functions P!A521, P!A541); 5, -1 and 0.5 are `#VALUE!`, also when
    /// the function takes one argument and does not use it (P!A461-A501, P!A581).
    pub(crate) fn relative_to(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<RelativeTo, CalcResult> {
        use Select::{All, Item, Parent};
        let Some(result) = arg else {
            return Ok(RelativeTo::column_total());
        };
        let (rows, columns) = match self.cast_to_number(result, cell)? {
            0.0 => (All, Item),
            1.0 => (Item, All),
            2.0 => (All, All),
            3.0 => (Item, Parent),
            4.0 => (Parent, Item),
            _ => return Err(value_error(cell, "relative_to must be 0, 1, 2, 3 or 4")),
        };
        Ok(RelativeTo { rows, columns })
    }

    /// The evaluated `relative_to` argument: 0 to 4 ([`Model::relative_to`]), checked whatever
    /// the function (P!A581), or an array, whose elements are returned to be lifted
    /// ([`Model::lifted_relative_to`]; probes/pivotby-followup P!A681).
    pub(crate) fn relative_to_arg(
        &mut self,
        arg: Option<CalcResult>,
        cell: CellReferenceIndex,
    ) -> Result<(RelativeTo, Option<Elements>), CalcResult> {
        match arg {
            Some(array @ (CalcResult::Array(_) | CalcResult::Range { .. })) => Ok((
                RelativeTo::column_total(),
                Some(self.result_to_array(array, cell)?),
            )),
            other => Ok((self.relative_to(other, cell)?, None)),
        }
    }

    /// An array `relative_to`, lifted per element as Excel lifts it (`{0;1}` is a 2x1 array,
    /// probes/pivotby-followup P!A681, probes/groupby-pivotby-functions P!A721): each element is
    /// the top-left cell of the PIVOTBY for that element, which is the `""` corner of the header
    /// rows, or the element's error (`{5;1}` is `#VALUE!` over `""`: P!A741). The other arguments
    /// are checked first, an error among them is the whole result (unprobed), and no function is
    /// called.
    pub(crate) fn lifted_relative_to(
        &mut self,
        elements: Vec<Vec<ArrayNode>>,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        elements
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|element| {
                        let element = array_node_to_calc_result(&element, cell);
                        match self.relative_to(Some(element), cell) {
                            Ok(_) => empty_text(),
                            Err(error) => calc_result_to_array_node(error),
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

/// The elements of an array `relative_to`, to be lifted ([`Model::lifted_relative_to`]).
pub(crate) type Elements = Vec<Vec<ArrayNode>>;

/// A row or column item of the grid: a group or subtotal line, or the Total (`None`).
pub(crate) type Item = Option<Line>;

/// The records an item selects on one axis: a key-prefix level and the prefix's id.
type Class = (usize, usize);

/// One axis of the grid for the relative sets: its groups' key prefixes at each level the sets
/// use, and each record's group.
struct AxisSets {
    select: Select,
    fields: usize,
    group_of: Vec<usize>,
    /// `prefixes[level][g]`: the id of group g's first `level` keys.
    prefixes: HashMap<usize, Vec<usize>>,
}

impl AxisSets {
    /// The levels an axis' items select, counted before anything is built.
    fn levels(select: Select, fields: usize, items: &[Line]) -> BTreeSet<usize> {
        let mut levels = BTreeSet::new();
        levels.insert(level(select, fields, None));
        for &line in items {
            levels.insert(level(select, fields, Some(line)));
        }
        levels
    }

    fn new(select: Select, grouping: &Grouping, height: usize, levels: &BTreeSet<usize>) -> Self {
        AxisSets {
            select,
            fields: grouping.fields(),
            group_of: grouping.group_of(height),
            prefixes: levels
                .iter()
                .map(|&level| (level, grouping.prefix_ids(level)))
                .collect(),
        }
    }

    /// The level of the prefix an item selects: 0 for all records.
    fn level(&self, item: Item) -> usize {
        level(self.select, self.fields, item)
    }

    /// The class (level, prefix id) of the records an item selects.
    fn class(&self, item: Item) -> Class {
        let level = self.level(item);
        let group = match item {
            Some(Line::Group(group)) => group,
            Some(Line::Subtotal { start, .. }) => start,
            None => 0,
        };
        (level, self.prefix(level, group))
    }

    /// The id of group `group`'s prefix of `level` keys (0 for level 0).
    fn prefix(&self, level: usize, group: usize) -> usize {
        self.prefixes
            .get(&level)
            .and_then(|ids| ids.get(group))
            .copied()
            .unwrap_or(0)
    }
}

/// The level of the key prefix a selection keeps of an item over `fields` key fields: 0 for all
/// records, the item's own for `Item`, one field shorter for `Parent`.
fn level(select: Select, fields: usize, item: Item) -> usize {
    let level = match item {
        None => 0,
        Some(Line::Group(_)) => fields,
        Some(Line::Subtotal { level, .. }) => level,
    };
    match select {
        Select::All => 0,
        Select::Item => level,
        Select::Parent => level.saturating_sub(1),
    }
}

/// Most record entries the relative sets of one PIVOTBY call may hold (each record once per pair
/// of key-prefix levels the cells use): 2^24, 128 MiB of indices, so that many key fields with
/// subtotals at every level cannot fill memory (directive 5). Excel has no such limit:
/// DEVIATION, docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY: two-parameter LAMBDAs and the
/// cell-visit cap".
const MAX_SET_ENTRIES: u64 = 1 << 24;

/// The relative sets of one PIVOTBY call, each built once and shared by its cells: for each pair
/// of levels the cells use (a row level, a column level), the records bucketed by their row and
/// column prefixes at those levels, in source order.
pub(crate) struct RelativeSets {
    rows: AxisSets,
    cols: AxisSets,
    index: HashMap<(Class, Class), usize>,
    sets: Vec<Vec<usize>>,
}

impl RelativeSets {
    /// The sets of `records` (in source order) for the grid of `rows` and `cols` with the row and
    /// column lines `row_lines` and `col_lines`. Each level pair holds every record once; past
    /// [`MAX_SET_ENTRIES`] the whole result is `#ERROR!`, checked from the level counts before any
    /// set, prefix table or call is made (only many key fields with subtotals at many levels get
    /// there).
    pub(crate) fn new(
        relative_to: RelativeTo,
        (rows, cols): (&Grouping, &Grouping),
        (row_lines, col_lines): (&[Line], &[Line]),
        records: &[usize],
        cell: CellReferenceIndex,
    ) -> Result<Self, CalcResult> {
        let row_levels = AxisSets::levels(relative_to.rows, rows.fields(), row_lines);
        let col_levels = AxisSets::levels(relative_to.columns, cols.fields(), col_lines);
        let pairs = (row_levels.len() as u64).saturating_mul(col_levels.len() as u64);
        if pairs.saturating_mul(records.len() as u64) > MAX_SET_ENTRIES {
            return Err(CalcResult::new_error(
                Error::ERROR,
                cell,
                "Too many relative set entries".to_string(),
            ));
        }
        let height = records.iter().max().map_or(0, |&row| row + 1);
        let mut sets = RelativeSets {
            rows: AxisSets::new(relative_to.rows, rows, height, &row_levels),
            cols: AxisSets::new(relative_to.columns, cols, height, &col_levels),
            index: HashMap::new(),
            sets: Vec::new(),
        };
        for &row_level in &row_levels {
            for &col_level in &col_levels {
                for &record in records {
                    let row_group = sets.rows.group_of.get(record).copied().unwrap_or(0);
                    let col_group = sets.cols.group_of.get(record).copied().unwrap_or(0);
                    let key = (
                        (row_level, sets.rows.prefix(row_level, row_group)),
                        (col_level, sets.cols.prefix(col_level, col_group)),
                    );
                    let next = sets.sets.len();
                    let at = *sets.index.entry(key).or_insert(next);
                    if at == next {
                        sets.sets.push(Vec::new());
                    }
                    if let Some(set) = sets.sets.get_mut(at) {
                        set.push(record);
                    }
                }
            }
        }
        Ok(sets)
    }

    /// The relative set of the cell in row item `row` and column item `col`.
    pub(crate) fn of_cell(&self, row: Item, col: Item) -> Relative<'_> {
        let key = (self.rows.class(row), self.cols.class(col));
        let at = self.index.get(&key).copied();
        Relative {
            id: at.map(SetId::Pivot),
            rows: at
                .and_then(|at| self.sets.get(at))
                .map_or(&[][..], Vec::as_slice),
        }
    }
}
