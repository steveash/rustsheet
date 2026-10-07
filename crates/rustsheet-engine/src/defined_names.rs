//! Workbook defined names: creating, validating, renaming and deleting them.

use crate::{
    calc_result::{CalcResult, Range},
    evaluation::stack_address,
    expressions::{
        lexer::LexerMode,
        parser::{
            stringify::{rename_defined_name_in_node, to_rc_format},
            NamedVariable, Node,
        },
        token::Error,
        types::{CellReferenceIndex, CellReferenceRC},
        utils::is_valid_identifier,
    },
    model::Model,
    types::DefinedName,
    utils as common,
};

/// A parsed formula for a defined name
#[derive(Clone)]
pub(crate) enum ParsedDefinedName {
    /// CellReference (`=C4`)
    CellReference(CellReferenceIndex),
    /// A Range (`=C4:D6`)
    RangeReference(Range),
    /// `=LAMBDA(params..., body)`
    LambdaDefinition(Vec<NamedVariable>, Node),
    /// An error literal, possibly sheet-qualified (`=Sheet1!#REF!`): the name evaluates to it
    ErrorLiteral(Error),
    /// Any other formula (`=SUM(Sheet1!$A$1:$A$3)`, `=Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2`,
    /// `=0.5`): evaluated where the name is used, in that cell's context
    Formula(std::sync::Arc<Node>),
    /// A body that does not parse
    InvalidDefinedNameFormula,
}

/// How many defined-name bodies may be evaluated inside one another. A name whose body
/// reaches itself, directly (`=selfref+1`) or through others (`a` is `=b`, `b` is `=a`), is
/// `#NAME?` in Excel (probes/defined-name-formula-bodies H11, H13); the engine gives that by
/// running out of depth or budget. A chain of distinct names deeper than this is `#NAME?` too,
/// where Excel would evaluate it (DEVIATION, decision 0014: references.md "A defined name may be
/// a formula").
const MAX_DEFINED_NAME_DEPTH: usize = 64;

/// How many name bodies one cell's formula may evaluate in all. The depth limit alone
/// does not bound the work: `x` = `=x+x` doubles it at every level. Past this the use is
/// `#NAME?` (same DEVIATION).
const MAX_DEFINED_NAME_EVALUATIONS: usize = 10_000;

/// The state of the defined-name bodies being evaluated right now (`Model::within_defined_name`).
#[derive(Default)]
pub(crate) struct DefinedNameRun {
    /// Bodies evaluated inside one another.
    depth: usize,
    /// Bodies evaluated for `owner` so far.
    evaluations: usize,
    /// The evaluation pass and the cell the count belongs to.
    owner: Option<(u64, Option<CellReferenceIndex>)>,
    /// Where the stack was when the outermost one started.
    stack_base: usize,
}

impl Model {
    /// Runs `f` one defined-name body deeper, or gives `None` past [`MAX_DEFINED_NAME_DEPTH`],
    /// [`MAX_DEFINED_NAME_EVALUATIONS`] or the evaluation's stack budget, so that names
    /// referring to themselves stay bounded in depth, time and stack.
    pub(crate) fn within_defined_name<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> Option<T> {
        let here = stack_address();
        // Inside a cell the budget belongs to that cell's formula in this `evaluate()`; outside
        // one (conditional formats, `evaluate_formula`) every outermost use starts afresh.
        let owner = (self.evaluation.depth > 0).then(|| {
            (
                self.evaluation.evaluations,
                self.evaluation.stack.last().copied(),
            )
        });
        // Inside a cell, measure from where its recursion started, so that names and cells
        // share one stack budget (evaluation.rs).
        let cell_base = (self.evaluation.depth > 0).then_some(self.evaluation.stack_base);
        let run = &mut self.defined_names_running;
        if run.depth == 0 {
            // One budget per cell formula, however many names it uses (`=x+x+x`).
            if owner.is_none() || run.owner != owner {
                run.owner = owner;
                run.evaluations = 0;
            }
            run.stack_base = cell_base.unwrap_or(here);
        }
        if run.depth >= MAX_DEFINED_NAME_DEPTH
            || run.evaluations >= MAX_DEFINED_NAME_EVALUATIONS
            || run.stack_base.abs_diff(here) >= self.evaluation.max_stack
        {
            return None;
        }
        run.depth += 1;
        run.evaluations += 1;
        let result = f(self);
        self.defined_names_running.depth -= 1;
        Some(result)
    }

