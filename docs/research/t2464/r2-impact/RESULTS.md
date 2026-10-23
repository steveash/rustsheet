# T-2464 r2-impact: from the research to an engine plan

Worker r2-impact, round 2, 2026-10-23. Method:
1. Run the eval and list every visibly-off cell of the incomplete-beta family.
2. Read the engine's incomplete beta and density code and compare it with TOMS 708 `bratio`.
3. Map each cell to a bratio branch and to the research piece that would fix it, and score each piece's model against
   Excel cell by cell.
4. Turn that into an ordered list of small engine tasks.

No engine code was edited and no probes were added.

Counts are bit-exact / within 4 ulp / within 15 significant digits ("w15"). The w15 test is the eval's own
`within_fifteen_digits` (closeness.rs), transcribed in `implib.within15`. "Visibly off" means outside 15 digits, or
a wrong type or error. "Fixed" means the engine is visibly off now and the model is within 15 digits. "Broke" means
the engine is within 15 digits now and the model is not. "Lost exact" means the engine is bit-exact now and the
model is not; in a gated suite that is a new baseline F line. Hold-out is the md5-odd half of (case, cell). Run every
script with `/home/user/rustsheet/bindings/python/.venv/bin/python` from this directory.

## 0. Data, models, controls

- **Eval run.** `target/release/rustsheet-eval run --out <scratch>` (release build of HEAD 505c10ab8, 2026-10-23
  10:50). It covers every suite, including the round-2 probe cases on disk at that moment. The script reads
  `report.json` (path in `implib.REPORT`, or override it with `$R2I_REPORT`).
- **Arguments.** `cells_all.tsv` comes from `../extract.py` with an empty filter: 138,915 single-call cells with
  resolved numeric arguments, in probes-numeric, probes, gen-m5/6/7/9. It is git-ignored. Rebuild it with
  `cd tools && uv run python ../docs/research/t2464/extract.py "" r2-impact/cells_all.tsv`.
- **Populations.**
  - **pre-T2464**: every cell outside the t2464 probe cases. This is the eval as it stood before the research:
    **5,533 visibly-off cells**, the README's "about 5,600".
  - **w2f-probes**: the tracked round-1 probe cases t2464-w2f-*. These are the tuning data of w2, w3 and w4.
  - **r2-probes**: the untracked round-2 probe cases. They change while this runs, so they are only counted.
- **Models, all imported and none copied** (`implib.py`):
  - r2-bratio's `bratio_excel`: a TOMS 708 port with every round-1 piece behind a switch. Its attribution ladder
    is R_LIKE, TOMS_XL, +closed, +w3 bpser, +z64, +lnΓ, +brcmp1 = pdf, +W arithmetic, BEST.
  - "BEST+log1p" swaps in r2-log1p's Excel log1p (`excel_log1p.log1p_series`) inside the density.
  - "LDe" is r2-density-large's Loader density for min(a, b) > 1 (`model.density`, stir = exact, lf = LF0).
  - The GAMMALN fetches of these modules are redirected to caches in this directory. No other worker's files were
    written (cache mtimes checked).
- **Cell to kernel mapping** (`implib.problem`). These are w5's verified argument forms:
  - F: x = 1/(1 + d2/(d1 X)), y = 1/(1 + d1 X/d2), d truncated.
  - T: y = fl(n/fl(n + t t)) with complement fl(1 - y) beyond t = 1 (and at df 2); in the centre,
    L = 0.5 - I_z(1/2, n/2)/2.
  - BINOM: 1 - I_p(k+1, n-k), with x = p and y = fl(1 - p).
  - NEGBINOM: I_p(r, f+1).
  - BETA.DIST: y = fl(1 - x). With [A, B], t = (x - A)/(B - A).
- **Positive control** (`impact.out`, top). Through these wrappers, BEST is bit-exact on every closed-form cell of
  every function: BETA.DIST 141/141, F.DIST 165/165, F.DIST.RT 180/180, FDIST 22/22, BINOM 128/128,
  NEGBINOM 142/142, T.DIST.RT 510/510, T.DIST 9/9, T.DIST.2T 7/7. A wrong argument form or tail would break these.
