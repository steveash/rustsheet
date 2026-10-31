# 0203 — T-3269's far-tail form merges with four named cells moved (owner ruling)

- **Status:** proposed (2026-10-31, T-3269). Owner ruling (b) "Ship, record ruling" (session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a) covers the four cells under Decision; the re-run against
  current main found four more ungated cells newly visibly off, and the baseline ratchet one gated regression against
  T-3285's head (Consequences), which the ruling did not name, so the record stays proposed and the PR may not merge
  until the owner rules on them. Under decision 0111 (an owner ruling may
  land a change that regresses named cells), for T-3269's PR only.
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
  - Re-run (main ea7470e4a, with T-3285's head merged into the branch, against the PR head; full runs on the same suite
    root, per cell). Gated: fixed 8, closer 18, farther 6, lost 1, visibly fixed 13, newly visibly off 0. Of these,
    T-3285 (decision 0202) accounts for the lost `CHIDIST(42,7)` and four farther `CHIDIST` cells; T-3269's own farther
    cells are cells 3 and 4 above, measured 44 to 72 and 5 to 25 as ruled.
  - Ungated, cases on main: cells 1 and 2 above, measured 15 to 129 and 13 to 189 as ruled, and **four more newly
    visibly off that the ruling did not name**, all at x / a = 10 exactly (probes-numeric/t3284-gamma-deep-upper-tail):
    Calc!A165 `POISSON(10,110,TRUE)` and its twin A166 `CHISQ.DIST.RT(220,22)`, 9 to 90 ulp; Calc!A183
    `POISSON(13,140,TRUE)` and its twin A184 `CHISQ.DIST.RT(280,28)`, 5 to 79 ulp. They were not in the earlier
    comparison: the main report it used (1248554a) predates T-3289's sweep. A246 `CHISQ.DIST.RT(56,7)` (2 to 3 ulp) is
    T-3285's, decision 0202.
  - **A gated regression against T-3285's head that the ruling did not name:** gen-m9/caa1eed48b1883fe Calc!A59
    `CHIDIST(Data!A4,9)` with Data!A4 = 100, Q(4.5, 50). Excel is 1.5735176303753944E-17, T-3285's correctly rounded Q
    has it bit for bit (T-3285 removed its baseline line), and the far-tail form gives 1.5735176303753947E-17, 1 ulp off,
    as main's old kernel did. It appears only since ruling (a) removed T-3285's 3a cut (decision 0202); against main the
    cell is unchanged, so the main-vs-HEAD counts above do not show it. `update-baseline` refuses it without
    `--allow-regressions`, which this ruling does not cover.
  - The two gated cells stay F lines of eval/baseline/gen-m9.txt (they failed before); the ungated cells are
    probes-numeric, which has no baseline, and count as Phase 1b evidence (decision 0188 R1(c)).
  - Excel's value stays the expected one; T-3469 owns the shape-6 and x / a = 10 misses.
