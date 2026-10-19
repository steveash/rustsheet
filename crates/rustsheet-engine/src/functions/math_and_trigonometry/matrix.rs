use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::lift::{number_or_error, LiftCost, LiftOptions, LiftedArg},
    limits::check_array_size,
    model::Model,
};

/// MUNIT's dimension: the number truncated, `#VALUE!` below 1.
fn munit_dimension(f: f64) -> Result<usize, Error> {
    let n = f.trunc();
    if n.is_nan() || n < 1.0 {
        Err(Error::VALUE)
    } else {
        // A saturating cast: a huge dimension is rejected by the size check.
        Ok(n as usize)
    }
}

fn coerce_to_f64(node: &ArrayNode, cell: CellReferenceIndex) -> Result<f64, CalcResult> {
    match node {
        ArrayNode::Number(v) => Ok(*v),
        // Excel: a blank cell anywhere in the matrix is #VALUE! (poi/MatrixFormulaEvalTestData.xlsx),
        // and so is a boolean, as a cell, a literal or an element
        // (probes/matrix-boolean-elements, docs/excel-semantics/arrays.md).
        ArrayNode::Boolean(_) | ArrayNode::Empty | ArrayNode::String(_) => {
            Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "matrix requires numeric values".to_string(),
            ))
        }
        ArrayNode::Error(e) => Err(CalcResult::new_error(
            e.clone(),
            cell,
            "matrix received an error value".to_string(),
        )),
    }
}

/// Coerce a 2-D `ArrayNode` grid into a flat Vec<f64> in row-major order,
/// returning the size n (asserts square).
fn to_square_matrix(
    arr: &[Vec<ArrayNode>],
    cell: CellReferenceIndex,
) -> Result<(usize, Vec<f64>), CalcResult> {
    let n = arr.len();
    if n == 0 {
        return Err(CalcResult::new_error(
            Error::VALUE,
            cell,
            "matrix must be non-empty".to_string(),
        ));
    }
    for row in arr {
        if row.len() != n {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "MDETERM/MINVERSE requires a square matrix".to_string(),
            ));
        }
    }
    let mut mat = vec![0.0f64; n * n];
    for (i, row) in arr.iter().enumerate() {
        for (j, node) in row.iter().enumerate() {
            mat[i * n + j] = coerce_to_f64(node, cell)?;
        }
    }
    Ok((n, mat))
}

/// LU decomposition with partial pivoting (in-place).
/// Returns (sign, swaps) where swaps[k] is the row swapped with row k at step k.
/// After the call `mat` holds the LU factors (L below diagonal, U on/above diagonal).
fn lu_decompose(mat: &mut [f64], n: usize) -> Result<(f64, Vec<usize>), ()> {
    let mut swaps: Vec<usize> = vec![0; n];
    let mut sign = 1.0f64;

    for col in 0..n {
        let mut max_val = mat[col * n + col].abs();
        let mut max_row = col;
        for row in (col + 1)..n {
            let v = mat[row * n + col].abs();
            if v > max_val {
                max_val = v;
                max_row = row;
            }
        }
        if max_val < f64::EPSILON {
            return Err(());
        }
        swaps[col] = max_row;
        if max_row != col {
            for k in 0..n {
                mat.swap(col * n + k, max_row * n + k);
            }
            sign = -sign;
        }
        let pivot = mat[col * n + col];
        for row in (col + 1)..n {
            mat[row * n + col] /= pivot;
            for k in (col + 1)..n {
                let tmp = mat[col * n + k];
                mat[row * n + k] -= mat[row * n + col] * tmp;
            }
        }
    }
    Ok((sign, swaps))
}

impl Model {
    // ── MDETERM ───────────────────────────────────────────────────────────────

