# 0180 — Owner ruling: T-2509 merges with five Student t cells newly visibly off

- **Status:** accepted (2026-10-30, T-2509; ruled by the owner in session
  https://claude.ai/code/session_01Xbmds2YtTrK9a9T4mMitSB). It amends decision 0083's standing allowance for PR #2055
  only, as decision 0152 did for T-0972.
- **Context:**
  - T-2509 (#2055) computes the Student t tails as one call into the ported incomplete beta, K(n/2, 1/2, y, z), with
    bratio's bgrat at b = 1/2 (`bratio/bgrat.rs`). The code was done on 2026-10-27; it was blocked because the
    workers' permission layer refused `update-baseline --allow-regressions`.
  - Since then `main` moved F onto the same bratio branches, so F with a df of 1 reached bgrat at b = 1/2 too, moving
    two gated FDIST cells by 7 ulp and making 13 F.DIST sweep cells visibly off. The PR now keeps bgrat at b = 1/2 for
    the Student t call only (`bratio::t_kernel_tails`); F stays on today's kernel until T-2510.
  - Measured on the branch merged with `main` ec865788 (2026-10-30):
    - gated suites: 81 lines fixed (gen-m9 69, ironcalc-calc 8, probes 4), 10 regressed, all numeric and each exactly
      1 ulp. Decision 0083's conditions 1, 3 and 4 hold for them.
    - over gen-m9, ironcalc-calc, probes and probes-numeric, against a `main` build: visibly-off cells fall from 82,100
      to 79,707 (2,398 fixed).
  - Condition 2 ("no cell becomes visibly off") does not hold for five cells, all Student t tails:
    - gen-m9 787ba8d801105a60 Calc!A91 `T.DIST.2T(0.5,1E+8)`: Excel 0.6170750785521777, ours 0.6170750785521784.
    - probes-numeric t2464-r2-tdist-p1 Calc!A10465 (df 16): Excel 0.0007059121118962457, ours 0.000705912111896245.
    - probes-numeric t2464-r2-tdist-p1 Calc!A3913 (df 1964): Excel 2.030639129174483E-08, ours 2.0306391291744768E-08.
    - probes-numeric t2464-r2-tdist-p1 Calc!A4416 (df 1665): Excel 1.5859946591496929E-10, ours 1.585994659149682E-10.
    - probes-numeric t2464-r2-tdist-p1 Calc!A9295 `T.DIST.RT(8.78041098985201,16)`: Excel 8.121918957904221E-08, ours
      8.121918957904214E-08.
- **Decision:** T-2509 (#2055) may merge with exactly these five cells newly visibly off. Follow-up T-2930 owns them
  and the ten gated 1-ulp cells. Excel stays the spec (directive 1): this accepts a temporary gap and settles no cell.
- **Consequences:**
  - The Excel-fidelity reviewer checks #2055's newly visibly-off cells against this list; any other cell needs its own
    decision record.
  - Decision 0083's condition 2 is unchanged for every other PR.
