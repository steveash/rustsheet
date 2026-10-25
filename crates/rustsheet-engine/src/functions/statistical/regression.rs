//! The data arguments of LINEST, LOGEST, TREND and GROWTH: how they are read, how `known_x` lines up
//! with `known_y`, and which x columns a singular fit drops (statistics.md "TREND: defaults,
//! shapes, single points and error inputs", "LINEST, LOGEST, GROWTH: probe findings").

use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// One element of a regression data argument as [`Model::read_data`] read it.
pub(super) enum Element {
    Number(f64),
    /// An empty cell, or an empty argument or array element.
    Blank,
    /// An error cell or element, or a lambda in an array.
    Error,
    /// Text, a boolean or a lambda.
    NotNumber,
}

/// A regression data argument, element by element, with where it came from.
pub(super) enum DataRead {
    Range(Vec<Vec<Element>>),
    Array(Vec<Vec<Element>>),
    Scalar(Element),
}

impl Model {
    /// Reads `known_y` as [`Model::read_data`] does. A computed blank y is `#VALUE!` at once,
    /// before x's and const's errors: `LINEST(IFERROR(B10,1),1/0)` and
    /// `LINEST(IFERROR(B10,1),{1;2},NA())` with B10 blank are `#VALUE!`, while the blank reference
    /// itself and a computed text or boolean y wait (`LINEST(B10,1/0)`, `LINEST("abc",1/0)` and
    /// `LINEST(IF(TRUE,"abc"),1/0)` are `#DIV/0!`; probes/t2455-transpose-single-value A62:A84).
    pub(super) fn read_known_y(
        &mut self,
        y_arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<DataRead, CalcResult> {
        let y = self.read_data(y_arg, cell)?;
        if matches!(y, DataRead::Scalar(Element::Blank))
            && !matches!(*y_arg.resolved_at(cell.row), Node::ReferenceKind { .. })
        {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Regression: known_y is blank".to_string(),
            ));
        }
        Ok(y)
    }

    /// Reads a data argument of the regression functions element by element, failing only when it
    /// is an error value itself; each caller decides what a blank, a text or an error element is.
    pub(super) fn read_data(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<DataRead, CalcResult> {
        Ok(match self.evaluate_node_in_context(arg, cell) {
            CalcResult::Range { left, right } => {
                let n_rows = (right.row - left.row + 1) as usize;
                let n_cols = (right.column - left.column + 1) as usize;
                let mut rows = Vec::with_capacity(n_rows);
                for r in 0..n_rows {
                    let mut row = Vec::with_capacity(n_cols);
                    for c in 0..n_cols {
                        let ref_idx = CellReferenceIndex {
                            sheet: left.sheet,
                            row: left.row + r as i32,
                            column: left.column + c as i32,
                        };
                        row.push(match self.evaluate_cell(ref_idx) {
                            CalcResult::Number(v) => Element::Number(v),
                            CalcResult::EmptyCell | CalcResult::EmptyArg => Element::Blank,
                            CalcResult::Error { .. } => Element::Error,
                            _ => Element::NotNumber,
                        });
                    }
                    rows.push(row);
                }
                DataRead::Range(rows)
            }
            CalcResult::Array(arr) => DataRead::Array(
                arr.into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|item| match item {
                                ArrayNode::Number(v) => Element::Number(v),
                                ArrayNode::Error(_)
                                | ArrayNode::Lambda(_)
                                | ArrayNode::Nested(_) => Element::Error,
                                ArrayNode::Empty => Element::Blank,
                                _ => Element::NotNumber,
                            })
                            .collect()
                    })
                    .collect(),
            ),
            CalcResult::Number(n) => DataRead::Scalar(Element::Number(n)),
            CalcResult::EmptyCell | CalcResult::EmptyArg => DataRead::Scalar(Element::Blank),
            err @ CalcResult::Error { .. } => return Err(err),
            _ => DataRead::Scalar(Element::NotNumber),
        })
    }
}

/// An optional data argument; an empty one (`TREND(A1:A6,,K1:K2)`, `LINEST(B2:B9,,TRUE,TRUE)`) is
/// the default (trend L12-L16, linest-modes K71).
pub(super) fn given(arg: Option<&Node>) -> Option<&Node> {
    arg.filter(|node| !matches!(node, Node::EmptyArgKind))
}

