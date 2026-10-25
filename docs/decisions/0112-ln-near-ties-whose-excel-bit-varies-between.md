# 0112 — LN near-ties whose Excel bit varies between fills stay named known failures

- **Status:** accepted (2026-10-25, T-2144). It changes no scoring, baseline, ratchet or CI check.
- **Context:**
  - probes/ln-fyl2x-kernel (tag M9; LN is catalog M5) has 27 known failures. Each is one ulp at an FYL2X near-tie, where the
    exact product `RN64(ln 2) * log2 x` lies about half a 64-bit ulp from a 53-bit midpoint, so the 64-bit rounding
    decides the last bit (numbers.md "LN is FLDLN2 then FYL2X").
  - T-2143 calculated one 380-point workbook three times within 35 s (probes-numeric/t2143-ln-near-ties-fill1..3). The
    fills disagree at 23 points, always between two adjacent doubles. So Excel's bit there is not a function of x.
  - T-2144 fitted guard-bit FYL2X models: a k-bit internal product, truncated or rounded, before RN64 and RN53, for k =
    66..72 (docs/research/t1941/ln_guard_bits.py). The best, k = 67 rounded, matches 354 of the 357 points where all
    fills agree, and keeps all 4,000 sampled fixture points. The shipped model matches 345 of the 357. No candidate
    matches all 357, so task T-2144's criterion for changing `excel_ln` is not met. The 30 fixture near-ties come from
    one research run, so they cannot arbitrate either.
  - Binding measures: decision 0085 item 3 (a cell within 4 ulp of Excel, owned by a blocked or settled fit, does not
    block an exit); decision 0042 (an accepted-pair rule needs evidence per cell); prime directive 2.
- **Options considered:**
  - *(a) Accept either neighbour at these cells* (a scoring rule like decision 0042's pair). It needs harness support
    for "Excel's bit varies here", per cell, and the evidence for that exists only at the 23 T-2143 points, not at
    the probe's 27. Not chosen.
  - *(b) Ship the k = 67 model.* It still misses 3 stable points and has no mechanism behind it (unlike the shipped
    model, which is the documented x87 sequence). Excel's bit at these points also varies by server. Rejected.
  - *(c) Keep the 27 cells as named known failures of probes/ln-fyl2x-kernel.* Chosen.
- **Decision:**
  1. The shipped model stays (`excel_ln`, `RN53(RN64(RN64(ln 2) * log2 x))`). The 27 cells stay F lines of
     eval/baseline/probes.txt. Their owner is T-2144, which is done.
  2. Every such cell is one ulp from Excel's cached value, so under decision 0085 item 3 none blocks a milestone exit.
     The same holds for any later LN cell (or LOG10, GAMMALN, GAMMA cell through `excel_ln`) that is one ulp off at an
     FYL2X near-tie, that is, with its exact product within 4e-4 ulp53 of a 53-bit midpoint. Such a cell may be named
     under this decision in the PR that finds it.
  3. A cell further than one ulp off, or one away from a near-tie, is an ordinary failure.
- **Consequences:**
  - No engine change. A later fit can reopen this only with a model that matches every fill-stable point. That would
    need more fills or Excel desktop evidence (M14, decision 0039).
  - The probe's tag (M9) differs from LN's catalog milestone (M5). Retagging is out of scope here. Under item 2 the
    cells do not block either row.