- **Bucket control.** offcells.py's visible counts agree with the report's per-function `visible` totals: for
  BETA.DIST, 24,980 standing plus 27,814 round-2 probe cells is 52,794, against 52,796 in the report. The 2-cell
  difference is attribution: the report counts a function wherever it is called, and this script uses the first
  family call in the formula (not checked further).

## 1. Every visibly-off cell (`offcells.py` → `offcells.out`, `offcells_pre.tsv`)

The eval has 74,686 visibly-off family cells in all. They split into 5,533 pre-T2464, 23,654 in the w2f probes and
45,499 in the round-2 probes. `offcells_pre.tsv` lists all 5,533 pre-T2464 cells, one row each, with function, case,
cell, args, Excel, engine, bratio branch, piece, the plan model's value and whether it fixes the cell.
`offcells.tsv` (git-ignored, rebuilt by `offcells.py`) lists all 74,686.

Pre-T2464 visibly-off cells by function (1,236 in gated suites):

| function | cells | gated | function | cells | gated |
|---|---|---|---|---|---|
| BETA.DIST | 1,326 | 36 | F.INV.RT | 148 | 93 |
| F.DIST | 826 | 23 | F.INV | 145 | 107 |
| BINOM.DIST | 642 | 18 | NEGBINOM.DIST | 126 | 6 |
| BETA.INV | 262 | 15 | FINV | 123 | 97 |
| F.TEST | 262 | 256 | T.INV | 110 | 11 |
| T.DIST.RT | 252 | 6 | CONFIDENCE.T | 107 | 59 |
| FTEST | 250 | 248 | BINOM.DIST.RANGE | 83 | 12 |
| F.DIST.RT | 247 | 27 | BETAINV | 57 | 49 |
| T.DIST | 209 | 10 | FDIST, BETADIST, TINV, BINOMDIST | 47, 37, 28, 20 | 38, 31, 5, 18 |
| T.INV.2T | 172 | 26 | T.DIST.2T, CRITBINOM, BINOM.INV, TTEST, T.TEST, TDIST, NEGBINOMDIST | 11, 11, 9, 7, 6, 6, 4 | 4, 11, 9, 7, 6, 6, 2 |

By suite: probes-numeric 4,297 (not gated), gen-m9 1,108, ironcalc-calc 86, probes 42.

What the 5,533 cells are (`annotate.out`):

| what | cells | gated |
|---|---|---|
| kernel cdf cells with resolved arguments (sections 3-4) | 1,519 | small |
| beta density cells (BETA.DIST(.,FALSE)) | 399 | 0 |
| **error or type mismatch (not the kernel)**: F.TEST/FTEST give #VALUE! where Excel gives #DIV/0!, #N/A or #NUM! (T-2093) on at least 409 cells; #NUM! where Excel has a number on 135 inverse and F.DIST cells | 879 | **792** |
| inverses with numeric arguments (BETA.INV, F.INV*, T.INV*) | 737 | 129 |
| same kernel, arguments not resolved here (nested or reference args: BETA.DIST(ref) 329, CONFIDENCE.T 101, BINOM.DIST.RANGE 73, F.TEST 81, ...) | 741 | 293 |
| other kernels: F density 580, BINOM pmf 368, t density 184, NEGBINOM pmf 94, BINOM k = 0 shortcut 32 | 1,258 | 3 |

**Finding 1:** the gated visible-parity lock barely depends on the kernel. Of the 1,236 gated visibly-off family
cells, 792 are argument and error handling and 129 + 293 are inverses or nested calls. Only about 23 mapped kernel
cells are gated. The kernel work moves the ungated probes-numeric cells, plus about 86 ironcalc-calc cells, which
are not mapped here.

## 2. The engine's incomplete beta, and how it differs from TOMS 708 bratio

Code: `crates/rustsheet-engine/src/functions/statistical/`.

