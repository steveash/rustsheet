//! A cell of a GROUPBY or PIVOTBY result that holds an array: Excel keeps the array a group's
//! function returns in that cell, shows it as `#VALUE!`, and INDEX hands it back (SPEC-0018;
//! docs/excel-semantics/arrays.md "A LAMBDA array result is shown as `#VALUE!`").

use crate::calc_result::CalcResult;
use crate::expressions::parser::ArrayNode;
use crate::expressions::token::Error;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::util::array_element_value;
use crate::limits::{MAX_NESTED_DEPTH, MAX_NESTED_KEPT_CELLS, MAX_NESTED_WALK_CELLS, MAX_SIZE};

/// What a nested cell holds (SPEC-0018 "Value").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NestedKind {
    /// A function's array result, read by INDEX as that array (rules 1, 3, 8).
    Array,
    /// A one-cell reference result, read by INDEX as the referenced value (rule 7: `TYPE` 1).
    Reference,
}

/// The array a nested cell holds: non-empty and rectangular, its elements possibly nested again
/// (rule 8). `cells` and `depth` are computed once by [`NestedArray::new`], which refuses a tree
/// past [`MAX_NESTED_DEPTH`] or [`MAX_SIZE`], so `Drop` and equality, which recurse, stay within
/// the depth cap (CLAUDE.md directive 5).
#[derive(Debug)]
pub struct NestedArray {
    rows: Vec<Vec<ArrayNode>>,
    kind: NestedKind,
    cells: usize,
    depth: usize,
}

/// The error of an array argument holding a nested cell of kind [`NestedKind::Array`] to the
/// consumers that read their data whole rather than through the aggregate walker: `#N/A` for
/// MODE.SNGL, LARGE, SMALL and PERCENTILE.INC over a GROUPBY result (probes/t2594-nested-aggregates
/// A15, A19:A21; MODE.MULT, QUARTILE.INC and PERCENTILE.EXC: t2625-nested-consumers A10:A12;
/// QUARTILE, PERCENTRANK and PERCENTRANK.EXC: gen-m9/46ac3ef5e61583ca A325, 2138b909d8f810ea A226,
/// 928d08e3e1eecb5f A374; T-2625).
pub(crate) fn nested_array_error(
    array: &[Vec<ArrayNode>],
    cell: CellReferenceIndex,
) -> Option<CalcResult> {
    array
        .iter()
        .flatten()
        .any(|node| matches!(node, ArrayNode::Nested(nested) if nested.kind() == NestedKind::Array))
        .then(|| CalcResult::new_error(Error::NA, cell, "Nested array".to_string()))
}

/// What one evaluation has spent of its nested-array budgets (SPEC-0018 "Bound"): the cells its
/// GROUPBY and PIVOTBY calls kept in nested cells' own grids (`NestedArray::grid_cells`: an inner
/// level was charged when its own call kept it), against [`MAX_NESTED_KEPT_CELLS`], and the
/// cells of nested arrays it read the contents of, against [`MAX_NESTED_WALK_CELLS`]. Reset with
/// the LAMBDA captures, per root cell (`Model::evaluate_root_cell`).
#[derive(Default, Debug)]
pub(crate) struct NestedBudget {
    kept: usize,
    walked: usize,
}

impl NestedBudget {
    /// Charges `cells` newly kept cells; `false`, charging nothing, past the budget.
    pub(crate) fn keep(&mut self, cells: usize) -> bool {
        let kept = self.kept.saturating_add(cells);
        if kept > MAX_NESTED_KEPT_CELLS {
            return false;
        }
        self.kept = kept;
        true
    }

    /// Charges `cells` read cells; `false`, charging nothing, past the budget.
    fn walk(&mut self, cells: usize) -> bool {
        let walked = self.walked.saturating_add(cells);
        if walked > MAX_NESTED_WALK_CELLS {
            return false;
        }
        self.walked = walked;
        true
    }
}

/// `#VALUE!`, what a lifted read of a `Reference` cell sees (unprobed, T-2609).
static VALUE_ERROR: ArrayNode = ArrayNode::Error(Error::VALUE);

