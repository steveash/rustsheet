---
id: T-1770
title: "SPEC-0011 argument-kind table, lift_call and static-analysis hooks; first consumers EDATE, EOMONTH, WEEKNUM and YEARFRAC"
milestone: M9
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1767, T-1784]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
SPEC-0011's argument-kind table and `lift_call` front end are in place, static analysis derives its signatures, shapes and legacy `@` from the table, and EDATE, EOMONTH, WEEKNUM and YEARFRAC are the first consumers. They lift over an array, and a multi-cell range in their probed arguments gives `#VALUE!`, with no implicit intersection, in dynamic and in legacy formulas.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md (Design, Test plan). Evidence: docs/excel-semantics/arrays.md "Scalar arguments lift: shapes, element errors, and the Analysis ToolPak range rule", and dates.md "Date function argument modes".
- Build on T-1784's `functions/lift/driver.rs`.
- Add `functions/lift/kinds.rs`: `ArgKind`, `KernelWork`, `ArgKinds` and `Function::arg_kinds`, one `match` returning `&'static`. Enter EDATE, EOMONTH, WEEKNUM and YEARFRAC as `LiftArrays` in every argument (all are probed over a range: EDATE's start and months, YEARFRAC's basis and dates, EOMONTH and WEEKNUM in date-argument-modes).
- Add the read phase: non-generic, `#[inline(never)]`, values held in an inline `ArgBuf`, and no heap or grid machinery on the unlifted path. Add `lift_call`.
- Add the borrowing casts (`cast_number_ref`, …). `cast_to_*` delegates to them.
- Add the three bounds: input cells, output cells, and work with `Prepared` and `Weighted`.
- Add the in-place single-grid path.
- Static analysis:
  - The table is consulted first.
  - The old arms move into `fallback_signature` and `fallback_shape`.
  - `lifted_static_result` uses `broadcast_static_result`.
  - `legacy_value_operand` takes the argument index.
  - The four functions' old arms are deleted.
