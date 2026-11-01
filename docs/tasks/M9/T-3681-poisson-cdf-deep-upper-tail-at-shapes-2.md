---
id: T-3681
title: "POISSON cdf deep upper tail at shapes 2 to 11: the 13 gated gen-m9 cells T-3269 left 1 to 4 ulp off Excel (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3469]
defer: phase-1b
---

## Goal
The 13 gated gen-m9 POISSON / POISSON.DIST cumulative cells below, which T-3269's far-tail form moved within 15
digits but not to Excel's bits, match Excel to the bit, by a rule for Excel's Q(k + 1, mean) in the deep upper tail
(mean well above k), or the Outcome lists the candidates tried with their bit-exact and within-4-ulp counts on a
hold-out split (decision 0056 time box). The cells stay named here until then.

## Context
- Gap triage 2026-11-01g (`make eval` on origin/main c0e330c2f, excel-graph). `scripts/exit_owners.py`'s `owned` rule,
  run over every failing cell (not only the visibly-off ones) against the open task files of origin/main and of every
  `origin/agent/*` task and triage branch, found exactly these POISSON cells with no open owner. T-3269 (done) named
  most of them as "visibly fixed" or "closer" in its eval diff, and T-2055 (done) before it. No open task names them:
  T-3469 owns the nine cells decision 0203 names as moved farther (shape 6 at x = 38, 42, 229.64986, 234.2428575 and
  x / a = 10), T-3461 the half-integer shapes, T-3640 / T-3639 (open PR) shapes above 11 and x past 600, T-3286 the
  mode at huge shapes.
- Every cell is within half a unit of Excel's 15th significant digit, so this is Phase 1b (`defer: phase-1b`). Remove
  the key only when the steward records the Phase 1 exit, or when an eval shows one of these cells visibly off (then
  give it a `campaign:` stage and a Visible-parity section).
- Excel is not correctly rounded here, so this is Excel's algorithm, not a more accurate kernel. Excel, ours, the
  50-digit Q(k + 1, mean) (mpmath `gammainc(k+1, mean, inf, regularized=True)`, this triage), then Excel's and our
  offsets from it in ulps of the value, and ours minus Excel in ulps:
  - gen-m9/097af1ca33638670 Calc!A214 `POISSON(1,100,TRUE)`: 3.7572767357810366E-42 / ...347E-42 /
    3.7572767357810443226E-42; -12.1 / -15.1; -3.
  - gen-m9/873f30d2bf76f8db Calc!A124 and gen-m9/c7664e494e418395 Calc!A294 `POISSON(2,100,TRUE)` (A124's k is
    `ERROR.TYPE(#DIV/0!)` = 2): 1.8976107553682424E-40 / ...242E-40 / 1.8976107553682284247E-40; +34.2 / +33.2; -1.
  - gen-m9/9735c5ab72fa5ecf Calc!A48 `POISSON.DIST(Data!$D8,100,Data!E3)`, D8 = 3.14159 (k = 3):
    6.389887702238139E-39 / ...136E-39 / 6.3898877022382161141E-39; -59.4 / -61.4; -2.
  - gen-m9/46ac3ef5e61583ca Calc!A240 and gen-m9/afc00c8b70e78baf Calc!A304 `POISSON(4,100,TRUE)`:
    1.6139305336977313E-37 / ...317E-37 / 1.613930533697730479E-37; +4.1 / +6.1; +2.
  - gen-m9/5389dd4d5b5739d2 Calc!A394 `POISSON.DIST(5,100,TRUE)`: 3.261456366720473E-36 / ...475E-36 /
    3.2614563667204696837E-36; +4.5 / +7.5; +3.
  - gen-m9/201d67f1d59436ed Calc!A173 `POISSON.DIST(5,SUM(Data!C5:C10),..)`, mean 105.14159: 2.4447084590093315E-38 /
    ...332E-38 / 2.444708459009333479E-38; -3.7 / -2.7; +1.
  - gen-m9/5919660b7bb4d195 Calc!A316 and gen-m9/7980ffb6095a340d Calc!A289 `POISSON(4,30,TRUE)`:
    3.6243009520614882E-09 / ...487E-09 / 3.6243009520614880262E-9; +0.5 / -2.5; -3. Here Excel is correctly rounded
    and ours is not: x = 30 is exactly `FAR_TAIL_MIN_X`, which the form excludes, so T-3285's accurate kernel answers.
  - gen-m9/2138b909d8f810ea Calc!A62 `POISSON(8,(Data!C12/Data!E2),0.5)`, mean 7/0.2 = 35: 4.519316457456951E-08 /
    ...949E-08 / 4.5193164574569458296E-8; +8.2 / +5.2; -3.
  - gen-m9/2138b909d8f810ea Calc!A214 `POISSON(3,Data!A$1,Data!$C3)`, mean 42: 7.631340901396825E-15 / ...831E-15 /
    7.6313409013968419314E-15; -10.9 / -6.9; +4.
  - gen-m9/3c06f9298f65a3ba Calc!A91 `CUMIPMT((Data!B12/POISSON.DIST(10,42,1)),197,1,1.5,13,0)`, Data!B12 = "1":
    -3409081450.3987784 vs ours -3409081450.398781 (5 ulp). Suspected: the inner `POISSON.DIST(10,42,TRUE)` (shape 11,
    x = 42); confirm with an oracle cell for the inner call before fitting it, and hand the cell back to the
    financial lane's owner if CUMIPMT alone moves it.
