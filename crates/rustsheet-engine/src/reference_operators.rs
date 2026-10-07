//! Evaluation of the reference operators: the range (`:`), the intersection (a space) and
//! the union (a comma inside parentheses).

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Most areas a union or an intersection may stand for. Unions nest in balanced trees
/// (`((A1,A2),(A3,A4))`) whose depth grows only with the logarithm of their leaves, so the
/// parser's depth cap does not bound them, and intersecting unions multiplies their areas.
/// Every area list the evaluator builds is checked against this cap before it grows, so a
/// formula cannot allocate without bound (CLAUDE.md directive 5; DEVIATION, decision 0014:
/// docs/excel-semantics/references.md "Union area cap").
pub(crate) const MAX_AREAS: usize = 1 << 16;

/// Most pairs of areas one top-level reference expression may compare. Each intersection compares at
/// most [`MAX_AREAS`] pairs, but a chain of cuts (`(U1 U2) A1 A1 A1 ...`) repeats that once
/// per three characters of formula, so the whole expression shares this budget.
pub(crate) const MAX_AREA_PAIRS: usize = 1 << 20;

/// Message of the `#VALUE!` for a union or intersection of more than [`MAX_AREAS`] areas.
pub(crate) const TOO_MANY_AREAS: &str = "Too many areas in a reference";

/// Message of the `#VALUE!` for an expression comparing more than [`MAX_AREA_PAIRS`] pairs.
pub(crate) const TOO_MANY_AREA_PAIRS: &str = "Too many areas to intersect";

/// `#VALUE!` for a reference of several areas where one area is needed.
fn several_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Reference of several areas".to_string())
}

fn too_many_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREAS.to_string())
}

/// Whether `node` may stand for several areas, so evaluating it needs an area list.
/// An intersection is multi-area only if a union sits somewhere under it: a chain of
/// plain references (`A1:C3 B1:D4 C1:C9`) stays on the allocation-free pair path.
/// Recursion is bounded by the parser's depth cap.
fn is_multi_area(node: &Node) -> bool {
    match node {
        Node::OpUnionKind { .. } => true,
        Node::OpIntersectKind { left, right, .. } => is_multi_area(left) || is_multi_area(right),
        _ => false,
    }
}

/// The cells `a` and `b` share: `None` if none, `#VALUE!` if they are on different sheets.
fn intersect_pair(
    a: &Range,
    b: &Range,
    cell: CellReferenceIndex,
) -> Result<Option<Range>, CalcResult> {
    if a.left.sheet != b.left.sheet {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Intersection of references on different sheets".to_string(),
        ));
    }
    let top = a.left.row.max(b.left.row);
    let bottom = a.right.row.min(b.right.row);
    let first = a.left.column.max(b.left.column);
    let last = a.right.column.min(b.right.column);
    if top > bottom || first > last {
        return Ok(None);
    }
    let sheet = a.left.sheet;
    Ok(Some(Range {
        left: CellReferenceIndex {
            sheet,
            row: top,
            column: first,
        },
        right: CellReferenceIndex {
            sheet,
            row: bottom,
            column: last,
        },
    }))
}

fn empty_intersection(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NULL, cell, "Empty intersection".to_string())
}

impl Model {
    /// `left:right`, the range operator between two operands that are not a plain `A1:B2`
    /// token (the lexer builds those): only two single cells on one sheet are joined, so
    /// `Data!A1:Other!B2` is `#VALUE!` (UNVERIFIED, references.md) and a `Range` across sheets
    /// is only ever a 3-D reference.
    pub(crate) fn get_range(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let left_result = self.evaluate_node_with_reference(left, cell);
        let right_result = self.evaluate_node_with_reference(right, cell);
        match (left_result, right_result) {
            (
                CalcResult::Range {
                    left: left1,
                    right: right1,
                },
                CalcResult::Range {
                    left: left2,
                    right: right2,
                },
            ) if left1 == right1 && left2 == right2 && left1.sheet == left2.sheet => {
                // Corners may be written in either order (`Data!B2:Data!A1` is `Data!A1:B2`).
                // UNVERIFIED against the oracle (references.md).
                CalcResult::Range {
                    left: CellReferenceIndex {
                        sheet: left1.sheet,
                        row: left1.row.min(right2.row),
                        column: left1.column.min(right2.column),
                    },
                    right: CellReferenceIndex {
                        sheet: right2.sheet,
                        row: left1.row.max(right2.row),
                        column: left1.column.max(right2.column),
                    },
                }
            }
            // An error operand's own error comes before the `#VALUE!`: `=(@(A1 B1)):C3` is
            // `#NULL!` (probes/at-operand-formulatext H12, T-1109). The right operand's error
            // is UNVERIFIED (references.md), taken from the union and intersection rule.
            (error @ CalcResult::Error { .. }, _) | (_, error @ CalcResult::Error { .. }) => error,
            _ => CalcResult::new_error(Error::VALUE, cell, "Invalid range".to_string()),
        }
    }

