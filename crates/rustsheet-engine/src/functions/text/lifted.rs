//! Lifting for the text functions that take lists (TEXTSPLIT, TEXTBEFORE, TEXTAFTER) or a
//! pattern (REGEX*) next to scalar arguments: the scalar arguments lift over a range or an
//! array, one call per element, while the list arguments are taken whole (docs/excel-semantics/
//! arrays.md "Dynamic-array and LAMBDA function argument modes"; probes/m7-argument-modes).

use crate::{
    calc_result::CalcResult,
    expressions::{
        parser::{ArrayNode, Node},
        token::Error,
        types::CellReferenceIndex,
    },
    functions::{
        lift::{grid_dims, LiftCost, LiftOptions, LiftedArg},
        util::array_node_to_calc_result,
    },
    limits::too_many_visits,
    model::Model,
};

/// Bytes of delimiter comparison counted as one cell visit when a position is compared with a
/// delimiter by `starts_with` (a `memcmp`, TEXTBEFORE and TEXTAFTER): about 25 ps a byte in a
/// release build against a few nanoseconds a visit (T-1272 performance review).
pub(super) const COMPARED_BYTES_PER_VISIT: u64 = 64;

/// The same when a delimiter is searched for from each piece (TEXTSPLIT's `memmem` finders): a
/// delimiter that matches at every position is verified at about 200 ps a byte (T-1272
/// performance review: `TEXTSPLIT(REPT("a",32767)&SEQUENCE(16),HSTACK("a",REPT("a",16000)))`).
pub(super) const SEARCHED_BYTES_PER_VISIT: u64 = 8;

impl Model {
    /// A one-cell reference as the value of its cell, any other value as it is.
    pub(super) fn collapse_single_cell(&mut self, value: CalcResult) -> CalcResult {
        match value {
            CalcResult::Range { left, right } if left == right => self.evaluate_cell(left),
            other => other,
        }
    }

    /// Evaluates the arguments of a lifting text call. Those at the indices in `lists` are
    /// returned whole, in order, and left as `EmptyArg` in the first vector, which holds the
    /// operands that lift (a single-cell reference is its value).
    pub(super) fn lift_text_operands(
        &mut self,
        args: &[Node],
        lists: &[usize],
        cell: CellReferenceIndex,
    ) -> (Vec<CalcResult>, Vec<CalcResult>) {
        let mut operands = Vec::with_capacity(args.len());
        let mut whole = Vec::with_capacity(lists.len());
        for (index, node) in args.iter().enumerate() {
            let value = self.evaluate_node_in_context(node, cell);
            if lists.contains(&index) {
                whole.push(value);
                operands.push(CalcResult::EmptyArg);
                continue;
            }
            // Read once here: the cost of a lifted call depends on the text it holds.
            let value = self.collapse_single_cell(value);
            operands.push(match LiftedArg::read(self, value, cell) {
                Ok(LiftedArg::Grid(grid)) => CalcResult::Array(grid),
                Ok(LiftedArg::Scalar(value)) => value,
                Err(error) => error,
            });
        }
        (operands, whole)
    }

    /// Runs `element` once on `operands`, or once per element when one lifts; `element` is told
    /// whether it lifts, as a lifted element keeps only the first value of an array result
    /// (TEXTSPLIT: probes/m7-argument-modes F13:H13; REGEXEXTRACT's `return_all` UNVERIFIED,
    /// text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar
    /// arguments"), so it need not build the rest.
    ///
    /// An element compares each position of its text with the delimiters of byte lengths
    /// `scanned` (one pass when empty), `bytes_per_visit` bytes of them costing one visit, and
    /// reads its other operands once. The work is the sum
    /// over the operands' elements, each counted as often as the broadcast repeats it, so one
    /// long cell in a mostly blank column costs its own length once. The call is `#ERROR!` when
    /// the work exceeds `limits::MAX_LIFTED_CELL_VISITS` (DEVIATION, decision 0014: Excel has no
    /// such limit), lifted or not.
    pub(super) fn lift_text_call(
        &mut self,
        operands: Vec<CalcResult>,
        cell: CellReferenceIndex,
        scanned: &[usize],
        bytes_per_visit: u64,
        mut element: impl FnMut(&mut Model, &[CalcResult], bool) -> CalcResult,
    ) -> CalcResult {
        let lifts = operands.iter().any(LiftedArg::is_grid);
        if let Some(error) =
            too_many_visits(text_call_cost(&operands, scanned, bytes_per_visit), cell)
        {
            return error;
        }
        self.lift_values(
            operands,
            LiftCost::Unit,
            &LiftOptions::EVERY_ELEMENT,
            cell,
            &mut |model, values| match element(model, values, lifts) {
                CalcResult::Array(grid) if lifts => match grid.first().and_then(|r| r.first()) {
                    Some(node) => array_node_to_calc_result(node, cell),
                    None => CalcResult::new_error(Error::VALUE, cell, String::new()),
                },
                other => other,
            },
        )
    }

