# 0197 — Allow seven CUMIPMT regressions from building the interest as PMT times the periods less CUMPRINC (T-3118)

- **Status:** accepted (2026-10-31, T-3118): the T-3118 PR's review panel (security, architecture, quality,
  Excel fidelity) approved this record. The orchestrating session relayed the repository owner's approval of
  regressing some cells when the overall result is a big win (2026-10-31, session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd); the run that applied this record did not see that
  approval itself. The change is a net win of 46 cells. Proposed 2026-10-30 as 0170; renumbered to 0190, then to 0197 (main took 0190) on merging
  main, where 0170 was taken.
- **Context:**
  - T-3067 run 4 (probes/fin-cumipmt-tiny-rate-structure, excel-graph) measured that Excel's
    CUMIPMT is `PMT(r,n,pv,0,t)·(e-s+1) - CUMPRINC(r,n,pv,s,e,t)` exactly in doubles, and a type 1
    range from period 1 `PMT·(e-1) - CUMPRINC(..,2,e,1)`. T-3014 has since given the engine
    Excel's PMT bits.
  - T-3118 builds the interest that way (`Periods::interest` in functions/financial/cumulative.rs)
    and drops the per-period interest forms (type 0's balance form and payments less principal,
    type 1's IPMT sum and tiny-rate form). On probes, gen-m9 and ironcalc-calc it fixes 53 known
    failures (probes 39, gen-m9 7, ironcalc-calc 7) and takes CUMIPMT's visibly-off cells from
    102 to 67. It changes no CUMPRINC cell.
  - It moves 7 cells off the bit: gen-m9 6e18550e9fc9c1b9 A329
    (`STDEVA(..,CUMIPMT(1E-10,77,380096,1,18,1))`), ironcalc-calc `CUMPRINC_CUMIPMT.xlsx` G3 and
    probes/fin-cumipmt-overflow-and-forms A8 (`CUMIPMT(0.1,10,345,2,2,1)`, 1 ulp), A12
    (`CUMIPMT(0.01,100,100000,100,100,0)`), A14 (`CUMIPMT(0.0083,36,8000,3,4,0)`),
    fin-cumipmt-gate-edges A1 (`CUMIPMT(6E-5,60,100000,3,10,0)`) and fin-cumipmt-tiny-rate-structure
    F2 (`CUMIPMT(2E-6,100,100000,4,4,1)`). Five of them are visibly off.
  - Each one is our CUMPRINC's last bits, not the rule: with our PMT, Excel's value is
    `PMT·m - x` for an x one or two ulps from our CUMPRINC (F2's x is Excel's own probed
    `CUMPRINC(2E-6,100,100000,4,4,1)` -999.9050029410097, tiny-rate-structure G2, where ours is
    ...096). The old forms were exact there by fit, not by Excel's construction.
  - No structural gate keeps all seven: keeping type 0's balance form inside its growth band, or
    type 1's IPMT sum at ordinary rates, gives up 7 to 25 of the fixes and still regresses 2 to 5
    (G3 at 0.1 regresses where G17 at 0.8, the same shape, is fixed). Deriving CUMPRINC from the
    old interest forms regresses 10 to 42 CUMPRINC cells.
  - Decision 0083's standing allowance does not cover the five visibly-off cells.
- **Decision:** run `rustsheet-eval update-baseline --suite probes --suite gen-m9 --suite
  ironcalc-calc --allow-regressions` once in the T-3118 PR for exactly these 7 cells. T-3413
  owns them (CUMPRINC's bits on those ranges).
- **Consequences:** the three baselines gain exactly these 7 lines and lose the 53 fixed ones.
  Reviewers confirm the baseline diff adds no other line. Every later CUMPRINC fix on a short
  range moves the matching CUMIPMT cell with it.
