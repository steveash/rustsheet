---
id: T-2143
title: "Oracle sweep for the LN FYL2X near-ties: refill the 30 points three times and probe 300 new near-ties"
milestone: M5
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
T-2144 gets enough excel-graph evidence to tell whether Excel's last bit of LN at an FYL2X near-tie is a fixed function
of x, which can be fitted, or changes from one calculation to the next.

## Context
- numbers.md "LN is FLDLN2 then FYL2X": the engine's `excel_ln` (crates/rustsheet-engine/src/x87_log10.rs) gives
  Excel's bits on 2,887,521 of the 2,887,551 T-1941 cells. The 30 misses are one ulp each, at points where the 64-bit
  product `RN64(RN64(ln 2) * log2 x)` is close to a 53-bit rounding midpoint. probes/ln-fyl2x-kernel (2026-10-20) gave
  the model's value at 3 of the 30 (2.426003200000124, 9.612345678912595, 1.2345670000006508e100), where the
  2026-10-19 research run gave the other neighbour. Its 27 other near-ties are known failures in
  eval/baseline/probes.txt (Calc!B10, B107, B113, ...).
- Nobody knows yet whether repeated fills of the same x agree, or how close to the midpoint a point must be before
  Excel can round it either way.

## Acceptance criteria
- [ ] A new probes-numeric case, `eval/suites/probes-numeric/cases/t2143-ln-near-ties.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, `=LN(A<row>)`), holds three sets of x. Its sidecar is filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill):
      - the 30 near-ties of probes/ln-fyl2x-kernel;
      - 300 new x whose model product lies within 2^-8 of a 53-bit midpoint, at several distances from it (pick them
        with the engine's own `excel_ln` or a short script, not by hand);
      - 50 x far from a midpoint, as controls.
- [ ] The same workbook is filled at least three times on different days, or in separate sessions. Each fill is
      kept as its own case file, or as a column of the same x. The case description records the fill dates. No
      pending cells.
- [ ] numbers.md "LN is FLDLN2 then FYL2X" records three results: how many near-ties agree across every fill,
      the distance from the midpoint below which fills disagree (if any), and whether the controls always match the
      model.
- [ ] T-2144 links the case.
- [ ] Fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `excel_ln` or the probes baseline (T-2144).
- LOG10, LOG and GAMMALN (they share FYL2X, but their near-ties are T-2144's follow-ups).