    /// A 3-D reference (`First:Last!A1`) where a value is needed: Excel gives `#REF!` for the
    /// bare reference, in an operator and in value functions such as ABS
    /// (probes/3d-ref-values C1:C3). A span of one sheet is that sheet's reference
    /// (`=Data1:Data1!A1`, C9).
    pub(crate) fn get_sheet_span_value(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if first_index == last_index {
            return self.evaluate_node_in_context(reference, cell);
        }
        CalcResult::new_error(
            Error::REF,
            cell,
            "3-D reference where a value is needed".to_string(),
        )
    }

    /// A 3-D reference where a single-sheet reference is needed: `#VALUE!` (INDEX, ROWS, ROW,
    /// AREAS, VLOOKUP, MATCH, SUMIF, COUNTIF, probes/3d-ref-functions). A span of one sheet is
    /// that sheet's reference.
    pub(crate) fn get_sheet_span_reference(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if first_index == last_index {
            return self.evaluate_node_with_reference(reference, cell);
        }
        CalcResult::new_error(
            Error::VALUE,
            cell,
            "3-D reference where one sheet is needed".to_string(),
        )
    }

    /// A 3-D reference as an aggregate argument: one `Range` whose corners sit on the first
    /// and the last sheet of the span in tab order, whichever way it was written
    /// (`Data3:Data1!A1` is `Data1:Data3!A1`, probes/3d-ref-aggregates C2). Only
    /// [`Model::evaluate_aggregate_arg`] builds a `Range` across sheets, and
    /// [`Model::for_each_cell_or_error_in_range`] walks it sheet by sheet.
    pub(crate) fn get_sheet_span_range(
        &mut self,
        first_index: u32,
        last_index: u32,
        reference: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.evaluate_node_with_reference(reference, cell) {
            CalcResult::Range { left, right } => CalcResult::Range {
                left: CellReferenceIndex {
                    sheet: first_index.min(last_index),
                    ..left
                },
                right: CellReferenceIndex {
                    sheet: first_index.max(last_index),
                    ..right
                },
            },
            other => other,
        }
    }

    /// `left right`, the intersection operator: the reference to the cells both operands
    /// contain. Excel (docs/excel-semantics/references.md, gen-m1/4dc8ed1de05a61be): an
    /// operand's error propagates, areas that share no cell give `#NULL!`, and areas on two
    /// different sheets give `#VALUE!` even where their coordinates overlap. An operand may be
    /// a union: a result of one area is that area (`(Data!A1:A12,Data!C1:C12) Data!A7` is
    /// Data!A7, gen-m1/c940d3cfb0d3b083), and one of several areas is `#VALUE!`, even when
    /// they are the same cell (`=A1:A3 (A2:B2,A2)`,
    /// probes/intersection-function-operands H5).
    pub(crate) fn get_intersection(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        match self.intersect_to_one_area(left, right, cell) {
            Ok(area) => CalcResult::Range {
                left: area.left,
                right: area.right,
            },
            Err(error) => error,
        }
    }

    /// The one area `left right` stands for, or the result to give instead.
    fn intersect_to_one_area(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        if !is_multi_area(left) && !is_multi_area(right) {
            // Two single areas, the common case: no area list.
            let a = self.evaluate_intersection_operand(left, cell)?;
            let b = self.evaluate_intersection_operand(right, cell)?;
            return intersect_pair(&a, &b, cell)?.ok_or_else(|| empty_intersection(cell));
        }
        let mut budget = MAX_AREA_PAIRS;
        let mut areas = self.intersect_areas(left, right, cell, &mut budget)?;
        match areas.pop() {
            Some(area) if areas.is_empty() => Ok(area),
            _ => Err(several_areas(cell)),
        }
    }

    /// `(left,right)`, the union operator, where a value is needed: `#VALUE!`, even for
    /// `(E4,E4)` (gen-m1/c940d3cfb0d3b083). The engine has no multi-area value yet (T-0133),
    /// so this is also what a function argument gets. An operand's own error comes first
    /// (UNVERIFIED, references.md). The operands are checked in the order
    /// [`Model::union_areas`] evaluates them, with the same errors, but only their area counts
    /// are kept.
    pub(crate) fn get_union(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let mut budget = MAX_AREA_PAIRS;
        match self.count_union_areas(left, right, cell, &mut budget) {
            Ok(_) => several_areas(cell),
            Err(error) => error,
        }
    }