| piece | engine today | Excel (round 1, verified) |
|---|---|---|
| BETA.DIST cdf (`beta.rs`) | **statrs `Beta::cdf`**, not `incomplete_beta` (TODO(T-2119)) | bratio |
| F.DIST / F.DIST.RT / FDIST (`fisher.rs`) | `incomplete_beta::beta_reg_complement` / `beta_reg_upper`, x = d1X/(d1X+d2), y = d2/(d1X+d2) | bratio; x = 1/(1+d2/(d1X)), y = 1/(1+d1X/d2) (w5: 197/197 against 180) |
| T.DIST* (`student_t.rs`) | its own composite: a double-double series for integer df up to 1000, a Cephes power series in the deep tail, `incbcf`/`incbd` from z, a GAMMALN scaling for df 13-16, and the df-2 closed form through `beta_reg_complement` | bratio on I_y(n/2, 1/2) beyond t = 1 and I_z(1/2, n/2) in the centre (w5) |
| BINOM / NEGBINOM cdf (`binom.rs`, `discrete.rs`) | a double-double pmf walk (an almost exact sum), with statrs as fallback | **bratio**: I_p(k+1, n-k) and I_p(r, f+1), with x = p and y = fl(1-p) (w5: 348/348 computed tails) |
| BINOM.DIST(0, n, p) | the walk | EXP(fl(n·l)), where l is Excel's log1p(-p) for p <= 0.375 and LN(fl(1-p)) above (w5, r2-log1p) |
| inverses (`inverse_beta.rs`, `beta.rs`, `fisher.rs`) | statrs AS 64/109 Newton on **statrs' `checked_beta_reg`**, with bisection fallbacks | a search to the crossing of Excel's own CDF (T-1745: 155/188) |
| beta density (`incomplete_beta::beta_density`) | powf(x, a-1)·powf(1-x, b-1)·tgamma ratio for a+b < 171.6; statrs otherwise | M1 for min <= 1 (w1); Loader dbinom_raw for min > 1 (w4, r2-density-large) |

Inside `incomplete_beta.rs`, compared with bratio:
1. **Closed forms on the unoriented arguments.** b == 1 gives exp(a ln x) and a == 1 gives 1 - exp(b ln(1-x)),
   with std ln/exp, before any swap.
   - Excel applies them after bratio's swap. It takes b0 == 1 → EXP(a0·LN x0); a0 == 1 with y0 <= 0.9 →
     EXP(b0·LN(fl(1-x0))); and the other tail as (0.5 - w) + 0.5.
   - Below the y0 cut it takes no closed form.
2. **Reflection at the mean**, x > a/(a+b), for every shape.
   - bratio swaps at x > 0.5 when min(a, b) <= 1, and at λ = a - (a+b)x < 0 otherwise. A mean switch gets 8 of the
     1,208 decidable F pairs wrong.
3. **Cephes `incbcf`/`incbd` continued fractions everywhere**, with no series, no bup, no bgrat and no basym.
   bratio instead picks among fpser/apser, bpser, w1 = bpser(b, a, y), bup + bgrat, bgrat, L140 bup + bpser,
   bup + bgrat, bfrac and basym. bfrac is only used at min(a, b) >= 40.
4. **Prefactor.** exp(a ln x)·exp(b ln y)·tgamma(a+b)/(tgamma(a) tgamma(b))/a below MAXGAM, else a brcomp/algdiv
   form.
   - Excel uses EXP(z), z = fl(a·LN x) - lnB, in bpser.
   - In bup and bfrac it uses its own density times x·y (brcmp1 and brcomp = pdf·x·y).
5. **Complement** 1 - v. Excel uses (0.5 - w) + 0.5.
6. **Elementary functions.** The engine uses std ln, exp, powf and musl tgamma. Excel uses its own EXP (V6), LN
   and GAMMALN, with x87 stored operations, and an extended EXP argument in bpser and in the density exponent.

Engine building blocks that already exist:
- `x87_exp::excel_exp` (double argument only);
- `x87_log10::excel_ln`;
- `gammaln::excel_gammaln`;
- `arithmetic::extended_{sum,product,quotient}`, which are fl53(fl64(.)) stored ops;
- `x87_register::Register`, an unstored 64-bit value.

Missing pieces:
- **EXP of an extended (Register) argument**, needed by the M1 density and by bpser z64;
- **Excel's log1p**, which r2-log1p solved.

## 3. Map to branches and pieces, scored per cell

### 3.1 The r2-bratio ladder against the engine (`impact.py` → `impact.out`)

