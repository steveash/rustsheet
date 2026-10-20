---
id: T-2096
title: "T.INV.2T and TINV accept p in (1,2) and return the negative quantile"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
T.INV.2T and TINV return a number for a probability strictly between 1 and 2, as Excel does:
`T.INV.2T(p,df)` = `T.INV(1 - p/2, df)`, a negative quantile. Today the engine gives `#NUM!`.

## Context
Gap triage 2026-10-20. Rule: docs/excel-semantics/statistics.md, "Student t sweep", bullet "T.INV.2T
does not reject p in (1,2)": `T.INV.2T(1.5,10)` = `T.INV(0.25,10)` = -0.6998120613124317 and
`T.INV.2T(1.9,5)` = `T.INV(0.05,5)` = -2.0150483733330233, equal to the last digit. `T.INV.2T(1,5)` is 0,
while p <= 0 and p >= 2 stay `#NUM!`. This is a domain fix carved out of T-1542, which lists A121-A128
in its context. The last digits stay with T-1542 and T-0967.
Failing cells (excel-graph; probes-numeric is not gated, so read them with
`make eval-show CASE=probes-numeric/<case>.case.json`). Each is Excel a negative number and ours `#NUM!`:
- m9-t-edges Calc!A1 `T.INV.2T(1.0001,5)`, A2 `(1.5,5)`, A3 `(1.999,5)`, A9 `(1.5,1)` = -1,
  A10 `(1.9,1)`, A43 `(1.5,10)`, A45 `(1.9,5)`.
- m9-t-inv-2t Calc!A134 `T.INV.2T(1.5,5)`.
- t1048-tinv Calc!A11 `TINV(1.1,3)` and B11 `T.INV.2T(1.1,3)`.
- t1531-t-tiny-p Calc!A121-A128 (p = 1.9999999999999998, 1.99999999999, 1.9999999999 and 1.5;
  df 1 and 99999).
Code: `t_inv_of` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs. With `two_tailed`, the
`p_out` test rejects p > 1. Accept 1 < p < 2 and return `-t_upper_quantile(&dist, df, (2.0 - p) / 2.0)`,
the negated magnitude for 2 - p. Use `(2.0 - p) / 2.0`, not `1.0 - p / 2.0` fed to the lower tail, so
p near 2 reaches the tiny-p tail path. 2 - p is exact for p in [1, 2] (Sterbenz). Keep p >= 2 and
p <= 0 `#NUM!`, and keep p = 1 giving 0.

## Acceptance criteria
- [ ] None of the 18 cells above is a TypeMismatch in `make eval`. Each is a number, and a unit test
      shows it is bit for bit `-1 ×` the engine's own `T.INV.2T(2 - p, df)`. Cells whose mirror is
      already bit-exact pass. List the rest with their verdict in the PR (NumericUlp/Close/Mismatch
      residue belongs to T-1542 and T-0967).
- [ ] `T.INV.2T(2,5)`, `T.INV.2T(2.5,5)` and `T.INV.2T(0,5)` stay `#NUM!` and `T.INV.2T(1,5)` stays 0
      (unit test). No gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] statistics.md's "Ours" note (T.INV.2T with p in (1,2) is `#NUM!` for us) is updated to say it is
      implemented (T-2096). The code comment cites m9-t-edges A2 and A43.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The quantile's last digits (T-0967, T-1542). T.INV with p outside (0,1) stays `#NUM!` as in Excel.
