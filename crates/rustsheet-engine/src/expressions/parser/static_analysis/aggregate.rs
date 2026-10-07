use super::{broadcast_static_result, run_static_analysis_on_node, Node, StaticResult};

/// AGGREGATE spills over an array function_num, options or fourth argument, broadcast against
/// each other (`Model::fn_aggregate`); references (ranges) are read, not lifted.
pub(super) fn aggregate_static_result(args: &[Node]) -> StaticResult {
    let lifted = args
        .iter()
        .enumerate()
        .filter(|(index, _)| *index < 2 || (*index == 3 && args.len() == 4))
        .map(|(_, arg)| arg)
        .filter(|arg| !matches!(run_static_analysis_on_node(arg), StaticResult::Range(_, _)));
    broadcast_static_result(lifted)
}