impl NestedArray {
    /// The nested array of `rows`, or `None` for an empty or ragged grid, a `Reference` that is
    /// not one scalar, and a tree deeper than [`MAX_NESTED_DEPTH`] or with more than
    /// [`MAX_SIZE`] cells, inner levels included (the caller gives `#ERROR!`; DEVIATION,
    /// decision 0014).
    pub(crate) fn new(rows: Vec<Vec<ArrayNode>>, kind: NestedKind) -> Option<Self> {
        let width = rows.first().map_or(0, Vec::len);
        if width == 0 || rows.iter().any(|row| row.len() != width) {
            return None;
        }
        let mut cells = rows.len().saturating_mul(width);
        let mut depth = 1;
        for node in rows.iter().flatten() {
            if let ArrayNode::Nested(inner) = node {
                cells = cells.saturating_add(inner.cells);
                depth = depth.max(inner.depth.saturating_add(1));
            }
        }
        if kind == NestedKind::Reference && (cells != 1 || depth != 1) {
            return None;
        }
        if depth > MAX_NESTED_DEPTH || cells > MAX_SIZE {
            return None;
        }
        Some(NestedArray {
            rows,
            kind,
            cells,
            depth,
        })
    }

    /// What it holds: an array, or a reference result's value.
    pub(crate) fn kind(&self) -> NestedKind {
        self.kind
    }

    /// Its cells plus those of every nested element, what reading its contents costs.
    pub(crate) fn cells(&self) -> usize {
        self.cells
    }

    /// Its own grid's cells, the memory it adds: a nested element is shared, not copied.
    pub(crate) fn grid_cells(&self) -> usize {
        self.rows
            .len()
            .saturating_mul(self.rows.first().map_or(0, Vec::len))
    }

    /// What INDEX selecting this cell returns (rules 3, 7): a `Reference` cell's value, or a copy
    /// of the array, its `cells` charged first to the evaluation's walk budget (`#ERROR!` past
    /// [`MAX_NESTED_WALK_CELLS`]; DEVIATION, decision 0014). Copies of one nested cell share it,
    /// so the charge per read is what bounds reading many copies.
    pub(crate) fn value(&self, budget: &mut NestedBudget, cell: CellReferenceIndex) -> CalcResult {
        if self.kind == NestedKind::Reference {
            return match self.rows.first().and_then(|row| row.first()) {
                Some(node) => array_element_value(node, cell),
                None => CalcResult::new_error(Error::VALUE, cell, String::new()),
            };
        }
        if !budget.walk(self.cells) {
            return CalcResult::new_error(
                Error::ERROR,
                cell,
                format!("reading nested arrays exceeds {MAX_NESTED_WALK_CELLS} cells"),
            );
        }
        CalcResult::Array(self.rows.clone())
    }
}

impl PartialEq for NestedArray {
    /// By content, kind included. Copies of one nested cell share it, so they compare in O(1);
    /// two distinct ones differing in size compare in O(1) too.
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
            || (self.kind == other.kind
                && self.cells == other.cells
                && self.depth == other.depth
                && self.rows == other.rows)
    }
}

