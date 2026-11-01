//! Excel's cap on nested function calls, applied on load wherever a formula is stored: cells,
//! defined-name bodies, and conditional-format and data-validation rules (syntax.md "Formula
//! nesting depth cap"; T-3097, T-3143, T-3263).

use rustsheet_engine::expressions::{
    parser::{Parser, MAX_NESTED_CALLS},
    types::CellReferenceRC,
};

use super::xml::XmlNode;
use crate::error::XlsxError;

/// The workbook error for a formula (`what`, e.g. `Formula in Sheet1!A1`) whose last parse by
/// `parser` nests more function calls than Excel takes ([`MAX_NESTED_CALLS`]): Excel refuses such
/// a workbook wherever the formula is stored (syntax.md "Formula nesting depth cap").
pub(super) fn refuse_too_many_calls(parser: &Parser, what: &str) -> Result<(), XlsxError> {
    if parser.nests_too_many_calls() {
        return Err(XlsxError::Workbook(format!(
            "{what} nests more than {MAX_NESTED_CALLS} function calls"
        )));
    }
    Ok(())
}

/// Refuses the workbook when a formula conditional format (`<cfRule type="expression">`) or a
/// custom data validation (`<dataValidation type="custom">`) on the sheet nests more than
/// [`MAX_NESTED_CALLS`] calls: Excel loads 65 and refuses the workbook at 66 for both
/// (createSession 501; probes/t3263-cf-abs-65, t3263-dv-abs-65, T-3263). Other rule types and the
/// x14 `extLst` forms are unprobed and not checked. Data validations are read for this only.
pub(super) fn check_rule_formulas(
    ws: &XmlNode,
    parser: &mut Parser,
    sheet_name: &str,
) -> Result<(), XlsxError> {
    // The cell a formula is parsed at moves its relative references but not its call nesting,
    // the only thing read here, so one origin serves every rule.
    let origin = CellReferenceRC {
        sheet: sheet_name.to_string(),
        row: 1,
        column: 1,
    };
    let rules = ws
        .children()
        .filter(|n| n.has_tag_name("conditionalFormatting"))
        .flat_map(|cf| cf.children().map(move |r| (cf, r)))
        .filter(|(_, r)| r.has_tag_name("cfRule") && r.attribute("type") == Some("expression"))
        .map(|(cf, r)| ("conditional format", cf, r, "formula"));
    let validations = ws
        .children()
        .filter(|n| n.has_tag_name("dataValidations"))
        .flat_map(|dv| dv.children())
        .filter(|v| v.has_tag_name("dataValidation") && v.attribute("type") == Some("custom"))
        .map(|v| ("data validation", v, v, "formula1"));
    // `sqref` is on the `<conditionalFormatting>` around a rule, and on the `<dataValidation>`.
    for (kind, scope, rule, tag) in rules.chain(validations) {
        let Some(formula) = rule
            .children()
            .find(|n| n.has_tag_name(tag))
            .and_then(|n| n.text())
        else {
            continue;
        };
        parser.parse(formula, &origin);
        let range = scope.attribute("sqref").unwrap_or_default();
        refuse_too_many_calls(
            parser,
            &format!("Formula of a {kind} on {sheet_name}!{range}"),
        )?;
    }
    Ok(())
}