- Code: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/excel_tail.rs (T-3269's Stirling ratio
  prefactor times T-3285's double-double fraction, shapes 3/2 to `FAR_TAIL_MAX_SHAPE` = 11 and 30 < x <= 600;
  `poisson_upper`), the accurate kernel below the cut. All cells but the two `POISSON(4,30,TRUE)` are on the form.
- Semantics: docs/excel-semantics/statistics.md, "Excel's far upper tail (T-3269)": Excel's deep upper tail is up to
  60 ulp from the math with no shared pattern; the cells above extend that entry (record the outcome there).
- Evidence already cached: T-3269's hold-out set (1058 literal cells, 1 < a < 20, x >= a + 1,
  docs/research/t3269/holdout-split.tsv and the mpmath values), probes-numeric/t3269-gamma-tail-switch (145 cells,
  x 26 to 40 at shapes 2 to 4, where the switch at 30 was fitted), t3269-shape6-misses (77), t3269-fresh-sweep (240),
  t3284-gamma-deep-upper-tail. These are training data now: the R4.2 sweep must be a new one, oracle-filled after the
  model is frozen, inside this task (as T-3311 did).
- Depends on T-3469: it changes the same form at shape 6 and at x / a = 10; fitting both at once would score each
  against a moving kernel. Its outcome may already move some of these cells; re-run the list first.

## Acceptance criteria
- [ ] Each of the 13 cells below is bit-exact, or the Outcome lists it with Excel, new and 50-digit values; the
      bit-exact count is stated against main's 0 of 13.
- [ ] Decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome:
      - hold-out counts: train and test side by side (bit-exact / within 4 ulp / within 15 digits) on a split fixed
        before scoring (T-3269's `holdout-split.tsv` or a new deterministic ~30% split by hash of (a, x)), the chosen
        candidate against T-3269's form, the correctly rounded value and main;
      - a fresh log-uniform sweep over integer shapes 2 to 11 and x from 30 to 600 (the two `POISSON(4,30,TRUE)` cells
        sit on the cut: include x = 30 and its neighbouring doubles), oracle-filled after the model is frozen, commit
        and fill times recorded, with its counts;
      - the Gate A envelope result on that sweep, every point outside it listed;
      - every fitted constant, cut or range tagged `FITTED(T-3681, n_cells, range)` and applied only inside its range,
        with accurate math outside it; quality/fitted.json updated through `scripts/check_fitted.py --update`;
      - no step fitted to differences below the reproducibility floor (4 ulp web against desktop,
        docs/research/oracle-calibration.md): with these cells 1 to 4 ulp off, say for each whether its miss is above
        the floor before fitting it;
      - no keep-matched branch (no special case that only preserves cells already matching).
- [ ] No gated cell regresses: the cells T-3269 made bit-exact (its Outcome lists them) and every CHISQ.DIST.RT,
      CHIDIST and GAMMA.DIST cell sharing the kernel stay as they are; no known-failure line is added to any
      eval/baseline file, and the lines that disappear from eval/baseline/gen-m9.txt are recorded with
      `make eval-baseline`.
- [ ] docs/excel-semantics/statistics.md "Excel's far upper tail (T-3269)" records the rule found or the rules ruled
      out, citing these cells.
- [ ] `make check` green; reviewers approve.

Cell list for the owner check:
- gen-m9/097af1ca33638670.case.json (1): Calc!A214
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A124
- gen-m9/c7664e494e418395.case.json (1): Calc!A294
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A48
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A240
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A304
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A394
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A173
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A316
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A289
- gen-m9/2138b909d8f810ea.case.json (2): Calc!A62, A214
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A91

## Out of scope
- T-3469's nine cells (shape 6 at isolated x, x / a = 10), T-3461 (half-integer shapes), T-3640 (shapes above 11, x
  past 600), T-3286 (near the mode at huge shapes).
- The POISSON pmf (T-2055's closed forms).
