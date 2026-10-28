---
id: T-1564
title: "Size array arms of CHOOSE, IF and HSTACK/VSTACK from range bounds and copy only picked arms"
milestone: M13
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
CHOOSE over an index array, IF over an array condition and HSTACK/VSTACK hold at most the output plus the arms they actually read: a formula with many large range arms (up to 254 for CHOOSE) is rejected or computed without first copying every arm in full.

## Context
T-1508 performance review: `choose_array` (functions/lookup_and_reference/mod.rs) evaluates every value argument through `evaluate_if_arg` (functions/logical/mod.rs), copying each range into a `Vec<Vec<ArrayNode>>` (each capped by `check_range_size`) before `check_array_size` checks the output, so `CHOOSE({1,2},A1:A1048576,…)` can hold about 254 × 1M nodes. `if_array` and `hstack_vstack.rs` do the same. Excel sizes the result from every arm, picked or not (lookup.md, "CHOOSE over an index array broadcasts its value arguments"), so take dims from `Range` bounds, check the output size, then copy only picked arms (a lazy `IfArg` variant). T-1508 architecture review: while there, move `IfArg`/`if_arg_at`/`if_arg_dims`/`evaluate_if_arg` to a neutral module with a neutral name (e.g. `BroadcastArg`), since CHOOSE shares them.

## Acceptance criteria
- [x] Measurement in the Outcome: peak memory and time of `CHOOSE({1,2},A1:A1048576,…)` with many unpicked whole-column arms, before/after (release).
- [x] probes/choose-array-index-range-arms and the IF/HSTACK/VSTACK cases still pass; no eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any change to the result values or shapes.

## Outcome
CHOOSE over an index array and IF over an array condition take each arm's extent from its range bounds
(`BroadcastArg::Range`, functions/broadcast_arg.rs; IF's `IfArg`, `if_arg_at`, `if_arg_dims` and `evaluate_if_arg`
moved there as `BroadcastArg`, `BroadcastArg::at`, `BroadcastArg::dims` and `evaluate_broadcast_arg`). They check the
output size before any range arm is read. A picked range arm is then copied. An unpicked one is read cell by cell
within the used area, in the same order, but not kept (`Model::read_range_cells`, cast.rs), so a cycle through it and
the dependencies it records are as before. It is charged only the cells it reads. HSTACK evaluates every argument
before reading any range, as VSTACK does, and both check the stacked shape from the arguments' extents before copying.

Measurement (release build, peak resident memory `VmHWM` of a process with C1:BJ10 filled and one formula in A1;
"N columns" are the whole-column arms `C:C,D:D,…`):

| formula | before (main) | after |
|---|---|---|
| `ROWS(CHOOSE({1,2},`60 columns`))` (2 picked, output refused) | `#ERROR!`, 21.7 s, 3.4 GB | `#ERROR!`, 0 ms, 8 MB |
| `ROWS(CHOOSE({1,…,60},`60 columns`))` (all picked, refused) | `#ERROR!`, 12.2 s, 3.4 GB | `#ERROR!`, 0 ms, 8 MB |
| `SUM(CHOOSE({1},`60 columns`))` | 5500, 14.9 s, 3.5 GB | 5500, 0.18 s, 123 MB |
| `SUM(CHOOSE({1},`254 columns`))` | `#VALUE!` (work budget), 13.0 s, 4.9 GB | 5500, 0.19 s, 123 MB |
| `SUM(IF({TRUE},C:C,D:D))` | 5500, 0.37 s, 180 MB | 5500, 0.16 s, 123 MB |
| `ROWS(HSTACK(`60 columns`))` | `#ERROR!`, 6.4 s, 3.4 GB | `#ERROR!`, 0 ms, 8 MB |
| `ROWS(HSTACK(`254 columns`))` | `#VALUE!` (work budget), 24.8 s, 4.9 GB | `#ERROR!` (array cells), 0 ms, 8 MB |

Two values change, both past our own limits: CHOOSE with many unpicked whole-column arms now computes, as Excel does,
instead of running out of the work budget; HSTACK of 254 whole columns is the array-cell `#ERROR!` (DEVIATION, arrays.md
"Array results above 1,048,576 cells") instead of the work budget's `#VALUE!`. Tests: test_fn_choose.rs,
test_fn_if.rs, test_fn_hstack_vstack.rs. Eval delta: none.