Pre-T2464 mapped cells: n = 13,655 (cdf + beta pdf). The engine is visibly off on 1,998 of them, bit-exact on
4,227, w4 on 10,420 and w15 on 11,657.

| model | fixed | broke | net | exact | w4 | w15 | lost exact (gated) |
|---|---|---|---|---|---|---|---|
| R_LIKE (plain TOMS 708, libm) | 1,226 | 599 | +627 | 4,620 | 10,681 | 12,284 | 1,740 (15) |
| TOMS_XL (Excel EXP/LN/GAMMALN) | 1,264 | 587 | +677 | 4,566 | 10,635 | 12,334 | 1,666 (13) |
| +closed | 1,275 | 572 | +703 | 5,214 | 10,764 | 12,360 | 1,107 (8) |
| +w3 bpser (z53) | 1,368 | 487 | +881 | 5,775 | 11,112 | 12,538 | 1,084 (10) |
| +z64 | 1,462 | 330 | +1,132 | 6,190 | 11,509 | 12,789 | 982 (8) |
| +w3 lnΓ (toms) | 1,465 | 330 | +1,135 | 6,194 | 11,508 | 12,792 | 992 (8) |
| +brcmp1 = pdf·x·y | 1,591 | 231 | +1,360 | 6,685 | 12,158 | 13,017 | 899 (6) |
| +W arithmetic | 1,592 | 230 | +1,362 | 6,682 | 12,157 | 13,019 | 897 (6) |
| BEST (+brcomp = pdf·x·y) | 1,609 | 214 | +1,395 | 6,700 | 12,196 | 13,052 | 895 (6) |
| **BEST+log1p** | **1,618** | 213 | **+1,405** | **6,714** | **12,220** | **13,062** | 893 (6) |
| BEST without closed forms | 1,604 | 260 | +1,344 | 5,986 | 12,059 | 13,001 | 1,508 (11) |
| BEST with TOMS bpser | 1,418 | 457 | +961 | 5,724 | 11,452 | 12,618 | 1,008 (9) |
| BEST with TOMS brcmp1 | 1,486 | 317 | +1,169 | 6,200 | 11,540 | 12,826 | 988 (8) |
| BEST with TOMS brcomp | 1,592 | 230 | +1,362 | 6,682 | 12,157 | 13,019 | 897 (6) |
| engine + closed forms only | 83 | 0 | +83 | 4,473 | 10,520 | 11,740 | 0 |
| engine + M1 density only | 149 | 1 | +148 | 4,660 | 10,772 | 11,805 | 69 (0) |
| engine + closed + M1 density + log1p | 232 | 1 | +231 | 4,905 | 10,874 | 11,888 | 69 (0) |

The hold-out half (6,812 cells, engine visibly off on 979) has the same shape:
- BEST+log1p fixes 787 and breaks 101;
- R_LIKE fixes 608 and breaks 280;
- engine + closed fixes 39 and breaks 0.

**Finding 2: a plain TOMS port is net-negative outside BETA.DIST.** On the 10,602 pre-T2464 F, T, BINOM and
NEGBINOM cells, which are out of sample for the bpser and L140 pieces, the visible change is:

| model | net visible change |
|---|---|
| R_LIKE | -103 |
| TOMS_XL | -51 |
| +closed | -35 |
| +w3 bpser | +108 |
| BEST | +543 |
| BEST+log1p | +550 |

So the engine must port the Excel pieces with the dispatcher, not plain TOMS first.

### 3.2 Per branch (BEST's dispatch; pre-T2464 in `impact.out`, per function group in `branches.out`)

