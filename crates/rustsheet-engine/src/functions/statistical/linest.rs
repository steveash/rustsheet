use super::regression::{given, independent_columns, known_points, layout, Grid};
use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{LiftCost, LiftOptions};
use crate::{
    calc_result::CalcResult, expressions::parser::Node, expressions::token::Error, model::Model,
};

// ─── Matrix helpers ──────────────────────────────────────────────────────────

/// Gauss-Jordan elimination: solves A·x = b in-place on the augmented [A|b] matrix.
/// Returns None if the matrix is singular (pivot < eps).
fn solve_system(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let k = a.len();
    let mut m: Vec<Vec<f64>> = (0..k)
        .map(|i| {
            let mut row = a[i].clone();
            row.push(b[i]);
            row
        })
        .collect();

    for col in 0..k {
        // partial pivoting
        let pivot_row = (col..k)
            .max_by(|&r1, &r2| {
                m[r1][col]
                    .abs()
                    .partial_cmp(&m[r2][col].abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(col);
        m.swap(col, pivot_row);

        let pivot = m[col][col];
        if pivot.abs() < 1e-12 {
            return None;
        }

        for val in &mut m[col][col..=k] {
            *val /= pivot;
        }

        let col_slice: Vec<f64> = m[col][col..=k].to_vec();
        for (row, mrow) in m.iter_mut().enumerate() {
            if row != col {
                let factor = mrow[col];
                for (offset, &v) in col_slice.iter().enumerate() {
                    mrow[col + offset] -= factor * v;
                }
            }
        }
    }

    Some((0..k).map(|i| m[i][k]).collect())
}

/// Inverts a square matrix via Gauss-Jordan on [A|I].
/// Returns None if singular.
fn invert_matrix(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let k = a.len();
    let mut m: Vec<Vec<f64>> = (0..k)
        .map(|i| {
            let mut row = a[i].clone();
            for j in 0..k {
                row.push(if i == j { 1.0 } else { 0.0 });
            }
            row
        })
        .collect();

    for col in 0..k {
        let pivot_row = (col..k)
            .max_by(|&r1, &r2| {
                m[r1][col]
                    .abs()
                    .partial_cmp(&m[r2][col].abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(col);
        m.swap(col, pivot_row);

        let pivot = m[col][col];
        if pivot.abs() < 1e-12 {
            return None;
        }

        for val in &mut m[col][col..2 * k] {
            *val /= pivot;
        }

        let col_slice: Vec<f64> = m[col][col..2 * k].to_vec();
        for (row, mrow) in m.iter_mut().enumerate() {
            if row != col {
                let factor = mrow[col];
                for (offset, &v) in col_slice.iter().enumerate() {
                    mrow[col + offset] -= factor * v;
                }
            }
        }
    }

    Some((0..k).map(|i| m[i][k..].to_vec()).collect())
}

// ─── Regression core ─────────────────────────────────────────────────────────

/// Result of the core linear regression.
pub(super) struct RegResult {
    /// Coefficients in Excel output order: [`β_p`, β_{p-1}, ..., `β_1`, `β_0`]
    /// (last x column first, intercept last, or omitted if const=FALSE)
    pub(super) coeffs: Vec<f64>,
    /// Standard errors in the same order (only if stats=TRUE)
    std_errors: Option<Vec<f64>>,
    r_squared: Option<f64>,
    se_y: Option<f64>,
    f_stat: Option<f64>,
    df_res: Option<f64>,
    ss_reg: Option<f64>,
    ss_res: Option<f64>,
}

/// Linear regression for any number of predictors, as LINEST: when the normal equations are
/// singular (or there are fewer points than parameters), the collinear x columns
/// ([`independent_columns`]) are dropped with coefficient 0 and standard error 0, and the degrees
/// of freedom count only the kept ones (`LINEST(B2:B9,A2:E9,TRUE,TRUE)`: A's coefficient and se 0,
/// df 3; one point `LINEST(B15,A15,TRUE,TRUE)` is `{0,9; 0,0; 1,0; #NUM!,0; 0,0}`, linest-modes
/// K106, K141). `None` is `#NUM!`.
pub(super) fn linear_regression(
    y: &[f64],
    x: &[Vec<f64>],
    p: usize,
    const_term: bool,
    compute_stats: bool,
) -> Option<RegResult> {
    if y.len() >= p + usize::from(const_term) {
        if let Some(reg) = regress(y, x, p, const_term, compute_stats) {
            return Some(reg);
        }
    }
    let kept = independent_columns(x, p, const_term);
    let reduced: Vec<Vec<f64>> = x
        .iter()
        .map(|row| kept.iter().map(|&j| row[j]).collect())
        .collect();
    let mut reg = regress(y, &reduced, kept.len(), const_term, compute_stats)?;
    reg.coeffs = expand_kept(&reg.coeffs, &kept, p);
    reg.std_errors = reg.std_errors.map(|se| expand_kept(&se, &kept, p));
    Some(reg)
}

/// Values of a fit over the `kept` columns (Excel order: the last x column first, the intercept
/// last) spread over all `p` columns, 0 for a dropped one.
fn expand_kept(values: &[f64], kept: &[usize], p: usize) -> Vec<f64> {
    let mut out = vec![0.0; p + 1];
    for (i, &j) in kept.iter().enumerate() {
        out[p - 1 - j] = values[kept.len() - 1 - i];
    }
    out[p] = values[kept.len()];
    out
}

/// The least-squares fit of the normal equations, `None` when they are singular.
/// Design matrix columns: [1, `x_1`, `x_2`, ..., `x_p`] (or [`x_1`, ..., `x_p`] if !`const_term`).
fn regress(
    y: &[f64],
    x: &[Vec<f64>],
    p: usize,
    const_term: bool,
    compute_stats: bool,
) -> Option<RegResult> {
    let n = y.len();
    let k = p + const_term as usize; // number of parameters

    // Build D'D (k×k) and D'Y (k×1)
    let mut dtd = vec![vec![0.0_f64; k]; k];
    let mut dty = vec![0.0_f64; k];

    for i in 0..n {
        // Build design row d[i]: length k
        let d: Vec<f64> = if const_term {
            let mut row = vec![1.0_f64];
            row.extend_from_slice(&x[i]);
            row
        } else {
            x[i].clone()
        };

        for r in 0..k {
            dty[r] += d[r] * y[i];
            for c in 0..k {
                dtd[r][c] += d[r] * d[c];
            }
        }
    }

    // Solve (D'D) β = D'Y
    let beta = solve_system(&dtd, &dty)?;
    // beta[0] = β_0 (intercept if const), beta[1..] = β_1..β_p

    // Excel output order: reverse x coefficients, intercept last.
    // When const=FALSE the intercept is forced to 0, but Excel still outputs a
    // p+1 wide array with 0 in the intercept column.
    let coeffs: Vec<f64> = {
        let x_part: Vec<f64> = beta[const_term as usize..].iter().rev().cloned().collect();
        let mut out = x_part;
        out.push(if const_term { beta[0] } else { 0.0 });
        out
    };

    if !compute_stats {
        return Some(RegResult {
            coeffs,
            std_errors: None,
            r_squared: None,
            se_y: None,
            f_stat: None,
            df_res: None,
            ss_reg: None,
            ss_res: None,
        });
    }

    // Compute fitted values and residuals
    let fitted: Vec<f64> = (0..n)
        .map(|i| {
            let d: Vec<f64> = if const_term {
                let mut row = vec![1.0_f64];
                row.extend_from_slice(&x[i]);
                row
            } else {
                x[i].clone()
            };
            d.iter().zip(beta.iter()).map(|(di, bi)| di * bi).sum()
        })
        .collect();

    let ss_res: f64 = y
        .iter()
        .zip(fitted.iter())
        .map(|(yi, fi)| (yi - fi).powi(2))
        .sum();
    let df = n as f64 - k as f64;

    let ss_tot = if const_term {
        let mean = y.iter().sum::<f64>() / n as f64;
        y.iter().map(|yi| (yi - mean).powi(2)).sum()
    } else {
        y.iter().map(|yi| yi.powi(2)).sum::<f64>()
    };

    let ss_reg = ss_tot - ss_res;
    let r_sq = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    let r_sq = r_sq.clamp(0.0, 1.0);

    // An exact fit with no degree of freedom left has se_y and standard errors 0
    // (`LINEST(A12:A13,B12:B13,TRUE,TRUE)`, two points, linest-modes K134:L138).
    let var_y = if df > 0.0 { ss_res / df } else { 0.0 };
    let se_y = var_y.sqrt();

    // F is `#NUM!` (not finite) for an exact fit or no degree of freedom (linest-modes K137,
    // K144, K249).
    let df_model = p as f64; // number of x predictors (not counting intercept)
    let f_stat = if df > 0.0 {
        (ss_reg / df_model) / (ss_res / df)
    } else {
        f64::NAN
    };

    // Standard errors: sqrt(se_y² · (D'D)^{-1}_{j,j})
    let se_beta: Vec<f64> = {
        let inv = invert_matrix(&dtd)?;
        (0..k).map(|j| (var_y * inv[j][j]).sqrt()).collect()
    };

    // Reorder standard errors to match coefficients output order.
    // const=FALSE: intercept se is #N/A (represented as NaN here).
    let std_errors: Vec<f64> = {
        let x_part: Vec<f64> = se_beta[const_term as usize..]
            .iter()
            .rev()
            .cloned()
            .collect();
        let mut out = x_part;
        out.push(if const_term { se_beta[0] } else { f64::NAN });
        out
    };

    Some(RegResult {
        coeffs,
        std_errors: Some(std_errors),
        r_squared: Some(r_sq),
        se_y: Some(se_y),
        f_stat: Some(f_stat),
        df_res: Some(df),
        ss_reg: Some(ss_reg),
        ss_res: Some(ss_res),
    })
}

/// Build LINEST/LOGEST output array from a `RegResult`.
/// ncols = `coeffs.len()` (one column per coefficient)
fn build_output_array(reg: RegResult, stats: bool) -> CalcResult {
    let ncols = reg.coeffs.len();

    let row0: Vec<ArrayNode> = reg.coeffs.iter().map(|&v| ArrayNode::Number(v)).collect();

    if !stats {
        return CalcResult::Array(vec![row0]);
    }

    // Row 1: standard errors
    let row1: Vec<ArrayNode> = reg
        .std_errors
        .as_deref()
        .unwrap_or(&vec![f64::NAN; ncols])
        .iter()
        .map(|&v| {
            if v.is_nan() {
                ArrayNode::Error(Error::NA)
            } else {
                ArrayNode::Number(v)
            }
        })
        .collect();

    // Row 2: [R², se_y, #N/A, ...]
    let mut row2 = vec![ArrayNode::Error(Error::NA); ncols];
    row2[0] = ArrayNode::Number(reg.r_squared.unwrap_or(f64::NAN));
    if ncols > 1 {
        let se = reg.se_y.unwrap_or(f64::NAN);
        row2[1] = if se.is_nan() {
            ArrayNode::Error(Error::NA)
        } else {
            ArrayNode::Number(se)
        };
    }

    // Row 3: [F, df, #N/A, ...]
    let mut row3 = vec![ArrayNode::Error(Error::NA); ncols];
    row3[0] = match reg.f_stat {
        Some(f) if f.is_finite() => ArrayNode::Number(f),
        _ => ArrayNode::Error(Error::NUM),
    };
    if ncols > 1 {
        row3[1] = match reg.df_res {
            Some(df) if !df.is_nan() => ArrayNode::Number(df),
            _ => ArrayNode::Error(Error::NA),
        };
    }

    // Row 4: [SS_reg, SS_res, #N/A, ...]
    let mut row4 = vec![ArrayNode::Error(Error::NA); ncols];
    row4[0] = match reg.ss_reg {
        Some(v) if !v.is_nan() => ArrayNode::Number(v),
        _ => ArrayNode::Error(Error::NA),
    };
    if ncols > 1 {
        row4[1] = match reg.ss_res {
            Some(v) if !v.is_nan() => ArrayNode::Number(v),
            _ => ArrayNode::Error(Error::NA),
        };
    }

    CalcResult::Array(vec![row0, row1, row2, row3, row4])
}

// ─── Public function implementations ─────────────────────────────────────────

impl Model {
    /// LINEST (`log` false) or LOGEST (true) of `known_y, [known_x], [const], [stats]`: LOGEST is
    /// LINEST of ln(y) with the coefficients exponentiated; its stats rows are those of the
    /// log-linear fit. The checks, in Excel's order:
    /// 1. an argument that is an error value, in argument order: `LINEST({1;2},1/0,"x")` and
    ///    `LINEST(1/0,{1;3},"x")` are `#DIV/0!` (probes/t2455-transpose-single-value A4:A27), while
    ///    `LINEST(Data!C3:C7,Data!B5:B10,NA(),"1")` is `#N/A`, not the size `#REF!` (gen-m9
    ///    05d44f6dd6cfc567 A195), and so is `LINEST(Data!D10:E10,Data!B1:B7,<text cell>,<#N/A>)`
    ///    (bcb24385baafb2f0 A40; statistics.md "LINEST and LOGEST report error values first");
    /// 2. the shapes (`#REF!`, [`layout`]): `LINEST(Data!B11:E12,(BYROW(..)+HOUR(0.5)),2,-1)` is
    ///    `#REF!` though its x holds errors (gen-m9 05d44f6dd6cfc567 A317);
    /// 3. an error element of `known_y` or `known_x` (`#VALUE!`, [`Grid::has_error`]);
    /// 4. a const or stats that is text and no boolean (`#VALUE!`; after the shapes:
    ///    `LOGEST(Data!A10:B10,Data!B8:B8,-1,<"a" cell>)` and `LOGEST(Data!A10:A11,-0,
    ///    LAMBDA(acc,Data!B3)(..))` with B3 = " x " are `#REF!`, f34f66fb7f81e673 A179,
    ///    bbdfc06cae0e0d71 A324);
    /// 5. the data ([`known_points`]).
    ///
    /// An array const or stats lifts, as TREND's const: each element is the top-left value of the
    /// call with those flags (`MAKEARRAY(2,3,..)` as stats is `#SPILL!`, gen-m9 c2974c061384fdd9
    /// A89), so `LOGEST(Data!C$10,0.1,EXPAND(Data!A10:C11,2,4))` spills 2x4 (`#SPILL!` there,
    /// gen-m9 165db58d1540f014 A327).
    fn linest_or_logest(
        &mut self,
        args: &[Node],
        log: bool,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if !(1..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }
        let data = match self.read_regression_grids(args, cell) {
            Ok(data) => data,
            Err(error) => return error,
        };
        // An omitted flag is an empty argument: its default.
        let mut flag_arg = |index: usize| match args.get(index) {
            Some(node) => flag_input(self, node, cell),
            None => (CalcResult::EmptyArg, true),
        };
        let (const_value, const_typed) = flag_arg(2);
        let (stats_value, stats_typed) = flag_arg(3);
        let is_array = |value: &CalcResult| matches!(value, CalcResult::Array(_));
        if !is_array(&const_value) && !is_array(&stats_value) {
            let const_flag = flag_value(self, const_value, true, const_typed, cell);
            let stats_flag = flag_value(self, stats_value, false, stats_typed, cell);
            return regression_output(&data, const_flag, stats_flag, log, cell);
        }
        let (y, x) = &data;
        let cost =
            u64::try_from(y.elements.len().saturating_add(x.elements.len())).unwrap_or(u64::MAX);
        self.lift_values(
            vec![const_value, stats_value],
            LiftCost::Cells(cost),
            &LiftOptions::REUSE_ANY_EQUAL_TOP_LEFT,
            cell,
            &mut |model, values| {
                let const_flag = flag_value(model, values[0].clone(), true, true, cell);
                let stats_flag = flag_value(model, values[1].clone(), false, true, cell);
                regression_output(&data, const_flag, stats_flag, log, cell)
            },
        )
    }

    /// Reads `known_y` and `known_x` (counting 1, 2, 3, ... in y's shape when omitted or empty),
    /// stopping at the first that is an error value.
    fn read_regression_grids(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(Grid, Grid), CalcResult> {
        let y = Grid::from_data(self.read_known_y(&args[0], cell)?);
        let x = match given(args.get(1)) {
            Some(node) => Grid::from_data(self.read_data(node, cell)?),
            None => Grid::counting(y.rows, y.cols),
        };
        Ok((y, x))
    }

    // LINEST(known_y's, [known_x's], [const], [stats])
    pub(crate) fn fn_linest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.linest_or_logest(args, false, cell)
    }

    // LOGEST(known_y's, [known_x's], [const], [stats])
    // Fits y = b * m1^x1 * m2^x2 * ... via linear regression on ln(y).
    pub(crate) fn fn_logest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.linest_or_logest(args, true, cell)
    }
}

/// A const or stats flag: `Err` is an error value, or typed or computed text that is no boolean
/// (step 1 of [`Model::linest_or_logest`]), `Ok(Err)` a text cell that is no boolean (step 4).
type Flag = Result<Result<bool, CalcResult>, CalcResult>;

/// LINEST's or LOGEST's array for one const (steps 1 to 5 of [`Model::linest_or_logest`]).
fn regression_output(
    data: &(Grid, Grid),
    const_term: Flag,
    stats: Flag,
    log: bool,
    cell: CellReferenceIndex,
) -> CalcResult {
    let run = || -> Result<CalcResult, CalcResult> {
        let (y, x) = data;
        let const_term = const_term?;
        let stats = stats?;
        let layout = layout(y, x, cell)?;
        if y.has_error() || x.has_error() {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Regression: error value in known_y or known_x".to_string(),
            ));
        }
        let (const_term, stats) = (const_term?, stats?);
        let known = known_points(y, x, layout, log, cell)?;
        let Some(mut reg) = linear_regression(&known.ys, &known.points, known.p, const_term, stats)
        else {
            return Err(CalcResult::new_error(
                Error::NUM,
                cell,
                "Regression: singular matrix".to_string(),
            ));
        };
        if log {
            for c in &mut reg.coeffs {
                *c = c.exp();
            }
        }
        Ok(build_output_array(reg, stats))
    };
    run().unwrap_or_else(|error| error)
}

/// The value of a const or stats argument, and whether it is typed (true) or the value of one
/// cell it refers to, a LAMBDA's result included (`LAMBDA(acc,Data!B3)(..)` with B3 = " x " waits
/// for the shapes, gen-m9 bbdfc06cae0e0d71 A324).
fn flag_input(model: &mut Model, node: &Node, cell: CellReferenceIndex) -> (CalcResult, bool) {
    match model.evaluate_node_with_reference(node, cell) {
        CalcResult::Range { left, right } if left == right => (model.evaluate_cell(left), false),
        value => (value, true),
    }
}

/// The [`Flag`] of a const or stats value. Text that is no boolean is `#VALUE!`
/// (`LINEST(B2:B9,A2:A9,"x")`, `LINEST(B2:B9,A2:A9,TRUE,"x")`, linest-modes K155, K162;
/// `LINEST({1;2},{1;3},"x")`, probes/t2455-transpose-single-value A7, A26): at once when `typed`
/// (not a cell reference: `LINEST(UNIQUE(Data!D9:E9),Data!B7,TRUE,"1")` and
/// `LOGEST(Data!C6:E9,Data!D$6,1,"")` are `#VALUE!` against unequal shapes, gen-m9
/// a013a45a8fbcab4f A276, fd9fa22d212080d6 A201), else after the shapes.
fn flag_value(
    model: &mut Model,
    value: CalcResult,
    default: bool,
    typed: bool,
    cell: CellReferenceIndex,
) -> Flag {
    match value {
        CalcResult::EmptyArg | CalcResult::EmptyCell => Ok(Ok(default)),
        CalcResult::Boolean(b) => Ok(Ok(b)),
        CalcResult::Number(n) => Ok(Ok(n != 0.0)),
        err @ CalcResult::Error { .. } => Err(err),
        text @ CalcResult::String(_) => match model.cast_to_bool(text, cell) {
            Err(error) if typed => Err(error),
            flag => Ok(flag),
        },
        _ => Ok(Ok(default)),
    }
}

/// Reconstruct internal beta vector [`β_0`, `β_1`, ..., `β_p`] from Excel-order coefficients.
pub(super) fn reconstruct_beta(coeffs: &[f64], const_term: bool, p: usize) -> Vec<f64> {
    // coeffs is: [β_p, β_{p-1}, ..., β_1, β_0] (if const) or [β_p, ..., β_1]
    let mut beta = vec![0.0_f64; p + const_term as usize];
    if const_term {
        // intercept is the last element of coeffs
        beta[0] = *coeffs.last().unwrap_or(&0.0);
        // x coefficients are reversed
        for j in 0..p {
            beta[j + 1] = coeffs[p - 1 - j];
        }
    } else {
        for j in 0..p {
            beta[j] = coeffs[p - 1 - j];
        }
    }
    beta
}

/// Predict y for a single x row given beta in internal order.
pub(super) fn predict(beta: &[f64], x_row: &[f64], const_term: bool) -> f64 {
    let mut pred = if const_term { beta[0] } else { 0.0 };
    let x_start = const_term as usize;
    for (j, &xj) in x_row.iter().enumerate() {
        if x_start + j < beta.len() {
            pred += beta[x_start + j] * xj;
        }
    }
    pred
}
