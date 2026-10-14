---
id: T-1518
title: "T.INV(0.9,5E10) hangs the evaluator (nightly fuzz timeout): reject Student-t df at or above Excel's upper limit"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
No Student-t function hangs on a huge `deg_freedom`: `=T.INV(0.9,5E10)` returns `#NUM!` at
once, as Excel does, and the nightly fuzz reproducer is a unit test (decision 0035).

## Context
Nightly fuzz run 38508146682 (2026-10-14, main cbf9c5c8) failed on `formula_eval` with a
libFuzzer **timeout** (artifact `fuzz-crash-formula_eval-38508146682`, file
`timeout-7612e496cb6ff434a0edb0ed172d18fb8ecba7c6`). The input bytes, escaped:
`[84, 46, 73, 78, 86, 40, 48, 46, 57, 44, 53, 69, 49, 48, 41]` = `T.INV(0.9,5E10)`.
Reproduce: `cd fuzz && cargo fuzz run --target "$(rustc -vV | sed -n 's/^host: //p')" formula_eval <file>`
with a file holding those bytes.

Cause (read from code, not yet timed): `fn_t_inv` in `functions/statistical/t_dist.rs` checks
only `df < 1` and hands any larger df to `statrs::StudentsT::inverse_cdf`, whose search does not
finish in reasonable time for a df that large. The other family members (`T.INV.2T`,
`T.DIST(...,TRUE)`, `T.DIST.RT`, `T.DIST.2T`, `T.TEST`, `CONFIDENCE.T`) may share the problem.

Excel evidence (docs/excel-semantics/statistics.md, "Student t sweep", T-1043): df = 1E10 and
9999999999 work everywhere; df >= 1.5E10 is `#NUM!` for `T.DIST(...,TRUE)`, `T.DIST.RT`,
`T.DIST.2T`, `T.INV` and `T.INV.2T` while the pdf `T.DIST(x,df,FALSE)` stays finite. The exact cut
between 1E10 and 1.5E10 is not probed: use the `excel-probe` skill to probe it (e.g. 1.0000001E10,
1.2E10, 1.4999E10) or, if the oracle is unavailable, reject df > 1E10 and leave the in-between
cells as a pending probe. If df = 1E10 itself is still too slow, that is a directive-5 limit and
needs a `DEVIATION` entry (decision 0014).

The steward's `make eval` on 2026-10-14 (main 6ab245f0) also lists four `probes-numeric` cases
that did not run ("Timeout: exceeded 60s"): `m9-t-edges` and `m9-t-nonint-df` (both hold
`T.DIST`/`T.DIST.RT`/`T.DIST.2T` at df 1E10, 1.5E10, 2E10 and 1E11), `t1048-tinv` and
`m9-legacy-finv`. Find which cells of each hang; the last two may hang on something other than a
huge df (FINV is the F family) — if so, file a separate P0 task for that cause rather than
widening this one.

## Acceptance criteria
- [ ] `=T.INV(0.9,5E10)` is `#NUM!`; a unit test holds the fuzz input (decision 0035).
- [ ] Every Student-t function listed in Context returns within a bounded time for df up to
      1E10 and is `#NUM!` above Excel's cut (the pdf stays finite, per the evidence); a test
      covers each.
- [ ] `m9-t-edges`, `m9-t-nonint-df` and `t1048-tinv` run to completion in `make eval` (they are
      no longer under "Cases that did not run"); `m9-legacy-finv` too, or it has its own task.
- [ ] The cut is oracle-backed or left as a pending probe, and statistics.md says which.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Student-t accuracy (T-0967, T-0969).
