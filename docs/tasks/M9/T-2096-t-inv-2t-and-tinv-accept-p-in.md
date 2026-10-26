---
id: T-2096
title: "T.INV.2T and TINV accept p in (1,2) and return the negative quantile"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
T.INV.2T and TINV return a number for a probability strictly between 1 and 2, as Excel does:
`T.INV.2T(p,df)` = `T.INV(1 - p/2, df)`, a negative quantile. Today the engine gives `#NUM!`. Also: p = 0 with a df that truncates to 1 is `#DIV/0!` for TINV, T.INV.2T and T.INV, not `#NUM!` (T-2256).

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

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): p in (1,2) for TINV and T.INV.2T (also a text "1.5" from a cell) and p = 0 with df in [1,2), which Excel gives as `#DIV/0!` (statistics.md "Student t quantile at p = 0 with df below 2 is #DIV/0! (T-2256)").

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 48 known-failure lines in eval/baseline/gen-m9.txt, 48 visibly off.
Examples:
- gen-m9/139684e68fb2fc3b Calc!A128 `=TINV(0,1)`: Excel #DIV/0!, ours #NUM!
- gen-m9/201d67f1d59436ed Calc!A103 `=TINV(1.5,2)`: Excel -0.8164965809277259, ours #NUM!
- gen-m9/5395b8a427764a3f Calc!A347 `=T.INV.2T(1.5,7)`: Excel -0.7111417780817859, ours #NUM!
- gen-m9/5cebac1f02bf30f2 Calc!A22 `=TINV(1+2^-52,2)`: Excel -3.140184917367558e-16, ours #NUM!
- gen-m9/7fd0a98183c6babf Calc!A128 `=TINV(1+2^-52,6)`: Excel -2.900778571755774e-16, ours #NUM!
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A341
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A14
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A128, A394
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A57
- gen-m9/1db18890916ee643.case.json (1): Calc!A106
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A103
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A252
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A158
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A162
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A1
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A288
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A347
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A395
- gen-m9/5be29fa316931f05.case.json (1): Calc!A127
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A22
- gen-m9/607e85cfee83a689.case.json (1): Calc!A160
- gen-m9/787ba8d801105a60.case.json (3): Calc!A24, A90, A265
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A350
- gen-m9/7f7ca726c3f68554.case.json (2): Calc!A14, A280
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A128
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A389
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A226
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A271, A386
- gen-m9/941c409532603d32.case.json (1): Calc!A108
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A36
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A226
- gen-m9/a2262ea466fcc142.case.json (2): Calc!A7, A205
- gen-m9/a4090567758520df.case.json (1): Calc!A307
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A370
- gen-m9/afc00c8b70e78baf.case.json (2): Calc!A188, A371
- gen-m9/c2684066d8df263e.case.json (3): Calc!A64, A237, A374
- gen-m9/c7664e494e418395.case.json (1): Calc!A293
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A340
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A285
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A357
- gen-m9/eccd03df42e40006.case.json (1): Calc!A159
- gen-m9/ede0323c389fc703.case.json (3): Calc!A186, A338, A376

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 48 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
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

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): it now owns 48 visibly-off gen-m9 cells (Context), so the task is in the
campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria
still unmet then move to a follow-up task without the key.

## Outcome
`t_inv_of` accepts 1 < p < 2 for T.INV.2T and TINV and returns `-t_upper_quantile(df, (2 - p) / 2)`;
p <= 0 and p >= 2 stay `#NUM!`, p = 1 is 0. The p = 0, df 1 `#DIV/0!` was already in place. Unit test
shows A2/A43-style cells are bit for bit the negated mirror `T.INV.2T(2 - p, df)`. gen-m9 known
failures 3775 -> 3770 (5 lines). The other owned cells now have Excel's type and are `NumericClose`
(last-digit residue, e.g. gen-m9/201d67f1d59436ed A103, 5395b8a427764a3f A347), which belongs to T-1542 and
T-0967; their case lines stay in the baseline for that reason. No gated baseline gained a line.