| branch (pre-T2464) | cells | engine vis | fixed by BEST+log1p | broke | left | the piece that fixes it |
|---|---|---|---|---|---|---|
| bpser(a,b>1) | 1,368 | 369 | 295 | 10 | 74 (48 BINOM) | P4 bpser (w3 series) |
| w_bpser | 3,218 | 341 | 323 | 9 | 18 | P4 |
| bgrat | 1,718 | 246 | 133 | **152** | 113 | U1 bgrat (unsolved); 145 of the broken cells are T at (n/2, 1/2) |
| L140 bup+bpser | 1,516 | 244 | 235 | 12 | 9 | P5 L140 with brcmp1 = pdf·x·y |
| pdf min>1, max>=8 | 583 | 224 | 148 (LDe 149) | 4 (LDe 17) | 76 | P6 Loader density |
| pdf M1 (min <= 1) | 1,107 | 155 | 149 | 1 | 6 | P2 M1 + log1p |
| bfrac | 298 | 128 | 84 | 1 | 44 | P7 bfrac (fraction arithmetic unsolved) |
| L140 bup+bgrat | 232 | 79 | 72 | 1 | 7 | U1 |
| closed:b0=1 / a0=1 | 1,304 | 83 | 83 | 0 | 0 | P1 closed forms |
| bup+bgrat | 1,502 | 48 | 34 | 16 | 14 | U1 |
| pdf min>1, max<8 | 345 | 37 | 19 (LDe 24) | 4 (LDe 8) | 18 | P6 |
| w1_bpser | 407 | 36 | 35 | 2 | 1 | P4 |
| L140 bup+bup+bgrat | 55 | 8 | 8 | 1 | 0 | U1 |

w2f probes (n = 41,554, engine visibly off on 23,657):
- BEST+log1p fixes 20,546 and breaks 101.
- By branch: L140 bup+bpser 8,944/8,944; w_bpser 5,182/5,182; pdf max>=8 3,082/5,305 (LDe 4,885);
  L140 bup+bgrat 2,290/2,494; bfrac 438/1,024.

The Loader density (`pdf_large.out`):
- On w2f it beats the correctly rounded stand-in clearly (max >= 8: fixes 4,885 and breaks 31, against 3,082 and
  67).
- On pre-T2464 cells it is no better: max >= 8 fixes 149 and breaks 17, against 148 and 4; max < 8 fixes 24 and
  breaks 8, against 19 and 4.
- Both pre-T2464 sets are round-1 data that r2-density-large used, so this is a weak point of LDe, not a hold-out
  win.
- The engine's own powf × tgamma density is bit-exact on 1,222 of the 7,168 w2f max<8 cells, against 378 for the
  correctly rounded value and 719 for LDe. It is still visibly off on more cells than LDe (153 against 0).

### 3.3 Inverses, and BINOM's k = 0 shortcut

- `inverse.py` → `inverse.out`: a bisection to the crossing of the BEST+log1p CDF, taking either neighbour.
  - It covers 1,207 numeric inverse cells (pre-T2464, gated included), of which the engine has 778 visibly off.
  - It fixes 418 of them and breaks 115.
  - Per function: BETA.INV 185/243, BETAINV 10/10, F.INV 43/100, F.INV.RT 56/102, FINV 47/80, T.INV 29/97,
    T.INV.2T 43/125, TINV 5/21.
  - Bit-exact on 310.
  - This is an estimate only. Excel's start and stop rule are unknown (T-1745 misses 11 of 188), and the model CDF
    itself is wrong on the bgrat cells.
- `binom_k0.py` → `binom_k0.out`: BINOM.DIST(0, n, p, ·) = EXP(fl(n·l)) with r2-log1p's log1p is
  **345/345 bit-exact**. It fixes all 68 visibly-off pre-T2464 cells (cdf and pmf) and breaks none.

## 4. The engine plan (`ladder.py` → `ladder.out`)

The ladder is cumulative. Each step switches one more group of bratio branches from the engine to the model; the
rest stay on the current engine. "vis left" counts the mapped pre-T2464 cells that are still visibly off; the engine
starts at 1,998.

| step | pre-T2464 vis left (Δ) | hold-out vis left (Δ) | non-BETA vis left (Δ) | w2f vis left (Δ) | lost exact, all / gated (cum.) |
|---|---|---|---|---|---|
| engine | 1,998 | 979 | 986 | 23,657 | 0 / 0 |
| S1 closed forms | 1,915 (-83) | 940 (-39) | 963 (-23) | 23,643 (-14) | 0 / 0 |
| S2 density M1 + log1p | 1,767 (-148) | 859 (-81) | 963 (0) | 23,233 (-410) | 69 / 0 |
| S3 bratio dispatch + bpser | 1,135 (-632) | 556 (-303) | 609 (-354) | 17,952 (-5,281) | 362 / 2 |
| S4 L140 bup+bpser | 912 (-223) | 447 (-109) | 475 (-134) | 9,008 (-8,944) | 446 / 5 |
| S5 bfrac / basym | 829 (-83) | 407 (-40) | 434 (-41) | 8,570 (-438) | 451 / 5 |
| S6 bgrat family, not T | 692 (-137) | 325 (-82) | 376 (-58) | 6,251 (-2,319) | 486 / 6 |
| S7 Loader density min > 1 | **544** (-148) | **257** (-68) | 376 (0) | **1,244** (-5,007) | 514 / 6 |
| (negative: S6 for T as well) | 604 (+60) | 294 (+37) | 436 (+60) | 1,244 (0) | 899 / 6 |

