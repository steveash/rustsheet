use crate::expressions::parser::ArrayNode;
use crate::expressions::types::CellReferenceIndex;
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

// ─── Data collection ─────────────────────────────────────────────────────────

struct RegressionData {
    /// Y values (n × 1)
    y: Vec<f64>,
    /// X matrix (n rows × p cols); p = 0 if auto-generated {1..n}
    x: Vec<Vec<f64>>,
    /// Number of X columns
    p: usize,
}

/// One element of a regression data argument as [`Model::read_data`] read it.
pub(super) enum Element {
    Number(f64),
    /// An empty cell, or an empty argument or array element.
    Blank,
    /// An error cell or element, or a lambda in an array.
    Error(CalcResult),
    /// Text, a boolean or a lambda.
    NotNumber,
}

/// A regression data argument, element by element, with where it came from: a blank cell of a
/// range is not an empty element of an array to LINEST.
pub(super) enum DataRead {
    Range(Vec<Vec<Element>>),
    Array(Vec<Vec<Element>>),
    Scalar(Element),
}

/// A data argument as [`Model::read_matrix`] read it, with the errors it found kept for later.
#[derive(Default)]
struct MatrixRead {
    values: Vec<Vec<f64>>,
    /// The first error element, lambda or non-numeric single value: reported before the sizes.
    early_error: Option<CalcResult>,
    /// The first text or boolean cell of a range: reported after the sizes.
    text_error: Option<CalcResult>,
}

/// `known_y` and `known_x` as read; `known_x` `None` if omitted.
struct RegressionInputs {
    y: MatrixRead,
    x: Option<MatrixRead>,
}

/// The checks after the read, in the order Excel makes them (statistics.md "LINEST and LOGEST
/// report error values first"):
/// 1. y's error element, lambda or non-numeric single value;
/// 2. too few y values (`#VALUE!`);
/// 3. x's error element, lambda or non-numeric single value;
/// 4. unequal rows (`#REF!`);
/// 5. a text cell. It comes after the size check: GROWTH(B2:B3,<number>) with B2:B3 = "?b", 0
///    is `#REF!`, not `#VALUE!` (gen-m9/89693ab0cafc29df A24).
///
/// LINEST and LOGEST report const's and stats' errors before all of these: a boolean y cell
/// waits (`LINEST(Data!C8,PERCENTILE.INC(0.5,0.975),"FALSE",NA())`, C8 = TRUE, is `#N/A`, c6cfda6bd10cae93
/// A110), and so does an error element of x (35e8168aae2b00f8 A400).
fn validate_regression_inputs(
    inputs: RegressionInputs,
    min_points: usize,
    cell: CellReferenceIndex,
) -> Result<RegressionData, CalcResult> {
    let MatrixRead {
        values: y_raw,
        early_error,
        mut text_error,
    } = inputs.y;
    if let Some(error) = early_error {
        return Err(error);
    }
    let y: Vec<f64> = y_raw.into_iter().flatten().collect();
    let n = y.len();

    if n < min_points {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            format!("Regression requires at least {min_points} data points"),
        ));
    }

    let (x, p) = match inputs.x {
        None => {
            // Auto-generate x = 1, 2, ..., n (single column)
            let x_auto: Vec<Vec<f64>> = (1..=n).map(|i| vec![i as f64]).collect();
            (x_auto, 1)
        }
        Some(MatrixRead {
            values: x_mat,
            early_error,
            text_error: x_text_error,
        }) => {
            if let Some(error) = early_error {
                return Err(error);
            }
            text_error = text_error.or(x_text_error);
            let rows = x_mat.len();
            let p = if rows > 0 { x_mat[0].len() } else { 0 };

            if rows != n {
                return Err(CalcResult::new_error(
                    Error::REF,
                    cell,
                    "Regression: known_y and known_x must have the same number of rows".to_string(),
                ));
            }
            (x_mat, p)
        }
    };
    if let Some(e) = text_error {
        return Err(e);
    }

    Ok(RegressionData { y, x, p })
}