/// How `known_x` lines up with `known_y` (Excel's LINEST rules): one variable read element by
/// element when both have the same shape, else a column of y with one variable per column of x,
/// or a row of y with one variable per row of x.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Layout {
    /// Same shape: TREND's `new_x` may have any shape, and the result has it.
    Elementwise,
    /// y is a column: each row of x and `new_x` is one point; the result is a column.
    Columns,
    /// y is a row: each column of x and `new_x` is one point; the result is a row.
    Rows,
}

/// A data argument's elements with its shape (a single value is 1x1).
pub(super) struct Grid {
    pub(super) rows: usize,
    pub(super) cols: usize,
    /// Row-major.
    pub(super) elements: Vec<Element>,
}

impl Grid {
    pub(super) fn from_data(data: DataRead) -> Grid {
        match data {
            DataRead::Range(rows) | DataRead::Array(rows) => {
                // Rows are as wide as the first, so `point` stays in bounds: a short row of a
                // ragged array (none is built today) reads as blanks, a long one is cut.
                let cols = rows.first().map_or(0, Vec::len);
                let n_rows = rows.len();
                let mut elements = Vec::with_capacity(n_rows.saturating_mul(cols));
                for row in rows {
                    let mut row = row.into_iter();
                    elements.extend((0..cols).map(|_| row.next().unwrap_or(Element::Blank)));
                }
                Grid {
                    rows: n_rows,
                    cols,
                    elements,
                }
            }
            DataRead::Scalar(element) => Grid {
                rows: 1,
                cols: 1,
                elements: vec![element],
            },
        }
    }

    /// The default `known_x`, {1, 2, 3, ...} in y's shape.
    pub(super) fn counting(rows: usize, cols: usize) -> Grid {
        Grid {
            rows,
            cols,
            elements: (1..=rows * cols)
                .map(|i| Element::Number(i as f64))
                .collect(),
        }
    }

    /// Whether an element is an error (or a lambda). LINEST and LOGEST report it as `#VALUE!`
    /// right after the shapes, before a text const or stats cell (`LINEST(Data!A1:C6,
    /// ROUND(GROUPBY(..)),..)`, gen-m9 589f42e67f4b4b8d A100; `LOGEST(<number>,IF(SORT(IF({1,0,1},
    /// 1,NA())),..),..)`, a scalar y against an x holding `#N/A`, d8266bc15533702a A117).
    pub(super) fn has_error(&self) -> bool {
        self.elements
            .iter()
            .any(|element| matches!(element, Element::Error))
    }

    /// Every element as a number, or `None` when one is a blank, text, a boolean or an error:
    /// those are all `#VALUE!` in `known_y` and `known_x` (trend L23-L29, an error cell included).
    fn numbers(&self) -> Option<Vec<f64>> {
        self.elements
            .iter()
            .map(|element| match element {
                Element::Number(v) => Some(*v),
                _ => None,
            })
            .collect()
    }

    /// The numbers of point `index` in `layout` (an element, a row or a column).
    pub(super) fn point(&self, numbers: &[f64], layout: Layout, index: usize) -> Vec<f64> {
        match layout {
            Layout::Elementwise => vec![numbers[index]],
            Layout::Columns => numbers[index * self.cols..(index + 1) * self.cols].to_vec(),
            Layout::Rows => (0..self.rows)
                .map(|r| numbers[r * self.cols + index])
                .collect(),
        }
    }
}

/// Matches `known_x`'s shape against `known_y`'s; a mismatch is `#REF!` (trend L17, L43;
/// `LINEST({1,2,3,5},{1;2;3;4})`, linest-modes K232). A single y is both a row and a column, so
/// `LINEST(5,A2:A9)` is one point of 8 variables (K204).
pub(super) fn layout(y: &Grid, x: &Grid, cell: CellReferenceIndex) -> Result<Layout, CalcResult> {
    if (y.rows, y.cols) == (x.rows, x.cols) {
        Ok(Layout::Elementwise)
    } else if y.cols == 1 && x.rows == y.rows {
        Ok(Layout::Columns)
    } else if y.rows == 1 && x.cols == y.cols {
        Ok(Layout::Rows)
    } else {
        Err(shape_error(cell))
    }
}

/// The `#REF!` of data arguments whose shapes do not match.
pub(super) fn shape_error(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(
        Error::REF,
        cell,
        "Regression: known_y, known_x and new_x do not match in size".to_string(),
    )
}

