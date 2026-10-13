---
id: T-1420
title: "Fuzz OOM: D:D+4:4 broadcasts a whole column against a whole row into a 2^34-cell array"
milestone: M1
lane: engine
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A binary operator whose operands are a whole column and a whole row (or any pair whose broadcast
result exceeds the engine's array limit), such as `=D:D+4:4` or `=Data!$D:$D>Data!$4:44`,
evaluates in bounded memory and time instead of exhausting memory.

## Context
Nightly fuzz run 38109900859 (2026-10-11, `main` 39790b9), target `formula_eval`, artifact
`oom-41f35e39044be6988fb8c7d9c946e71cb529e886` (libFuzzer exit 71, out of memory). The input is
the 22 bytes `Data!$D:$D >Data!$4:44`, set as `=...` in `C3` of the target's small model
(`fuzz/fuzz_targets/formula_eval.rs`). Reproduced by the steward on 1337fe2 with a debug test
through the facade under `ulimit -v 4000000`: allocation failure and abort. Smaller forms abort
the same way: `=D:D>4:4`, `=D:D+4:4`, `=D1:D100000>4:4` and `=SUM(D:D>4:4)`.
A column of R rows against a row of C columns broadcasts to an R x C array (here 2^20 x 2^14), and
the arithmetic/comparison paths (`arithmetic.rs` `arithmetic_on_values`, `handle_comparison`)
build it without `check_array_size` (`limits.rs`, the
decision-0014 DEVIATION limit of 2^20 cells; docs/excel-semantics/arrays.md "Array results above
1,048,576 cells"). T-0047 bounded lifting a whole range into one array; this is the two-operand
broadcast. Check the result size before allocating (from the operand shapes, before
materialising both ranges where possible) and return the same error the limit gives elsewhere.
Keep Excel's results for broadcasts under the limit unchanged (existing probes and eval).

## Acceptance criteria
- [x] The reproducer and `=D:D+4:4` are unit tests (decision 0035) that finish quickly in a debug build with bounded memory.
- [x] Every binary operator (arithmetic, comparison, `&`) checks the broadcast size before allocating.
- [x] docs/excel-semantics/arrays.md "Array results above 1,048,576 cells" (the decision-0014 DEVIATION) names the operator broadcast path, or is extended to.
- [x] 0 eval regressions; `make check` green; reviewers approve (performance reviewer included).

## Out of scope
- Raising or changing the 2^20-cell limit itself.

## Outcome
The three array-by-array operator broadcasts (arithmetic, `&`, comparisons) became one
`broadcast_arrays` in crates/rustsheet-engine/src/arithmetic.rs, which checks the result shape with
`check_array_size` before allocating: the fuzz reproducer, `=D:D+4:4`, `=SUM(D:D>4:4)` and the other
forms are `#ERROR!` (DEVIATION, decision 0014) in ~0.5 s of a debug test; without the fix the test
aborts on allocation under `ulimit -v 4000000`. Duplicated lines 6647 → 6598. Eval delta: none
(0 fixed, 0 regressions). Follow-up: T-1448 (IF and IFERROR/IFNA broadcast the same way, unbounded).