    /// `=MDETERM(array)`
    ///
    /// Returns the determinant of a square matrix using LU decomposition
    /// with partial pivoting.
    pub(crate) fn fn_mdeterm(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let arr = match self.eval_to_array(&args[0], cell) {
            Ok(a) => a,
            Err(e) => return e,
        };
        let (n, mut mat) = match to_square_matrix(&arr, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        match lu_decompose(&mut mat, n) {
            Err(()) => CalcResult::Number(0.0),
            Ok((sign, _)) => {
                let mut det = sign;
                for i in 0..n {
                    det *= mat[i * n + i];
                }
                CalcResult::Number(det)
            }
        }
    }

    // ── MINVERSE ──────────────────────────────────────────────────────────────

    /// `=MINVERSE(array)`
    ///
    /// Returns the inverse of a square matrix via LU decomposition.
    /// Returns `#VALUE!` for a singular matrix.
    pub(crate) fn fn_minverse(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        if args.len() != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let arr = match self.eval_to_array(&args[0], cell) {
            Ok(a) => a,
            Err(e) => return e,
        };
        let (n, mut mat) = match to_square_matrix(&arr, cell) {
            Ok(v) => v,
            Err(e) => return e,
        };

        let (_, swaps) = match lu_decompose(&mut mat, n) {
            Ok(v) => v,
            Err(()) => {
                return CalcResult::new_error(Error::NUM, cell, "matrix is singular".to_string())
            }
        };

        // Solve A*X = I column by column: apply P, then forward/back substitution.
        let mut inv = vec![0.0f64; n * n];
        let mut col_vec = vec![0.0f64; n];
        for col in 0..n {
            // Build identity column, then apply the same row swaps (P * e_col).
            col_vec.fill(0.0);
            col_vec[col] = 1.0;
            for (i, &s) in swaps.iter().enumerate() {
                col_vec.swap(i, s);
            }
            // Forward substitution (L * y = P*e_col); L has implicit 1s on diagonal.
            for i in 1..n {
                for k in 0..i {
                    col_vec[i] -= mat[i * n + k] * col_vec[k];
                }
            }
            // Back substitution (U * x = y).
            for i in (0..n).rev() {
                for k in (i + 1)..n {
                    col_vec[i] -= mat[i * n + k] * col_vec[k];
                }
                col_vec[i] /= mat[i * n + i];
            }
            for row in 0..n {
                inv[row * n + col] = col_vec[row];
            }
        }

        let result: Vec<Vec<ArrayNode>> = (0..n)
            .map(|i| (0..n).map(|j| ArrayNode::Number(inv[i * n + j])).collect())
            .collect();
        CalcResult::Array(result)
    }

    // ── MUNIT ─────────────────────────────────────────────────────────────────

    /// `=MUNIT(dimension)`
    ///
    /// Returns the identity matrix of the given size. Not a lifting-table entry: its scalar
    /// result is a matrix, so its static shape is unknown, but its argument is read as a `Lift`
    /// one (`read_lift_arg`; a 3-D reference is `#REF!`, probes/date-math-lift-driver-edges G29).
    pub(crate) fn fn_munit(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        let [arg] = args else {
            return CalcResult::new_args_number_error(cell);
        };
        let value = match self.read_lift_arg(arg, cell) {
            Ok(LiftedArg::Scalar(value)) => value,
            // An array or range argument lifts: each element becomes the 1 at the top left of its
            // own identity matrix, or the error it would give alone (probes/radians-sqrtpi-munit,
            // date-math-lift-driver-edges G33-G35, docs/excel-semantics/numbers.md "RADIANS,
            // SQRTPI and MUNIT", T-1282).
            Ok(LiftedArg::Grid(grid)) => {
                return self.lift_values(
                    vec![CalcResult::Array(grid)],
                    LiftCost::Unit,
                    &LiftOptions::EVERY_ELEMENT,
                    cell,
                    &mut |model, values| {
                        let dimension = match values {
                            [value] => model.kernel_number(value, cell),
                            _ => return CalcResult::new_args_number_error(cell),
                        };
                        let one = dimension.and_then(|f| {
                            let n = munit_dimension(f).map_err(|error| {
                                CalcResult::new_error(error, cell, String::new())
                            })?;
                            match check_array_size(n, n) {
                                Some((error, _)) => {
                                    Err(CalcResult::new_error(error, cell, String::new()))
                                }
                                None => Ok(1.0),
                            }
                        });
                        number_or_error(one)
                    },
                );
            }
            Err(error) => return error,
        };
        let n = match self.kernel_number(&value, cell).and_then(|f| {
            munit_dimension(f).map_err(|error| {
                CalcResult::new_error(error, cell, "MUNIT requires a positive integer".to_string())
            })
        }) {
            Ok(n) => n,
            Err(error) => return error,
        };
        if let Some((error, message)) = check_array_size(n, n) {
            return CalcResult::new_error(error, cell, message);
        }
        let result: Vec<Vec<ArrayNode>> = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| ArrayNode::Number(if i == j { 1.0 } else { 0.0 }))
                    .collect()
            })
            .collect();
        CalcResult::Array(result)
    }
}
