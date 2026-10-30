//! The operands `parse_operand` dispatches to, one method per kind of token. They are methods of
//! their own so that the frame of `parse_operand`, which every nesting level of a formula keeps on
//! the stack, stays small: an unoptimized build gives every local of every arm its own slot, and
//! the whole match took about 11 KB per level (T-1110).

use super::super::token::{self, TableReference, TokenType};
use super::super::types::ParsedReference;
use super::super::utils::{
    is_out_of_range_r1c1_name, number_to_column, parse_dotted_a1_range, parse_reference_a1,
    quote_name,
};
use super::reference_operators::is_reference_operand;
use super::structured_reference::{
    is_unresolved_structured_reference, TableExtent, COLUMN_NOT_FOUND, TABLE_NOT_FOUND,
    TABLE_SHEET_NOT_FOUND,
};
use super::*;
use crate::functions::kinds::arg_counts;
use crate::language::get_default_language;

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

    /// A reference token, or a call of one: in the stored R1C1 form `R[-7]C[27](1)` (A1 text
    /// `AB1(1)`; in A1 text a plain cell followed by `(` is lexed as a name instead, T-1758), and
    /// in A1 text an anchored or sheet-qualified cell, `$D$1(1)`, `Sheet1!D1(1)`: `#REF!` like any
    /// called reference (probes/cell-call-edges-a A5-A7, T-1806).
    pub(super) fn reference_or_call_operand(
        &mut self,
        sheet: Option<String>,
        row: i32,
        column: i32,
        absolute_column: bool,
        absolute_row: bool,
    ) -> Node {
        let node = self.reference_operand(sheet, row, column, absolute_column, absolute_row);
        self.call_if_parenthesis_follows(node)
    }

    /// A range token, or a call of one: `Sheet1!$A$1:$A$2(2)` and `$A$1:A2(2)` are `#REF!` like
    /// any called reference (probes/callee-unparenthesized-range-*, T-2008).
    pub(super) fn range_or_call_operand(
        &mut self,
        sheet: Option<String>,
        left: ParsedReference,
        right: ParsedReference,
    ) -> Node {
        let node = self.range_operand(sheet, left, right);
        self.call_if_parenthesis_follows(node)
    }

    /// `node` called with the argument list that follows it, when a `(` follows with no space.
    pub(super) fn call_if_parenthesis_follows(&mut self, node: Node) -> Node {
        if !self.lexer.whitespace_follows() && self.lexer.peek_token() == TokenType::LeftParenthesis
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
            None => crate::expressions::utils::unquote_sheet_prefix(prefix)?,
        };
        let scope = match self.get_sheet_index_by_name(&sheet) {
            _ if sheet.is_empty() => None,
            Some(index) => Some(index),
            // `Budget.xlsx!loc`, the text Excel shows for `Feb!loc`, reads the workbook name.
            None if self.is_workbook_file(&sheet) => None,
            None => return None,
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
        // Excel shows a workbook name read through a sheet, or through the file's own-workbook
        // prefix `[0]!`, with the workbook's file name as its prefix, quoted as a sheet name
        // would be (`=Feb!loc` is `='53aa….xlsx'!loc`), and a sheet rename leaves it alone
        // (docs/excel-semantics/references.md "A sheet-qualified workbook name is rewritten to
        // the workbook's file name", T-2834).
        let text = match &self.workbook_file {
            Some(file) if scope.is_none() && !prefix.is_empty() && !formula.is_empty() => {
                format!("{}!{bare}", quote_name(file))
            }
            _ => name.to_string(),
        };
        Some(Node::DefinedNameKind((text, scope, formula)))
    }

    /// Whether a sheet prefix spells this workbook's file name (`Budget.xlsx`, any case).
    fn is_workbook_file(&self, prefix: &str) -> bool {
        self.workbook_file
            .as_deref()
            .is_some_and(|file| file.to_lowercase() == prefix.to_lowercase())
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
        // A LET name or LAMBDA parameter shadows a defined name of the same name in any case
        // (probes/let-name-shadows-defined-name, T-2057).
        if !is_out_of_range_r1c1_name(&name) && !self.binds_name(&name) {
            if let Some((scope, formula)) = self.get_defined_name(&name, context_sheet_index) {
                return Node::DefinedNameKind((name, scope, formula));
            }
        }
        let name_lower = name.to_lowercase();
        for table_name in self.tables.keys() {
            if table_name.to_lowercase() == name_lower {
                // A bare table name is its data area, like `Table[]` (gen-m8b specials
                // e804cf2f8af00680 K116, lookups 27252667210b1d03).
                return self.structured_reference_operand(
                    name,
                    None,
                    None,
                    token::TableSpacing::default(),
                    true,
                );
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
        if !self.lexer.is_a1_mode() || self.binds_reference_shaped_name(name) {
            return None;
        }
        let (left, right) = parse_dotted_a1_range(name)?;
        Some(self.range_operand(None, left, right))
    }

    /// Whether an enclosing LET or LAMBDA binds `name` ([`Parser::bind_name`]),
    /// in any grammar mode.
    fn binds_name(&self, name: &str) -> bool {
        self.bound_names
            .iter()
            .any(|bound| bound.eq_ignore_ascii_case(name))
    }

    /// Whether a call of `name`, spelled like a function, is a call of the LET name or LAMBDA
    /// parameter of that name instead: only in unprefixed stored text (A1 grammar, not typed, the
    /// `typed` split of decision 0103), where the name was declared bare ([`Parser::bind_name`]).
    /// Such text (`Input::StoredFormula`, the harness's case formulas) stands for the `_xlpm.`
    /// form the file holds, and there Excel calls the variable: stored as `_xlpm.Pv(1)`,
    /// `LET(Pv,LAMBDA(a,a),Pv(1))` is 1 and `LET(Sum,LAMBDA(a,a*2),SUM(1,2))` `#VALUE!`
    /// (probes/let-name-function-spelled-call, let-lambda-body-own-name, T-2567). Typed, the
    /// function wins in any case: `=LET(Sum,LAMBDA(a,a*2),Sum(3))` is `SUM(3)`, 3, and
    /// `=LET(Pv,LAMBDA(a,a),Pv(1))` is refused (probes/let-function-name-typed-*). Excel's files
    /// (xlsx import, also A1 grammar) declare the name `_xlpm.Sum` and call it `_xlpm.Sum(`, so
    /// a bare `SUM(` there is the function (`LET(_xlpm.sum,SUM(1,2),_xlpm.sum*2)` is 6); the R1C1
    /// storage form writes such a call `_xlpm.Sum(` too (`callee_text`).
    fn calls_bound_name(&self, name: &str) -> bool {
        self.lexer.is_a1_mode() && !self.typed && self.binds_name(name)
    }

    /// [`Parser::binds_name`] in the A1 grammar, asked for a name it would otherwise read as a
    /// cell or a Lotus-style range.
    pub(super) fn binds_reference_shaped_name(&self, name: &str) -> bool {
        self.lexer.is_a1_mode() && self.binds_name(name)
    }

    /// `node`, parsed from the argument `text` starting at `token`, where a LET or LAMBDA
    /// expects a name: a name spelled like a Lotus-style range or a relative cell on this sheet
    /// was read as the range or cell while unbound, so it becomes the variable again, and stays
    /// one for the rest of the call. Excel: `LET(R1.C1,5,R1.C1*2)` is 10
    /// (probes/r1c1-let-name-r1-c1, T-1659), and stored as `_xlpm.f1`, `LET(f1,1,f1+1)` is 2
    /// (probes/let-cell-shaped-name-stored, T-2040). Excel refuses the cell-shaped name typed
    /// (probes/let-cell-shaped-name-typed-*): it binds here too, and typed entry refuses the
    /// formula afterwards ([`Parser::typed_refusal`], decision 0103). A name that was read as a
    /// defined name becomes the variable in any grammar mode, shadowing the defined name
    /// (probes/let-name-shadows-defined-name, T-2057). Any other plain name is recorded as bound
    /// too when declared bare, for [`Parser::calls_bound_name`] (T-2567).
    /// The whitespace written around the name stays around the variable (`LET(x, 1, y, 2, x+y)`,
    /// SPEC-0023).
    pub(super) fn bind_name(&mut self, token: &TokenType, text: &str, node: Node) -> Node {
        let (space, node) = node.unspaced();
        self.bind_bare_name(token, text, node).respaced(space)
    }

    /// [`Parser::bind_name`] of a name with no whitespace around it.
    fn bind_bare_name(&mut self, token: &TokenType, text: &str, node: Node) -> Node {
        // A name that is also a defined name parsed as that defined name; it declares the
        // variable instead, and shadows the defined name up to the end of the scope. In any
        // grammar mode: a stored formula (R1C1 mode) prints the name without `_xlpm.`.
        if let (TokenType::Ident(_), Node::DefinedNameKind((name, ..))) = (token, &node) {
            if !name.contains(['!', '[']) {
                let name = name.clone();
                self.bound_names.push(name.clone());
                return Node::NamedVariableKind { name, id: None };
            }
        }
        // Any other plain name declared bare is bound too: called in unprefixed stored text, it
        // is the variable, not a function of its name (`calls_bound_name`, T-2567). A name
        // declared `_xlpm.Sum`, as Excel and the export write it, is not: there every call of
        // the variable is `_xlpm.Sum(` too, and a bare `SUM(` is the function.
        if let (TokenType::Ident(_), Node::NamedVariableKind { name, .. }) = (token, &node) {
            if !text.trim_start().starts_with("_xlpm.") {
                self.bound_names.push(name.clone());
            }
            return node;
        }
        if !self.lexer.is_a1_mode() {
            return node;
        }
        let bound = match (token, &node) {
            (TokenType::Ident(name), Node::RangeKind { .. } | Node::WrongRangeKind { .. })
                if parse_dotted_a1_range(name).is_some() =>
            {
                name.clone()
            }
            (
                TokenType::Reference {
                    sheet: None,
                    absolute_column: false,
                    absolute_row: false,
                    ..
                },
                Node::ReferenceKind { .. } | Node::WrongReferenceKind { .. },
            ) if parse_reference_a1(&text.to_ascii_uppercase()).is_some() => {
                if self.cell_shaped_binding.is_none() {
                    self.cell_shaped_binding = Some(text.to_string());
                }
                text.to_string()
            }
            _ => return node,
        };
        self.bound_names.push(bound.clone());
        Node::NamedVariableKind {
            name: bound,
            id: None,
        }
    }

    /// Records the first call of `kind` with a `count` of arguments the function does not take,
    /// for [`Parser::typed_refusal`]: Excel refuses such a formula typed (`=PV(1)`, T-2567; `=SUM()`,
    /// `=IF(1)`, `=ABS(1,2)`, `ACCRINTM` with three arguments, T-2632) and in a file
    /// (docs/excel-semantics/syntax.md "Minimum argument counts"); stored text keeps it, and the
    /// call evaluates to `#ERROR!`. The counts are the catalog's for every function
    /// ([`arg_counts::takes_count`], which also refuses IFS and LET off whole pairs, T-2682); a
    /// lifted function must also end on a whole `repeat` cycle
    /// ([`ArgKinds::accepts`]: Excel refuses `=SORTBY(C1:C2,D1:D2,1,E1:E2)` typed, T-2632). A
    /// function the catalog does not list is never refused here.
    fn note_argument_count(&mut self, kind: &Function, name: &str, count: usize) {
        // Only typed text is refused, and the count table is a linear scan: skip it for stored
        // formulas and xlsx import.
        if !self.typed || self.wrong_count_call.is_some() {
            return;
        }
        let english = kind.to_localized_name(get_default_language());
        let refused = !arg_counts::takes_count(&english, count)
            || kind.arg_kinds().is_some_and(|kinds| !kinds.accepts(count));
        if refused {
            self.note_wrong_count(name);
        }
    }

    /// Whether the formula is being typed ([`Parser::set_typed`]).
    pub(crate) fn is_typed(&self) -> bool {
        self.typed
    }

    /// Why Excel refuses the formula just parsed when it is typed: it binds a LET name or LAMBDA
    /// parameter spelled like a cell (`=LET(f1,1,f1+1)`; T-2040 Graph PATCH 400, decision 0103),
    /// or it calls a function with fewer or more arguments than the function takes (`=PV(1)`,
    /// also `=LET(Pv,LAMBDA(a,a),Pv(1))`, where the typed call is the function: T-2567 Graph
    /// PATCH 400, T-2568), or it nests more function calls than Excel takes
    /// ([`MAX_NESTED_CALLS`], T-3097). `None` when parsing stored text or nothing is refused.
    pub(crate) fn typed_refusal(&self) -> Option<String> {
        if !self.typed {
            return None;
        }
        if self.nests_too_many_calls() {
            return Some(format!("Formula: {TOO_MANY_NESTED_CALLS_MESSAGE}"));
        }
        if let Some(name) = &self.cell_shaped_binding {
            return Some(format!(
                "Formula: the LET or LAMBDA name '{name}' is spelled like a cell"
            ));
        }
        let name = self.wrong_count_call.as_ref()?;
        Some(format!(
            "Formula: {name} is called with the wrong number of arguments"
        ))
    }

    /// Records `name` as the first call of the formula with a count of arguments Excel refuses
    /// typed ([`Parser::typed_refusal`]); stored text records nothing.
    fn note_wrong_count(&mut self, name: &str) {
        if self.typed && self.wrong_count_call.is_none() {
            self.wrong_count_call = Some(name.to_ascii_uppercase());
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
        let empty_call_run = self.empty_call_run();
        let mut args = match self.parse_function_args(&display_name) {
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
        // Implicit intersection: `_xlfn.SINGLE(x)` is how a file stores `@x`, and Excel takes a
        // typed `SINGLE(x)`, in any case, as `@x` too and shows it so (`=SINGLE(C1:C3)` beside row 2
        // is the row's cell, FORMULATEXT `=@C1:C3`; probes/single-typed-*, xlsx-storage.md "Typed
        // SINGLE(x) is the @ operator", T-2938). Excel refuses `=SINGLE()` and `=SINGLE(1,2)` typed.
        if display_name.eq_ignore_ascii_case("SINGLE") {
            if args.len() != 1 {
                self.note_wrong_count(&display_name);
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
        // text it is a cell, so `NLG1(1)` is a call of that cell, `#REF!` (below, T-1936).
        if let Some(kind) =
            Function::from_legacy_spelling(&name).filter(|_| !self.lexer.is_a1_mode())
        {
            return self.function_call(kind, args);
        }
        // We should do this *only* importing functions from xlsx
        if let Some(function_kind) = self
            .language
            .functions
            .lookup_stored(&name)
            .filter(|_| !self.calls_bound_name(&name))
        {
            self.note_argument_count(&function_kind, &display_name, args.len());
            if self.loaded && function_kind == Function::Let {
                result_of_even_let(&mut args);
            }
            let node = self
                .function_call(function_kind, args)
                .with_empty_call_run(empty_call_run);
            // The result of a function that can return a reference (or a LAMBDA) may be called:
            // `INDEX(A1:A3,1)(2)` is `#REF!` and `IF(TRUE,LAMBDA(x,x*2))(3)` 6, while Excel
            // refuses `SUM(1)(2)`, `ABS(1)(2)` and `IFERROR(LAMBDA(x,x*2),0)(3)`
            // (probes/chained-calls-builtin-*, T-2405): every built-in `is_reference_operand` takes was probed
            // callable.
            if is_reference_operand(&node) {
                return self.chain_calls(node);
            }
            return node;
        }
        // A callee spelled like a cell, and no function (`LOG10` stays one), is a call of that cell:
        // `#REF!` like any called reference (probes/unknown-function-name-spelling A8, T-1758).
        // `$` shapes are unprobed, so they stay names; a LET or LAMBDA name spelled so is called
        // (`LET(mk,..,add5,mk(5),add5(1))`, T-2040). A1 grammar only: in R1C1 mode (stored
        // forms, and the raw A1 text of a formula that did not parse) `AB1` is no cell, and reading
        // it as one would turn A1 coordinates into R1C1 offsets.
        if let Some(cell) = parse_reference_a1(&name.to_ascii_uppercase()).filter(|_| {
            self.lexer.is_a1_mode()
                && !name.contains('$')
                && !self.binds_reference_shaped_name(&name)
        }) {
            let callee = self.reference_operand(None, cell.row, cell.column, false, false);
            return self.chain_calls(Node::LambdaCallKind {
                lambda: Box::new(callee),
                args,
            });
        }
        self.chain_calls(Node::NamedFunctionKind {
            name: name.trim_start_matches("_xlpm.").to_string(),
            args,
            id: None,
        })
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
            let empty_call_run = self.empty_call_run();
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
            let kind = if value {
                Function::True
            } else {
                Function::False
            };
            return Node::FunctionKind { kind, args }.with_empty_call_run(empty_call_run);
        }
        Node::BooleanKind(value)
    }

    /// A structured reference, `Table1[Column]`, kept as written around the area it stands for
    /// (SPEC-0016). `bare` marks a bare table name (`ROWS(Sales)`). `#This Row` lowers to a
    /// [`Node::ThisRowKind`], resolved at the evaluating cell's row (T-1699); a reference that
    /// does not resolve (unknown table or column) stays a parse error, which keeps the formula
    /// text.
    pub(super) fn structured_reference_operand(
        &mut self,
        table_name: String,
        specifier: Option<token::TableSpecifier>,
        table_reference: Option<TableReference>,
        spacing: token::TableSpacing,
        bare: bool,
    ) -> Node {
        let mut reference = StructuredReference {
            table_name: table_name.clone(),
            specifier: specifier.clone(),
            table_reference: table_reference.clone(),
            spacing,
            bare,
            unqualified: false,
            extent: None,
        };
        let lowered = self.lower_structured_reference(table_name, specifier, table_reference);
        if let Node::ThisRowKind {
            data_start,
            data_end,
            ..
        } = &lowered
        {
            reference.extent = self
                .table_named(&reference.table_name)
                .and_then(|table| TableExtent::of(table, Some((*data_start, *data_end))));
        }
        if let Node::ParseErrorKind { message, .. } = &lowered {
            if self.loaded
                && !message.starts_with(TABLE_SHEET_NOT_FOUND)
                && is_unresolved_structured_reference(message)
            {
                return Node::ErrorKind(token::Error::REF);
            }
            return lowered;
        }
        Node::StructuredReferenceKind {
            reference: Box::new(reference),
            lowered: Box::new(lowered),
        }
    }

    /// The table `name` names in the parser's table list. Table names match ignoring case
    /// (gen-m8b names-case 5b5c0e20e82153bb): an exact hit is deterministic; the case-folded scan
    /// covers the rest.
    pub(super) fn table_named(&self, name: &str) -> Option<&Table> {
        self.tables.get(name).or_else(|| {
            let lower = name.to_lowercase();
            self.tables
                .iter()
                .find(|(table_name, _)| table_name.to_lowercase() == lower)
                .map(|(_, table)| table)
        })
    }

    /// The area a structured reference resolves to against the parser's table list: a
    /// `ReferenceKind`, a `RangeKind`, a `ThisRowKind`, an `ErrorKind` or a `ParseErrorKind`.
    fn lower_structured_reference(
        &mut self,
        table_name: String,
        specifier: Option<token::TableSpecifier>,
        table_reference: Option<TableReference>,
    ) -> Node {
        // We will try to convert to a normal reference
        // table_name[column_name] => cell1:cell2
        // table_name[[#This Row], [column_name]:[column_name]] => the columns on the cell's row
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
        let table = match self.table_named(&table_name) {
            Some(t) => t,
            None => {
                let message = format!(
                    "{TABLE_NOT_FOUND}'{table_name}' at '{}!{}{}'",
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
                    message: format!("{TABLE_SHEET_NOT_FOUND}{}", table.sheet_name),
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

        let this_row = specifier == Some(token::TableSpecifier::ThisRow);
        match specifier {
            // Decided at the evaluating cell's row (`Node::ThisRowKind`), below.
            Some(token::TableSpecifier::ThisRow) => {}
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
        let one_column = matches!(table_reference, Some(TableReference::ColumnReference(_)));
        let lowered = match table_reference {
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
                            message: format!("{COLUMN_NOT_FOUND}{s} in table {table_name}"),
                        };
                    }
                };
                if row_start == row_end && !this_row {
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
                            message: format!("{COLUMN_NOT_FOUND}{left} in table {table_name}"),
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
                            message: format!("{COLUMN_NOT_FOUND}{right} in table {table_name}"),
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
        };
        if !this_row {
            return lowered;
        }
        // Off the data rows (header, totals, above, below) it is #VALUE!, at the evaluating
        // cell's row (docs/excel-semantics/references.md, gen-m8b this-row e1bf5aa0ffb4ed87).
        match lowered {
            Node::RangeKind {
                sheet_name,
                sheet_index,
                column1,
                column2,
                ..
            } => Node::ThisRowKind {
                sheet_name,
                sheet_index,
                column1,
                column2,
                data_start,
                data_end,
                cell: one_column,
            },
            other => other,
        }
    }
}

/// A stored LET with an even argument count ends on a name and a value, and Excel loads it with
/// the last name appended as the result: `<f>LET( a ,1, b ,a+1)</f>` prints
/// `=LET( a,1, b,a+1,b)` and returns 2 (probes/t2949-stored-11, syntax.md "Remaining whitespace
/// shapes (T-2949)"). Typed, Excel refuses the arity.
fn result_of_even_let(args: &mut Vec<Node>) {
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return;
    }
    let name = args[args.len() - 2].unwrap_spaced().clone();
    args.push(name);
}
