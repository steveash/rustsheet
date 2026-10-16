//! The `function` argument of GROUPBY and PIVOTBY: one function, or an array of functions with
//! their labels, how many arguments each gets, and the call of one aggregate cell with its rows
//! and relative set (SPEC-0006 Design step 3, T-1325). Excel: docs/excel-semantics/arrays.md
//! "GROUPBY and PIVOTBY", evidence probes/groupby-pivotby-functions and -functions2
//! (excel-graph, 2026-10-16, T-1325).

use std::collections::HashMap;

use crate::{
    calc_result::CalcResult,
    cast::calc_result_to_array_node,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::logical::{as_lambda, FunctionValue, LambdaCaller},
    functions::math_and_trigonometry::percent_of,
    functions::Function,
    language::get_default_language,
    limits::{check_array_size, too_many_visits},
    model::Model,
};

use super::grouping::{at, empty_text, value_error, Relative, SetId};

/// What one aggregate cell is computed from: the rows of its group (or grid cell) and its
/// relative set, the second argument of a two-argument function (SPEC-0006 `Aggregate`).
#[derive(Clone, Copy)]
pub(crate) struct Aggregate<'a> {
    pub(crate) rows: &'a [usize],
    pub(crate) relative: Relative<'a>,
}

/// How a function is called: with the group's values, with the relative set's values too, or,
/// for the built-in PERCENTOF, as the SUM of the group over the SUM of the relative set, the
/// latter computed once per relative set and value column so the work is O(N).
enum Call {
    One,
    Two,
    PercentOf {
        sum: Box<LambdaCaller>,
        totals: HashMap<(SetId, usize), CalcResult>,
    },
}

/// One function of the `function` argument and its label.
struct Aggregator {
    caller: LambdaCaller,
    call: Call,
    label: ArrayNode,
}

/// How several functions are laid out: one function, or an array of them as columns (a row of
/// functions, `HSTACK(SUM,AVERAGE)`, under a row of their labels: G!S121) or as rows (a column,
/// `VSTACK(SUM,MAX)`, each group repeated per function after a label column: G!AB121).
#[derive(Clone, Copy, PartialEq)]
enum Layout {
    Single,
    Columns,
    Rows,
}

/// The output shape the functions give: value columns, rows per group, and the extra label
/// column or label row.
#[derive(Clone, Copy)]
pub(crate) struct Shape {
    pub(crate) columns: usize,
    pub(crate) rows_per_group: usize,
    pub(crate) label_column: bool,
    pub(crate) label_row: bool,
}

/// The functions of a GROUPBY or PIVOTBY call, paired with the value columns.
pub(crate) struct Functions {
    list: Vec<Aggregator>,
    layout: Layout,
    /// (function, value column) of each output value, in output order ([`Functions::pair_with`]).
    pairs: Vec<(usize, usize)>,
    /// The cells the calls have been given (the cell's rows, and the relative set where one is
    /// passed or summed), charged before each call.
    visits: u64,
    /// The cap error once the charges passed the cap: the result is that error.
    refused: Option<CalcResult>,
}

/// One element of the `function` argument ([`Model::function_elements`]).
#[derive(Clone)]
pub(crate) enum Element {
    Function(CalcResult),
    Value(ArrayNode),
}

/// Whether the `function` argument is an array of functions written in place: HSTACK or VSTACK
/// (`HSTACK(SUM,AVERAGE)`, G!S121), or TRANSPOSE of one (functions2 G!A681). An array of functions
/// cannot be a value, so it is read from the formula ([`Model::function_grid`]).
pub(crate) fn is_function_array(arg: &Node) -> bool {
    match arg.ungrouped() {
        Node::FunctionKind {
            kind: Function::Hstack | Function::Vstack,
            ..
        } => true,
        Node::FunctionKind {
            kind: Function::Transpose,
            args,
        } => args.first().is_some_and(is_function_array),
        _ => false,
    }
}

