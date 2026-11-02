//! Area and reference-argument types of the reference operators, and their error helpers.

use std::{ops::Deref, sync::Arc};

use crate::{
    calc_result::{CalcResult, Range},
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
};

/// Most areas a union or an intersection may stand for. Unions nest in balanced trees
/// (`((A1,A2),(A3,A4))`) whose depth grows only with the logarithm of their leaves, so the
/// parser's depth cap does not bound them, and intersecting unions multiplies their areas.
/// Every area list the evaluator builds is checked against this cap before it grows, so a
/// formula cannot allocate without bound (CLAUDE.md directive 5; DEVIATION, decision 0014:
/// docs/excel-semantics/references.md "Union area cap").
pub(crate) const MAX_AREAS: usize = 1 << 16;

/// Message of the `#VALUE!` for a union or intersection of more than [`MAX_AREAS`] areas.
pub(crate) const TOO_MANY_AREAS: &str = "Too many areas in a reference";

/// The areas [`Model::evaluate_areas`] finds: one area, held without an allocation (the
/// common `LET(r,A1:C3,r B2)`), a list built for this expression, or a list shared with the
/// binding or name it came from, read without a copy.
pub(super) enum AreaList {
    One(Range),
    Owned(Vec<Range>),
    Shared(Arc<[Range]>),
}

impl Deref for AreaList {
    type Target = [Range];

    fn deref(&self) -> &[Range] {
        match self {
            AreaList::One(range) => std::slice::from_ref(range),
            AreaList::Owned(areas) => areas,
            AreaList::Shared(areas) => areas,
        }
    }
}

impl AreaList {
    pub(super) fn into_vec(self) -> Vec<Range> {
        match self {
            AreaList::One(range) => vec![range],
            AreaList::Owned(areas) => areas,
            AreaList::Shared(areas) => areas.to_vec(),
        }
    }
}

/// The areas of a reference that has more than one, in formula order, duplicates kept
/// (`SUM((A1:B2,B2))` counts B2 twice, probes/multiarea-agg H3): 2..=[`MAX_AREAS`] ranges, all
/// on one sheet, each with its corners in order. Shared (`Arc`, so a `Model` holding one in a LET
/// binding stays `Send`), so a copy is never made on read.
#[derive(Clone)]
pub(crate) struct Areas(pub(super) Arc<[Range]>);

impl Areas {
    /// The first area in formula order (an `Areas` holds at least two).
    pub(crate) fn first(&self) -> Option<Range> {
        self.0.first().cloned()
    }

    /// How many areas the reference has: 2..=[`MAX_AREAS`].
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

/// The ranges a reference argument stands for: one range (a 3-D reference included, whose
/// corners sit on two sheets), or the areas of a multi-area reference. Derefs to the ranges in
/// formula order; a single range is borrowed, never copied.
#[derive(Clone)]
pub(crate) enum Ranges {
    One(Range),
    Areas(Areas),
}

impl Ranges {
    /// The ranges of an area list built by [`Model::evaluate_areas`]: one area is a plain
    /// range; several must sit on one sheet, as a reference across sheets is `#VALUE!` where a
    /// function takes a union (`SUM((Data!A1,Other!A1))`, probes/multiarea-xsheet).
    pub(super) fn from_list(
        areas: AreaList,
        cell: CellReferenceIndex,
    ) -> Result<Ranges, CalcResult> {
        let mut areas = match areas {
            AreaList::One(area) => return Ok(Ranges::One(area)),
            // The areas of an `Areas`, which hold to its rules already.
            AreaList::Shared(areas) => return Ok(Ranges::Areas(Areas(areas))),
            AreaList::Owned(areas) => areas,
        };
        if areas.len() > MAX_AREAS {
            return Err(too_many_areas(cell));
        }
        let Some(first) = areas.first() else {
            return Err(empty_intersection(cell));
        };
        let sheet = first.left.sheet;
        if areas
            .iter()
            .any(|area| area.left.sheet != sheet || area.right.sheet != sheet)
        {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Reference of areas on several sheets".to_string(),
            ));
        }
        if areas.len() == 1 {
            if let Some(area) = areas.pop() {
                return Ok(Ranges::One(area));
            }
        }
        Ok(Ranges::Areas(Areas(Arc::from(areas))))
    }
}

impl Deref for Ranges {
    type Target = [Range];

    fn deref(&self) -> &[Range] {
        match self {
            Ranges::One(range) => std::slice::from_ref(range),
            Ranges::Areas(Areas(areas)) => areas,
        }
    }
}

