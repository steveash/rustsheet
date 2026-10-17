use super::util::{text_after, text_before, Case};
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

impl Model {
    /// TEXTAFTER(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])
    pub(crate) fn fn_textafter(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.text_around(args, cell, Side::After)
    }

    /// TEXTBEFORE(text, delimiter, [instance_num], [match_mode], [match_end], [if_not_found])
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
            Some(list) => self.around_delimiters(list, cell),
            None => Ok(vec![]),
        };
        self.lift_text_call(operands, cell, |model, values| {
            model.text_around_element(values, arg_count, &delimiters, side, cell)
        })
    }

    /// The delimiter list of TEXTBEFORE and TEXTAFTER: a scalar is read as text (a blank is the
    /// empty delimiter, a boolean its spelling), a range or array is a list.
    fn around_delimiters(
        &mut self,
        value: CalcResult,
        cell: CellReferenceIndex,
    ) -> Result<Vec<String>, CalcResult> {
        let value = match value {
            CalcResult::Range { left, right } if left == right => self.evaluate_cell(left),
            other => other,
        };
        match value {
            CalcResult::Range { .. } | CalcResult::Array(_) => self.delimiter_list(value, cell),
            scalar => self.cast_to_string(scalar, cell).map(|text| vec![text]),
        }
    }

    /// One call of TEXTBEFORE or TEXTAFTER on the values of the lifted arguments.
    fn text_around_element(
        &mut self,
        values: &[CalcResult],
        arg_count: usize,
        delimiters: &Result<Vec<String>, CalcResult>,
        side: Side,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        let text = match self.cast_to_string(values[0].clone(), cell) {
            Ok(s) => s,
            Err(error) => return error,
        };
        let delimiters = match delimiters {
            Ok(list) => list,
            Err(error) => return error.clone(),
        };
        let delimiters: Vec<&str> = delimiters.iter().map(String::as_str).collect();
        let instance_num = if arg_count > 2 {
            match self.cast_to_number(values[2].clone(), cell) {
                Ok(f) => f.floor() as i32,
                Err(s) => return s,
            }
        } else {
            1
        };
        let match_mode = if arg_count > 3 {
            match self.cast_to_number(values[3].clone(), cell) {
                Ok(f) if f == 0.0 => Case::Sensitive,
                Ok(_) => Case::Insensitive,
                Err(s) => return s,
            }
        } else {
            Case::Sensitive
        };
        let match_end = if arg_count > 4 {
            match self.cast_to_number(values[4].clone(), cell) {
                Ok(f) => f,
                Err(s) => return s,
            }
        } else {
            // disabled by default
            // the delimiter is specified in the formula
            0.0
        };
        if instance_num == 0 {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "instance_num must be <> 0".to_string(),
            );
        }
        // With no delimiter shorter than the text a match is impossible. UNVERIFIED for a list
        // (text.md "TEXTBEFORE and TEXTAFTER delimiter lists"): every entry longer than the text.
        if !delimiters.is_empty() && delimiters.iter().all(|d| d.len() > text.len()) {
            // so this is fun(!)
            // if the function was provided with two arguments is a #VALUE!
            // if it had more is a #N/A (irrespective of their values)
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
        if match_end != 0.0 && match_end != 1.0 {
            return CalcResult::new_error(
                Error::VALUE,
                cell,
                "argument must be 0 or 1".to_string(),
            );
        }
        let found = match side {
            Side::Before => text_before(&text, &delimiters, instance_num, match_mode),
            Side::After => text_after(&text, &delimiters, instance_num, match_mode),
        };
        if let Some(s) = found {
            return CalcResult::String(s.into());
        }
        if match_end == 1.0 {
            // The end of the text is the delimiter that is missing: before the first match from
            // the start is the whole text, after it is empty, and the other way for a search
            // from the end.
            let from_start = instance_num == 1;
            let from_end = instance_num == -1;
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