Two more groups lie outside the ladder:
- the BINOM k = 0 shortcut: -68;
- inverses, by a crossing search: about -418 / +115 of the 778 numeric inverse cells.

All in, the plan removes about 1,454 + 68 + ~300 = **~1,800 of the 5,533 pre-T2464 visibly-off cells**. On w2f it
removes 22,413 of 23,657.

**The gated cells a port would regress.** These need a decision record and `--allow-regressions`, or a narrower
change. `branches.out` names every one: 6 lost bit-exact cells and 2 that become visibly off.
- `probes/stat-argument-modes` F.DIST H215, F.DIST.RT H221 and FDIST H226 at (2.5, 2.5, x = 0.6000000000000001),
  L140: 2-4 ulp.
- `gen-m9` F.DIST.RT 0a614cfe A19 (bpser(a,b>1), 1 ulp), FDIST 05d44f6d A175 (w_bpser, 1 ulp) and FDIST c18dbf21
  A151 (bup+bgrat, 7 ulp).
- Visible: F.DIST.RT 60b0b6e6 A151 (6 ulp at 6.6e-44) and FDIST 2801c1c1 A374 (6 ulp).
- Not scored here: the ironcalc-calc workbooks (args unresolved). T-2119's nine chained BETA.INV cells in
  BETADIST_BETAINV.xlsx are known to move.

### Task list (S/M, one piece each, in order)