/// The known points of a fit: y (ln y when `log`), and each point's x values.
pub(super) struct KnownPoints {
    pub(super) ys: Vec<f64>,
    pub(super) points: Vec<Vec<f64>>,
    /// The number of x variables.
    pub(super) p: usize,
}

/// The known points once the shapes match: a blank, text, boolean or error element of `known_y` or
/// `known_x` is `#VALUE!` (trend L23-L29; `LINEST(B2:B9,A2:A9&"")`, linest-modes K113). For LOGEST
/// and GROWTH (`log`) `known_y` is read in row-major order and its first element that is no positive
/// number decides: a blank, text, boolean or error is `#VALUE!` at once, a non-positive number
/// `#NUM!` once `known_x` is read too. `GROWTH(Data!D6:D10)` holding 1.005, 0, a blank is `#NUM!`,
/// `GROWTH(Data!A2:A12)` holding "1.5", 2, .., -0.0001 `#VALUE!`, `GROWTH(Data!D8:E9)` holding a
/// blank, then 1, -0.5 `#VALUE!` (gen-m9 58e3253bbd3a7a82 A291, 40425341c5034eed A158,
/// 58e3253bbd3a7a82 A383), `GROWTH(0,Data!A7)` with A7 = "abc" `#VALUE!` (fec39efaba9284c3 A35),
/// and `LOGEST(Data!C6:E11)` with E6 = -0.0001 before C7 = "1.5" `#NUM!` (58e3253bbd3a7a82 A250).
pub(super) fn known_points(
    y: &Grid,
    x: &Grid,
    layout: Layout,
    log: bool,
    cell: CellReferenceIndex,
) -> Result<KnownPoints, CalcResult> {
    let first_bad = y
        .elements
        .iter()
        .find(|element| !matches!(element, Element::Number(v) if *v > 0.0));
    let not_positive = log && matches!(first_bad, Some(Element::Number(_)));
    let (Some(y_values), Some(x_values)) = (
        if not_positive {
            Some(Vec::new())
        } else {
            y.numbers()
        },
        x.numbers(),
    ) else {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "Regression: non-numeric value in known_y or known_x".to_string(),
        ));
    };
    if not_positive {
        return Err(CalcResult::new_error(
            Error::NUM,
            cell,
            "Regression: all known_y values must be positive".to_string(),
        ));
    }
    let points: Vec<Vec<f64>> = (0..y_values.len())
        .map(|i| x.point(&x_values, layout, i))
        .collect();
    let p = points.first().map_or(0, Vec::len);
    let ys = if log {
        y_values.iter().map(|v| v.ln()).collect()
    } else {
        y_values
    };
    Ok(KnownPoints { ys, points, p })
}

/// The x columns kept when the normal equations are singular, in ascending order: a column that
/// is, to rounding, a combination of the intercept and the columns kept after it is dropped with
/// coefficient 0, as Excel's LINEST drops a collinear column. The last column is tried first:
/// `LINEST(B2:B9,A2:E9,TRUE,TRUE)` with E = 2A drops A, not E (A's se is exactly 0, linest-modes
/// K106:O107), and one point drops its x (K141). So a zero-variance x gives the mean of y
/// (`TREND(A1:A6,G1:G6)` is 4.5, trend O20). The cut-off is ours, unprobed: TODO(T-2569).
pub(super) fn independent_columns(x: &[Vec<f64>], p: usize, const_term: bool) -> Vec<usize> {
    let n = x.len();
    let mut basis: Vec<Vec<f64>> = Vec::new();
    if const_term && n > 0 {
        basis.push(vec![1.0 / (n as f64).sqrt(); n]);
    }
    let mut kept = Vec::new();
    for j in (0..p).rev() {
        let column: Vec<f64> = x.iter().map(|row| row[j]).collect();
        let norm = column.iter().map(|v| v * v).sum::<f64>().sqrt();
        let mut residual = column;
        for b in &basis {
            let dot: f64 = residual.iter().zip(b).map(|(r, bi)| r * bi).sum();
            for (r, bi) in residual.iter_mut().zip(b) {
                *r -= dot * bi;
            }
        }
        let rest = residual.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 0.0 && rest > 1e-10 * norm {
            basis.push(residual.iter().map(|v| v / rest).collect());
            kept.push(j);
        }
    }
    kept.reverse();
    kept
}
