# T-1941 second review scripts (Fable 5.1, 2026-10-19)

Companion to `../HANDOFF.md` section 11. Run from this directory. Row dumps go in a scratch directory named by the
`SCR` environment variable (default: the reviewer's scratchpad); make them from the repo root with
`python3 docs/research/t1941/g_rows.py $SCR/g_rows.txt` and `python3 docs/research/t1941/review/ln_rows.py $SCR/ln_rows.txt`.
`lib2.py` caches both dumps as `.npy` on first use. Needs numpy (x87 longdouble) and gcc; `gsim.c` builds with
`gcc -O2 -shared -fPIC -o libgsim.so gsim.c` (180 ns per point, 5x numpy).

| Script | Result file | What it showed |
|---|---|---|
| (gammaln_big_check.c with `typedef double L`, SSE) | section 11.1.1 | x87 double rounding is real: fl53(fl64) 295,720 / 295,724 on x >= 8; pure RN53 295,641 |
| `k1_lattice.py` | `results_k1_lattice.txt` | AN2 stage table: every N and D constant sits on its sum's grid (u(S)/u(c) in {1,2,4}; D0 is 2^20 finer); stage weights |
| `k1_fingerprint.py` | `results_k1_fingerprint.txt` | output change for a +1 ulp move of each stored value at 1.85-2.0 and 1.0-1.05 (mean, std, P(0)); error histograms of Excel - AN2 |
| `k1_sawtooth.py N 1,3,5`, `D 2,4,6` | `results_k1_sawtooth_*.txt` | single-coefficient exact-match landscape: a 1-ulp-wide V, depth = stage visibility; no lattice |
| `xk_reach.py` | `results_xk_reach.txt` | model-free composition test on [0.7,3]: K3 = fl(t*A), (1.5,2] = fl(t*fl(m*R)), K2 = fl(m*S); nested variants at base rate |
| `k2_inner_reach.py` | `results_k2inner.txt` | K2's stored inner S is NOT fl((x-1)*R): reachable at the base rate (0.796/0.803, 0.886/0.890) where the test has power |
| `rehash_matrix.py` | `results_rehash.txt` | the re-hash cascade: fraction of points where each stored sum changes after +1 ulp on each coefficient, K1/K2/K3 |
| `fm_sideband_ctl.py 1.88`, `1.05` | `results_fm188.txt`, `results_fm105.txt` | positive control: two invisible product phases read from one run by an exact-template (nudge) scan; survives one wrong downstream stage at 1.88, not at 1.05 |
| `posctl_fm.py`, `fm_decode.py` | (negative, not kept) | per-run decoding of all 28 phases from a cold start fails: coordinate ascent on matches and single-template correlation both have no gradient when many stages are wrong |
| `top_bump.py` | `results_top_bump.txt` | Excel vs AN2: the (d2,d3) scan (phases of S_1*y and S_2*y) is a V centred at (0,0) at anchors 1, 1.5, 1.75, 1.875, 2- and on 150 runs at 1.70-1.96; synthetic top moves give bumps of +133..+548 points |
| `top_bump2.py` | `results_top_bump2.txt` | a null-direction truth 1e4 ulps off gives a FLAT collective profile (range 70 on 311k points); per-run Excel scans: (0,0) best on most runs |
| `stage_scan.py p1`, `anch` | `results_stage_scan_*.txt` | per-run 1-D phase scans of all 14 products and 14 sums on Excel's P1/P1b runs and the 1.875/2- rungs: best signed phase and gain per stage |
| `k2_upstream_fm2.py` | `results_k2_upstream_fm2.txt` | K2 meter control: upstream p7-2/p6+3/q7+2/q6-3 makes single-phase readings of N s4/s5, D s3/s4 pseudo-random (+-0.03..0.5, r 0.07-0.39); downstream p2+1/q1-1 changes nothing |
| `k23_midscan.py` | `results_k23_midscan.txt` | +-6 ulp scans of mid coefficients: ss_fc and grid_a are local maxima in every coordinate; grid_a's p5 is pinned to the ulp (+-1 ulp costs 25 points); grid_a q7 and ss_fc N7/D7 show small monotone gradients |

Nudge convention (gsim.c, lib2.py): a product nudge `pn[k]` (k = 0 is the top product c[n]*y) adds `pn[k]` times the
ulp of the double nearest the pre-rounding product before it is stored; a sum nudge `sn[k]` does the same for the
pre-rounding sum. A constant product nudge at a run is the local coordinate for the sub-grid part of the upstream
stage value `S_{k-1}(y0)*y0`, i.e. it reads `frac(dS_{k-1}(y0)*y0/u(P_k))` where `dS` is Excel's stage value minus
the candidate's. Phases are reported signed in (-0.5, 0.5].