| # | task (size) | depends | can start now? | acceptance criteria | expected visibly-off change |
|---|---|---|---|---|---|
| E0 | **Excel log1p and EXP of an extended argument as engine primitives (S).** `excel_log1p` is r2-log1p's x87-stored atanh series in alnrel's variable (14 terms, LN(fl(1+a)) below -0.375). `excel_exp_ext(Register)` is EXP V6 on a 64-bit argument. Wire BINOM.DIST(0, n, p) = EXP(fl(n·l)). | none | **yes** | Unit rows: r2-log1p's pinned l, 16,000/16,027 (every miss 1 ulp). EXP-ext against w1's mpmath V6 model on random extended inputs, plus the double-input subset bit-identical to `excel_exp`. BINOM k = 0 cells 345/345. | **-68** (BINOM k = 0, cdf and pmf), 0 broke |
| E1 | **Oriented closed forms (S).** In `incomplete_beta` and the BETA.DIST cdf path: swap at x > 0.5 (min <= 1) or λ < 0, then b0 == 1 → EXP(a0·LN x0); a0 == 1 and y0 <= 0.9 → EXP(b0·LN(fl(1-x0))); other tail (0.5 - w) + 0.5. Use Excel's EXP/LN. Also take F's x, y forms from w5. | none | **yes** | w5's `closed_final.py` set 1,286/1,286. Positive control here: 1,312/1,312 through every wrapper. No new baseline F lines. | **-83** pre-T2464 (-39 hold-out), -14 w2f; 0 broke, 0 lost |
| E2 | **Beta density M1 for min(a, b) <= 1 (S/M).** brcomp's ln(1-x) switch at 0.375 (E0's log1p), tx = fl(fl(a-1)·LN x), ty likewise, TOMS betaln with Excel GAMMALN/LN (GL form up to max <= 8 inclusive, smaller shape first; algdiv above; a = 1 → -LN b), E = tx + (ty - lnB) extended (b = 1: stored), EXP-ext. Keep F.DIST(.,FALSE) on the old density until it is measured (its d <= 2 cells use `beta_density`). | E0 | after E0 (the CR-log1p variant can start now: the same -148) | M1 scores (w1: 685/2,035 old, 10,301/16,000 probe1; r2-log1p: 10,375/12,800). No F-density regression. | **-148** pre-T2464 (-81 hold-out), -410 w2f; 1 broke; 69 bit-exact lost in probes-numeric, 0 gated |
| E3a | **Spec: the bratio port (S, docs/specs).** Module layout, K(a, b, x, y) → (w, w1, branch), each caller's argument forms (w5 2.3), the migration rule (a branch moves only when its piece lands; other branches delegate to today's kernel), bounded loops, no panics, the gated regressions above. | none | **yes** | Spec approved. | 0 |
| E3 | **bratio dispatch + bpser branches (M).** TOMS dispatch with x0 >= 0.3 (r2-bratio), fpser/apser. bpser is w3's forward x87 series from fl(1/a) with I = fl(EXP-ext(z)·s), z = fl64(fl(a·LN x) - L). L is r2-bpser-small's GL53 / glalgdiv ln B, which beats this table's BEST on its 2,000-shape probe. Covers BETA.DIST cdf (subsumes T-2119), F, BINOM, NEGBINOM and T in the w_bpser / w1_bpser / bpser(a,b>1) branches only. | E0, E1, E3a | after E3a | Per branch, at least: w_bpser 6,767/11,845 and w1_bpser 1,666/1,845 (r2-bratio BEST), bpser(a,b>1) 978/1,523. Decision record for the 2 gated lost cells. | **-632** pre-T2464 (-303 hold-out; -354 non-BETA), -5,281 w2f; 22 broke |
| E4 | **L140 = bup with brcmp1 = Excel density·x·y, plus bpser (M).** w4's bup arithmetic: P·fl(y0·x)/b0', ratios stored, ratio·y in a register. b0' <= 1, so the prefactor density is E2's M1. | E2, E3 | after E3 | r2-bratio: L140 bup+bpser 7,505/11,673/11,713 of 11,742. ix L140 1,011/1,620. | **-223** pre-T2464, **-8,944** w2f (all of the branch); 3 more gated lost (stat-argument-modes) |
| E5 | **Beta density for min > 1: Loader dbinom_raw (M).** r2-density-large's LDe: q = fl(1-x), bd0 log form with stored ratio and Excel LN, cut 0.2, stirlerr and lf constants. | E0 | in parallel with E3 (r2-density-large's model is ready to port; its constant is unexplained, ±8 units) | LDe's 5,765/25,254/31,385 of 32,264, and on its own probe hold-out 13,824/13,824 w15. | **-148** pre-T2464 (it breaks 25 there), -5,007 w2f |
| E6 | **bfrac with brcomp = Excel density·x·y (S).** TOMS bfrac, λ = fl(a - fl((a+b)x)), for min(a,b) >= 40; basym. | E3, E5 | after E5 | (100, 120) run: at least 185/1,024 exact (w4). | **-83** pre-T2464, -438 w2f; the fraction's own noise (sd 3.8 units) stays open |
| E7 | **bgrat family through TOMS bgrat with Excel pieces, except T (S/M).** bup+bgrat, bgrat, L140 bup+bgrat and bup+bup+bgrat for BETA.DIST/F/BINOM/NEGBINOM. T.DIST* keeps `student_t.rs` in these branches. | E3, E4 | after E4 | Net visible gain per function. No T change. | **-137** pre-T2464, -2,319 w2f; 17 broke. Switching T too would add +60 (negative) |
| E8 | **Inverses solve the ported CDF to the crossing (M).** Replaces statrs' inversion for BETA.INV, F.INV, F.INV.RT and T.INV*. Settles T-2119's chained BETA.INV cells and T-0974. | E3-E7 | last | T-1745's crossing rule: 155/188, or better. | about **-418 / +115** of 778 numeric inverse cells; stop rule open |

Ready now from the round-1/2 results alone: **E0, E1, E2 (with the CR log1p if E0 is not merged yet), E3a and E5**.
E3 can start once the spec lands. All its pieces are fixed by round 1 and r2-bpser-small.