/// How many arguments GROUPBY and PIVOTBY pass to a function passed by name: its minimum argument
/// count when that is 1 or 2, `None` (the whole result `#VALUE!`) otherwise. Two arguments, the
/// values and the relative set: PERCENTOF (G!S97), CORREL, PEARSON, RSQ, COVARIANCE.P (`#N/A` per
/// group, as the sizes differ; in the total, the same array twice: about 1 for CORREL, PEARSON
/// and RSQ, the variance 161.7225 for COVARIANCE.P, functions2 G!B326), and IF, ROUND, SUMIF (`#VALUE!` per cell: an
/// array result); none: NOW, TEXTJOIN, XLOOKUP (probes/groupby-pivotby-functions G!A421,
/// -functions2 G!A281-A501). Other functions whose minimum is not 1 are unprobed and get one.
// TODO(T-1706): every function's minimum argument count (eval/catalog/excel-functions.tsv).
fn eta_arity(kind: &Function) -> Option<usize> {
    match kind {
        Function::Percentof
        | Function::Correl
        | Function::Pearson
        | Function::Rsq
        | Function::CovarianceP
        | Function::If
        | Function::Round
        | Function::Sumif => Some(2),
        Function::Now | Function::Textjoin | Function::Xlookup => None,
        _ => Some(1),
    }
}

/// How many arguments a LAMBDA gets: one when it needs at most one (`LAMBDA([x],1)`, and
/// `LAMBDA(x,[y],ISOMITTED(y))` is TRUE: an optional second parameter is not passed), two when it
/// needs two; three or more required parameters make the whole result `#VALUE!` (G!A381, G!A401,
/// functions2 G!A601). A LAMBDA without parameters: unprobed, `#VALUE!`.
fn lambda_arity(required: usize, total: usize) -> Option<usize> {
    match required {
        0 | 1 if total >= 1 => Some(1),
        2 => Some(2),
        _ => None,
    }
}

/// `#VALUE!` for a `function` argument that is not a function or a valid array of them.
fn not_a_function(cell: CellReferenceIndex, name: &str) -> CalcResult {
    value_error(cell, &format!("{name} expects a function"))
}

impl Functions {
    /// Pairs the functions with `value_columns` value columns: each function with the one value
    /// column, the one function with each value column, or, as many of each, function i with
    /// column i (`HSTACK(SUM,MAX)` over Sales and Units is SUM of Sales and MAX of Units: G!A41,
    /// G!A81, PIVOTBY functions2 P!A1, P!A21); other counts are `#VALUE!` (functions2 G!A1-A61).
    pub(crate) fn pair_with(
        &mut self,
        value_columns: usize,
        cell: CellReferenceIndex,
    ) -> Result<(), CalcResult> {
        let functions = self.list.len();
        self.pairs = if functions == 1 {
            (0..value_columns).map(|column| (0, column)).collect()
        } else if value_columns == 1 {
            (0..functions).map(|function| (function, 0)).collect()
        } else if functions == value_columns {
            (0..functions).map(|i| (i, i)).collect()
        } else {
            return Err(value_error(
                cell,
                "there must be one function, one value column or as many of each",
            ));
        };
        Ok(())
    }

    /// The output shape ([`Shape`]) once paired: a row layout with several functions gives one
    /// value column and a row per function; otherwise one value column per pair.
    pub(crate) fn shape(&self) -> Shape {
        let by_rows = self.layout == Layout::Rows && self.list.len() > 1;
        Shape {
            columns: if by_rows { 1 } else { self.pairs.len() },
            rows_per_group: if by_rows { self.pairs.len() } else { 1 },
            label_column: self.layout == Layout::Rows,
            label_row: self.layout == Layout::Columns,
        }
    }

    /// The pair shown in row `row` (0-based among a group's rows) and value column `column`.
    pub(crate) fn pair_at(&self, row: usize, column: usize) -> usize {
        if self.shape().rows_per_group > 1 {
            row
        } else {
            column
        }
    }