impl Model {
    /// Reads `known_y` and `known_x`, failing only when one is an error value itself, y's first:
    /// [`validate_regression_inputs`] does every other check, so LINEST and LOGEST can report an
    /// error in const or stats between the two.
    fn read_regression_inputs(
        &mut self,
        y_arg: &Node,
        x_arg: Option<&Node>,
        cell: CellReferenceIndex,
    ) -> Result<RegressionInputs, CalcResult> {
        let y = self.read_known_y(y_arg, cell)?;
        let y = matrix_from_data(y, cell);
        let x = match x_arg {
            Some(x_node) => Some(matrix_from_data(self.read_data(x_node, cell)?, cell)),
            None => None,
        };
        Ok(RegressionInputs { y, x })
    }

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
            && !matches!(y_arg, Node::ReferenceKind { .. })
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
                            err @ CalcResult::Error { .. } => Element::Error(err),
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
                                ArrayNode::Error(e) => {
                                    Element::Error(CalcResult::new_error(e, cell, String::new()))
                                }
                                ArrayNode::Lambda(_) => {
                                    Element::Error(Model::lambda_as_number_error(cell))
                                }
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

/// A data argument as [`Model::read_data`] read it, as a matrix for LINEST and LOGEST. What is
/// wrong with it is kept for the caller to report in its own order: an error element of a range or
/// array, a lambda, or a single value that is no number (`early_error`), and a text or boolean
/// cell in a range (`text_error`, reported after the size checks). A blank cell of a range and
/// a text or boolean element of an array read as 0.
fn matrix_from_data(data: DataRead, cell: CellReferenceIndex) -> MatrixRead {
    let mut read = MatrixRead::default();
    match data {
        DataRead::Range(rows) => {
            for row in rows {
                let mut mrow = Vec::with_capacity(row.len());
                for element in row {
                    mrow.push(match element {
                        Element::Number(v) => v,
                        Element::Blank => 0.0,
                        Element::Error(err) => {
                            read.early_error.get_or_insert(err);
                            0.0
                        }
                        Element::NotNumber => {
                            read.text_error.get_or_insert_with(|| {
                                CalcResult::new_error(
                                    Error::VALUE,
                                    cell,
                                    "Regression: non-numeric value in data range".to_string(),
                                )
                            });
                            0.0
                        }
                    });
                }
                read.values.push(mrow);
            }
        }
        DataRead::Array(rows) => {
            for row in rows {
                let mut mrow = Vec::with_capacity(row.len());
                for element in row {
                    mrow.push(match element {
                        Element::Number(v) => v,
                        Element::Error(err) => {
                            read.early_error.get_or_insert(err);
                            0.0
                        }
                        Element::Blank | Element::NotNumber => 0.0,
                    });
                }
                read.values.push(mrow);
            }
        }
        DataRead::Scalar(Element::Number(n)) => read.values = vec![vec![n]],
        DataRead::Scalar(Element::Blank) => {}
        DataRead::Scalar(Element::Error(err)) => read.early_error = Some(err),
        DataRead::Scalar(Element::NotNumber) => {
            read.early_error = Some(CalcResult::new_error(
                Error::VALUE,
                cell,
                "Expected numeric range or array".to_string(),
            ));
        }
    }
    read
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

/// Core linear regression for any number of predictors.
/// Design matrix columns: [1, `x_1`, `x_2`, ..., `x_p`] (or [`x_1`, ..., `x_p`] if !`const_term`).
/// Returns coefficients in internal order: [`β_0`, `β_1`, ..., `β_p`] (intercept first if const).
pub(super) fn linear_regression(
    y: &[f64],
    x: &[Vec<f64>],
    p: usize,
    const_term: bool,
    compute_stats: bool,
) -> Option<RegResult> {
    let n = y.len();
    let k = p + const_term as usize; // number of parameters

    if n < k || k == 0 {
        return None;
    }

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

    let (mean_y, ss_tot) = if const_term {
        let mean = y.iter().sum::<f64>() / n as f64;
        let ss_tot: f64 = y.iter().map(|yi| (yi - mean).powi(2)).sum();
        (mean, ss_tot)
    } else {
        let ss_tot: f64 = y.iter().map(|yi| yi.powi(2)).sum();
        (0.0, ss_tot)
    };

    let ss_reg = ss_tot - ss_res;
    let r_sq = if ss_tot > 0.0 {
        1.0 - ss_res / ss_tot
    } else {
        1.0
    };
    let r_sq = r_sq.clamp(0.0, 1.0);

    let se_y = if df > 0.0 {
        (ss_res / df).sqrt()
    } else {
        f64::NAN
    };

    let df_model = p as f64; // number of x predictors (not counting intercept)
    let f_stat = if df > 0.0 && df_model > 0.0 {
        (ss_reg / df_model) / (ss_res / df)
    } else {
        f64::NAN
    };

    // Standard errors: sqrt(se_y² · (D'D)^{-1}_{j,j})
    let se_beta: Vec<f64> = {
        let inv = invert_matrix(&dtd)?;
        let var_y = if df > 0.0 { ss_res / df } else { f64::NAN };
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

    let _ = mean_y;

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
        Some(f) if !f.is_nan() => ArrayNode::Number(f),
        _ => ArrayNode::Error(Error::NA),
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
    /// LINEST's and LOGEST's arguments: the data, const and stats. Error values come first, in
    /// argument order, then the data checks: `LINEST({1;2},1/0,"x")` and `LINEST(1/0,{1;3},"x")`
    /// are `#DIV/0!` (probes/t2455-transpose-single-value A4:A27), while
    /// `LINEST(Data!C3:C7,Data!B5:B10,NA(),"1")` is `#N/A`, not the size `#REF!` (gen-m9
    /// 05d44f6dd6cfc567 A195; statistics.md "LINEST and LOGEST report error values first").
    fn regression_fit_args(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> Result<(RegressionData, bool, bool), CalcResult> {
        let inputs = self.read_regression_inputs(&args[0], args.get(1), cell)?;
        let const_term = eval_optional_bool(self, args.get(2), cell, true)?;
        let stats = eval_optional_bool(self, args.get(3), cell, false)?;
        Ok((
            validate_regression_inputs(inputs, 2, cell)?,
            const_term,
            stats,
        ))
    }

    // LINEST(known_y's, [known_x's], [const], [stats])
    pub(crate) fn fn_linest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }

        let (data, const_term, stats) = match self.regression_fit_args(args, cell) {
            Ok(fit_args) => fit_args,
            Err(e) => return e,
        };

        match linear_regression(&data.y, &data.x, data.p, const_term, stats) {
            Some(reg) => build_output_array(reg, stats),
            None => CalcResult::new_error(Error::NUM, cell, "LINEST: singular matrix".to_string()),
        }
    }

    // LOGEST(known_y's, [known_x's], [const], [stats])
    // Fits y = b * m1^x1 * m2^x2 * ... via linear regression on ln(y).
    pub(crate) fn fn_logest(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if !(1..=4).contains(&args.len()) {
            return CalcResult::new_args_number_error(cell);
        }

        let (mut data, const_term, stats) = match self.regression_fit_args(args, cell) {
            Ok(fit_args) => fit_args,
            Err(e) => return e,
        };

        // Transform y → ln(y); return #NUM! if any y ≤ 0
        for yi in &data.y {
            if *yi <= 0.0 {
                return CalcResult::new_error(
                    Error::NUM,
                    cell,
                    "LOGEST: all known_y values must be positive".to_string(),
                );
            }
        }
        data.y = data.y.iter().map(|&v| v.ln()).collect();

        match linear_regression(&data.y, &data.x, data.p, const_term, stats) {
            Some(mut reg) => {
                // Transform coefficients: exp(each)
                for c in &mut reg.coeffs {
                    *c = c.exp();
                }
                // Stats rows (rows 1-4) are about the log-linear fit, not transformed
                build_output_array(reg, stats)
            }
            None => CalcResult::new_error(Error::NUM, cell, "LOGEST: singular matrix".to_string()),
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Evaluate an optional boolean argument, returning `default` when the argument
/// is absent or an empty cell/arg (which Excel treats as omitted).
fn eval_optional_bool(
    model: &mut Model,
    arg: Option<&Node>,
    cell: CellReferenceIndex,
    default: bool,
) -> Result<bool, CalcResult> {
    let node = match arg {
        None => return Ok(default),
        Some(n) => n,
    };
    match model.evaluate_node_in_context(node, cell) {
        CalcResult::EmptyArg | CalcResult::EmptyCell => Ok(default),
        CalcResult::Boolean(b) => Ok(b),
        CalcResult::Number(n) => Ok(n != 0.0),
        err @ CalcResult::Error { .. } => Err(err),
        _ => Ok(default),
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
