use super::{broadcast_static_result, run_static_analysis_on_node, Node, StaticResult};

/// AGGREGATE spills over an array function_num, options or fourth argument, broadcast against
/// each other (`Model::fn_aggregate`). A multi-cell range function_num or options lifts too
/// (probes/aggregate-range-args); a single cell and a range fourth argument are read, not lifted.
pub(super) fn aggregate_static_result(args: &[Node]) -> StaticResult {
    let lifted = args
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < 2 || (*index == 3 && args.len() == 4))
        .filter(|(index, arg)| match run_static_analysis_on_node(arg) {
            StaticResult::Range(rows, columns) => *index < 2 && (rows, columns) != (1, 1),
            _ => true,
        })
        .map(|(_, arg)| arg);
    broadcast_static_result(lifted)
}
