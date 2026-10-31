# T-2512 r3-bgrat: bgrat for b ≠ 1/2 and the L140 bup+bgrat cells

Worker r3-bgrat, 2026-10-31 (T-2512 item 1). Method: docs/research/excel-kernel-playbook.md. Base model: r2-tdist's
FINAL (r2-bratio `bratio_excel.py` BEST + Excel log1p, Excel ERFC at b = 1/2, bgrat's lnx cut at y = 0.01), imported
through `r2-tdist/xlk.py`, not copied. Scores are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit (w15). Hold-out = md5(case, cell) odd. Nothing below is fitted: every model is a fixed composition,
so the hold-out halves are reported only where a choice between forms was made (`bupdens.out`, `split.out`,
`zstore.out` carry both halves).

Python: `/home/user/rustsheet/bindings/python/.venv/bin/python` (`make py-bindings`, plus `uv pip install mpmath
numpy` into it), run from this directory. Order: `branches.py` → `exact.py -a` → the probes' `make_probe*.py` →
the scorers. `calls.pkl`, `exact.pkl`, `exact_l140.pkl` and `qxl.pkl` are git-ignored caches the scripts rebuild.

Data: `data.py` reads BETA.DIST(x, a, b, TRUE) cells straight from the probes-numeric case and oracle files (45,375
cells; formula literals decoded as Excel reads them: exact `M*2^E` forms exactly, decimals truncated to 15 digits).
The derived tables of rounds 1-2 (w4-fractions/data.tsv and so on) are not needed. F, T, BINOM and NEGBINOM cells are
not scored here.

## 1. Headline

1. **L140 bup+bgrat at 1 < b < 2 is solved as a composition of Excel's own pieces**, bit-exact on 2,515 of 3,072 and
   within 4 ulp on all 3,072 (hold-out 1,306/1,613/1,613):
   ```
   I_x(a, b) = RN( W + D )          b0 = b - 1 in (0, 1), a > 15, x > 0.7 (bratio's L140 with n = 1)
   W = fl( x87( P · fl(x·y) / b0 ) )     P = Excel's BETA.DIST(y, b0, a, FALSE): the density in bup's own
                                           orientation (y; b0, a), not the caller's (x; a, b0)
   D = Excel's BETA.DIST(x, a, b0, TRUE): bratio's direct bgrat branch, w = 0, the same increment alone
   ```
   (`bupdens.py` → `bupdens.out`; shared helpers in `common.py`). The orientation matters: with
   P = BETA.DIST(x, a, b0, FALSE) it is 1,267/1,767/1,815. The two densities differ on 1,355 of 2,908 points, by up
   to ±8 ulp (`orient.py` → `orient.out`). M2 takes LN or log1p by its own x, so the two orientations take different
   paths (not scored further here). With fl(fl(P·fl(xy))/b0) (two stored ops) it is 1,998/3,072/3,072, and with the
   sum also kept extended it is 2,126/3,072/3,072.
2. **The visible error on these cells is the density, not bgrat.** Ablations of the best form (`bupdens.out`,
   `engdens.out`):

   | W from | increment | exact/w4/w15 of 3,072 |
   |---|---|---|
   | Excel's density | Excel's D | **2,515/3,072/3,072** |
   | Excel's density | exact F (the true I_x(a, b0) at the double lnx) | 1,726/3,072/3,072 |
   | the engine's density (today's main) | exact F | 282/2,824/3,042 |
   | the engine's density | Excel's D | 478/3,057/3,057 |
   | r2-bratio's model density (M1 lnB) | Excel's D | 89/2,154/2,356 |
   | r2-tdist FINAL as is (M0) | | 102/1,973/2,162 |
   | the engine today (BETA.DIST cdf) | | 26/113/344 |

   So an engine port that takes W from the engine's density and an accurate increment moves about 2,700 cells to
   within 15 digits. Below that, the density's lnB for min < 1 < 8 < max (item 2, algdiv) is what is left.
