---
id: T-1078
title: "Fuzz timeout: chained postfix % on a whole-column range re-lifts 1M cells per operator"
milestone: M1
lane: engine
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A formula that chains unary/postfix operators over a whole-column (or whole-row) range, such as
`=LOOKUP(20,A:A,B:B%%%…%)` with ~95 `%`, evaluates in bounded time (well under the fuzz job's 30 s
per-input timeout) with Excel's value, so the nightly `formula_eval` fuzz target stops timing out on it.

## Context
Found by the first working nightly fuzz run (T-1067 PR #286, closed as a duplicate of #287; Actions run 37589979810, job "fuzz (formula_eval)",
artifact `fuzz-crash-formula_eval-37589979810`, `timeout-140caa1b2f85b508dd452f0b30b4d4dae9aa6e82`).
The reproducer is the 108-byte formula `LOOKUP(20,A:A,B:B` + 95 × `%` + `)`, typed into Sheet1!C3 of
the fuzz model (fuzz/fuzz_targets/formula_eval.rs). libFuzzer reported a timeout (> 30 s).
Locally (ASan fuzz build), `SUM(B:B%%%%%)` and `LOOKUP(20,A:A,B:B%%%%%)` each take about 8.7 s. That is
about 1.7 s per `%`: each postfix operator lifts the whole 1,048,576-row column into a fresh array and
walks it again, so the cost grows with operator count × column height, not with the used area. T-0047
bounded the memory of such lifts, and T-0049 (used-area tracking, todo) is related but does not by itself
stop the per-operator re-lift. Unary minus/plus chains (`-----B:B`) are likely the same path; check them.
Repro: `cd fuzz && cargo fuzz run formula_eval <file with the formula>` (decision 0035).
Excel semantics (cells beyond the used area are blank → 0%) must be kept; see
docs/excel-semantics/ for whole-column lifts.

## Acceptance criteria
- [x] The reproducer (and a unary-minus chain variant) is a unit test that runs in well under a second in
      a debug test build and returns the same value as before the fix.
- [x] Operator chains over whole-column/row ranges cost O(used area), or lift once, not once per operator.
- [x] `make check` green; reviewers approve.

## Out of scope
- General used-area tracking (T-0049).

## Outcome
A unary `-`/`%` chain is now evaluated by `Model::apply_unary_chain` (cast.rs). It does one lift and
one element-wise pass, using the minus parity and the `%` count (the operators commute), with no
allocation and an early stop at 0. `apply_number_unary` now rewrites the lifted array in place instead
of reallocating a million one-element rows per operator. Results are unchanged (the excel-fidelity
reviewer checked the old values on main). The fuzz reproducer takes 0.21 s in a debug test, down from
4 s in release before. In the ASan fuzz build it went from a > 30 s timeout to 3.6 s. `model.rs` shrank
by 9 lines. Eval delta: none (0 fixed, 0 regressions). Binary-operator chains over whole columns
(`B:B*1*1…`) still lift once per operator; T-0049/T-0405 cover bounding those walks by the used area.
