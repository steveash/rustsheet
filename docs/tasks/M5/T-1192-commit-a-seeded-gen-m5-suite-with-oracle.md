---
id: T-1192
title: "Commit a seeded gen-m5 suite with oracle values and file its gaps"
milestone: M5
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
eval/suites/gen-m5 exists with Excel-for-the-web oracle values, scores in `make eval`, and every engine mismatch in it is a filed task or a baseline entry tied to one.

## Context
T-1082 gave the M5 math functions generator signatures. Its acceptance run
(`rustsheet-gen --milestone M5 --count 10 --seed 1`, then `rustsheet-oracle fill -b excel-graph
--suite gen-m5`) was accepted by Excel for the web (10 sidecars, 0 failures) but not committed
(T-1082 out of scope). The engine passed 350 of 360 scored cells. The mismatches it showed (from
that run, re-check them): `COS(1E+15)` is `#NUM!` in Excel (the engine computes it); `ACOTH(FACT(14))`
differs in the 5th significant digit; `COTH(0.5)` and `POWER(1E+15,-2.5)` (approximately) differ by ulps;
`DECIMAL` of a number it cannot read gave `#NUM!` in Excel vs `#VALUE!`; `PERCENTOF` is `#NAME?` on
the web (decision 0039: a web golden). Regenerate with the same seed (output is deterministic
unless the catalog changed; if T-1087/T-1088 landed first, the sample also covers text, logical
and information functions, which is fine).

## Acceptance criteria
- [x] eval/suites/gen-m5 committed (suite.toml non-gating, cases + oracle sidecars), `make eval-baseline` run.
- [x] Each distinct mismatch family has a task (via the gap-triager or `make new-task`) or an existing one is cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the mismatches.

## Outcome
Committed eval/suites/gen-m5 (10 cases, seed 1, excel-graph sidecars, non-gating so no baseline file
is written). Scores 350/360 cells; one case times out. Mismatch families: COS(1E+15) `#NUM!` → T-1196;
ACOS(1E+15) NaN leaking through SERIESSUM → T-1199;
MULTINOMIAL(1E+15) hang (the timeout) → T-1197; DECIMAL error kind → T-1198; ACOTH large-x precision loss, COTH/POWER ulp
differences → existing T-0917 (and T-0905 for evaluation order); PERCENTOF `#NAME?` on the web is a
web golden (decision 0039), no task.
