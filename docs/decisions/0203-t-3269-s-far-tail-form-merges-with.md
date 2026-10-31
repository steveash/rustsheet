# 0203 — T-3269's far-tail form merges with four named cells moved (owner ruling)

- **Status:** accepted (2026-10-31, T-3269; owner ruling (b) "Ship, record ruling", session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a). Under decision 0111 (an owner ruling may land a change that
  regresses named cells), for T-3269's PR only.
- **Context:**
  - Past x = 30 at shapes below 20, Excel's Q(a, x) (POISSON's cdf, CHISQ.DIST.RT, CHIDIST, CHISQ.TEST) is Stirling's
    ratio prefactor sqrt(a / 2 pi) exp(a ln(x/a) + (a - x) - correction(a)) times the continued fraction, 10 to 400 ulp
    from the truth (docs/excel-semantics/statistics.md "Excel's far upper tail (T-3269)"). T-3269 reproduces it
    (`incomplete_gamma/excel_tail.rs`) for 30 < x <= 600, with the exact Stirling correction at each multiple of 1/2 and
    T-3285's double-double fraction; the switch at x = 30 is a measured cut, `FITTED(T-3269, 145 cells, x 26..40)`.
  - Decision 0188 R4: hold-out (30% by (a, x) hash, 502 literal cells) bit-exact test 52 of 157 against 22 for the
    correctly rounded Q (train 79 of 345 against 50); fresh log-uniform sweep after the freeze (probes-numeric/
    t3269-fresh-sweep) 20 of 88 bit-exact in the far range (old kernel 1, correctly rounded 2). Every value is within
    1e-9 of the math (Gate A envelope).
  - The form misses at isolated points: shape 6 at x = 38, 42, 229.64986 and 234.2428575 (probes-numeric/
    t3269-shape6-misses) and x / a = 10 at some shapes. Decision 0056's two attempts are spent without explaining them.
- **Decision:**
  - T-3269's far-tail form merges although it moves these four cells farther from Excel (ulps of Excel's value, main
    against the PR head; see Consequences for the measured run):
    1. probes-numeric/t2561-chisq-dist-rt-past-edge R!A73 `CHISQ.DIST.RT(459.29972,12)`: 15 to 129 ulp, newly visibly
       off;
    2. probes-numeric/t2561-chisq-dist-rt-past-edge R!A75 `CHISQ.DIST.RT(468.485715,12)`: 13 to 189 ulp, newly visibly
       off;
    3. gen-m9/5919660b7bb4d195 Calc!A80 `POISSON(9,100,..)`: 44 to 72 ulp, already visibly off (gated);
    4. gen-m9/43c6227e0ff368c7 Calc!A108 `POISSON(5,Data!A8,(-1))`, Data!A8 = 42: 5 to 25 ulp, already visibly off
       (gated).
  - The shape-6 misses and x / a = 10 go to follow-up T-3469 (`campaign: visible`), which owns these four cells.
- **Consequences:**
  - The two gated cells stay F lines of eval/baseline/gen-m9.txt (they failed before); the two ungated cells are
    probes-numeric, which has no baseline, and count as Phase 1b evidence (decision 0188 R1(c)).
  - Excel's value stays the expected one for all four; T-3469 looks for Excel's exponent at shape 6 and at x / a = 10.
