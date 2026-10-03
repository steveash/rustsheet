use super::{run_static_analysis_on_node, Node, StaticResult};

/// ROW and COLUMN give a single number for a single cell, and when the
/// reference stays within one row (ROW) or one column (COLUMN). Otherwise they
/// give a vector of numbers, one for each row or column of the reference.
pub(super) fn static_analysis_row_column(args: &[Node], is_row: bool) -> StaticResult {
    let Some(reference) = args.first() else {
        return StaticResult::Scalar;
    };
    // The vector is `Unknown` and not `Array`: a formula from before dynamic
    // arrays expects a single number here, and it is the unknown results that
    // are intersected (see `add_implicit_intersection`).
    match reference {
        Node::RangeKind {
            absolute_row1,
            absolute_column1,
            row1,
            column1,
            absolute_row2,
            absolute_column2,
            row2,
            column2,
            ..
        } => {
            // The two ends are only comparable if both are absolute or both
            // are relative to the cell.
            let single = if is_row {
                absolute_row1 == absolute_row2 && row1 == row2
            } else {
                absolute_column1 == absolute_column2 && column1 == column2
            };
            if single {
                StaticResult::Scalar
            } else {
                StaticResult::Unknown
            }
        }
        other => match run_static_analysis_on_node(other) {
            StaticResult::Scalar => StaticResult::Scalar,
            _ => StaticResult::Unknown,
        },
    }
}
