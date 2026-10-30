# 0181 — Owner ruling: T-2507 merges the Loader beta density with 661 sweep cells newly visibly off and one gated 1-ulp regression

- **Status:** accepted (2026-10-30, T-2507; ruled by the owner in session
  https://claude.ai/code/session_01Xbmds2YtTrK9a9T4mMitSB, who accepted the trade-off T-2507's "Blocked because" put to
  the owner: "Unblock when T-2586 pins the shape constant and the series branch, or when the owner accepts the losses
  with a decision record"). It amends decision 0083's standing allowance for T-2507's PR only, as decisions 0152 and
  0180 did for T-0972 and T-2509.
- **Context:**
  - T-2507 computes BETA.DIST(x,a,b,FALSE) for min(a,b) > 1 as Excel's Loader saddle point in Excel's arithmetic
    (`functions/statistical/loader.rs` `excel_beta_density`; statistics.md "Beta density for min(a,b) > 1 is Loader's
    saddle point"). The model's shape constant keeps a residual of 2-4 units of 2^-53 and its bd0 series branch 3-41
    units of noise (T-2586 items 1 and 2, still open), so it is not bit-exact everywhere.
  - Measured on the branch merged with `main` ec865788 (2026-10-30), against a `main` build, over gen-m9,
    ironcalc-calc, probes and probes-numeric: visibly-off cells fall from 82,100 to 46,065 (36,696 fixed), all of the
    fixes and losses BETA.DIST densities. probes-numeric t2464-r2-density-large-p1 is within 15 digits on all 13,824
    cells (10,935 on `main`), t2513-r3-loader-const-hold1 on 4,021 of 4,114 (2,282).
  - Two of decision 0083's conditions do not hold:
    - condition 2: 661 probes-numeric BETA.DIST cells become visibly off, listed with Excel's and our values in
      docs/research/t2507/newly_visibly_off.tsv (all in ungated research sweeps; none in a gated suite);
    - the gated suites: ironcalc-calc fixes 2 lines and regresses one, STATISTICAL/BETADIST_BETAINV.xlsx Sheet1!G3, by 1 ulp
      (Excel 0.17924096936442618; the correctly rounded density is 0.17924096936442616, so a correctly rounded kernel
      loses G3 too). It is numeric and within 4 ulp, so conditions 1 and 4 hold for it.
- **Decision:** T-2507 may merge with exactly the cells in docs/research/t2507/newly_visibly_off.tsv newly visibly off
  and the one gated regression above. T-3220 owns them. Excel stays the spec (directive 1): this accepts a temporary gap
  and settles no cell.
- **Consequences:**
  - The Excel-fidelity reviewer checks T-2507's newly visibly-off cells against the TSV; any other cell needs its own
    decision record.
  - Decision 0083's condition 2 is unchanged for every other PR.
