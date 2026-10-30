# 0176 — Allow the IRR ulp regression of T-3120's secant in d

- **Status:** accepted (2026-10-30, T-3120)
- **Context:**
  - T-3120 found where Excel's IRR results get their 2^-53 grid in v (financial.md "IRR is a secant
    in d = 1 - v"): the secant runs on d = 1 - v from d0 = guess/(1 + guess), each step anchored on
    the newer iterate, NPV by worksheet NPV's running product, leading zero flows dropped, and a zero
    step stops on a residual relative to the largest flow. In the fit lab it makes the IRR fit set
    262/263 bit-exact (the secant in v: 189), probes/t2565-irr-guess-sweeps 180/181 and every far-root
    cell of probes/irr-fallback-roots.
  - In the engine it fixes 273 known failures (probes 215, ironcalc-calc 32, gen-m9 26), among them
    every visibly-off t2565 cell but P!C157 and irr-fallback-roots P!A25-A28 and A36-A39.
  - It moves one cell off the bit: gen-m9/67a4c03afd63f988 Calc!A395, `IRR(Data!C10:E12,0.5)` over
    the flows {0.5, 0.1, -0.5}: Excel -0.09501243788791103, ours -0.0950124378879108 (16 ulp, within
    the 15 significant digits). Decision 0083's standing allowance covers only regressions within
    4 ulp, so it does not cover this one.
- **Options considered:**
  - *Keep the secant in v.* 273 fewer fixed cells; the t2565 and far-root cells stay visibly off.
  - *Start from d0 = 1 - 1/(1 + guess).* It gets A395 and loses 27 fit-set cells (235/263).
  - *Fit further first.* No named candidate gets A395 and keeps the fit set; T-3205 owns the search.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3120 PR, for
  exactly this one cell. The follow-up T-3205 owns it.
- **Consequences:** the gen-m9 baseline gains exactly the line for 67a4c03afd63f988 Calc!A395 and the
  baselines lose the fixed ones. Reviewers check that it is the only added line. No gate is lowered.
