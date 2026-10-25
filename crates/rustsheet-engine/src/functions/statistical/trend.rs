//! TREND and GROWTH: the predictions of LINEST's and LOGEST's fit (statistics.md "TREND:
//! defaults, shapes, single points and error inputs").

use super::linest::{linear_regression, predict, reconstruct_beta};
use super::regression::{
    given, known_points, layout, shape_error, Element, Grid, KnownPoints, Layout,
};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{LiftCost, LiftOptions};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

/// The fit's coefficients in internal order, `[β_0, β_1, ..., β_p]` (`β_0` only with a constant
/// term), collinear columns dropped (see [`linear_regression`]); `None` is `#NUM!`.
fn fit(y: &[f64], x: &[Vec<f64>], p: usize, const_term: bool) -> Option<Vec<f64>> {
    linear_regression(y, x, p, const_term, false)
        .map(|reg| reconstruct_beta(&reg.coeffs, const_term, p))
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
        let y = &grids.y;
        let counting;
        let x = match &grids.x {
            Some(x) => x,
            None => {
                counting = Grid::counting(y.rows, y.cols);
                &counting
            }
        };
        let layout = match layout(y, x, cell) {
            Ok(layout) => layout,
            Err(e) => return e,
        };
        let const_term = match const_term {
            Ok(b) => b,
            Err(e) => return e,
        };
        // The sign of a GROWTH y comes before new_x's text (`GROWTH(Data!A2:A4,Data!A2:A4,<text
        // cells>)` is `#NUM!`, d8266bc15533702a A396).
        let KnownPoints { ys, points, p } = match known_points(y, x, layout, self.growth, cell) {
            Ok(known) => known,
            Err(e) => return e,
        };

        let new_x = grids.new_x.as_ref().unwrap_or(x);
        let (out_rows, out_cols, count) = match layout {
            Layout::Elementwise => (new_x.rows, new_x.cols, new_x.rows * new_x.cols),
            Layout::Columns if new_x.cols == p => (new_x.rows, 1, new_x.rows),
            Layout::Rows if new_x.rows == p => (1, new_x.cols, new_x.cols),
            Layout::Columns | Layout::Rows => return shape_error(cell),
        };
        if new_x
            .elements
            .iter()
            .any(|element| matches!(element, Element::Blank | Element::NotNumber))
        {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "Regression: non-numeric value in new_x".to_string(),
            );
        }

        // One point with a constant term is that point whatever new_x is, its x dropped as
        // collinear with the intercept (GROWTH's is its ln/exp round trip: `GROWTH(5,5,5)` is
        // 4.999999999999999, growth-modes K385).
        let Some(beta) = fit(&ys, &points, p, const_term) else {
            return CalcResult::new_error(
                Error::NUM,
                cell,
                "Regression: singular matrix".to_string(),
            );
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
        let typed = !matches!(
            *const_node.resolved_at(cell.row),
            Node::ReferenceKind { .. }
        );
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
