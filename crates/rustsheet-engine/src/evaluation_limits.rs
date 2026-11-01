//! How deep the evaluation's recursion may go, in formulas and in bytes of stack, before it unwinds
//! and replays (`evaluation.rs`).

use crate::model::Model;

/// How deep the recursion may go when nobody has said otherwise, in formulas
/// and in bytes of stack. Measured on a chain of `=A2+1`: a formula takes about
/// 3 KB of stack in a release build and 16 KB in a debug build, and three to
/// four times that when it nests a few functions. The limits are far below
/// what the smallest common stacks hold (1 MB in WebAssembly, 2 MB for a
/// thread) and cost nothing to workbooks that never reach them.
pub(crate) const DEFAULT_MAX_DEPTH: usize = 64;
pub(crate) const DEFAULT_MAX_STACK: usize = 256 * 1024;

/// Roughly where the stack is: the address of a local.
#[inline(never)]
pub(crate) fn stack_address() -> usize {
    let marker = 0u8;
    std::hint::black_box(&marker) as *const u8 as usize
}

/// The limit a new model starts with. Under test it can be set from outside,
/// `IRONCALC_MAX_DEPTH=1 cargo test`, so that every test of the crate runs
/// through the unwind and the replay.
pub(crate) fn default_max_depth() -> usize {
    #[cfg(test)]
    if let Some(max_depth) = std::env::var("IRONCALC_MAX_DEPTH")
        .ok()
        .and_then(|value| value.parse().ok())
    {
        return max_depth;
    }
    DEFAULT_MAX_DEPTH
}

impl Model {
    /// Sets how many formulas deep the evaluation may recurse. A formula that
    /// reads a formula that reads a formula, and so on, uses the stack of the
    /// thread for every step. Beyond this limit the evaluation unwinds and
    /// carries on from where it stopped instead of going deeper, so the limit
    /// bounds the stack used and never changes a result, except a recalculation
    /// whose thrown-away work reaches the bound of SPEC-0028 (docs/excel-semantics/
    /// recalc.md "One recalculation does at most 2^32 work units"). Lower it where the
    /// stack is small. The least it can be is 1. See also
    /// `set_max_evaluation_stack`: whichever limit is reached first applies.
    pub fn set_max_evaluation_depth(&mut self, max_depth: usize) {
        self.evaluation.max_depth = max_depth.max(1);
    }

    /// The limit set by `set_max_evaluation_depth`.
    pub fn get_max_evaluation_depth(&self) -> usize {
        self.evaluation.max_depth
    }

    /// Sets how many bytes of stack the evaluation may use before it unwinds,
    /// counted from the first formula of the recursion. The evaluation can
    /// overshoot by what one formula needs, so leave room: a quarter of the
    /// stack is a reasonable value. Like `set_max_evaluation_depth`, it never
    /// changes a result, except a recalculation whose thrown-away work reaches
    /// the bound of SPEC-0028.
    pub fn set_max_evaluation_stack(&mut self, bytes: usize) {
        self.evaluation.max_stack = bytes;
    }

    /// The limit set by `set_max_evaluation_stack`.
    pub fn get_max_evaluation_stack(&self) -> usize {
        self.evaluation.max_stack
    }
}
