use std::collections::HashMap;

use crate::expressions::parser::stringify::retarget_unqualified_references;
use crate::expressions::parser::tests::utils::new_parser;
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceRC;

/// The sheet index of every reference and range of `node`, with whether it was written with a
/// sheet prefix.
fn reference_sheets(node: &Node) -> Vec<(bool, u32)> {
    let mut out = Vec::new();
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        if let Node::ReferenceKind {
            sheet_name,
            sheet_index,
            ..
        }
        | Node::RangeKind {
            sheet_name,
            sheet_index,
            ..
        } = node
        {
            out.push((sheet_name.is_some(), *sheet_index));
        }
        node.push_children(&mut pending);
    }
    out.sort_unstable();
    out
}

#[test]
fn only_unprefixed_references_move_to_the_database_sheet() {
    let worksheets = vec!["Db".to_string(), "Data".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    let context = CellReferenceRC {
        sheet: "Db".to_string(),
        row: 2,
        column: 15,
    };
    let mut node = parser.parse("AND(B2>3,SUM(C1:C3)>0,Db!B2<0,Data!B2>0)", &context);
    assert_eq!(
        reference_sheets(&node),
        [(false, 0), (false, 0), (true, 0), (true, 1)]
    );
    retarget_unqualified_references(&mut node, 1);
    assert_eq!(
        reference_sheets(&node),
        [(false, 1), (false, 1), (true, 0), (true, 1)]
    );
}