/// What a reference-accepting argument evaluates to ([`Model::evaluate_reference_arg`]). Kept
/// apart from [`CalcResult`] so that a reference of several areas reaches only the code that
/// asks for one; everywhere a value is needed it stays `#VALUE!` (SPEC-0004).
pub(crate) enum RefArg {
    /// A reference: one range or several areas.
    Ranges(Ranges),
    /// A lifted call whose elements are references (SPEC-0014).
    Grid(RefGrid),
    /// Anything else: a value, an array or an error. Never a `CalcResult::Range`.
    Value(CalcResult),
}

/// What an IF or CHOOSE stands for once its condition or index is read: the argument it picks,
/// or the value it gives instead ([`Model::if_pick`], [`Model::choose_pick`],
/// [`Model::pick_branch`]).
pub(crate) enum Picked<'a> {
    /// The argument picked, still unevaluated, to read in place of the IF or CHOOSE.
    Node(&'a Node),
    /// What it gives when it picks no one argument: an argument-count error, the condition's or
    /// index's error, IF's FALSE for a missing branch, or its result lifted over an array.
    Value(CalcResult),
}

/// What an array stands for where a function takes a reference (SUBTOTAL, AREAS, COUNTBLANK):
/// `#VALUE!` per element, an element that is an error kept (`SUBTOTAL(9,IF(TRUE,{1,#N/A}))` is
/// `#VALUE!`, `#N/A`; probes/subtotal-reference-operands, T-1163; probes/countblank-array-values).
pub(crate) fn not_a_reference_array(rows: Vec<Vec<ArrayNode>>) -> CalcResult {
    CalcResult::Array(
        rows.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|value| match value {
                        ArrayNode::Error(error) => ArrayNode::Error(error),
                        _ => ArrayNode::Error(Error::VALUE),
                    })
                    .collect()
            })
            .collect(),
    )
}

impl From<CalcResult> for RefArg {
    fn from(result: CalcResult) -> RefArg {
        match result {
            CalcResult::Range { left, right } => RefArg::Ranges(Ranges::One(Range { left, right })),
            other => RefArg::Value(other),
        }
    }
}

impl RefArg {
    /// The areas of a reference that sits on one sheet: borrowed, never copied. `None` for a
    /// 3-D reference (corners on two sheets) and for anything that is not a reference, where
    /// AREAS, INDEX and ISREF keep their `#VALUE!` (probes/3d-ref-functions).
    pub(crate) fn single_sheet_areas(&self) -> Option<&[Range]> {
        match self {
            RefArg::Ranges(ranges)
                if ranges
                    .iter()
                    .all(|area| area.left.sheet == area.right.sheet) =>
            {
                Some(ranges)
            }
            _ => None,
        }
    }

    /// [`RefArg::into_single_area`] where the area must sit on one sheet: a 3-D range (a span
    /// bound by LET, from IF or CHOOSE) is `#VALUE!`, as the written span is there
    /// (`ROWS(x)`, `COUNTBLANK(x)` for `x` = `Data1:Data3!A1`: probes/3d-ref-let-bound-span C7,
    /// 3d-ref-let-bound-span-readers C11, 3d-ref-let-values C5; T-1158). Not every caller of
    /// `into_single_area` may refuse one: making that refuse 3-D ranges regressed probe cells.
    pub(crate) fn into_single_sheet_area(self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            RefArg::Ranges(Ranges::One(range)) if range.left.sheet != range.right.sheet => {
                three_d_needs_one_sheet(cell)
            }
            other => other.into_single_area(cell),
        }
    }

    /// The argument's reference, or its value: an array of references as its value view
    /// ([`RefGrid::to_values`]), for a consumer that reads no element of one yet (SPEC-0014
    /// Rollout steps 4-5, T-2063 and T-2064).
    pub(crate) fn into_ranges_or_value(self) -> Result<Ranges, CalcResult> {
        match self {
            RefArg::Ranges(ranges) => Ok(ranges),
            RefArg::Grid(grid) => Err(grid.to_values()),
            RefArg::Value(value) => Err(value),
        }
    }

    /// The argument where only one area is taken: a reference of several areas is `#VALUE!`,
    /// as in a value context, and one range is a `CalcResult::Range`.
    pub(crate) fn into_single_area(self, cell: CellReferenceIndex) -> CalcResult {
        match self {
            RefArg::Ranges(Ranges::One(Range { left, right })) => CalcResult::Range { left, right },
            RefArg::Ranges(Ranges::Areas(_)) => several_areas(cell),
            RefArg::Grid(grid) => grid.to_values(),
            RefArg::Value(value) => value,
        }
    }
}

/// One element of an array of references: the reference a scalar call returned, or its error.
#[derive(Clone)]
pub(crate) enum RefElement {
    Reference(Range),
    Error(Error),
}

/// A lifted call whose elements are references (SPEC-0014 "Representation"): `rows` x
/// `columns` elements in row-major order, at least one, within `check_array_size`; each
/// reference on one sheet with its corners in order. Shared, so a pass-through never copies it.
/// It is not a [`CalcResult`], so only code that asks for a [`RefArg`] ever sees it; everywhere
/// else it is its value view, [`RefGrid::to_values`].
#[derive(Clone)]
pub(crate) struct RefGrid {
    rows: usize,
    columns: usize,
    elements: Arc<[RefElement]>,
}

