use super::{
    lifted::COMPARED_BYTES_PER_VISIT,
    util::{text_after, text_before, Case, Delimiters},
};
use crate::{
    calc_result::CalcResult,
    expressions::{parser::Node, token::Error, types::CellReferenceIndex},
    model::Model,
};

/// Which side of the match TEXTBEFORE and TEXTAFTER return.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Before,
    After,
}

/// The scalar options of one TEXTBEFORE or TEXTAFTER element.
#[derive(Clone, Copy)]
struct AroundOptions {
    instance_num: i32,
    case: Case,
    match_end: f64,
}

impl Model {
    /// `TEXTAFTER(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])`
    pub(crate) fn fn_textafter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.text_around(args, cell, Side::After)
    }

    /// `TEXTBEFORE(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])`
    pub(crate) fn fn_textbefore(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.text_around(args, cell, Side::Before)
    }

    /// TEXTBEFORE and TEXTAFTER. The delimiter is a list (any of its entries matches); the other
    /// arguments lift over a range or an array (probes/m7-argument-modes F13-K13).
    fn text_around(&mut self, args: &[Node], cell: CellReferenceIndex, side: Side) -> CalcResult {
        let arg_count = args.len();
        if !(2..=6).contains(&arg_count) {
            return CalcResult::new_args_number_error(cell);
        }
        // Which arguments are written: an empty value an argument evaluates to is not omitted.
        let written: Vec<bool> = args
            .iter()
            .map(|arg| !matches!(arg, Node::EmptyArgKind))
            .collect();
        let (operands, mut lists) = self.lift_text_operands(args, &[1], cell);
        let delimiters = match lists.pop() {
            Some(list) => self.around_delimiters(list, cell).map(Delimiters::new),
            None => Ok(Delimiters::new(vec![])),
        };
        let scanned: Vec<usize> = delimiters
            .as_ref()
            .map_or(vec![], |d| d.exact().iter().map(String::len).collect());
        self.lift_text_call(
            operands,
            cell,
            &scanned,
            COMPARED_BYTES_PER_VISIT,
            |model, values, _| model.text_around_element(values, &written, &delimiters, side, cell),
        )
    }

    /// The delimiter list of TEXTBEFORE and TEXTAFTER: a scalar is read as text (a blank is the
    /// empty delimiter, a boolean its spelling), an array or a multi-cell range of any shape is a
    /// list (probes/m7-argument-modes row 17, probes/textbefore-delimiter-range).
    fn around_delimiters(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let value = self.collapse_single_cell(value);
        match value {
            CalcResult::Array(_) | CalcResult::Range { .. } => self.delimiter_list(value, cell),
            scalar => self.cast_to_string(scalar, cell).map(|text| vec![text]),
        }
    }

    /// One call of TEXTBEFORE or TEXTAFTER on the values of the lifted arguments.
    fn text_around_element(
        &mut self,
        values: &[CalcResult],
        written: &[bool],
        delimiters: &Result<Delimiters, CalcResult>,
        side: Side,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let text = match Self::cast_string_ref(&values[0], cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let delimiters = match delimiters {
            Ok(list) => list,
            Err(error) => return error.clone(),
        };
        let arg_count = written.len();
        let options = match self.around_options(values, written, cell) {
            Ok(options) => options,
            Err(error) => return error,
        };
        // With no delimiter shorter than the text a match is impossible. UNVERIFIED for a list
        // (text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar
        // arguments", T-1824): every entry longer than the text.
        let exact = delimiters.exact();
        if !exact.is_empty() && exact.iter().all(|d| d.len() > text.len()) {
            // With more than two arguments this is `#VALUE!`, with two `#N/A`, whatever their values.
            let error = if arg_count > 2 {
                Error::VALUE
            } else {
                Error::NA
            };
            return CalcResult::new_error(
                error,
                cell,
                "The delimiter is longer than the text is trying to match".to_string(),
            );
        }
        if options.match_end != 0.0 && options.match_end != 1.0 {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "argument must be 0 or 1".to_string(),
            );
        }
        if exact.iter().any(String::is_empty) {
            return Self::around_empty_delimiter(text, options.instance_num, side);
        }
        // Excel: see docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: instance_num past the
        // text, a number if_not_found": the size of instance_num is bounded by the number of
        // positions the shortest delimiter can start at, the text's length less the delimiter's
        // plus one in UTF-16 units, whatever match_end and if_not_found (probes/
        // t2314-textbefore-instance-bound, t2581-textsplit-textbefore A30-A50).
        let shortest = exact
            .iter()
            .map(|d| d.encode_utf16().count())
            .min()
            // The list is never empty (the delimiter is required and a list has an entry); were
            // it, 1 keeps the text-length bound.
            .unwrap_or(1);
        let positions = (text.encode_utf16().count() + 1).saturating_sub(shortest);
        if usize::try_from(options.instance_num.unsigned_abs()).map_or(true, |n| n > positions) {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "instance_num is larger than the text".to_string(),
            );
        }
        let (n, case, match_end) = (options.instance_num, options.case, options.match_end == 1.0);
        let found = match side {
            Side::Before => text_before(&text, delimiters, n, case, match_end),
            Side::After => text_after(&text, delimiters, n, case, match_end),
        };
        match found {
            Some(s) => CalcResult::String(s.into()),
            None => Self::around_not_found(values, arg_count, cell),
        }
    }

    /// The `instance_num`, `match_mode` and `match_end` of one element, read in that order (the
    /// defaults 1, case-sensitive and 0 when omitted, also as an empty argument:
    /// probes/textbefore-delimiter-range A22, A37). An `instance_num` of 0 is `#VALUE!`. A written
    /// argument that is the empty value is 0, not omitted: `TEXTAFTER(..,"-",CELL("contents",
    /// <blank>),1,1)` is `#VALUE!` (gen-m7/9359f03972e94026 A1420, T-3053).
    fn around_options(
        &self,
        values: &[CalcResult],
        written: &[bool],
        cell: CellReferenceIndex,
    ) -> Result<AroundOptions, CalcResult> {
        let number_at = |index: usize| -> Result<Option<f64>, CalcResult> {
            match values.get(index) {
                Some(CalcResult::EmptyArg) if written.get(index) == Some(&true) => {
                    self.cast_number_ref(&CalcResult::EmptyCell, cell).map(Some)
                }
                Some(CalcResult::EmptyArg) | None => Ok(None),
                Some(value) => self.cast_number_ref(value, cell).map(Some),
            }
        };
        let instance_num = number_at(2)?.map_or(1, |f| f.floor() as i32);
        let case = match number_at(3)? {
            None => Case::Sensitive,
            Some(mode) => Case::from_match_mode(mode, cell)?,
        };
        // Also cut toward zero: -0.0001 is 0, not `#VALUE!` (gen-m7/2d9b48c6363e7a16 A660,
        // T-2412).
        let match_end = number_at(4)?.map_or(0.0, f64::trunc);
        if instance_num == 0 {
            return Err(CalcResult::new_error(
                Error::VALUE,
                cell,
                "instance_num must be <> 0".to_string(),
            ));
        }
        Ok(AroundOptions {
            instance_num,
            case,
            match_end,
        })
    }

    /// The result of an element whose delimiter list holds the empty delimiter. The other
    /// delimiters, the size of `instance_num`, `match_mode`, `match_end` and `if_not_found` do not
    /// matter. The empty delimiter matches at the start of the text, or at its end when
    /// `instance_num` is negative or the text is one UTF-16 unit long (`TEXTBEFORE("é","")` is
    /// "é", `TEXTBEFORE("😀","")` is ""). Excel: see docs/excel-semantics/text.md "TEXTBEFORE
    /// and TEXTAFTER with the empty delimiter" (probes/textbefore-empty-delimiter-list).
    fn around_empty_delimiter(text: String, instance_num: i32, side: Side) -> CalcResult {
        let at_end = instance_num < 0 || text.encode_utf16().nth(1).is_none();
        let whole = match side {
            Side::Before => at_end,
            Side::After => !at_end,
        };
        CalcResult::String(if whole { text } else { String::new() }.into())
    }

    /// The result of an element whose delimiter is not found (`match_end` already counted the end
    /// of the text): `if_not_found`, else `#N/A`.
    fn around_not_found(
        values: &[CalcResult],
        arg_count: usize,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if arg_count == 6 {
            // An empty cell is converted to empty string (not 0), a number or a boolean comes back
            // as its text (probes/t2314-textbefore-instance-bound A27-A29).
            match &values[5] {
                CalcResult::EmptyCell => CalcResult::String("".into()),
                number @ (CalcResult::Number(_) | CalcResult::Boolean(_)) => {
                    match Self::cast_string_ref(number, cell) {
                        Ok(text) => CalcResult::String(text.into()),
                        Err(error) => error,
                    }
                }
                result => result.clone(),
            }
        } else {
            CalcResult::new_error(Error::NA, cell, "Value not found".to_string())
        }
    }
}
