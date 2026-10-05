//! Text-preserving relative shift of an A1 formula (shared-formula expansion).

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::expressions::types::ParsedReference;
use crate::expressions::utils::number_to_column;
use crate::language::get_language;
use crate::locale::get_locale;

/// Shifts the relative parts of every cell/range reference in `formula` (no leading `=`) by
/// `d_row` rows and `d_col` columns, keeping every other character (spacing, case, sheet-name
/// quoting) as written. This is how a shared-formula member derives its own formula from the
/// master's text. A reference shifted off the sheet becomes `#REF!`.
///
/// Returns `None` if the formula does not lex (the caller keeps the original text).
#[must_use]
pub fn shift_formula_text(formula: &str, d_row: i32, d_col: i32) -> Option<String> {
    let locale = get_locale("en").ok()?;
    let language = get_language("en").ok()?;
    let mut lexer = Lexer::new(formula, LexerMode::A1, locale, language);
    let chars: Vec<char> = formula.chars().collect();
    let mut out = String::new();
    let mut copied = 0usize;
    loop {
        let before = usize::try_from(lexer.get_position()).ok()?;
        let token = lexer.next_token();
        let end = usize::try_from(lexer.get_position()).ok()?;
        let (left, right) = match token {
            TokenType::EOF => break,
            TokenType::Illegal(_) => return None,
            TokenType::Reference {
                row,
                column,
                absolute_row,
                absolute_column,
                ..
            } => (
                ParsedReference {
                    row,
                    column,
                    absolute_row,
                    absolute_column,
                },
                None,
            ),
            TokenType::Range { left, right, .. } => (left, Some(right)),
            _ => continue,
        };
        // The token's span starts after any leading whitespace; the sheet prefix (if any) ends at
        // the last `!`, and sheet names cannot contain one after their closing quote.
        let mut start = before;
        while chars.get(start).is_some_and(|c| c.is_whitespace()) {
            start += 1;
        }
        let span = chars.get(start..end)?;
        let tail_at = span.iter().rposition(|&c| c == '!').map_or(0, |i| i + 1);
        out.extend(chars.get(copied..start + tail_at)?);
        out.push_str(&shifted_tail(&left, right.as_ref(), d_row, d_col));
        copied = end;
    }
    out.extend(chars.get(copied..)?);
    Some(out)
}

fn shift_one(r: &ParsedReference, d_row: i32, d_col: i32) -> Option<ParsedReference> {
    let row = if r.absolute_row {
        r.row
    } else {
        r.row.checked_add(d_row)?
    };
    let column = if r.absolute_column {
        r.column
    } else {
        r.column.checked_add(d_col)?
    };
    ((1..=LAST_ROW).contains(&row) && (1..=LAST_COLUMN).contains(&column)).then_some(
        ParsedReference {
            row,
            column,
            absolute_row: r.absolute_row,
            absolute_column: r.absolute_column,
        },
    )
}

fn cell_text(r: &ParsedReference) -> Option<String> {
    Some(format!(
        "{}{}{}{}",
        if r.absolute_column { "$" } else { "" },
        number_to_column(r.column)?,
        if r.absolute_row { "$" } else { "" },
        r.row
    ))
}

fn shifted_tail(
    left: &ParsedReference,
    right: Option<&ParsedReference>,
    d_row: i32,
    d_col: i32,
) -> String {
    let text = (|| {
        let l = shift_one(left, d_row, d_col)?;
        let Some(right) = right else {
            return cell_text(&l);
        };
        let r = shift_one(right, d_row, d_col)?;
        let col = |p: &ParsedReference| {
            Some(format!(
                "{}{}",
                if p.absolute_column { "$" } else { "" },
                number_to_column(p.column)?
            ))
        };
        let row =
            |p: &ParsedReference| format!("{}{}", if p.absolute_row { "$" } else { "" }, p.row);
        // Whole-column (`D:D`) and whole-row (`5:5`) ranges are stored with sentinel bounds.
        if left.absolute_row && right.absolute_row && left.row == 1 && right.row == LAST_ROW {
            return Some(format!("{}:{}", col(&l)?, col(&r)?));
        }
        if left.absolute_column
            && right.absolute_column
            && left.column == 1
            && right.column == LAST_COLUMN
        {
            return Some(format!("{}:{}", row(&l), row(&r)));
        }
        Some(format!("{}:{}", cell_text(&l)?, cell_text(&r)?))
    })();
    text.unwrap_or_else(|| "#REF!".to_string())
}

#[cfg(test)]
mod tests {
    use super::shift_formula_text as shift;

    #[test]
    fn keeps_spacing_and_shifts_relative_parts() {
        assert_eq!(
            shift("TEXT(C41, B41)", 55, 0).as_deref(),
            Some("TEXT(C96, B96)")
        );
        assert_eq!(
            shift("$A1+B$2 + $C$3", 2, 3).as_deref(),
            Some("$A3+E$2 + $C$3")
        );
    }

    #[test]
    fn ranges_sheets_strings_and_whole_columns() {
        assert_eq!(
            shift("SUM('My Sheet'!A1:B2, \"A1\", D:D, 5:6, Sheet2!$A$1)", 1, 1).as_deref(),
            Some("SUM('My Sheet'!B2:C3, \"A1\", E:E, 6:7, Sheet2!$A$1)")
        );
    }

    #[test]
    fn off_sheet_is_ref_error_and_bad_input_is_none() {
        assert_eq!(shift("A1+1", -1, 0).as_deref(), Some("#REF!+1"));
        assert_eq!(shift("A1+\"x", 1, 0), None);
    }
}
