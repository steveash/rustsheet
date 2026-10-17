use super::util::{text_after, text_before, Case, Delimiters};
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
        let (operands, mut lists) = self.lift_text_operands(args, &[1], cell);
        let delimiters = match lists.pop() {
            Some(list) => self.around_delimiters(list, cell).map(Delimiters::new),
            None => Ok(Delimiters::new(vec![])),
        };
        let scanned: Vec<usize> = delimiters
            .as_ref()
            .map_or(vec![], |d| d.exact().iter().map(String::len).collect());
        self.lift_text_call(operands, cell, &scanned, |model, values, _| {
            model.text_around_element(values, arg_count, &delimiters, side, cell)
        })
    }

    /// The delimiter list of TEXTBEFORE and TEXTAFTER: a scalar is read as text (a blank is the
    /// empty delimiter, a boolean its spelling), an array is a list (probes/m7-argument-modes
    /// row 17). A multi-cell range is still read as one text, as before T-1272: Excel's rule for
    /// a range with blanks, numbers or booleans is unprobed (T-1361), and
    /// `TEXTAFTER(" x ",Data!A6:A10,Data!A9)` is `#VALUE!` (gen-m7/851eb2099ae3f805 Calc!A25).
    fn around_delimiters(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let value = self.collapse_single_cell(value);
        match value {
            CalcResult::Array(_) => self.delimiter_list(value, cell),
            scalar => self.cast_to_string(scalar, cell).map(|text| vec![text]),
        }
    }

    /// One call of TEXTBEFORE or TEXTAFTER on the values of the lifted arguments.
    fn text_around_element(
        &mut self,
        values: &[CalcResult],
        arg_count: usize,
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
        let options = match self.around_options(values, arg_count, cell) {
            Ok(options) => options,
            Err(error) => return error,
        };
        // With no delimiter shorter than the text a match is impossible. UNVERIFIED for a list
        // (text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift their text and scalar
        // arguments", T-1816): every entry longer than the text.
        let exact = delimiters.exact();
        if !exact.is_empty() && exact.iter().all(|d| d.len() > text.len()) {
            // With two arguments this is `#VALUE!`, with more `#N/A`, whatever their values.
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
        let found = match side {
            Side::Before => text_before(&text, delimiters, options.instance_num, options.case),
            Side::After => text_after(&text, delimiters, options.instance_num, options.case),
        };
        match found {
            Some(s) => CalcResult::String(s.into()),
            None => Self::around_not_found(values, arg_count, text, options, side, cell),
        }
    }

    /// The `instance_num`, `match_mode` and `match_end` of one element, read in that order (the
    /// defaults 1, case-sensitive and 0 when omitted). An `instance_num` of 0 is `#VALUE!`.
    fn around_options(
        &self,
        values: &[CalcResult],
        arg_count: usize,
        cell: CellReferenceIndex,
    ) -> Result<AroundOptions, CalcResult> {
        let number_at = |index: usize| -> Result<Option<f64>, CalcResult> {
            if arg_count > index {
                self.cast_number_ref(&values[index], cell).map(Some)
            } else {
                Ok(None)
            }
        };
        let instance_num = number_at(2)?.map_or(1, |f| f.floor() as i32);
        let case = match number_at(3)? {
            Some(f) if f != 0.0 => Case::Insensitive,
            _ => Case::Sensitive,
        };
        let match_end = number_at(4)?.unwrap_or(0.0);
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

    /// The result of an element whose delimiter is not found: with `match_end` the end of the text
    /// is the missing delimiter, then `if_not_found`, else `#N/A`.
    fn around_not_found(
        values: &[CalcResult],
        arg_count: usize,
        text: String,
        options: AroundOptions,
        side: Side,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        if options.match_end == 1.0 {
            // Before the first match from the start is the whole text, after it is empty, and
            // the other way for a search from the end.
            let from_start = options.instance_num == 1;
            let from_end = options.instance_num == -1;
            match side {
                Side::Before if from_end => return CalcResult::String("".into()),
                Side::Before if from_start => return CalcResult::String(text.into()),
                Side::After if from_start => return CalcResult::String("".into()),
                Side::After if from_end => return CalcResult::String(text.into()),
                _ => {}
            }
        }
        if arg_count == 6 {
            // An empty cell is converted to empty string (not 0)
            match &values[5] {
                CalcResult::EmptyCell => CalcResult::String("".into()),
                result => result.clone(),
            }
        } else {
            CalcResult::new_error(Error::NA, cell, "Value not found".to_string())
        }
    }
}
