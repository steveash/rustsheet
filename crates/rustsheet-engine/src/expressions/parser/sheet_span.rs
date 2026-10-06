//! Sheet prefixes of references: one sheet (`Data!A1`) or a 3-D span of sheets
//! (`First:Last!A1`, docs/excel-semantics/references.md "3-D sheet-range references").

use crate::expressions::parser::{Node, Parser};

/// The two ends of a 3-D reference prefix (`First:Last!`), as written and as sheet indices.
pub(super) struct SheetSpan {
    first_sheet: String,
    last_sheet: String,
    first_index: u32,
    last_index: u32,
}

/// `node`, a reference written without a sheet prefix, under the 3-D prefix `span` if any.
pub(super) fn with_sheet_span(span: Option<SheetSpan>, node: Node) -> Node {
    match span {
        Some(span) => Node::SheetSpanKind {
            first_sheet: span.first_sheet,
            last_sheet: span.last_sheet,
            first_index: span.first_index,
            last_index: span.last_index,
            reference: Box::new(node),
        },
        None => node,
    }
}

impl Parser {
    /// The sheet a reference token's prefix names: `(span, sheet, index)`. For a 3-D prefix the
    /// span is `Some` and the reference itself is built on the first sheet with no prefix; a
    /// missing prefix is the formula's own sheet; an unknown sheet has no index (`#REF!`).
    pub(super) fn resolve_sheet_prefix(
        &self,
        sheet: Option<String>,
    ) -> (Option<SheetSpan>, Option<String>, Option<u32>) {
        if let Some(span) = sheet.as_deref().and_then(|name| self.get_sheet_span(name)) {
            let index = span.first_index;
            return (Some(span), None, Some(index));
        }
        let index = match &sheet {
            Some(name) => self.get_sheet_index_by_name(name),
            None => self.get_sheet_index_by_name(&self.context.sheet),
        };
        (None, sheet, index)
    }

    /// The sheets of a 3-D reference prefix `First:Last` (a `:` cannot occur in a sheet name),
    /// or `None` if `name` is not a span or either end is not a sheet (then the reference is
    /// `#REF!` like any other on a missing sheet).
    fn get_sheet_span(&self, name: &str) -> Option<SheetSpan> {
        let (first, last) = name.split_once(':')?;
        Some(SheetSpan {
            first_index: self.get_sheet_index_by_name(first)?,
            last_index: self.get_sheet_index_by_name(last)?,
            first_sheet: first.to_string(),
            last_sheet: last.to_string(),
        })
    }

    pub(super) fn get_sheet_index_by_name(&self, name: &str) -> Option<u32> {
        let worksheets = &self.worksheets;
        for (i, sheet) in worksheets.iter().enumerate() {
            if sheet == name {
                return Some(i as u32);
            }
        }
        None
    }
}
