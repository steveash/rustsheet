# 0160 — Allow seven near-1E15 CUMPRINC and CUMIPMT regressions from the T-3067 underflowed-balance principal

- **Status:** accepted (2026-10-29, T-3067)
- **Context:**
  - T-3067 run 5 (probes/fin-cumprinc-underflow-ends, excel-graph 2026-10-29) found that Excel's
    type 0 CUMPRINC over the last m periods does not depend on nper once `v^nper` underflows: nper
    1E6, 1E10, 1E12 and 1E15 give the same doubles at rates 0.001, 0.01 and 0.1. The per-period sum
    the kernel used there is 1 to 3 ulps off Excel on most of those principals, and CUMIPMT, which is
    the payments less that principal, turns each ulp into a visibly-off interest
    (fin-cumipmt-overflow-and-forms A9, A11; fin-remaining-later-argument-ranges K3).
  - The balance drop `-(pv·(v^(nper-end)·(1-v^m)))` on Excel's EXP, Kahan's expm1 and `growth_log`,
    taken only where `1 - v^nper` is 1 and the leading exponent is below `GEOMETRIC_MIN_LEAD`, fixes
    22 known failures in the probes suite (fin-cumipmt-overflow-and-forms A9, A11; fin-cumipmt-period-bound
    A19; fin-remaining-later-argument-ranges K3; fin-cumprinc-near-1e15 A1, A5, A6, A8, A11, B2, B3,
    B5-B8, C3, C12, E1, E5, E6, E8, E11) and changes no cell in any other suite. Visibly-off in-scope
    cells go from 2,833 to 2,822 (the new case 8 to 4, the rest 2,825 to 2,818).
  - It moves 7 cells of fin-cumprinc-near-1e15 off the bit: the principals A7 (+1 ulp), B1, B4, B9,
    C10 (-1 ulp each) and C6 (+1 ulp), none visibly, and E7 `CUMIPMT(0.01,1E15,100000,1E15-6,1E15,0)`,
    Excel -271.8054707135361, now -271.80547071353703 (16 ulps, visibly off), which is A7's one ulp
    amplified by the payments less the principal.
  - Decision 0083's standing allowance does not cover E7 (more than 4 ulps and visibly off).
- **Options considered:**
  - *Keep the per-period sum.* It leaves A9, A11, K3 and 4 more cells visibly off.
  - *Keep looking for the exact form.* Three T-3067 runs fitted more than 70 forms to these
    principals (financial.md "T-3067 notes"); none is exact on all, and the errors are not one
    biased L (the implied L of m = 2 and m = 8 at 0.01 do not overlap). Decision 0056 time-boxes the fit.
- **Decision:** run `rustsheet-eval update-baseline --suite probes --allow-regressions` once in the
  T-3067 run 5 PR for exactly these 7 cells. The follow-up ulp task T-3141 owns them.
- **Consequences:** `eval/baseline/probes.txt` gains exactly these 7 lines and the new case's failing
  cells, and loses the 22 fixed ones. Reviewers confirm the baseline diff adds no other line.