    /// The value column (0-based among the values) shown in output column `column`, for the
    /// header names.
    pub(crate) fn value_column(&self, column: usize) -> usize {
        self.pairs
            .get(self.pair_at(0, column))
            .map_or(0, |&(_, value)| value)
    }

    /// The label of the function shown in pair `pair`.
    pub(crate) fn label(&self, pair: usize) -> ArrayNode {
        self.pairs
            .get(pair)
            .and_then(|&(function, _)| self.list.get(function))
            .map_or_else(empty_text, |f| f.label.clone())
    }

    /// The label row of a column layout: `""` over the `key_columns` keys, then each value
    /// column's function label (G!S121, PIVOTBY P!A121 per column group).
    pub(crate) fn label_cells(&self, columns: impl Iterator<Item = usize>) -> Vec<ArrayNode> {
        columns
            .map(|column| self.label(self.pair_at(0, column)))
            .collect()
    }

    /// Whether detected headers are shown, as `field_headers` 3 shows them: with a row of
    /// several functions (`HSTACK(SUM,MAX)` over Data!A1:A11: functions2 G!A641, functions3
    /// P!A1), not with one function, a column of them or labelled ones (G!J1; functions3 G!A1,
    /// G!A61, G!A81, P!A31).
    pub(crate) fn shows_detected_headers(&self) -> bool {
        self.layout == Layout::Columns && self.list.len() > 1
    }

    /// Whether PIVOTBY's Total columns are `""`: one function over several value columns (P!Y97);
    /// several functions fill them, paired with several values too (functions2 P!A1).
    pub(crate) fn blank_totals(&self, value_columns: usize) -> bool {
        self.list.len() == 1 && value_columns > 1
    }

    /// Whether any function takes the relative set, so PIVOTBY builds the sets.
    pub(crate) fn needs_relative(&self) -> bool {
        self.list
            .iter()
            .any(|function| !matches!(function.call, Call::One))
    }

    /// The cap error if a two-parameter LAMBDA call was refused: the whole result is that error.
    pub(crate) fn refused(&mut self) -> Option<CalcResult> {
        self.refused.take()
    }
}