impl RefGrid {
    /// The grid of `rows` x `columns` `elements`, row-major; `None` unless the counts agree and
    /// the grid is not empty.
    pub(crate) fn new(rows: usize, columns: usize, elements: Vec<RefElement>) -> Option<RefGrid> {
        (rows > 0 && columns > 0 && rows.checked_mul(columns) == Some(elements.len())).then(|| {
            RefGrid {
                rows,
                columns,
                elements: Arc::from(elements),
            }
        })
    }

    /// Rows and columns of elements.
    pub(crate) fn dims(&self) -> (usize, usize) {
        (self.rows, self.columns)
    }

    /// The element at 0-based (`row`, `column`), `None` past the grid.
    pub(crate) fn get(&self, row: usize, column: usize) -> Option<&RefElement> {
        if row >= self.rows || column >= self.columns {
            return None;
        }
        self.elements.get(row * self.columns + column)
    }

    /// The elements in row-major order.
    pub(crate) fn elements(&self) -> &[RefElement] {
        &self.elements
    }

    /// Whether `other` is a clone of this grid, holding the same elements in memory.
    pub(crate) fn shares_elements(&self, other: &RefGrid) -> bool {
        Arc::ptr_eq(&self.elements, &other.elements)
    }

    /// The value view (SPEC-0014 rule 2): every reference element is `#VALUE!` and an error
    /// element is itself, so `=OFFSET(A1,{0;1},0)` spills two `#VALUE!` (probes/
    /// reference-lift-elements K1, K4, K13).
    pub(crate) fn to_values(&self) -> CalcResult {
        CalcResult::Array(self.to_array(ArrayNode::Error(Error::VALUE)))
    }

    /// The grid as array rows: every reference element is `reference`, an error element is
    /// itself.
    pub(crate) fn to_array(&self, reference: ArrayNode) -> Vec<Vec<ArrayNode>> {
        self.elements
            .chunks(self.columns)
            .map(|row| {
                row.iter()
                    .map(|element| match element {
                        RefElement::Reference(_) => reference.clone(),
                        RefElement::Error(error) => ArrayNode::Error(error.clone()),
                    })
                    .collect()
            })
            .collect()
    }
}

/// `#REF!`, what a 3-D reference gives where a value is needed (probes/3d-ref-values C1:C3).
pub(crate) fn three_d_value_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::REF,
        cell,
        "3-D reference where a value is needed".to_string(),
    )
}

/// `#VALUE!` for a 3-D reference where a single-sheet reference is needed (INDEX, AREAS,
/// VLOOKUP, MATCH, ..., probes/3d-ref-functions).
pub(crate) fn three_d_needs_one_sheet(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::VALUE,
        cell,
        "3-D reference where one sheet is needed".to_string(),
    )
}

/// The smallest range holding every area of a union (all on one sheet, corners in order).
pub(super) fn bounding_area(areas: &[Range]) -> CalcResult {
    let mut bounds = areas.iter();
    // Callers pass at least one area (a union has two or more); the empty arm is never taken.
    let Some(first) = bounds.next() else {
        return CalcResult::EmptyCell;
    };
    let (mut left, mut right) = (first.left, first.right);
    for area in bounds {
        left.row = left.row.min(area.left.row);
        left.column = left.column.min(area.left.column);
        right.row = right.row.max(area.right.row);
        right.column = right.column.max(area.right.column);
    }
    CalcResult::Range { left, right }
}

/// `#VALUE!` for a reference of several areas where one area is needed.
pub(crate) fn several_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "Reference of several areas".to_string())
}

/// `#REF!` for a reference of several areas in the contexts where Excel gives `#REF!` rather than
/// the `#VALUE!` of [`several_areas`] (ROW/COLUMN/ROWS/COLUMNS, a VLOOKUP/HLOOKUP index).
pub(crate) fn several_areas_ref(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::REF, cell, "Reference of several areas".to_string())
}

pub(crate) fn too_many_areas(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, TOO_MANY_AREAS.to_string())
}

/// The cells `a` and `b` share: `None` if none, `#VALUE!` if they are on different sheets.
pub(super) fn intersect_pair(
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

pub(crate) fn empty_intersection(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::NULL, cell, "Empty intersection".to_string())
}

/// Whether a sheet prefix names a sheet of another workbook: `[book]Sheet` (a sheet name cannot
/// hold `[`). That workbook is never opened (directive 5).
pub(crate) fn names_other_workbook(sheet_name: Option<&str>) -> bool {
    sheet_name.is_some_and(|name| name.contains('['))
}