    /// The delimiters a list argument holds: the strings, numbers and booleans (as text) of an
    /// array or a range, a blank cell being the empty delimiter; an error element is that error
    /// (probes/textbefore-delimiter-range, probes/textbefore-delimiter-list-elements A1-A7, A22).
    pub(super) fn delimiter_list(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let grid = match value {
            CalcResult::Range { left, right } => self.evaluate_range(left, right, cell)?,
            CalcResult::Array(grid) => grid,
            // An omitted argument is no delimiter; a blank cell is the empty one
            // (probes/textsplit-empty-delimiter A5, A6).
            CalcResult::EmptyArg => return Ok(vec![]),
            CalcResult::EmptyCell => return Ok(vec![String::new()]),
            err @ CalcResult::Error { .. } => return Err(err),
            // A scalar boolean is its spelling, as a list element is (probes/textsplit-lift-guesses
            // A71-A73).
            scalar @ (CalcResult::String(_) | CalcResult::Number(_) | CalcResult::Boolean(_)) => {
                return Self::cast_string_ref(&scalar, cell).map(|text| vec![text])
            }
            _ => return Err(not_a_string(cell)),
        };
        grid.into_iter()
            .flatten()
            .map(|item| delimiter_text(item, cell))
            .collect()
    }
}

/// The shape of an operand: an array's rows and columns, 1x1 for a scalar.
fn operand_dims(operand: &CalcResult) -> (usize, usize) {
    match operand {
        CalcResult::Array(grid) => grid_dims(grid),
        _ => (1, 1),
    }
}

/// The work of one text call ([`Model::lift_text_call`]): over every operand, each element's
/// length plus 1 (a non-string 1), times how often the broadcast repeats it (a dimension of 1
/// stretches over the output's); the text operand's length is further multiplied by the
/// delimiter comparisons at each position ([`ScanCost`]). Saturating.
fn text_call_cost(operands: &[CalcResult], scanned: &[usize], bytes_per_visit: u64) -> u64 {
    let (rows, columns) = operands
        .iter()
        .map(operand_dims)
        .fold((1, 1), |acc, d| (acc.0.max(d.0), acc.1.max(d.1)));
    let scan = ScanCost::new(scanned, bytes_per_visit);
    operands
        .iter()
        .enumerate()
        .map(|(index, operand)| {
            let per_position = |len: usize| if index == 0 { scan.at(len) } else { 1 };
            operand_work(operand, rows, columns, per_position)
        })
        .fold(0u64, u64::saturating_add)
}

/// `n` as a `u64`, saturating.
pub(super) fn to_u64(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// The work of reading one operand over a `rows` x `columns` output: each element's length
/// plus 1 times `per_position(length)`, times how often the broadcast repeats it.
fn operand_work(
    operand: &CalcResult,
    rows: usize,
    columns: usize,
    per_position: impl Fn(usize) -> u64,
) -> u64 {
    let length = |len: usize| {
        to_u64(len)
            .saturating_add(1)
            .saturating_mul(per_position(len))
    };
    match operand {
        CalcResult::Array(grid) => {
            let (own_rows, own_columns) = operand_dims(operand);
            let repeats = to_u64(if own_rows == 1 { rows } else { 1 })
                .saturating_mul(to_u64(if own_columns == 1 { columns } else { 1 }));
            grid.iter()
                .flatten()
                .map(|node| match node {
                    ArrayNode::String(s) => length(s.len()),
                    _ => length(0),
                })
                .fold(0u64, u64::saturating_add)
                .saturating_mul(repeats)
        }
        CalcResult::String(s) => {
            length(s.len()).saturating_mul(to_u64(rows.saturating_mul(columns)))
        }
        _ => length(0).saturating_mul(to_u64(rows.saturating_mul(columns))),
    }
}

/// The delimiter comparisons at one position of a text: each delimiter costs 1 plus the bytes
/// it compares (at most the text's length) over `bytes_per_visit`; at least 1 (one pass).
struct ScanCost {
    /// The delimiter lengths, ascending.
    lengths: Vec<usize>,
    /// `prefix[i]` is the sum of the `i` shortest lengths.
    prefix: Vec<u64>,
    bytes_per_visit: u64,
}

impl ScanCost {
    fn new(scanned: &[usize], bytes_per_visit: u64) -> Self {
        let mut lengths = scanned.to_vec();
        lengths.sort_unstable();
        let mut prefix = Vec::with_capacity(lengths.len() + 1);
        prefix.push(0u64);
        for &len in &lengths {
            let last = prefix.last().copied().unwrap_or(0);
            prefix.push(last.saturating_add(to_u64(len)));
        }
        ScanCost {
            lengths,
            prefix,
            bytes_per_visit: bytes_per_visit.max(1),
        }
    }

    /// The cost at each position of a text of `text_len` bytes, in logarithmic time.
    fn at(&self, text_len: usize) -> u64 {
        let shorter = self.lengths.partition_point(|&len| len <= text_len);
        let compared = self
            .prefix
            .get(shorter)
            .copied()
            .unwrap_or(0)
            .saturating_add(to_u64(self.lengths.len() - shorter).saturating_mul(to_u64(text_len)));
        to_u64(self.lengths.len())
            .saturating_add(compared / self.bytes_per_visit)
            .max(1)
    }
}

/// One element of a delimiter list as text ([`Model::cast_string_ref`]: a blank is "").
fn delimiter_text(item: ArrayNode, cell: CellReferenceIndex) -> Result<String, CalcResult> {
    match item {
        ArrayNode::String(s) => Ok(s.into_lossy_string()),
        other => Model::cast_string_ref(&array_node_to_calc_result(&other, cell), cell),
    }
}

fn not_a_string(cell: CellReferenceIndex) -> CalcResult {
    CalcResult::new_error(Error::VALUE, cell, "delimiter must be a string".to_string())
}
