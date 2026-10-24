//! TREND and GROWTH: the predictions of LINEST's and LOGEST's fit (statistics.md "TREND:
//! defaults, shapes, single points and error inputs").

use super::linest::{linear_regression, predict, reconstruct_beta, DataRead, Element};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{LiftCost, LiftOptions};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// How `known_x` lines up with `known_y` (Excel's LINEST rules): one variable read element by
/// element when both have the same shape, else a column of y with one variable per column of x,
/// or a row of y with one variable per row of x.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// Same shape: `new_x` may have any shape, and the result has it.
    Elementwise,
    /// y is a column: each row of x and `new_x` is one point; the result is a column.
    Columns,
    /// y is a row: each column of x and `new_x` is one point; the result is a row.
    Rows,
}

/// A data argument's elements with its shape (a single value is 1x1).
struct Grid {
    rows: usize,
    cols: usize,
    /// Row-major.
    elements: Vec<Element>,
}

impl Grid {
    fn from_data(data: DataRead) -> Grid {
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
    fn counting(rows: usize, cols: usize) -> Grid {
        Grid {
            rows,
            cols,
            elements: (1..=rows * cols)
                .map(|i| Element::Number(i as f64))
                .collect(),
        }
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
    fn point(&self, numbers: &[f64], layout: Layout, index: usize) -> Vec<f64> {
        match layout {
            Layout::Elementwise => vec![numbers[index]],
            Layout::Columns => numbers[index * self.cols..(index + 1) * self.cols].to_vec(),
            Layout::Rows => (0..self.rows)
                .map(|r| numbers[r * self.cols + index])
                .collect(),
        }
    }
}

/// Matches `known_x`'s shape against `known_y`'s; `None` is `#REF!` (trend L17, L43).
fn layout(y: &Grid, x: &Grid) -> Option<Layout> {
    if (y.rows, y.cols) == (x.rows, x.cols) {
        Some(Layout::Elementwise)
    } else if y.cols == 1 && x.rows == y.rows {
        Some(Layout::Columns)
    } else if y.rows == 1 && x.cols == y.cols {
        Some(Layout::Rows)
    } else {
        None
    }
}

/// The x columns kept when the normal equations are singular: a column that is, to rounding, a
/// combination of the intercept and the columns kept before it is dropped with coefficient 0, as
/// Excel's LINEST drops a collinear column (statistics.md "LINEST": `LINEST(B2:B9,A2:E9,TRUE,TRUE)`
/// drops A). So a zero-variance x gives the mean of y (`TREND(A1:A6,G1:G6)` is 4.5, trend O20). The
/// cut-off and the column order are ours, unprobed: TODO(T-2569).
fn independent_columns(x: &[Vec<f64>], p: usize, const_term: bool) -> Vec<usize> {
    let n = x.len();
    let mut basis: Vec<Vec<f64>> = Vec::new();
    if const_term && n > 0 {
        basis.push(vec![1.0 / (n as f64).sqrt(); n]);
    }
    let mut kept = Vec::new();
    for j in 0..p {
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
    kept
}

/// The fit's coefficients in internal order, `[β_0, β_1, ..., β_p]` (`β_0` only with a constant
/// term), with collinear columns dropped when the full fit is singular; `None` is `#NUM!`.
fn fit(y: &[f64], x: &[Vec<f64>], p: usize, const_term: bool) -> Option<Vec<f64>> {
    if let Some(reg) = linear_regression(y, x, p, const_term, false) {
        return Some(reconstruct_beta(&reg.coeffs, const_term, p));
    }
    let kept = independent_columns(x, p, const_term);
    let offset = usize::from(const_term);
    let mut beta = vec![0.0; p + offset];
    if kept.is_empty() && !const_term {
        return Some(beta);
    }
    let reduced: Vec<Vec<f64>> = x
        .iter()
        .map(|row| kept.iter().map(|&j| row[j]).collect())
        .collect();
    let reg = linear_regression(y, &reduced, kept.len(), const_term, false)?;
    let reduced_beta = reconstruct_beta(&reg.coeffs, const_term, kept.len());
    if const_term {
        beta[0] = reduced_beta[0];
    }
    for (i, &j) in kept.iter().enumerate() {
        beta[offset + j] = reduced_beta[offset + i];
    }
    Some(beta)
}

/// An optional data argument; an empty one (`TREND(A1:A6,,K1:K2)`) is the default (trend L12-L16).
fn given(arg: Option<&Node>) -> Option<&Node> {
    arg.filter(|node| !matches!(node, Node::EmptyArgKind))
}

/// The data arguments of one TREND or GROWTH call, read once.
struct Grids {
    y: Grid,
    /// `None` when omitted: {1, 2, 3, ...} in y's shape.
    x: Option<Grid>,
    /// `None` when omitted: `known_x`.
    new_x: Option<Grid>,
}

/// One TREND or GROWTH call's data, or the first of its data arguments that is an error value.
struct TrendData {
    grids: Result<Grids, CalcResult>,
    growth: bool,
}

/// What `const` reads as for one call (or one lifted element).
enum Const {
    /// An error value, or typed text that is no boolean: reported right after the data
    /// arguments' errors, before the shapes.
    Early(CalcResult),
    /// The boolean, or why a cell holding something else is none (reported after the shapes).
    Value(Result<bool, CalcResult>),
}

impl TrendData {
    /// The predictions for one `const` (steps 1 to 5 of [`Model::trend_or_growth`]).
    fn predictions(&self, const_arg: Const, cell: CellReferenceIndex) -> CalcResult {
        let grids = match &self.grids {
            Ok(grids) => grids,
            Err(error) => return error.clone(),
        };
        let const_term = match const_arg {
            Const::Early(error) => return error,
            Const::Value(const_term) => const_term,
        };
        let value_error =
            |message: &str| CalcResult::new_error(Error::VALUE, cell, message.to_string());
        let ref_error = || {
            CalcResult::new_error(
                Error::REF,
                cell,
                "Regression: known_y, known_x and new_x do not match in size".to_string(),
            )
        };
        let y = &grids.y;
        let counting;
        let x = match &grids.x {
            Some(x) => x,
            None => {
                counting = Grid::counting(y.rows, y.cols);
                &counting
            }
        };
        let Some(layout) = layout(y, x) else {
            return ref_error();
        };
        let const_term = match const_term {
            Ok(b) => b,
            Err(e) => return e,
        };
        // GROWTH reads known_y in row-major order and its first element that is no positive
        // number decides: a blank, text, boolean or error is `#VALUE!` at once, a non-positive
        // number `#NUM!` once known_x is read too. `GROWTH(Data!D6:D10)` holding 1.005, 0, a blank
        // is `#NUM!`, `GROWTH(Data!A2:A12)` holding "1.5", 2, .., -0.0001 `#VALUE!`,
        // `GROWTH(Data!D8:E9)` holding a blank, then 1, -0.5 `#VALUE!` (gen-m9 58e3253bbd3a7a82
        // A291, 40425341c5034eed A158, 58e3253bbd3a7a82 A383), and `GROWTH(0,Data!A7)` with A7 =
        // "abc" `#VALUE!` (fec39efaba9284c3 A35). The sign comes before new_x's text
        // (`GROWTH(Data!A2:A4,Data!A2:A4,<text cells>)` is `#NUM!`, d8266bc15533702a A396).
        let first_bad = y
            .elements
            .iter()
            .find(|element| !matches!(element, Element::Number(v) if *v > 0.0));
        let not_positive = self.growth && matches!(first_bad, Some(Element::Number(_)));
        let (Some(y_values), Some(x_values)) = (
            if not_positive {
                Some(Vec::new())
            } else {
                y.numbers()
            },
            x.numbers(),
        ) else {
            return value_error("Regression: non-numeric value in known_y or known_x");
        };
        if not_positive {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "GROWTH: all known_y values must be positive".to_string(),
            );
        }
        let n = y_values.len();
        let points: Vec<Vec<f64>> = (0..n).map(|i| x.point(&x_values, layout, i)).collect();
        let p = points.first().map_or(0, Vec::len);

        let new_x = grids.new_x.as_ref().unwrap_or(x);
        let (out_rows, out_cols, count) = match layout {
            Layout::Elementwise => (new_x.rows, new_x.cols, new_x.rows * new_x.cols),
            Layout::Columns if new_x.cols == p => (new_x.rows, 1, new_x.rows),
            Layout::Rows if new_x.rows == p => (1, new_x.cols, new_x.cols),
            Layout::Columns | Layout::Rows => return ref_error(),
        };
        if new_x
            .elements
            .iter()
            .any(|element| matches!(element, Element::Blank | Element::NotNumber))
        {
            return value_error("Regression: non-numeric value in new_x");
        }

        let ys = if self.growth {
            y_values.iter().map(|v| v.ln()).collect()
        } else {
            y_values
        };
        // One point with a constant term is that point whatever new_x is: LINEST of one point is
        // slope 0, intercept y (linest-modes K141). GROWTH's is its ln/exp round trip
        // (`GROWTH(5,5,5)` is 4.999999999999999, growth-modes K385). Without a constant term the
        // fit through the origin is unprobed and left to the regression.
        let beta = match (&ys[..], const_term) {
            (&[single], true) => {
                let mut beta = vec![0.0; p + 1];
                beta[0] = single;
                beta
            }
            _ => match fit(&ys, &points, p, const_term) {
                Some(beta) => beta,
                None => {
                    return CalcResult::new_error(
                        Error::NUM,
                        cell,
                        "Regression: singular matrix".to_string(),
                    )
                }
            },
        };

        // An error element of new_x is `#VALUE!` for its own point.
        let new_numbers: Vec<f64> = new_x
            .elements
            .iter()
            .map(|element| match element {
                Element::Number(v) => *v,
                _ => f64::NAN,
            })
            .collect();
        let predictions: Vec<ArrayNode> = (0..count)
            .map(|i| {
                let point = new_x.point(&new_numbers, layout, i);
                if point.iter().any(|v| v.is_nan()) {
                    return ArrayNode::Error(Error::VALUE);
                }
                let fitted = predict(&beta, &point, const_term);
                ArrayNode::Number(if self.growth { fitted.exp() } else { fitted })
            })
            .collect();
        CalcResult::Array(
            predictions
                .chunks(out_cols.max(1))
                .take(out_rows)
                .map(<[ArrayNode]>::to_vec)
                .collect(),
        )
    }

    /// The data's cells, what one lifted element's fit walks.
    fn cells(&self) -> u64 {
        let Ok(grids) = &self.grids else {
            return 1;
        };
        let count = |grid: &Grid| u64::try_from(grid.elements.len()).unwrap_or(u64::MAX);
        let y = count(&grids.y);
        let x = grids.x.as_ref().map_or(y, count);
        let new_x = grids.new_x.as_ref().map_or(x, count);
        y.saturating_add(x).saturating_add(new_x)
    }
}

impl Model {
    /// TREND (`growth` false) or GROWTH (true) of `known_y, [known_x], [new_x], [const]`. The checks,
    /// in Excel's order (probes/regression-trend; gen-m9 T-2256 cells, statistics.md "TREND:
    /// defaults, shapes, single points and error inputs"):
    /// 1. an argument that is an error value, in argument order, const's included: `TREND(<#NUM!>,
    ///    B8,A5:C11,NA())` is `#NUM!`, `TREND(B4:C10,A11:B12,<#VALUE!>)` with unequal shapes `#VALUE!`
    ///    (gen-m9 3b7990bd4e3307ba A345, 11eb3ab4f4f8104c A326), then a typed or computed text
    ///    const that is no boolean (`#VALUE!`, f82473275380882e A100);
    /// 2. `known_x`'s shape against `known_y`'s (`#REF!`), a text `known_x` being one value
    ///    (`TREND(A1:A6,"a")`, trend L43) and an error element waiting (gen-m9 a4090567758520df A333);
    /// 3. a const cell that is no boolean (`#VALUE!`; after the shapes, bbab6562e88b70e8 A13);
    /// 4. a blank, text, boolean or error element of `known_y` or `known_x` (`#VALUE!`, trend L23-L29);
    ///    for GROWTH a non-positive `known_y` number before it in row-major order is `#NUM!`;
    /// 5. `new_x`'s shape (`#REF!`), after the data (80586b85 A265), and a blank, text or boolean
    ///    element of `new_x` (`#VALUE!`, trend L38, L39); an error element of `new_x` is `#VALUE!` for
    ///    its own prediction only (trend L37).
    ///
    /// An array const lifts: each element is the top-left value of the call with that const
    /// (`TREND(..,SORTBY(Data!A1:D12,..))` spills, gen-m9 80586b85 A61).
    fn trend_or_growth(
        &mut self,
        args: &[Node],
        growth: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let data = TrendData {
            grids: self.read_trend_grids(args, cell),
            growth,
        };
        let Some(const_node) = args.get(3) else {
            return data.predictions(Const::Value(Ok(true)), cell);
        };
        // A typed or computed text const is `#VALUE!` at once, before the shapes
        // (`TREND(Data!A2:D6,Data!C2:E4,Data!E2:E6,"x")`, `GROWTH(..,"1")`, `GROWTH(..,"")`;
        // gen-m9 f82473275380882e A100, a5828d3925575713 A284, caa1eed48b1883fe A19), and so is a
        // text element of a lifted const (`GROWTH(..,TEXTSPLIT(..))`, 8fec7b207f4dd753 A14), while
        // a text cell waits for them (`TREND(Data!C10:E11,..,Data!$A12)` with A12 = "abc" is
        // `#REF!`, bbab6562e88b70e8 A13). A multi-cell range does not lift
        // (`TREND(..,OFFSET(Data!B6:C12,-1,3))` is `#VALUE!`, 43c6227e0ff368c7 A165).
        let typed = !matches!(const_node, Node::ReferenceKind { .. });
        let kernel = |model: &mut Model, value: CalcResult| {
            if value.is_error() {
                return data.predictions(Const::Early(value), cell);
            }
            let is_text = typed && matches!(value, CalcResult::String(_));
            let const_arg = match model.cast_to_bool(value, cell) {
                Err(e) if is_text => Const::Early(e),
                const_term => Const::Value(const_term),
            };
            data.predictions(const_arg, cell)
        };
        let const_value = self.evaluate_node_in_context(const_node, cell);
        if !matches!(const_value, CalcResult::Array(_)) {
            return kernel(self, const_value);
        }
        // The lift comes before the data arguments' errors: each element is that error
        // (`GROWTH(LOOKUP(NA(),..),..,GROUPBY(..))` spills, gen-m9 165db58d1540f014 A239).
        self.lift_values(
            vec![const_value],
            LiftCost::Cells(data.cells()),
            &LiftOptions::REUSE_ANY_EQUAL_TOP_LEFT,
            cell,
            &mut |model, values| kernel(model, values[0].clone()),
        )
    }

    /// Reads `known_y`, `known_x` and `new_x`, stopping at the first that is an error value.
    fn read_trend_grids(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<Grids, CalcResult> {
        let y = Grid::from_data(self.read_known_y(&args[0], cell)?);
        let x = match given(args.get(1)) {
            Some(node) => Some(Grid::from_data(self.read_data(node, cell)?)),
            None => None,
        };
        let new_x = match given(args.get(2)) {
            Some(node) => Some(Grid::from_data(self.read_data(node, cell)?)),
            None => None,
        };
        Ok(Grids { y, x, new_x })
    }

    // TREND(known_y's, [known_x's], [new_x's], [const])
    pub(crate) fn fn_trend(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.trend_or_growth(args, false, cell)
    }

    // GROWTH(known_y's, [known_x's], [new_x's], [const]): TREND of ln(y), exponentiated.
    pub(crate) fn fn_growth(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.trend_or_growth(args, true, cell)
    }
}
