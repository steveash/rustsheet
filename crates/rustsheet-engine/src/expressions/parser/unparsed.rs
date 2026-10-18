//! The stored (R1C1) form of a formula that did not parse (T-1805).
//!
//! A cell formula that does not parse keeps its typed A1 text. Formulas are stored in R1C1 and
//! every reload or structural edit (`rename_sheet`, a defined-name rename) re-parses them in R1C1
//! mode, where that text can mean something else: `R[1]C[1]+1` is an error in A1 but a reference
//! in R1C1, and `Ab1 (1)` a call. So the stored form is marked and carries the error itself, and
//! the R1C1 parser hands it back unchanged instead of parsing the text.

use super::{ExpectedTokens, Node};

/// Starts the stored form. The lexer rejects it, so no formula that parses is stored with it first.
const MARK: char = '\u{1}';

/// The stored form of `node` if it is a parse error: [`MARK`], the error position, `:`, the
/// length of the message in bytes, `:`, then the message and the formula text.
pub(crate) fn stored_text(node: &Node) -> Option<String> {
    match node {
        Node::ParseErrorKind {
            formula,
            message,
            position,
            ..
        } => Some(format!(
            "{MARK}{position}:{}:{message}{formula}",
            message.len()
        )),
        _ => None,
    }
}

/// The parse error stored as `text` by [`stored_text`]; `None` for any other text, malformed
/// stored forms included (a loaded workbook is untrusted), which are then parsed as usual.
pub(crate) fn parse_stored_text(text: &str) -> Option<Node> {
    let (position, rest) = text.strip_prefix(MARK)?.split_once(':')?;
    let (length, rest) = rest.split_once(':')?;
    let length: usize = length.parse().ok()?;
    Some(Node::ParseErrorKind {
        formula: rest.get(length..)?.to_string(),
        message: rest.get(..length)?.to_string(),
        position: position.parse().ok()?,
        expecting: vec![ExpectedTokens::Other],
    })
}