impl ArrayNode {
    /// The value an element-wise lift reads of this element: a `Nested` cell's top-left,
    /// taken again while that is nested (assumed for two levels, T-2609), `#VALUE!` for a
    /// `Reference` cell (unprobed, T-2609); any other element itself (SPEC-0018 "One reader",
    /// rule 4: `SUM(--ISERROR(GROUPBY(..)))` counts only top-left errors, probes/t2360 P!C33).
    pub(crate) fn scalar(&self) -> &ArrayNode {
        let mut node = self;
        // Each step goes one level down, so this ends within `MAX_NESTED_DEPTH` steps.
        while let ArrayNode::Nested(nested) = node {
            if nested.kind == NestedKind::Reference {
                return &VALUE_ERROR;
            }
            match nested.rows.first().and_then(|row| row.first()) {
                Some(first) => node = first,
                None => return &VALUE_ERROR,
            }
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    const AT: CellReferenceIndex = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };

    fn number(n: f64) -> ArrayNode {
        ArrayNode::Number(n)
    }

    fn nested(rows: Vec<Vec<ArrayNode>>) -> ArrayNode {
        ArrayNode::Nested(Arc::new(NestedArray::new(rows, NestedKind::Array).unwrap()))
    }

    #[test]
    fn new_refuses_empty_ragged_and_bad_references() {
        assert!(NestedArray::new(Vec::new(), NestedKind::Array).is_none());
        assert!(NestedArray::new(vec![Vec::new()], NestedKind::Array).is_none());
        let ragged = vec![vec![number(1.0), number(2.0)], vec![number(3.0)]];
        assert!(NestedArray::new(ragged, NestedKind::Array).is_none());
        assert!(NestedArray::new(vec![vec![number(10.0)]], NestedKind::Reference).is_some());
        let two = vec![vec![number(1.0), number(2.0)]];
        assert!(NestedArray::new(two, NestedKind::Reference).is_none());
        let inner = vec![vec![nested(vec![vec![number(1.0)]])]];
        assert!(NestedArray::new(inner, NestedKind::Reference).is_none());
    }

    #[test]
    fn cells_and_depth_count_every_level() {
        let inner = nested(vec![vec![number(1.0)], vec![number(2.0)]]);
        let outer = NestedArray::new(
            vec![vec![ArrayNode::String("a".into()), inner.clone(), inner]],
            NestedKind::Array,
        )
        .unwrap();
        // Its 3 cells and the 2 of each nested element (SPEC-0018 rule 8: two levels).
        assert_eq!(outer.cells(), 7);
        assert_eq!(outer.grid_cells(), 3);
        assert_eq!(outer.depth, 2);
    }

    #[test]
    fn depth_cap() {
        let mut node = number(1.0);
        for _ in 0..MAX_NESTED_DEPTH {
            node = nested(vec![vec![node]]);
        }
        assert!(NestedArray::new(vec![vec![node]], NestedKind::Array).is_none());
    }

    #[test]
    fn cell_cap_counts_inner_levels() {
        let inner = nested(vec![vec![number(0.0); MAX_SIZE]]);
        let ArrayNode::Nested(kept) = &inner else {
            unreachable!()
        };
        assert_eq!(kept.cells(), MAX_SIZE);
        // One more cell around it passes the cap, though the outer grid is 1x1.
        assert!(NestedArray::new(vec![vec![inner]], NestedKind::Array).is_none());
    }

    #[test]
    fn scalar_reads_the_top_left_through_two_levels() {
        let inner = nested(vec![vec![ArrayNode::Error(Error::NA)], vec![number(3.0)]]);
        let outer = nested(vec![vec![inner], vec![number(4.0)]]);
        assert_eq!(outer.scalar(), &ArrayNode::Error(Error::NA));
        assert_eq!(number(5.0).scalar(), &number(5.0));
        let reference = ArrayNode::Nested(Arc::new(
            NestedArray::new(vec![vec![number(10.0)]], NestedKind::Reference).unwrap(),
        ));
        assert_eq!(reference.scalar(), &ArrayNode::Error(Error::VALUE));
    }

    #[test]
    fn value_is_the_array_or_the_referenced_value() {
        let mut budget = NestedBudget::default();
        let array = NestedArray::new(
            vec![vec![number(1.0)], vec![number(2.0)]],
            NestedKind::Array,
        )
        .unwrap();
        let CalcResult::Array(rows) = array.value(&mut budget, AT) else {
            panic!("not an array")
        };
        assert_eq!(rows, vec![vec![number(1.0)], vec![number(2.0)]]);
        assert_eq!(budget.walked, 2);
        let reference = NestedArray::new(vec![vec![number(10.0)]], NestedKind::Reference).unwrap();
        assert!(reference.value(&mut budget, AT) == CalcResult::Number(10.0));
        assert_eq!(budget.walked, 2);
    }

    #[test]
    fn value_past_the_walk_budget_is_an_error() {
        let mut budget = NestedBudget {
            kept: 0,
            walked: MAX_NESTED_WALK_CELLS - 1,
        };
        let array =
            NestedArray::new(vec![vec![number(1.0), number(2.0)]], NestedKind::Array).unwrap();
        assert!(matches!(
            array.value(&mut budget, AT),
            CalcResult::Error {
                error: Error::ERROR,
                ..
            }
        ));
        assert_eq!(budget.walked, MAX_NESTED_WALK_CELLS - 1);
    }

    #[test]
    fn keep_budget() {
        let mut budget = NestedBudget::default();
        assert!(budget.keep(MAX_NESTED_KEPT_CELLS));
        assert!(!budget.keep(1));
        assert_eq!(budget.kept, MAX_NESTED_KEPT_CELLS);
    }

    #[test]
    fn equality_is_by_content_and_kind() {
        let a = nested(vec![vec![number(1.0)], vec![number(2.0)]]);
        let b = nested(vec![vec![number(1.0)], vec![number(2.0)]]);
        let c = nested(vec![vec![number(1.0)], vec![number(3.0)]]);
        assert_eq!(a, a.clone());
        assert_eq!(a, b);
        assert_ne!(a, c);
        let array = nested(vec![vec![number(10.0)]]);
        let reference = ArrayNode::Nested(Arc::new(
            NestedArray::new(vec![vec![number(10.0)]], NestedKind::Reference).unwrap(),
        ));
        assert_ne!(array, reference);
    }
}