    /// The value of the defined name `name` whose body is the formula `body`, used in `cell`.
    pub(crate) fn evaluate_defined_name_formula(
        &mut self,
        name: &str,
        body: &Node,
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.within_defined_name(|model| model.evaluate_node_in_context(body, cell))
            .unwrap_or_else(|| {
                CalcResult::new_error(
                    Error::NAME,
                    cell,
                    format!("Defined name \"{name}\" refers to itself."),
                )
            })
    }

    /// Adds a new defined name.
    /// If scope is None it is a global defined name, otherwise it is local to the sheet with index scope.
    pub fn new_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        self.is_valid_defined_name(name, scope, formula)?;
        self.define_name(name, scope, formula)
    }

    /// Adds a defined name whose formula is any formula that parses, as Excel stores them: an
    /// intersection is a valid name body (docs/excel-semantics/references.md "A defined name may
    /// be an intersection"). [`Model::new_defined_name`] additionally restricts the body to a
    /// reference or a LAMBDA.
    ///
    /// If scope is None it is a global defined name, otherwise it is local to the sheet with
    /// index scope. Reparses and re-evaluates the workbook.
    ///
    /// # Errors
    /// An invalid or duplicate name, an invalid scope, or a formula that does not parse.
    pub fn define_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<(), String> {
        let sheet_id = self.defined_name_slot(name, scope)?;
        // Defined-name formulas are stored internally in English so they keep
        // working when the user switches language/locale.
        let context = self.defined_name_context();
        let internal_formula = self.user_formula_to_internal(formula, &context)?;
        self.workbook.defined_names.push(DefinedName {
            name: name.to_string(),
            formula: internal_formula,
            sheet_id,
        });
        self.reset_parsed_structures();

        Ok(())
    }

    /// The context used to parse/stringify defined-name formulas. Defined names
    /// have no natural anchor cell, so we use the first worksheet's A1.
    pub(crate) fn defined_name_context(&self) -> CellReferenceRC {
        CellReferenceRC {
            sheet: self
                .workbook
                .worksheets
                .first()
                .map(|ws| ws.get_name())
                .unwrap_or_else(|| "Sheet1".to_string()),
            row: 1,
            column: 1,
        }
    }

    /// The `sheet_id` a new defined name `name` in `scope` would get, or an error when the name
    /// is not a valid identifier, the scope is not a sheet index, or the name already exists there.
    fn defined_name_slot(&self, name: &str, scope: Option<u32>) -> Result<Option<u32>, String> {
        if !is_valid_identifier(name) {
            return Err("Name: Invalid defined name".to_string());
        }
        let name_upper = name.to_uppercase();
        let sheet_id = match scope {
            Some(index) => match self.workbook.worksheet(index) {
                Ok(ws) => Some(ws.sheet_id),
                Err(_) => return Err("Scope: Invalid sheet index".to_string()),
            },
            None => None,
        };
        if self
            .workbook
            .defined_names
            .iter()
            .any(|df| df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id)
        {
            return Err("Name: Defined name already exists".to_string());
        }
        Ok(sheet_id)
    }

    /// Validates if a defined name can be created
    pub fn is_valid_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        formula: &str,
    ) -> Result<Option<u32>, String> {
        let sheet_id = self.defined_name_slot(name, scope)?;

        // Make sure the formula is valid: a cell/range reference, a LAMBDA definition or an error
        // literal such as `Sheet1!#REF!`.
        let is_reference =
            common::ParsedReference::parse_reference_formula(None, formula, self.locale, |name| {
                self.get_sheet_index_by_name(name)
            })
            .is_ok();

        if !is_reference {
            // Try the full parser to see if it is a LAMBDA definition.
            // Defined-name formulas may carry a leading '='; strip it before parsing.
            let formula_body = formula.strip_prefix('=').unwrap_or(formula);
            let dummy_ref = self.defined_name_context();
            // Accept the formula whether it is written in the active language
            // or already in the internal English form (e.g. generated by
            // undo/redo or cut & paste).
            let mut node = self.parser.parse(formula_body, &dummy_ref);
            if let Node::ParseErrorKind { .. } = node {
                node = self.parse_internal_formula(formula_body, &dummy_ref);
            }
            if !matches!(node, Node::LambdaDefKind { .. } | Node::ErrorKind(_)) {
                return Err("Formula: Invalid defined name formula".to_string());
            }
        }

        Ok(sheet_id)
    }

    /// Delete defined name of name and scope
    pub fn delete_defined_name(&mut self, name: &str, scope: Option<u32>) -> Result<(), String> {
        let name_upper = name.to_uppercase();
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(self.workbook.worksheet(index)?.sheet_id),
            None => None,
        };
        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        if let Some(i) = index {
            self.workbook.defined_names.remove(i);
            self.reset_parsed_structures();
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }

    /// Update defined name
    pub fn update_defined_name(
        &mut self,
        name: &str,
        scope: Option<u32>,
        new_name: &str,
        new_scope: Option<u32>,
        new_formula: &str,
    ) -> Result<(), String> {
        if !is_valid_identifier(new_name) {
            return Err("Name: Invalid defined name".to_string());
        };
        let name_upper = name.to_uppercase();
        let new_name_upper = new_name.to_uppercase();

        if name_upper != new_name_upper || scope != new_scope {
            for key in self.parsed_defined_names.keys() {
                if key.1.to_uppercase() == new_name_upper && key.0 == new_scope {
                    return Err("Name: Defined name already exists".to_string());
                }
            }
        }
        let defined_names = &self.workbook.defined_names;
        let sheet_id = match scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let new_sheet_id = match new_scope {
            Some(index) => Some(
                self.workbook
                    .worksheet(index)
                    .map_err(|_| "Scope: Invalid sheet index")?
                    .sheet_id,
            ),
            None => None,
        };

        let mut index = None;
        for (i, df) in defined_names.iter().enumerate() {
            if df.name.to_uppercase() == name_upper && df.sheet_id == sheet_id {
                index = Some(i);
            }
        }
        // Defined-name formulas are stored internally in English.
        let context = self.defined_name_context();
        let internal_formula = self.user_formula_to_internal(new_formula, &context)?;
        if let Some(i) = index {
            if let Some(df) = self.workbook.defined_names.get_mut(i) {
                if new_name != df.name {
                    // We need to rename the name in every formula:

                    // Parse all formulas with the old name
                    // All internal formulas are R1C1
                    self.parser.set_lexer_mode(LexerMode::R1C1);
                    let worksheets = &mut self.workbook.worksheets;
                    for worksheet in worksheets {
                        let cell_reference = CellReferenceRC {
                            sheet: worksheet.get_name(),
                            row: 1,
                            column: 1,
                        };
                        let mut formulas = Vec::new();
                        for formula in &worksheet.shared_formulas {
                            let mut t = self.parser.parse(formula, &cell_reference);
                            rename_defined_name_in_node(&mut t, name, scope, new_name);
                            formulas.push(to_rc_format(&t));
                        }
                        worksheet.shared_formulas = formulas;
                    }
                    // Se the mode back to A1
                    self.parser.set_lexer_mode(LexerMode::A1);
                }
                df.name = new_name.to_string();
                df.sheet_id = new_sheet_id;
                df.formula = internal_formula;
                self.reset_parsed_structures();
            }
            Ok(())
        } else {
            Err("Defined name not found".to_string())
        }
    }
}