    /// Whether `(left,right)` evaluates to a reference: a union with an error operand (`(A1,#REF!)`)
    /// is that error, not a reference (probes/union-accepted H9, docs/excel-semantics/references.md).
    pub(crate) fn union_is_reference(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
    ) -> bool {
        let mut budget = MAX_AREA_PAIRS;
        self.count_union_areas(left, right, cell, &mut budget)
            .is_ok()
    }

    /// The areas of a reference argument: several for a union, one for any other reference.
    /// `Err` holds the error the argument evaluates to, or `#VALUE!` if it is not a reference
    /// (AGGREGATE's reference form; probes/aggregate-args).
    pub(crate) fn reference_areas(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Range>, CalcResult> {
        let mut budget = MAX_AREA_PAIRS;
        self.evaluate_areas(node, cell, &mut budget)
    }

    /// The number of areas `(left,right)` stands for, or the result to give instead, exactly
    /// as [`Model::union_areas`] finds them but without building the union's area list.
    fn count_union_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
        budget: &mut usize,
    ) -> Result<usize, CalcResult> {
        let mut count = 0;
        for node in [left, right] {
            count += match node {
                Node::OpUnionKind { left, right, .. } => {
                    self.count_union_areas(left, right, cell, budget)?
                }
                _ => self.evaluate_areas(node, cell, budget)?.len(),
            };
            if count > MAX_AREAS {
                return Err(too_many_areas(cell));
            }
        }
        Ok(count)
    }

    /// The areas a reference expression stands for: several for a union, and for an
    /// intersection with one; one for any other reference. Never more than [`MAX_AREAS`].
    /// `Err` holds the result to give instead (see [`Model::evaluate_intersection_operand`]).
    fn evaluate_areas(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
        budget: &mut usize,
    ) -> Result<Vec<Range>, CalcResult> {
        match node {
            Node::OpUnionKind { left, right, .. } => self.union_areas(left, right, cell, budget),
            Node::OpIntersectKind { left, right, .. } => {
                self.intersect_areas(left, right, cell, budget)
            }
            _ => Ok(vec![self.evaluate_intersection_operand(node, cell)?]),
        }
    }

    /// The areas of `left` followed by those of `right`. More than [`MAX_AREAS`] is `#VALUE!`,
    /// checked before the list grows (DEVIATION, decision 0014: references.md "Union area
    /// cap").
    fn union_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
        budget: &mut usize,
    ) -> Result<Vec<Range>, CalcResult> {
        let mut areas = self.evaluate_areas(left, cell, budget)?;
        let right = self.evaluate_areas(right, cell, budget)?;
        if areas.len() + right.len() > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        areas.extend(right);
        Ok(areas)
    }

    /// The pairwise intersections of the areas of `left` and `right` that hold a cell. A pair
    /// on different sheets is `#VALUE!`, and no shared cell at all is `#NULL!`. Unions of
    /// unions multiply the areas, so more than [`MAX_AREAS`] pairs is `#VALUE!`, and so is
    /// going over what is left of the expression's `budget` of [`MAX_AREA_PAIRS`] (DEVIATION,
    /// decision 0014: references.md "Union area cap").
    fn intersect_areas(
        &mut self,
        left: &Node,
        right: &Node,
        cell: CellReferenceIndex,
        budget: &mut usize,
    ) -> Result<Vec<Range>, CalcResult> {
        let left = self.evaluate_areas(left, cell, budget)?;
        let right = self.evaluate_areas(right, cell, budget)?;
        let pairs = left.len().saturating_mul(right.len());
        if pairs > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        *budget = budget.checked_sub(pairs).ok_or_else(|| {
            CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREA_PAIRS.to_string())
        })?;
        let mut areas = Vec::new();
        for a in &left {
            for b in &right {
                areas.extend(intersect_pair(a, b, cell)?);
            }
        }
        if areas.is_empty() {
            return Err(empty_intersection(cell));
        }
        Ok(areas)
    }

    /// An operand of the intersection as a single-sheet area with its corners in order, or
    /// the result to give instead: the operand's own error, or `#VALUE!` for anything that
    /// is not a reference (`(1+0) A1`) or spans sheets.
    fn evaluate_intersection_operand(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Range, CalcResult> {
        match self.evaluate_node_with_reference(node, cell) {
            CalcResult::Range { left, right } if left.sheet == right.sheet => Ok(Range {
                left: CellReferenceIndex {
                    sheet: left.sheet,
                    row: left.row.min(right.row),
                    column: left.column.min(right.column),
                },
                right: CellReferenceIndex {
                    sheet: left.sheet,
                    row: left.row.max(right.row),
                    column: left.column.max(right.column),
                },
            }),
            error @ CalcResult::Error { .. } => Err(error),
            _ => Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Intersection of a value that is not a reference".to_string(),
            )),
        }
    }
}
