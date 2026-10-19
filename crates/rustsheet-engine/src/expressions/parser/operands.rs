//! The operands `parse_operand` dispatches to, one method per kind of token. They are methods of
//! their own so that the frame of `parse_operand`, which every nesting level of a formula keeps on
//! the stack, stays small: an unoptimized build gives every local of every arm its own slot, and
//! the whole match took about 11 KB per level (T-1110).

use super::super::token::{self, TableReference, TokenType};
use super::super::types::ParsedReference;
use super::super::utils::{
    is_out_of_range_r1c1_name, number_to_column, parse_dotted_a1_range, parse_reference_a1,
};
use super::reference_operators::is_reference_operand;
use super::*;

impl Parser {
    /// The parse error for a token that cannot start an operand.
    pub(super) fn unexpected_operand(&self, message: String) -> Node {
        Node::ParseErrorKind {
            formula: self.lexer.get_formula(),
            expecting: vec![ExpectedTokens::Other],
            position: 0,
            message,
        }
    }

    /// An array constant, `{1,2;3,4}`, after its `{`.
    pub(super) fn parse_array_operand(&mut self) -> Node {
        // It's an array. It's a collection of rows all of the same dimension
        let column_separator_token = self.get_column_separator_token();

        let first_row = match self.parse_array_row() {
            Ok(s) => s,
            Err(error) => return error,
        };
        let length = first_row.len();

        let mut matrix = Vec::new();
        matrix.push(first_row);
        let mut next_token = self.lexer.peek_token();
        while next_token == column_separator_token {
            self.lexer.advance_token();
            let row = match self.parse_array_row() {
                Ok(s) => s,
                Err(error) => return error,
            };
            next_token = self.lexer.peek_token();
            if row.len() != length {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: self.lexer.get_position() as usize,
                    message: "All rows in an array should be the same length".to_string(),
                };
            }
            matrix.push(row);
        }