3. **Why TOMS bgrat lost (T-2510's "net visible loss").** TOMS bgrat passes its own tolerance, 15·eps = 15e-15, to
   grat1, and grat1's continued fraction (every z here is >= 1.1) stops tens of ulps short. On the 2,908 direct bgrat
   cells (`epsscan.out`, `bgld.out`):

   | bgrat form | exact/w4/w15 of 2,908 |
   |---|---|
   | TOMS in doubles, Excel's EXP/LN/GAMMALN, eps 15e-15 (M0) | 13/142/162 |
   | the same, eps 1e-15 | 139/1,263/1,834 |
   | the same, eps 1e-16 or 1e-17 | 106/1,073/1,643 |
   | x87 extended, stored lnx and z, grat1 CF in extended at eps 15e-15 | 1/27/41 |
   | the same at eps 1e-15 | **206/1,672/2,158** |
   | the same at eps 1e-17 | 196/1,548/2,030 |
   | x87 extended with the exact Q(b, z) (any eps) | 191-202/1,587-1,592/2,069-2,075 |
   | CR: exact F at the double lnx | 208/1,598/2,091 |
   | RN(F · Q(b, z_s) / Q(b, z_e)), z_s stored, z_e = -nu·lnx exact (nu exact) | 246/1,881/2,357 |

   Excel's D sits a median -4 units of 2^-53 relative below F (sd 7, `ident.out`). It is not correctly rounded and is
   not TOMS in any precision tried, so a bit-level D is open. At the cell level an accurate increment is enough
   (finding 2).

## 2. The b ≠ 1/2 identity test (the task's "first test")

r2-tdist's b = 1/2 law is I = RN(F · Q_xl / Q), with Q_xl Excel's own ERFC. For general b it needs Excel's Q(b, z) at
bgrat's stored z. Probe **t2512-r3-bgrat-qgamma** (9,960 cells; `make_probe.py`, `read_probe.py`) asks for it two ways
at the 4,980 (b, z) pairs of every recorded bgrat call with b ≠ 1/2:
- `CHISQ.DIST.RT(2z, 2b)` cannot serve: CHISQ.DIST.RT truncates its df to an integer, so 2b < 1 is `#NUM!` and the
  rest evaluate Q(int(2b)/2, z) (4/10/10 of 3,490, `score_all.out`). The worksheet has no direct Q(b, z) at a
  non-half-integer b.
- 1 - `GAMMA.DIST(z, b, 1, TRUE)` loses the digits: Q < 0.01 on 5,196 of the 5,228 cells (`qsize.out`), so 1 - P
  carries more than 100 units of error relative to Q. On the 32 cells with Q >= 0.01 it scores 6/20/31 (exact F:
  9/19/32).

So the identity cannot be tested at generic b through the worksheet. That is a negative for the method, not for the law.
The split probe (section 3) replaces it: it measures Excel's bgrat increment directly.

## 3. Probes

| probe (probes-numeric) | cells | what | script |
|---|---|---|---|
| t2512-r3-bgrat-qgamma | 9,960 | GAMMA.DIST(z, b, 1, TRUE) and CHISQ.DIST.RT(2z, 2b) at every bgrat call's (b, z), b ≠ 1/2 | `make_probe.py` |
| t2512-r3-bgrat-split | 2,908 | BETA.DIST(x, a, b0, TRUE) at every L140 bup+bgrat cell: Excel's bgrat increment with w = 0 | `make_probe2.py` |
| t2512-r3-bgrat-bupdens | 5,816 | BETA.DIST(x, a, b0, FALSE) and BETA.DIST(y, b0, a, FALSE) there: bup's prefactor | `make_probe3.py` |

The source cells are the L140 bup+bgrat cells of t2464-w2f-absweep, -xruns-1 and -xruns-2 (shapes (30.7, 1.7) and
(40, 1.03) with their ulp neighbours). The split probe uses only cells whose direct counterpart takes bgrat in the
model (all of them did). Its 2,908 points serve 3,072 L140 cells (some cells repeat a point).

## 4. Where the bgrat cells are (`branches.py`, `bscore.py` → `score_all.out`)

On the 45,375 BETA.DIST cdf cells, FINAL reaches bgrat 5,960 times. For b ≠ 1/2 the open part is L140 bup+bgrat:
3,082 cells, of which M0 has 106/1,983/2,172. The direct bgrat branch (604) and bup+bgrat (1,542) with b ≠ 1/2 are
already 599/603/603 and 1,356/1,530/1,542 in M0. They are mostly t2464-r2-bratio-x29 cells (2,100 of 2,146), where
the increment is a small part of a tail near 1 and its error does not reach the 15th digit.

Per shape (`direct.py` → `direct.out`, `bupres.py` → `bupres.out`): D - F is a median -3, -4 and -1 ulp of D on
(30.7, 0.7) absweep, xruns-1 and (40, 0.03) xruns-2. The bup sum Excel implies (Excel - D) is offset from r2-bratio's bup
by a per-shape constant (-5.5, +3.25 and -2.1 ulp of w), and from the exact sum by -2.61, +0.82 and -0.15. That is the
density prefactor finding 2 isolates.

## 5. What is left (handed on)

- **D to the bit** (Excel's direct bgrat at b ≠ 1/2). Best form: the stored-z ratio, 246/1,881/2,357 of 2,908. Excel is
  a median 4 units low. Next: a run probe in x at fixed (a, b0), 8,192 consecutive doubles, to see which stored
  intermediate makes the sawtooth (playbook section 2); grat1's CF in x87 with Excel's own EXP for r.
- **The density's lnB for min < 1 < 8 < max** (item 2): now the whole visible gap on L140 one-term cells.
- **n > 1** (2 <= b < 40 at a > 15, x > 0.7): bup's sum of n terms with this prefactor. No such cells are in the
  probes yet. The m9-beta-dist integer-b cells reach bgrat at b0 = 1, where Q(1, z) = e^-z (10 cells, M0 4/10/10).
- Items 2-6 of T-2512 were not attempted in this round; they move to the follow-up research task.

## 6. Negatives (decision 0056)

| tried | on | score |
|---|---|---|
| Q_xl = CHISQ.DIST.RT(2z, 2b) | bgrat calls b ≠ 1/2 | 4/10/10 of 3,490 (df truncation) |
| Q_xl = 1 - GAMMA.DIST(z, b, 1, TRUE) | L140 cells | 161/1,113/1,209 of 3,082 (EXQ 114/2,082/2,448) |
| TOMS bgrat, doubles, eps 15e-15 / 1e-15 / converged | direct cells | 13/142/162, 139/1,263/1,834, 106/1,073/1,643 |
| grat1 CF in doubles inside the identity, eps 15e-15 / 1e-15 | direct cells | 18/222/251, 134/1,150/1,742 |
| x87 bgrat, CF eps 15e-15 / 1e-17; exact Q | direct cells | 1/27/41, 196/1,548/2,030; 191/1,587/2,069 |
| stored z with nu in doubles (x87 or double product) | direct cells | 212/1,650/2,133 (CR 208/1,598/2,091) |
| W from BETA.DIST(x, a, b0, FALSE) (caller orientation) | L140 cells | best 1,267/1,767/1,815 |
| r2-bratio's bup with Excel's D | L140 cells | 83/2,145/2,375 |
| exact bup sum with D or F | L140 cells | 526/2,609/2,818, 517/2,632/2,821 (`split.out`) |