Outside the kernel, in the same family:
- T-2093 covers F.TEST/FTEST error precedence: at least 409 visibly-off cells, mostly gated, the largest gated block.
- The F density (580), t density (184) and BINOM/NEGBINOM pmf (462) are separate kernels with no model yet.
- The T bgrat family (T.DIST.RT at (n/2, 1/2), n/2 >= 8) stays on `student_t.rs` until r2-tdist or a successor
  solves it.

## 5. Negatives (all scored, scripts named)

- **A plain TOMS 708 port as the first step** (R_LIKE, TOMS_XL), on the non-BETA functions: net -103 and -51
  visible, and 1,740 / 1,666 lost exact (`impact.out`).
- **TOMS bgrat with Excel pieces for T.DIST*:** bgrat fixes 89 cells and breaks 145, bup+bgrat fixes 4 and breaks 8
  (`branches.out`). That is the +60 row of the ladder.
- **TOMS bpser / brcmp1 / brcomp in place of the Excel pieces** (one-piece ablations of BEST):
  - TOMS bpser: net +961, against +1,395;
  - TOMS brcmp1: +1,169;
  - TOMS brcomp: +1,362;
  - no closed forms: +1,344, and 613 more lost exact.
- **The correctly rounded density for min > 1, max < 8.** It loses 1,146 of the engine's 1,222 bit-exact w2f cells.
  LDe loses 1,150 too. The engine's powf × tgamma form is closer to Excel's last bits on those runs: 1,222 bit-exact
  against LDe's 719, though not within 15 digits more often.
- **LDe on pre-T2464 pdf cells:** no gain over the correctly rounded stand-in (section 3.2).
- **The crossing search on the model CDF** breaks 115 inverse cells the engine has within 15 digits (`inverse.out`).
- **BINOM**: bpser(a,b>1) leaves 48 cells visibly off and breaks 8; w_bpser breaks 4 and fixes 0. Integer shapes
  with b up to 100 are the weakest bpser region (w5 noted median 6 ulp).

## 6. Next hypotheses

1. **Gated F cells at (2.5, 2.5, x = 0.6000000000000001)** (stat-argument-modes H215/H221/H226). The engine is exact
   and L140 is 2-4 ulp off.
   - Check whether F's x is 1/(1 + (d2/d1)/X), which w5 could not separate from the chosen form.
   - Check L140's W rounding at b0' = 0.5, n = 2.
   - Either result would avoid a decision record in E4.
2. **Score r2-bpser-small's GL53 / glalgdiv ln B through this pipeline** (`implib.configs` takes a Cfg). That would
   firm up E3's acceptance numbers. Its own probe says +466 / +47 / +74 exact over BEST in the three bpser
   sub-branches.
3. **Map the 741 nested and reference cells.** BETA.DIST(ref) via w5's `BETA.DIST(ref)` rows, CONFIDENCE.T via
   T.INV, and BINOM.DIST.RANGE as a difference of two BINOM cdfs. Most should fall into the same pieces.
4. **T bgrat** (181 visible, 145 broken by TOMS): the open problem with the largest pre-T2464 count left after the
   plan.

## Scripts and outputs

| file | what |
|---|---|
| `implib.py` | report buckets (closeness.rs), argument join, cell → kernel mapping, model configurations, runner |
| `offcells.py` → `offcells.out`, `offcells.tsv` (ignored) | step 1: every visibly-off family cell, by population, function, suite, kind |
| `impact.py` → `impact.out`, `impact.pkl` (ignored) | step 3: every model against the engine per cell; the positive control; tables per population and branch |
| `branches.py` → `branches.out` | function group × branch; the gated cells a port would regress |
| `pdf_large.py` → `pdf_large.out` | min > 1 density: engine vs correctly rounded vs LDe |
| `annotate.py` → `annotate.out`, `offcells_pre.tsv` | the 5,533 pre-T2464 cells with branch, piece, plan value and fix flag; PLAN totals |
| `ladder.py` → `ladder.out` | step 4: the cumulative port ladder per subset |
| `inverse.py` → `inverse.out` | inverses: a crossing search on the model CDF |
| `binom_k0.py` → `binom_k0.out` | BINOM.DIST(0, n, p) with Excel's log1p |

Order: `offcells.py`, `impact.py` (3 min; it caches `impact.pkl`), then the rest.

Probes added: none.