        if let Err(err) = self.lexer.expect(TokenType::RightBrace) {
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting: vec![ExpectedTokens::Other],
                position: err.position,
                message: err.message,
            };
        }
        Node::ArrayKind(matrix)
    }

    /// A reference token, or in the stored R1C1 form a call of one, `R[-7]C[27](1)` (A1 text
    /// `AB1(1)`; in A1 text a cell followed by `(` is lexed as a name instead, T-1758).
    pub(super) fn reference_or_call_operand(
        &mut self,
        sheet: Option<String>,
        row: i32,
        column: i32,
        absolute_column: bool,
        absolute_row: bool,
    ) -> Node {
        let node = self.reference_operand(sheet, row, column, absolute_column, absolute_row);
        if !self.lexer.is_a1_mode()
            && !self.lexer.whitespace_follows()
            && self.lexer.peek_token() == TokenType::LeftParenthesis
        {
            return self.parse_call(node);
        }
        node
    }

    /// A cell reference token, made relative to the formula's cell in A1 mode.
    pub(super) fn reference_operand(
        &mut self,
        sheet: Option<String>,
        row: i32,
        column: i32,
        absolute_column: bool,
        absolute_row: bool,
    ) -> Node {
        let (span, sheet, sheet_index) = self.resolve_sheet_prefix(sheet);
        let context = &self.context;
        let a1_mode = self.lexer.is_a1_mode();
        let row = if absolute_row || !a1_mode {
            row
        } else {
            row - context.row
        };
        let column = if absolute_column || !a1_mode {
            column
        } else {
            column - context.column
        };
        let node = match sheet_index {
            Some(index) => Node::ReferenceKind {
                sheet_name: sheet,
                sheet_index: index,
                row,
                column,
                absolute_row,
                absolute_column,
            },
            None => Node::WrongReferenceKind {
                sheet_name: sheet,
                row,
                column,
                absolute_row,
                absolute_column,
            },
        };
        with_sheet_span(span, node)
    }

    /// A range token, its corners ordered and made relative to the formula's cell in A1 mode.
    pub(super) fn range_operand(
        &mut self,
        sheet: Option<String>,
        left: ParsedReference,
        right: ParsedReference,
    ) -> Node {
        let (span, sheet, sheet_index) = self.resolve_sheet_prefix(sheet);
        let context = &self.context;
        let mut row1 = left.row;
        let mut column1 = left.column;
        let mut row2 = right.row;
        let mut column2 = right.column;

        let mut absolute_column1 = left.absolute_column;
        let mut absolute_column2 = right.absolute_column;
        let mut absolute_row1 = left.absolute_row;
        let mut absolute_row2 = right.absolute_row;

        if self.lexer.is_a1_mode() {
            if row1 > row2 {
                (row2, row1) = (row1, row2);
                (absolute_row2, absolute_row1) = (absolute_row1, absolute_row2);
            }
            if column1 > column2 {
                (column2, column1) = (column1, column2);
                (absolute_column2, absolute_column1) = (absolute_column1, absolute_column2);
            }
        }

        if self.lexer.is_a1_mode() {
            if !absolute_row1 {
                row1 -= context.row;
            };
            if !absolute_column1 {
                column1 -= context.column;
            };
            if !absolute_row2 {
                row2 -= context.row;
            };
            if !absolute_column2 {
                column2 -= context.column;
            };
        }

        let node = match sheet_index {
            Some(index) => Node::RangeKind {
                sheet_name: sheet,
                sheet_index: index,
                row1,
                column1,
                row2,
                column2,
                absolute_column1,
                absolute_column2,
                absolute_row1,
                absolute_row2,
            },
            None => Node::WrongRangeKind {
                sheet_name: sheet,
                row1,
                column1,
                row2,
                column2,
                absolute_column1,
                absolute_column2,
                absolute_row1,
                absolute_row2,
            },
        };
        with_sheet_span(span, node)
    }

    /// An identifier: a call when `(` follows, otherwise a name.
    pub(super) fn parse_identifier(&mut self, name: String) -> Node {
        let next_token = self.lexer.peek_token();
        // If this identifier is the last token before the cursor (its
        // peek is EOF), the user is mid-typing a name here. Record it so
        // `parse_at_cursor` can offer function/name completion. We are in
        // a name/operand position by construction, so this never fires
        // for an identifier glued to a completed operand: the `m` in
        // `1m` is leftover input, never reached as a primary.
        if next_token == TokenType::EOF {
            let end = self.lexer.get_position() as usize;
            let start = end.saturating_sub(name.chars().count());
            self.trailing_name = Some((name.clone(), start));
        }
        if next_token == TokenType::LeftParenthesis {
            self.lexer.advance_token();
            return self.parse_named_call(name);
        }
        self.name_operand(name)
    }

    /// `Sheet!Name`, `[0]!Name` or `[0]Sheet!Name`: a defined name of this workbook written with
    /// its scope, resolved here so every lookup finds it. The node keeps the prefix in its name
    /// so the formula text round-trips as typed (`utils::bare_defined_name` strips it for
    /// lookups). `None` for an unprefixed name, a linked workbook's name (`[1]!Name`) or a sheet
    /// this workbook lacks.
    fn prefixed_name_operand(&self, name: &str) -> Option<Node> {
        let (prefix, bare) = name.rsplit_once('!')?;
        let sheet = match prefix.strip_prefix('[') {
            Some(external) => {
                let (book, sheet) = external.split_once(']')?;
                if book != "0" {
                    return None;
                }
                sheet.to_string()
            }
            None => match prefix.strip_prefix('\'') {
                Some(quoted) => quoted.strip_suffix('\'')?.replace("''", "'"),
                None => prefix.to_string(),
            },
        };
        let scope = if sheet.is_empty() {
            None
        } else {
            Some(self.get_sheet_index_by_name(&sheet)?)
        };
        // A sheet with no local definition reads the workbook-scoped name (docs/excel-semantics/
        // references.md "A sheet-qualified name falls back to the workbook name"; `Feb!loc`).
        let (scope, formula) = match scope {
            Some(sheet) => self
                .get_defined_name(bare, sheet)
                .unwrap_or((scope, String::new())),
            None => {
                let bare_lower = bare.to_lowercase();
                let formula = self
                    .defined_names
                    .iter()
                    .find(|(n, s, _)| s.is_none() && n.to_lowercase() == bare_lower)
                    .map(|(.., formula)| formula.clone())
                    .unwrap_or_default();
                (None, formula)
            }
        };
        Some(Node::DefinedNameKind((name.to_string(), scope, formula)))
    }

    /// A name that is not called: a defined name, a table or a LAMBDA parameter.
    fn name_operand(&mut self, name: String) -> Node {
        // A defined name of a linked workbook, `[1]!Name` or `[1]Sheet!Name` (one identifier to
        // the lexer), keeps its prefix: evaluation reads it from the link's cache.
        if let Some(node) = self.prefixed_name_operand(&name) {
            return node;
        }
        if name.starts_with('[') {
            return Node::DefinedNameKind((name, None, String::new()));
        }
        if let Some(node) = self.dotted_range_operand(&name) {
            return node;
        }
        let context = &self.context;

        let context_sheet_index = match self.get_sheet_index_by_name(&context.sheet) {
            Some(i) => i,
            None => {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message: format!("sheet not found: {}", context.sheet),
                };
            }
        };

        // Could be a defined name or a table. A name spelled like an out-of-range R1C1 reference
        // is defined but never resolved (T-1657).
        if !is_out_of_range_r1c1_name(&name) {
            if let Some((scope, formula)) = self.get_defined_name(&name, context_sheet_index) {
                return Node::DefinedNameKind((name, scope, formula));
            }
        }
        let name_lower = name.to_lowercase();
        for table_name in self.tables.keys() {
            if table_name.to_lowercase() == name_lower {
                // A bare table name is its data area, like `Table[]` (gen-m8b specials
                // e804cf2f8af00680 K116, lookups 27252667210b1d03).
                return self.structured_reference_operand(name, None, None);
            }
        }
        // xlpm: Excel Lambda Parameter
        let name = name.trim_start_matches("_xlpm.").to_string();
        Node::NamedVariableKind { name, id: None }
    }

    /// The range a name spelled like a Lotus-style range is (`A1.B2` is `A1:B2`; see
    /// `utils::parse_dotted_a1_range`), in A1 mode and unless an enclosing LET or LAMBDA binds
    /// it. Stored formulas (R1C1 mode) already print such a range as a range.
    fn dotted_range_operand(&mut self, name: &str) -> Option<Node> {
        if !self.lexer.is_a1_mode()
            || self
                .dotted_bound_names
                .iter()
                .any(|bound| bound.eq_ignore_ascii_case(name))
        {
            return None;
        }
        let (left, right) = parse_dotted_a1_range(name)?;
        Some(self.range_operand(None, left, right))
    }

    /// `node`, parsed from the argument starting at `token`, where a LET or LAMBDA expects a
    /// name: a name spelled like a Lotus-style range was read as the range while unbound, so it
    /// becomes the variable again, and stays one for the rest of the call (Excel:
    /// `LET(R1.C1,5,R1.C1*2)` is 10, probes/r1c1-let-name-r1-c1, T-1659).
    pub(super) fn bind_dotted_name(&mut self, token: &TokenType, node: Node) -> Node {
        match (token, &node) {
            (TokenType::Ident(name), Node::RangeKind { .. } | Node::WrongRangeKind { .. })
                if self.lexer.is_a1_mode() && parse_dotted_a1_range(name).is_some() =>
            {
                self.dotted_bound_names.push(name.clone());
                Node::NamedVariableKind {
                    name: name.clone(),
                    id: None,
                }
            }
            _ => node,
        }
    }

    /// A call by name, `SUM(...)` or `_xlfn.LAMBDA(...)(...)`, after its `(`.
    fn parse_named_call(&mut self, name: String) -> Node {
        // It's a function call "SUM(.."
        // _xlfn.LAMBDA(_xlpm.a,_xlpm.b, SQRT(_xlpm.a*_xlpm.a+_xlpm.b*_xlpm.b))(3,4)
        if &name == "_xlfn.LAMBDA" || &name.to_uppercase() == "LAMBDA" {
            return self.parse_lambda();
        }
        // The user-facing name, without the xlsx import prefixes.
        let display_name = name
            .trim_start_matches("_xlfn._xlws.")
            .trim_start_matches("_xlfn.")
            .trim_start_matches("_xlpm.")
            .to_string();
        let args = match self.parse_function_args(&display_name) {
            Ok(s) => s,
            Err(e) => return e,
        };
        if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
            // `SUM(A1` parses a complete argument and then fails to
            // find the `)`. If we ran out of input we are still
            // inside the call, sitting right after a complete
            // argument: report the (correct) argument index so the
            // signature tooltip works, but NOT `Range` — a fresh
            // reference cannot begin after a complete operand. (A
            // separator or `)` is what is expected here.)
            let at_eof = err.position >= self.lexer.get_formula().chars().count();
            let expecting = if at_eof {
                let index = (args.len() as u32).max(1);
                vec![ExpectedTokens::Argument(display_name.clone(), index)]
            } else {
                vec![ExpectedTokens::Other]
            };
            return Node::ParseErrorKind {
                formula: self.lexer.get_formula(),
                expecting,
                position: err.position,
                message: err.message,
            };
        }
        // We should do this *only* importing functions from xlsx: Implicit Intersection
        if &name == "_xlfn.SINGLE" {
            if args.len() != 1 {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: self.lexer.get_position() as usize,
                    message: "Implicit Intersection requires just one argument".to_string(),
                };
            }
            return Node::at(args[0].clone(), false);
        }
        // The spill range operator; typed `ANCHORARRAY(D1)` is `D1#` for Excel too
        // (probes/at-operand-formulatext J1, T-1109).
        if display_name.eq_ignore_ascii_case("ANCHORARRAY") {
            if args.len() != 1 {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: self.lexer.get_position() as usize,
                    message: "ANCHORARRAY requires one argument".to_string(),
                };
            }
            // `=ANCHORARRAY(1)` is refused like a value operand of `:` (T-1111, T-1117).
            if !is_reference_operand(&args[0]) {
                return self.range_operand_error();
            }
            return Node::SpillRangeOperator {
                child: Box::new(args[0].clone()),
            };
        }
        // The stored spelling of a legacy N or T (SPEC-0012), in R1C1 (stored) text only: in A1
        // text it is a cell (`NLG1(1)` is `#REF!`, below) or a name call (T-1936).
        if !self.lexer.is_a1_mode() {
            if let Some(kind) = Function::from_legacy_spelling(&name, |name| {
                self.defined_names
                    .iter()
                    .any(|(defined, _, _)| defined.eq_ignore_ascii_case(name))
            }) {
                return self.function_call(kind, args);
            }
        }
        // We should do this *only* importing functions from xlsx
        if let Some(function_kind) = self.language.functions.lookup_stored(&name) {
            return self.function_call(function_kind, args);
        }
        // A callee spelled like a cell, and no function (`LOG10` stays one), is a call of that cell:
        // `#REF!` like any called reference (probes/unknown-function-name-spelling A8, T-1758).
        // `$` shapes are unprobed, so they stay names. A1 grammar only: in R1C1 mode (stored
        // forms, and the raw A1 text of a formula that did not parse) `AB1` is no cell, and reading
        // it as one would turn A1 coordinates into R1C1 offsets.
        if let Some(cell) = parse_reference_a1(&name.to_ascii_uppercase())
            .filter(|_| self.lexer.is_a1_mode() && !name.contains('$'))
        {
            let callee = self.reference_operand(None, cell.row, cell.column, false, false);
            return Node::LambdaCallKind {
                lambda: Box::new(callee),
                args,
            };
        }
        Node::NamedFunctionKind {
            name: name.trim_start_matches("_xlpm.").to_string(),
            args,
            id: None,
        }
    }

    /// `TRUE` or `FALSE`, or the calls `TRUE()` and `FALSE()`.
    pub(super) fn parse_boolean_operand(&mut self, value: bool) -> Node {
        // Could be a function call "TRUE()"
        let next_token = self.lexer.peek_token();
        if next_token == TokenType::LeftParenthesis {
            self.lexer.advance_token();
            // We parse all the arguments, although technically this is moot
            // But is has the upside of transforming `=TRUE( 4 )` into `=TRUE(4)`
            let fn_name = if value { "TRUE" } else { "FALSE" };
            let args = match self.parse_function_args(fn_name) {
                Ok(s) => s,
                Err(e) => return e,
            };
            if let Err(err) = self.lexer.expect(TokenType::RightParenthesis) {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: err.position,
                    message: err.message,
                };
            }
            if value {
                return Node::FunctionKind {
                    kind: Function::True,
                    args,
                };
            } else {
                return Node::FunctionKind {
                    kind: Function::False,
                    args,
                };
            }
        }
        Node::BooleanKind(value)
    }

    /// A structured reference, `Table1[Column]`, resolved to the range it stands for.
    pub(super) fn structured_reference_operand(
        &mut self,
        table_name: String,
        specifier: Option<token::TableSpecifier>,
        table_reference: Option<TableReference>,
    ) -> Node {
        // We will try to convert to a normal reference
        // table_name[column_name] => cell1:cell2
        // table_name[[#This Row], [column_name]:[column_name]] => cell1:cell2
        let context = &self.context;
        let context_sheet_index = match self.get_sheet_index_by_name(&context.sheet) {
            Some(i) => i,
            None => {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message: format!("sheet not found: {}", context.sheet),
                };
            }
        };
        // table-name => table
        // Table names match ignoring case (gen-m8b names-case 5b5c0e20e82153bb).
        // An exact hit is deterministic; the case-folded scan covers the rest.
        let table_name_lower = table_name.to_lowercase();
        let table = match self.tables.get(&table_name).or_else(|| {
            self.tables
                .iter()
                .find(|(name, _)| name.to_lowercase() == table_name_lower)
                .map(|(_, t)| t)
        }) {
            Some(t) => t,
            None => {
                let message = format!(
                    "Table not found: '{table_name}' at '{}!{}{}'",
                    context.sheet,
                    number_to_column(context.column).unwrap_or(format!("{}", context.column)),
                    context.row
                );
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message,
                };
            }
        };
        let table_sheet_index = match self.get_sheet_index_by_name(&table.sheet_name) {
            Some(i) => i,
            None => {
                return Node::ParseErrorKind {
                    formula: self.lexer.get_formula(),
                    expecting: vec![ExpectedTokens::Other],
                    position: 0,
                    message: format!("table sheet not found: {}", table.sheet_name),
                };
            }
        };

        let sheet_name = if table_sheet_index == context_sheet_index {
            None
        } else {
            Some(table.sheet_name.clone())
        };

        // The table metadata comes from the workbook (untrusted): a stored reference that does not
        // parse is #REF! (directive 5: no panics; docs/excel-semantics/references.md, DEVIATION).
        let Ok((column_start, mut row_start, column_end, mut row_end)) =
            parse_range(&table.reference)
        else {
            return Node::ErrorKind(token::Error::REF);
        };

        // The counts are workbook values too: arithmetic that does not fit is #REF!, not a wrap.
        let (Ok(totals_row_count), Ok(header_row_count)) = (
            i32::try_from(table.totals_row_count),
            i32::try_from(table.header_row_count),
        ) else {
            return Node::ErrorKind(token::Error::REF);
        };
        let (Some(data_end), Some(data_start)) = (
            row_end.checked_sub(totals_row_count),
            row_start.checked_add(header_row_count),
        ) else {
            return Node::ErrorKind(token::Error::REF);
        };
        let Some(totals_row) = data_end.checked_add(1) else {
            return Node::ErrorKind(token::Error::REF);
        };
        row_end = data_end;

        match specifier {
            Some(token::TableSpecifier::ThisRow) => {
                // Off the data rows (header, totals, above, below) it is #VALUE!
                // (docs/excel-semantics/references.md, gen-m8b this-row e1bf5aa0ffb4ed87).
                if context.row < data_start || context.row > data_end {
                    return Node::ErrorKind(token::Error::VALUE);
                }
                row_start = context.row;
                row_end = context.row;
            }
            Some(token::TableSpecifier::Totals) => {
                if totals_row_count != 0 {
                    row_start = totals_row;
                    row_end = row_start;
                } else {
                    // Table1[#Totals] is #REF! if Table1 does not have totals
                    return Node::ErrorKind(token::Error::REF);
                }
            }
            Some(token::TableSpecifier::Headers) => {
                row_end = row_start;
            }
            Some(token::TableSpecifier::Data) => {
                row_start = data_start;
            }
            Some(token::TableSpecifier::All) => {
                if totals_row_count != 0 {
                    row_end = totals_row;
                }
            }
            // [#Headers],[#Data]: header row through the last data row (row_end already excludes totals).
            Some(token::TableSpecifier::HeadersData) => {}
            // [#Data],[#Totals]: data rows plus the totals row when there is one.
            Some(token::TableSpecifier::DataTotals) => {
                row_start = data_start;
                if totals_row_count != 0 {
                    row_end = totals_row;
                }
            }
            None => {
                // skip the headers
                row_start = data_start;
            }
        }
        match table_reference {
            None => Node::RangeKind {
                sheet_name,
                sheet_index: table_sheet_index,
                absolute_row1: true,
                absolute_column1: true,
                row1: row_start,
                column1: column_start,
                absolute_row2: true,
                absolute_column2: true,
                row2: row_end,
                column2: column_end,
            },
            Some(TableReference::ColumnReference(s)) => {
                let column_index = match get_table_column_by_name(&s, table) {
                    Some(s) => s + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("Expecting column: {s} in table {table_name}"),
                        };
                    }
                };
                if row_start == row_end {
                    return Node::ReferenceKind {
                        sheet_name,
                        sheet_index: table_sheet_index,
                        absolute_row: true,
                        absolute_column: true,
                        row: row_start,
                        column: column_index,
                    };
                }
                Node::RangeKind {
                    sheet_name,
                    sheet_index: table_sheet_index,
                    absolute_row1: true,
                    absolute_column1: true,
                    row1: row_start,
                    column1: column_index,
                    absolute_row2: true,
                    absolute_column2: true,
                    row2: row_end,
                    column2: column_index,
                }
            }
            Some(TableReference::RangeReference((left, right))) => {
                let left_column_index = match get_table_column_by_name(&left, table) {
                    Some(f) => f + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("Expecting column: {left} in table {table_name}"),
                        };
                    }
                };

                let right_column_index = match get_table_column_by_name(&right, table) {
                    Some(f) => f + column_start,
                    None => {
                        return Node::ParseErrorKind {
                            formula: self.lexer.get_formula(),
                            expecting: vec![ExpectedTokens::Other],
                            position: self.lexer.get_position() as usize,
                            message: format!("Expecting column: {right} in table {table_name}"),
                        };
                    }
                };
                // The same area whichever column is written first (references.md, column-range
                // 44cecbb1ecd5b2da K143).
                Node::RangeKind {
                    sheet_name,
                    sheet_index: table_sheet_index,
                    absolute_row1: true,
                    absolute_column1: true,
                    row1: row_start,
                    column1: left_column_index.min(right_column_index),
                    absolute_row2: true,
                    absolute_column2: true,
                    row2: row_end,
                    column2: left_column_index.max(right_column_index),
                }
            }
        }
    }
}
