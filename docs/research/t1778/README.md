# T-1778 research files on main (copied by T-1916)

The small text files the engine's ERF/ERFC kernels and Excel EXP model are built from, copied from branch
`agent/research/T-1778-kernel-map` at `59c2c3724a3c` (docs/research/t1778/ there). Everything else of the research
(probe generators, fitting scripts, and the about 1.8 GB of `kmyst-*` probe cases) stays on that branch; the
round READMEs below mention such files, which are not copied. The write-up is
docs/research/t1778-erf-kernel-research-plan.md and docs/excel-semantics/statistics.md, "One incomplete-gamma
kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)".

| File | What | Used by |
|---|---|---|
| `round3/sol100.txt` | Small-x ERF coefficients, P quartic and Q quintic in z = x*x, lowest degree first. No inline provenance line: `round2/sc.c` parses every token of it. | `erf.rs` `SMALL_P`, `SMALL_Q` |
| `round2/sc.c` | Exact x87 scorer for the small-x form (mode 0300). | |
| `round9/erfc_mid_coef.txt` | ERFC (9,10) rational for 0.5 < x <= 4, highest degree first. | `erf.rs` `MID_N`, `MID_D` |
| `round9/mid_check.c` | Exact x87 checker for it (`gcc -O1 -mfpmath=387`). | |
| `round7/erfc_tail_coef.txt` | ERFC (8,9) rational for 4 < x <= 26.55, highest degree first. | `erf.rs` `TAIL_N`, `TAIL_D` |
| `round7/erfc_tail_check.c` | Exact x87 checker for it. | |
| `round8/exp_v6.py` | Excel EXP model V6 reference (mpmath). | `x87_exp.rs` |

Each copied file has a provenance line at its top (at the end of `erfc_mid_coef.txt`, whose checker reads only
its first two lines). The curated oracle evidence on main is probes-numeric `t1916-*`.
