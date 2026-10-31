---
id: T-2839
title: "T.INV.2T and TINV just above p = 1 have an absolute error of about 1e-8 (p = 1+2^-52 and 1+1e-15)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
T.INV.2T and TINV for p just above 1 (the negative branch `-t_upper_quantile(df, (2 - p) / 2)` added by T-2096) match
Excel, which resolves the quantile near 0 to full relative precision. Ours has an absolute error of about 1e-8.

## Context
Found by the T-2096 Excel-fidelity review (about 10^8 too large in magnitude, `NumericMismatch`, not last-digit residue):
- gen-m9/5cebac1f02bf30f2 A22 `TINV(1+2^-52,2)`: Excel -3.140184917367558e-16, ours -1.4901161193847656e-8
- gen-m9/7fd0a98183c6babf A128 `TINV(1+2^-52,6)`: Excel -2.900778571755774e-16, ours -2.580956827951785e-8
- gen-m9/eccd03df42e40006 A159 `TINV(1+1e-15,6)`: Excel -1.45e-15, ours -2.58e-8
- probes-numeric m9-t-edges A1 `T.INV.2T(1.0001,5)`: Excel -0.0001317152766640225, ours -0.00013171527638163228
- and the other p = 1+2^-52 cells of the T-2096 list (0620505280d2ea6e A14, 139684e68fb2fc3b A394, 43c6227e0ff368c7 A162,
  46ac3ef5e61583ca A1, 787ba8d801105a60 A24, 7980ffb6095a340d A350, 873f30d2bf76f8db A389, 941c409532603d32 A108,
  9e2225ead8a6a07b A226, a2262ea466fcc142 A205, a4090567758520df A307, c2684066d8df263e A237, c7664e494e418395 A293,
  ede0323c389fc703 A376).
- Gap triage 2026-10-28 (`make eval` on dea28aa5, excel-graph; no open task named it): gen-m9/5389dd4d5b5739d2 Calc!A288
  `=T.INV.2T(Data!E4,Data!C$10)` = `T.INV.2T(1.005,3.14159)` (df truncated to 3): Excel -0.006801817544729202, ours
  -0.006801817544722885 (relative 9E-13, visibly off). A 60-digit mpmath solve of `I_{3/(3+t^2)}(3/2,1/2) = 2 - 1.005`
  gives -0.0068018175447292014, so Excel is correctly rounded and ours loses digits on the same `1 - y` cancellation,
  further from 1 than the cells above (absolute error 6E-15 here, 3E-13 at 1.0001).
Suspected cause (untraced): the quantile at q = 0.5 - tiny fed to `t_upper_quantile`/statrs has an absolute floor of about 1e-8;
the p < 1 branch of T.INV near 0.5 may share it. Also the NumericClose cells that are off at the 15th digit, e.g.
201d67f1d59436ed A103 and c2684066d8df263e A374 `T.INV.2T(1.5,1)` (-1.0000000000000016, Excel -1), belong to T-0967/T-1542.

Gap triage 2026-10-27: the same centre loss on the p <= 1 side (`T.INV.2T(1-1E-10,10)` 1.28E-10 in Excel, ours
3.3E-8) is owned by T-2724, which solves the centre from the complement P(|T| < t) = 1 - p. Build one centre path
and reuse it in whichever of the two lands second (T-2859 owns the general region only). The df = 1 cell above (a4090567758520df A307 `TINV(1+2^-52,1)`)
and the other df = 1 cells above p = 1 (probes-numeric m9-t-edges A9/A10, t1531-t-tiny-p A121-A127) are T-2858's
closed form, not this task's.

Gap triage 2026-10-28 (eval on origin/main abefb50): one more cell of this branch that no task named, at a fractional
df and p further from 1: gen-m9/5389dd4d5b5739d2 Calc!A288 `T.INV.2T(Data!E4,Data!C$10)` = `T.INV.2T(1.005,3.14159)`,
Excel -0.006801817544729202, ours -0.006801817544722885 (9E-13 relative, visibly off; known failure in
eval/baseline/gen-m9.txt). It counts toward the first criterion.
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/941c409532603d32.case.json: Calc!A108
  - gen-m9/139684e68fb2fc3b.case.json: Calc!A394
  - gen-m9/a4090567758520df.case.json: Calc!A307

Note from T-2724 (2026-10-31): the p in (1, 2) branch reaches T-2724's centre kernel through
`t_upper_quantile` at c = 1 - 2q = p - 1, so these cells moved from about 1E-8 absolute to last-digit residue
(`TINV(1+2^-52,2)` -3.14018491736755e-16 against Excel -3.140184917367558e-16; eccd03df42e40006 A159 is now
bit-exact). Re-measure before working this task.

## Acceptance criteria
- [ ] The cells above are within half a unit of the 15th significant digit of Excel's value, or the residue is filed.
- [ ] statistics.md records the near-1 behavior and the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