impl Model {
    /// The evaluated `function` argument, `arg`: an array of functions read from the formula
    /// ([`is_function_array`]), or the argument's value, a one-cell reference read as its value.
    /// An error value is returned as the error, before the later options are evaluated, as
    /// [`Model::evaluate_options`] orders them.
    pub(crate) fn function_elements(
        &mut self,
        arg: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<Element>>, CalcResult> {
        if is_function_array(arg) {
            return self.function_grid(arg, cell);
        }
        let value = self
            .evaluate_options(std::slice::from_ref(arg), cell)?
            .pop()
            .flatten()
            .unwrap_or(CalcResult::EmptyArg);
        // The "is it a LAMBDA" check BYROW and BYCOL share; anything else is not a function.
        Ok(vec![vec![match as_lambda(value) {
            Ok(lambda) => Element::Function(lambda),
            Err(_) => Element::Value(ArrayNode::Empty),
        }]])
    }

    /// The elements of an array of functions written with HSTACK, VSTACK and TRANSPOSE, stacked as
    /// those functions stack values (a shorter part padded with `#N/A`, which then is the
    /// result: unprobed). An error value in it is the result (`HSTACK(SUM,NA())` is `#N/A`: G!A781,
    /// functions2 G!A701).
    fn function_grid(
        &mut self,
        node: &Node,
        cell: CellReferenceIndex,
    ) -> Result<Vec<Vec<Element>>, CalcResult> {
        let (kind, args) = match node.ungrouped() {
            Node::FunctionKind { kind, args } if is_function_array(node) => (kind, args),
            other => {
                let value = self.evaluate_node_in_context(other, cell);
                return match value {
                    lambda @ CalcResult::Lambda(_) => Ok(vec![vec![Element::Function(lambda)]]),
                    error @ CalcResult::Error { .. } => Err(error),
                    value => Ok(self
                        .result_to_array(value, cell)?
                        .into_iter()
                        .map(|row| row.into_iter().map(Element::Value).collect())
                        .collect()),
                };
            }
        };
        let mut parts = Vec::with_capacity(args.len());
        for arg in args {
            parts.push(self.function_grid(arg, cell)?);
        }
        // The stacked size is checked before anything is padded or copied (directive 5):
        // `VSTACK(SEQUENCE(1048576),SEQUENCE(1,16384))` would pad to 2^34 cells.
        let mut dims = parts
            .iter()
            .map(|part| (part.len(), part.first().map_or(0, Vec::len)));
        let (height, width) = match kind {
            Function::Transpose => dims.next_back().map_or((0, 0), |(h, w)| (w, h)),
            Function::Hstack => dims.fold((0usize, 0usize), |(h, w), (ph, pw)| {
                (h.max(ph), w.saturating_add(pw))
            }),
            _ => dims.fold((0usize, 0usize), |(h, w), (ph, pw)| {
                (h.saturating_add(ph), w.max(pw))
            }),
        };
        if let Some((error, message)) = check_array_size(height, width) {
            return Err(CalcResult::new_error(error, cell, message));
        }
        let grid = match kind {
            Function::Transpose => transpose(parts.pop().unwrap_or_default()),
            Function::Hstack => {
                let height = parts.iter().map(Vec::len).max().unwrap_or(0);
                (0..height)
                    .map(|row| {
                        parts
                            .iter_mut()
                            .flat_map(|part| {
                                let width = part.first().map_or(0, Vec::len);
                                let cells = part.get_mut(row).map(std::mem::take);
                                cells.unwrap_or_else(|| {
                                    (0..width).map(|_| not_available()).collect()
                                })
                            })
                            .collect()
                    })
                    .collect()
            }
            _ => {
                let width = parts
                    .iter()
                    .map(|part| part.first().map_or(0, Vec::len))
                    .max()
                    .unwrap_or(0);
                parts
                    .into_iter()
                    .flatten()
                    .map(|mut row| {
                        row.resize_with(width, not_available);
                        row
                    })
                    .collect()
            }
        };
        Ok(grid)
    }

    /// The functions of the `function` argument of GROUPBY or PIVOTBY (`name`), as
    /// [`Model::function_elements`] read it. A grid
    /// of one function is that function; a row or a column of functions is laid out as columns or
    /// as rows, labelled with their names ("SUM", "STDEV.S", "CUSTOM" for a LAMBDA: G!A181,
    /// G!A521); functions in the first column with values beside them are rows labelled with those
    /// values (`HSTACK(SUM,5)` is a label column of 5, a third column ignored: G!A221, functions2
    /// G!A121-A181), and functions in the first row with values below them are columns labelled
    /// with those (`VSTACK(SUM,5)`: functions2 G!A161). Anything else is `#VALUE!`: a value, an
    /// array constant, a 2-D array of functions, a value first (G!A201, G!A261, functions2 G!A101;
    /// probes/groupby-lambda-forms G!A217). So is a function that takes no arguments or more than
    /// two ([`eta_arity`], [`lambda_arity`]).
    pub(crate) fn function_argument(
        &mut self,
        function: &[Vec<Element>],
        name: &str,
        cell: CellReferenceIndex,
    ) -> Result<Functions, CalcResult> {
        let invalid = || not_a_function(cell, name);
        let is_function = |element: &Element| matches!(element, Element::Function(_));
        let height = function.len();
        let width = function.first().map_or(0, Vec::len);
        let column = |index: usize| function.iter().filter_map(move |row| row.get(index));
        let all_functions = function.iter().flatten().all(is_function);
        // (functions, labels given, layout)
        let (functions, labels, layout): (Vec<&Element>, Option<Vec<&Element>>, Layout) =
            if all_functions && height == 1 && width == 1 {
                (column(0).collect(), None, Layout::Single)
            } else if all_functions && height == 1 {
                (function.iter().flatten().collect(), None, Layout::Columns)
            } else if all_functions && width == 1 {
                (column(0).collect(), None, Layout::Rows)
            } else if width >= 2 && column(0).all(is_function) && !column(1).any(is_function) {
                (column(0).collect(), Some(column(1).collect()), Layout::Rows)
            } else if height >= 2
                && function
                    .first()
                    .is_some_and(|row| row.iter().all(is_function))
                && !function
                    .get(1)
                    .is_some_and(|row| row.iter().any(is_function))
            {
                let first = function.first().map(|row| row.iter().collect());
                let second = function.get(1).map(|row| row.iter().collect());
                (first.unwrap_or_default(), second, Layout::Columns)
            } else {
                return Err(invalid());
            };
        if functions.is_empty() {
            return Err(invalid());
        }
        let mut list = Vec::with_capacity(functions.len());
        for (index, element) in functions.into_iter().enumerate() {
            let Element::Function(lambda) = element else {
                return Err(invalid());
            };
            let (arity, default_label, percentof) = match self.function_value(lambda) {
                Some(FunctionValue::Eta(kind)) => (
                    eta_arity(&kind),
                    ArrayNode::String(kind.to_localized_name(get_default_language()).into()),
                    kind == Function::Percentof,
                ),
                Some(FunctionValue::Lambda { required, total }) => (
                    lambda_arity(required, total),
                    ArrayNode::String("CUSTOM".into()),
                    false,
                ),
                None => return Err(invalid()),
            };
            let Some(arity) = arity else {
                return Err(value_error(
                    cell,
                    &format!("{name} calls a function with one or two arguments"),
                ));
            };
            let call = if percentof {
                let sum = self.eta_lambda(Function::Sum);
                Call::PercentOf {
                    sum: Box::new(self.lambda_caller(&sum, 1)),
                    totals: HashMap::new(),
                }
            } else if arity == 2 {
                Call::Two
            } else {
                Call::One
            };
            let label = match labels.as_ref().and_then(|labels| labels.get(index)) {
                Some(Element::Value(value)) => value.clone(),
                _ => default_label,
            };
            list.push(Aggregator {
                caller: self.lambda_caller(lambda, arity).keeping_references(),
                call,
                label,
            });
        }
        Ok(Functions {
            list,
            layout,
            pairs: Vec::new(),
            visits: 0,
            refused: None,
        })
    }

    /// The cells right of the keys of one group's rows: the label (a row layout) and each output
    /// value column's aggregate. `reuse` is the value-sorted column and the group's value there,
    /// shown instead of calling the function again (the first row's only).
    pub(crate) fn value_rows(
        &mut self,
        functions: &mut Functions,
        values: &[Vec<ArrayNode>],
        at: Aggregate<'_>,
        reuse: Option<(usize, Option<&ArrayNode>)>,
        cell: CellReferenceIndex,
    ) -> Vec<Vec<ArrayNode>> {
        let shape = functions.shape();
        (0..shape.rows_per_group)
            .map(|sub_row| {
                let mut row = Vec::with_capacity(shape.columns + 1);
                if shape.label_column {
                    row.push(functions.label(functions.pair_at(sub_row, 0)));
                }
                for column in 0..shape.columns {
                    let value = match reuse {
                        Some((by, Some(value))) if by == column && sub_row == 0 => value.clone(),
                        _ => {
                            let pair = functions.pair_at(sub_row, column);
                            self.aggregate(functions, pair, values, at, cell)
                        }
                    };
                    row.push(value);
                }
                row
            })
            .collect()
    }

    /// The function of pair `pair` ([`Functions::pair_with`]) over the values of its column in
    /// `cell_rows.rows`, passed as one column array in source order (ARRAYTOTEXT gives "Apple, Pear,
    /// Apple", G!AB97), as values, not a reference (ISREF is FALSE), with blank cells as blanks
    /// (ISBLANK is TRUE, COUNTA counts them: probes/groupby-lambda-forms G!S217, G!A241,
    /// probes/groupby-blank-values). A two-argument function also gets the relative set's values
    /// that way (ISREF FALSE, source order: G!A341, G!A441). An array result is `#VALUE!`, a 1x1
    /// array too (G!J121, one-record groups included), and so is a reference, also one picked by IF
    /// or INDEX (`LAMBDA(x,Data!D2)`: groupby-lambda-forms G!J193; G!A541-A581): each cell on its
    /// own, not the whole result.
    ///
    /// Every call is charged the cells it is given (the cell's rows, and the relative set a
    /// two-parameter LAMBDA gets or PERCENTOF sums) before anything is built; past the cap no
    /// further call is made and the whole result is the cap error ([`Functions::refused`];
    /// directive 5, DEVIATION in arrays.md).
    pub(crate) fn aggregate(
        &mut self,
        functions: &mut Functions,
        pair: usize,
        values: &[Vec<ArrayNode>],
        cell_rows: Aggregate<'_>,
        cell: CellReferenceIndex,
    ) -> ArrayNode {
        let Some(&(function, column)) = functions.pairs.get(pair) else {
            return ArrayNode::Error(Error::VALUE);
        };
        if functions.refused.is_some() {
            return ArrayNode::Error(Error::ERROR);
        }
        let relative = cell_rows.relative;
        let (rows, all) = (cell_rows.rows.len() as u64, relative.rows.len() as u64);
        let cost = match functions.list.get(function).map(|f| &f.call) {
            Some(Call::One) => rows,
            Some(Call::Two) => rows.saturating_add(all),
            Some(Call::PercentOf { totals, .. }) => {
                let cached = relative
                    .id
                    .is_some_and(|id| totals.contains_key(&(id, column)));
                rows.saturating_add(if cached { 0 } else { all })
            }
            None => return ArrayNode::Error(Error::VALUE),
        };
        let visits = functions.visits.saturating_add(cost);
        if let Some(error) = too_many_visits(visits, cell) {
            functions.refused = Some(error);
            return ArrayNode::Error(Error::ERROR);
        }
        functions.visits = visits;
        let column_of = |rows: &[usize]| {
            CalcResult::Array(
                rows.iter()
                    .map(|&row| vec![at(values, row, column)])
                    .collect(),
            )
        };
        let input = column_of(cell_rows.rows);
        let Some(aggregator) = functions.list.get_mut(function) else {
            return ArrayNode::Error(Error::VALUE);
        };
        let result = match &mut aggregator.call {
            Call::One => self.call_lambda_caller(&mut aggregator.caller, [input], cell),
            Call::Two => {
                let all = column_of(relative.rows);
                self.call_lambda_caller(&mut aggregator.caller, [input, all], cell)
            }
            Call::PercentOf { sum, totals } => {
                let part = self.call_lambda_caller(sum, [input], cell);
                percent_of(
                    part,
                    || {
                        let key = relative.id.map(|id| (id, column));
                        if let Some(total) = key.and_then(|key| totals.get(&key)) {
                            return total.clone();
                        }
                        let total = self.call_lambda_caller(sum, [column_of(relative.rows)], cell);
                        if let Some(key) = key {
                            totals.insert(key, total.clone());
                        }
                        total
                    },
                    cell,
                )
            }
        };
        calc_result_to_array_node(result)
    }
}

/// `#N/A`, the padding of a shorter part of a stacked array.
fn not_available() -> Element {
    Element::Value(ArrayNode::Error(Error::NA))
}

/// The grid with rows and columns swapped.
fn transpose(grid: Vec<Vec<Element>>) -> Vec<Vec<Element>> {
    let width = grid.first().map_or(0, Vec::len);
    let mut columns: Vec<Vec<Element>> =
        (0..width).map(|_| Vec::with_capacity(grid.len())).collect();
    for row in grid {
        for (column, element) in columns.iter_mut().zip(row) {
            column.push(element);
        }
    }
    columns
}