- Element results go through `cast::calc_result_to_array_node`, with no new helper.
- The engine intersects today: `ROWS(EDATE(A1:A60,1))` is 1 on the range's rows. Legacy `=EDATE(A1:A60,1)` gets an `@` that Excel does not add.
Cells, all failing in eval/baseline/probes.txt:
- probes/date-argument-modes F/G 20, 22, 24, 26, 54, 56, 66, 68.
- probes/scalar-lift-shapes H1, H2, H3, H5, H6, H22, H30, H36, H37, H45, H49, H50, J2, J6, J7, K2, K6, K7.
- probes/scalar-lift-shapes-b C3, C70, D3, D70, E3, E70, F3, F70, G3, K3, L3, L70, N3, N70, P3, Q3.
- probes/scalar-lift-shapes-c E3, E70, H3, H70, P3, S3.
- gen-m9/4df1b22d3ed63f58 A24 (EOMONTH over an array start: `#SPILL!`) and gen-m9/b58b6d7cd39d195f A9 (EOMONTH's months a TAKE reference: `#VALUE!`). Both moved here from T-1236.
- corpus-bench/45738 (26 cells, T-0427, excel-graph): `Sheet1!L48` `=$B6*ISNUMBER(MATCH(L$4,EOMONTH($D6,-ROW($AI$1:$AI$8)*12/$C6),))+(EOMONTH($D6,0)=L$4)*($A6+$B6)` expected 1000, engine 0: EOMONTH's months argument is a computed array, EOMONTH gives `#N/IMPL!`, so MATCH never matches (also O49, M50, F51, I52, E53, J54, ...). Its `F` line should leave eval/baseline/corpus-bench.txt.

## Acceptance criteria
- [x] The cells listed under Context pass and leave eval/baseline/probes.txt (and gen-m9's baseline), with no new F line in any suite.
- [x] The unit tests of SPEC-0011's Test plan pass. They cover shapes and `#N/A` padding, errors per element, `LiftArrays` alone and next to an array, `Whole` evaluated once, the unlifted path calling the kernel once without a `LiftedArg` or grid, and the table-wins test (no function in the table also has a fallback arm).
- [x] Size tests on a filled fixture with explicit ranges:
  - `=ROWS(EDATE(A1:A1048576+0,A1:B1+0))` is `#ERROR!` with 0 kernel calls.
  - The summed-input-cells bound refuses before the next grid is read.
  - `LiftCost::Cells` trips the work bound before the first element, and `Prepared` runs.
- [x] Kernels on the driver build element errors with an empty message (`String::new()`), so an error element allocates nothing.
- [x] `test_nested_formula_stack` passes with a deeply nested lifted EDATE formula added.
- [x] docs/excel-semantics/dates.md and arrays.md say "Implemented in" for the four functions.
- [x] `make check` green.
- [ ] Reviewers approve.

## Out of scope
- WORKDAY, NETWORKDAYS and their .INTL forms (T-1236). The other families (T-1372, T-1290, T-1772, T-1383). Moving the old helpers (T-1773, T-1785, T-1774, T-1775).

## Outcome
The lifting table, `lift_call` and the static-analysis hooks are in, and EDATE, EOMONTH, WEEKNUM and YEARFRAC are their first consumers (`LiftArrays` in every argument).

- **Table** (`functions/lift/kinds.rs`): `ArgKind`, `KernelWork`, `ArgKinds` (`fixed`, `kind`, `accepts`) and `Function::arg_kinds`, one `match` over shared `const`s, each entry citing its probes. All four are `KernelWork::Constant`: YEARFRAC's basis-1 year-length loop is now a closed-form leap-year count, so no kernel loops on argument values. The closed form is not the loop for a start year after the end year (the empty loop gave `#NUM!`, the formula a number), so that case returns `#NUM!` explicitly, exactly as before (Excel swaps the dates, gen-m7/c263b43217a5898b Calc!A38; T-1354). The first review found that regression: gen-m7 has no gate.
- **Read phase and `lift_call`** (`functions/lift/call.rs`): `read_lift_args` is non-generic and `#[inline(never)]`, holds the values in an inline `ArgBuf` (8 slots, `Vec` fallback, `functions/lift/arg_buf.rs`) and the grids apart, so an unlifted call hands the kernel a borrowed slice and builds no `LiftedArg`. A `LiftArrays` multi-cell range (or a range across sheets) is `#VALUE!` in its slot; INDEX, XLOOKUP, TAKE, DROP and TRIMRANGE are read in their reference form there, so `EOMONTH(..,TAKE(..))` is refused like a written range (gen-m9/b58b6d7cd39d195f A9). A one-cell range is its value. A 3-D range is `#VALUE!` in a `LiftArrays` argument and `#REF!` in a `Lift` one (T-1777's probes/scalar-lift-open-b W2-W11, -d W17-W29, now on main; unit-tested here, no `Lift` consumer yet). A one-cell 3-D range in a `LiftArrays` argument and the XLOOKUP, DROP and TRIMRANGE reference-form reads are inferred, not probed; T-1801's context now lists them to probe. The input bound sums grid cells and refuses before the next range is read.
- **Driver** (`functions/lift/driver.rs`): `lift_values` (the direct callers' entry, unchanged signature) and `lift_call` share `lift_grids`: output bound, then the spec's work model (`Unit`, `Cells`, `Prepared`, `Walk { range, walks }`, times the `KernelWork` weight), then either the in-place single-grid path or the general broadcast; element texts count against the 64 MiB budget. `LiftOptions` gains `work`. The direct callers keep their T-1784 totals exactly (*IF(S) `Walk { walks: criteria }`, `lookup_each` and AGGREGATE multiply by their lifted arguments, IFS and SWITCH `Cells(arguments)`); XLOOKUP charges one search per element (it had divided the search by its three criteria to undo the old factor, so it only loosens by that rounding). The general path no longer clones every result when it does not reuse runs. `reuse_equal_runs`' 0/−0 equality is documented on the field (only its callers can see it, and they match 0 and −0 alike).
- **Borrowing casts** (cast.rs): `cast_number_ref`, `cast_number_no_bools_ref`, `cast_bool_ref`, `cast_string_ref`; `cast_to_number`, `cast_to_number_no_bools`, `cast_to_bool` and `cast_to_string` are `dereference` + the borrowing cast (the error arm moves, as before). Kernel-built element errors have empty messages (unit-tested); the casts keep their existing messages, so a cast failure still allocates (recorded in SPEC-0011 "Kernels"). `lift_in_place` holds its values in an `ArgBuf` too.
- **Kernels**: EDATE and EOMONTH moved to `date_and_time/month_shift.rs` (`edate_of`, `eomonth_of`); `weeknum_of`, `yearfrac_of`. Checks keep each kernel's own order: the driver runs no error pre-scan, and a refused range reaches the kernel as `#VALUE!` in its slot, so the kernel decides which error wins (`EDATE(NA(),A1:A60)` is `#N/A`, `EDATE(A1:A60,NA())` `#VALUE!`, as T-1777 found). YEARFRAC still checks start, end, basis; Excel checks basis first (T-1777 -d W1-W7), which T-1800 fixes.
- **Direct callers of the four**: ACCRINTM handed its raw basis node to `fn_yearfrac`, which then lifted it (`ACCRINTM(39083,39448,0.1,1000,{0;0})` gave YEARFRAC's {1;1}, so probes/fin-argument-modes H17 passed for the wrong reason). It now reads basis as a scalar (`get_number_no_bools`, as YEARFRAC did) and calls `get_yearfrac`, as DISC and PRICEMAT do, so it behaves as on main; its lifting is T-1372's. `get_yearfrac` passes number nodes only. No other code calls `fn_edate`, `fn_eomonth`, `fn_weeknum` or `fn_yearfrac`.
- **Static analysis** (`static_analysis/lifted.rs`): the table is consulted first. The old arms are `fallback_signature`/`fallback_shape` (the four functions' hand-written arms replaced by explicit `return None` arms, which the table-wins test checks); `lifted_static_result` feeds `broadcast_shapes` (split out of `broadcast_static_result`, so each argument is analysed once); `legacy_value_operand` takes the argument index and is true for a value operation in a `LiftArrays` position. OFFSET/INDIRECT shapes moved to `static_analysis/reference_result.rs` and `broadcast_shapes` to `static_analysis/broadcast.rs` to make room. `functions/mod.rs` only changes `mod lift;` to `pub(crate) mod lift;`.
- **Tests**: `functions/lift/tests/` (SPEC-0011 Test plan: shapes, padding, 1×1, in place, errors per element, leftmost error, `LiftArrays` alone/next to an array/returned references, `Whole` read once, `WholeByCaller` unevaluated, the unlifted inline path, the size, input, work (`Cells` vs `Prepared`), weight and text bounds on a filled 2^20-row fixture); the table-wins tests in `lifted.rs`; legacy `@`/export and static shapes in test_implicit_intersection_export.rs; the nested lifted EDATE in test_nested_formula_stack.rs.
- **Eval delta** (after merging main with T-1777's probes; 0 regressions and 0 new-coverage failures in any suite, `make eval-baseline` run with `--timeout 240` because a loaded box times slow unrelated cases out at 60 s). Every suite, the non-baselined ones included (gen-m5..gen-m9, probes-numeric), was compared cell by cell against main, failing values too: 0 new failures and one changed failing value, probes/scalar-lift-open-a W7 `INDEX(YEARFRAC({45000;45031},B1:B60,NA()),1)`, `#N/IMPL!` → `#VALUE!` (Excel `#N/A`: YEARFRAC checks basis first, T-1800). Fixed:
  - probes: 83 cells (known failures 3159 → 3076): the 56 cells of date-argument-modes and scalar-lift-shapes, -b, -c, and 27 of T-1777's scalar-lift-open-a (20), -b (4) and -d (3). probes/fin-argument-modes H17 (ACCRINTM over an array basis) is no longer fixed: it passed only through YEARFRAC's lift.
  - gen-m9: 2 cells (4df1b22d3ed63f58 A24, b58b6d7cd39d195f A9).
  - corpus-bench: case 45738 (26 cells, its `F` line removed: 21 → 20).
- **Bench** (`crates/rustsheet/benches/lift.rs`, release, one case per process, parent origin/main a395c79b vs branch, interleaved ABBA, pinned to one CPU, 6 rounds; COUNTIFS re-run alone for 10 more rounds on a quieter box; Δ is the mean paired per-round change in min with its 95% CI):

  | Case | parent min / median / spread | branch min / median / spread | Δ min (95% CI) |
  |---|---|---|---|
  | `=SUM(COUNTIFS(A:A,A1:A100000))` (6 rounds) | 983 / 1222 / 323 ms | 1023 / 1245 / 307 ms | +4.9% ± 9.1% |
  | same, 10 rounds alone | 1022 / 1221 / 404 ms | 1110 / 1272 / 369 ms | +5.2% ± 3.6% |
  | `=SUM(SUMIFS(B1:B100000,A1:A100000,SEQUENCE(10,1,0)))` | 964 / 1079 / 231 ms | 988 / 1123 / 405 ms | +2.2% ± 2.7% |
  | `=SUM(SWITCH(A1:A1000000,0,1,1,2,3))` | 560 / 643 / 204 ms | 569 / 630 / 126 ms | +0.1% ± 3.6% |
  | `=ROWS(SWITCH(A1:A1000000,0,"a","b"))` | 666 / 776 / 212 ms | 671 / 763 / 178 ms | +1.5% ± 4.6% |

  The COUNTIFS wall time sits at the 5% budget, but its instruction count does not move: callgrind over the whole bench process gives 85,715,348,539 instructions on the parent and 85,720,453,094 on the branch (+0.006%). The SWITCH cases, where the driver dominates, are at parity. So the COUNTIFS gap is code layout or allocator state, not added work; T-1804 (P3) re-measures it with perf counters on a quiet machine.
- **Not done here / deviations from the spec**: `lift_values` keeps its `Vec<CalcResult>` signature (its callers' one-cell ranges lift as grids, their T-1784 contract); the `lift_kinds` testing hook is left to T-1776, its only user; `cast_text_ref` is `cast_string_ref`. SPEC-0011 is updated for `LiftOptions::work`, `REUSE_EQUAL_RUNS`, `Walk { range, walks }`, the direct callers' costs and the reference-form read.

