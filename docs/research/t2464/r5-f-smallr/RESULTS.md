# T-2586 r5-f-smallr: Excel's -h·L(r) term of the F density for r <= 0.375

Worker r5-f-smallr, round 5, 2026-10-24. Counts are bit-exact / within 4 ulp / within half a unit of the 15th
significant digit ("w15"), "of n". "Units" are 2^-53, absolute in ln F. a = d1/2, b = d2/2, h = a + b, m = d1,
n = d2, mx = fl(m·x), r = the smaller of fl(mx/n) (side 1) and fl(n/mx) (side 2), L = Excel's log1p (r2-log1p's
14-term series, stored), H = -h·L (extended product), P = +fl(a·LN x) (side 1) | -fl(b·LN x) (side 2). "Free-C
score" = cells of a run within 3 units of the best fit of a chain with one free constant per run (`r5lib.scan`).
Run scripts from this directory with `/home/user/rustsheet/bindings/python/.venv/bin/python`.

## 1. Headline
**Not cracked.** The -h·L(r) term for r <= 0.375 is still not identified, and no frozen model reaches visible
parity. What this round established:

1. **The round-4 "double-quantum jumps" are not proof of two stored H pieces.** A free-constant scan on an
   inexact lattice (round 4: the constant phase was K' plus a non-dyadic K_lo) cannot reach the exact ties of a
   stored sum. With the constant on an exact 1/4-unit lattice, the one-chain structure S1 = fl(C + H),
   S2 = fl(S1 + P) reproduces whole runs bit for bit, double jumps included, where m = n and r ≈ 0.3:
   - (40,40), (36,36), (80,80) side 1, (120,120), (200,200): 106-112 of 112 on each run (`s8_own_a.out`);
   - pw:a (100,100,x0.37), (100,100,x100), (100,50,x100): 224 of 224;
   - (256,64,s2,r0.31): 112/112 (57 on the old lattice). There, a = 2^7 and b = 2^5 make -a·L and -b·L exact, and
     Excel's S1 sits on an exact tie (the constant ≡ 8 mod 16 units). So **Excel's constant is not one double on its
     own grid.** It carries a low part with an exact dyadic phase inside the first stored sum.
   - Positive control: (256,64) reaches 112 with C+Ha!+Hb!+P! and with C+H!+P! once ties are reachable, and both
     chains are algebraically identical there (`s8_chains.py`, inline check in the session).
2. **The series L is right, and so are its inputs.** For L itself, the r form and the log1p variant change nothing
   that helps, on every failing run tested:
   - r formed as fl(mx/n), fl(n/(m x)) in extended, fl(fl(n/m)/x), fl(1/fl(mx/n)), fl(fl(n/x)/m), or from
     y = mx/(n+mx);
   - L as the series (summed in either order, Horner in t², 2t + 2s, t = r·fl(1/(r+2)), 8 or 14 terms), TOMS
     alnrel's rational, the correctly rounded log1p (stored or extended), the exact log1p of r or of the exact ratio,
     or LN(fl(1+r)).

   The stored 14-term series is the best or tied-best everywhere. For example, (70,110,s2,r0.31) scores 77 with
   the series against 61-73 for every other L (`r5lib` chain terms `Hv_*`, `Hr_*`, `Hcr*`, `HxE`, `HxR`, `HxS`, `HS`).
   The H steps in the data are whole L-ulps times h (45 and 50 units on consecutive doubles), so L is a stored
   double.
3. **Every x term is inside the stored sums.** A fraction (1 - α) of H added outside the roundings fails for every
   α ≠ 1 (`s17_alpha.py`: 5-18 of 112 against 69-97 at α = 1). The -ln x part is outside them: a value division,
   or an unstored -LN x. C+H!+P!+X ties the division at 111 of 112, while every chain that stores after -LN x scores
   13-21.
4. **The first stored sum is not fl(K' + H).** At (80,80,s2,r0.31), K' + H = 32.07 lies just above 32, but Excel's
   first sum has a 32-unit grid. The chain D+H!+C+P! (a constant piece D meets H first, |D + H| < 32, then the rest
   of the constant and P in one rounding) gives 112 of 112, against 70 for C+H!+P! (`s12_two.py`, `s13_grid.py`).
   Either Loader piece of r4-f-small-df's constant fits there: A = (h - 1/2)·ln(h/o), or B = the rest.
5. **What still fails:** m ≠ n at every r, and m = n at r ≈ 0.04-0.25 (r4's (300,300,x6) and (160,160,x0.21)
   included).
   - Free-C score: 70-110 of 112 per run, 3,471 of 4,144 on the informative runs of probe a (`s8_own_a.out`). On the
     untouched hold-out it is 877 of 1,056 (83%, `s8_own_hold.out`, structure test only).
   - The misses are whole ±1-quantum plateaus of S1 or S2.
   - The data need an extra x-dependent offset of about ±30-80 units inside the first rounding, varying over 1-10
     cells. No candidate regressor explains it: no rounding error of a stored product c·v (c in {a, b, h, a±1,
     b±1, h-1}; v in {LN x, LN mx, LN r, LN(1+r), L, LN(n+mx), r, mx}), no r rounding, no LN(x) error and no P
     rounding (`s11_regneed.py`, `s14_regs.py`, `s15_corr.py`).
6. **One lead: at m = n and small r, the power term looks like two stored products.** The chain C + H + fl(h·LN x)
   - fl(b·LN x), in extended with one final rounding (on side 2, fl(a·LN x) - fl(h·LN x)), gives:

   | run | pair chain | C+H!+P! |
   |---|---|---|
   | (80,80,s1,r0.06) | **110 of 112** | 78 |
   | (120,120,s2,r0.045) | 107 | 97 |
   | (120,120,s1,r0.29) | 107 | 111 |
   | (150,50,s1,r0.29) | 89 | 72 |
   | (70,110,s2,r0.31) | 58 | 77 |
   | (50,150,s2,r0.045) | 62 | 93 |

   - Over all informative runs of probe a (`s20_own_a.out`), the pair form totals 3,106 of 4,144, against 3,471
     for C+H!+P!. By run class (pair against single):

     | class | pair | single |
     |---|---|---|
     | m = n, r ≈ 0.05 | 105-110 | 78-102 |
     | side 1, m ≠ n, r ≈ 0.29 | 89-100 | 72-92 |
     | m = n, r ≈ 0.3 | 62-108 | 70-112 |
     | side 2, m ≠ n | 41-67 | 69-109 |

     So it is a partial signal tied to side and r, not the answer.
   - The same form with LN r, LN mx or LN R in either product is worse (`s19_pairs.py`, `s20_pairgrid.py`).
   - This matches h·ln x - o·ln x = w·ln x (o = b on side 1, a on side 2), "the x power as a ratio of two powers".
   - It is not established: it fails at m ≠ n, and it loses at m = n, r ≈ 0.3.

## 2. Frozen models (frozen before the hold-out was read; `final_model.py`)
All variants use M-sd's constant K = `sdmodel.K_of(m, n, side)` (r4-f-small-df), H, P, and F = fl(EXP_V6(E)/x).

| variant | E |
|---|---|
| msd | K + P + H, extended (r4's M-sd) |
| dd | fl(fl(fl(K) + H) + P) + (K - fl(K)) |
| xs | fl(fl(K + H) + P) |
| xs1 | fl(K + H + P) |

| set | CR | msd | dd | xs | xs1 |
|---|---|---|---|---|---|
| own:a (7,168, all r <= 0.375, in-sample) | 85/802/2,006 | **512**/2,443/**3,663** | 333/**2,563**/3,607 | 330/2,530/3,330 | 397/2,361/3,321 |
| pw:a r <= 0.375 (3,808) | 52/451/1,095 | 118/926/1,631 | 71/**1,147/1,802** | 120/744/1,813 | **146**/874/1,634 |
| br:big r <= 0.375 (912) | 25/160/409 | 69/366/578 | **89/430**/579 | 53/270/508 | 73/359/**587** |
| **own:hold (1,536, untouched)** | 20/226/459 | **65**/489/851 | 0/241/533 | 9/302/847 | 14/**537/986** |

(`final_insample.out`, `final_hold.out`.)
- Hold-out discipline: `t2586-r5-f-smallr-hold` was built and filled together with `a`, before any analysis. It was
  read once, by `final_model.py own:hold`, after the variants were frozen. Then `s8_chains.py own:hold` was run, as a
  structure test only (free C, no model choice).
- Nothing reaches bit-exactness or visible parity on the hold-out. xs1 is the best within 15 digits there (986 of
  1,536), msd the best bit-exact (65).
- The stored-sum variants fail because the constant's phase inside the sum is not known. M-sd's K is right only to
  about 1-3 units, while the sums round at 32-512 units, so a wrong phase costs a whole quantum on long plateaus.
- dd's 0 bit-exact on the hold-out shows that fl(K) as the phase is wrong. The free-C fits put the constant at exact
  dyadic fractions (finding 1.1), not at fl(K').

## 3. Probes and controls
| case | cells | content |
|---|---|---|
| `t2586-r5-f-smallr-a` | 14,336 | in-sample: 16 shapes ((40,40) (80,80) (120,120) (200,200) (30,90) (90,30) (50,150) (150,50) (24,120) (120,24) (70,110) (110,70) (256,64) (64,256) (36,36) (180,60)) × 4 runs (side 1 r ≈ 0.29, 0.06; side 2 r ≈ 0.31, 0.045) × 112 consecutive doubles |
| `t2586-r5-f-smallr-hold` | 3,072 | **hold-out**: (44,100) (100,44) (66,66) (140,140) (28,56) (56,28) (170,90) (90,170) × 2 runs (side 1 r ≈ 0.17, side 2 r ≈ 0.21) × 96 |

- Total 17,408 cells. A = x by w2-forensics' exact encoder, B = F.DIST(A,d1,d2,FALSE).
- `mkprobe5.rows` asserts that every echo equals x: 0 misreads. Every run stays at r <= 0.375 on one side (checked
  at build time).
- Both cases are filled (excel-graph, 2026-10-24).
- Also used: r4-f-power's `a` (pw:a), r3's `a`, r4-f-brcomp's `big`.
- Positive control for the free-C scan: the (256,64) degenerate shape, where the split and unsplit chains are
  identical. Both reach 112/112 once exact ties are reachable, and only one of them reached it on the old lattice.

## 4. Hypotheses tried (negatives included; free-C score per run of 112 unless stated)
- **H split as -a·L and -b·L, each stored** (C+Ha!+Hb!+P!, C+Hb!+Ha!+P!): 3,037 and 3,120 of 4,144, against 3,471
  for one H (`s8_own_a.out`). Rejected as the general form. It only wins on runs where it acts as a finer first-sum
  grid.
- **H split into the series pieces**: -fl(2h·t) and -fl(2h·s) in any order and storage. At most +3 on any run.
- **H stored** (fl(h·L)): no better than H extended on any set (round 4's 'fl(hL)' agrees).
- **Mixed L:** -w·series - o·LN(fl(1+r)) (both orders), and o·LN(q) or o·(LN(n+mx) - LN(n_o)) for the o part: 44-76
  on the failing runs, against 69-78.
- **First-sum binade** (D+H!+C+P! and D+H!+C!+P!, |D + H| in [0.375, 768], both signs; `s13_own_a.out`): fixes
  (80,80,s2,r0.31) only (112). No binade helps any small-r run.
- **A second constant between H and P** (C+H!+D!+P!, D = ±1.5·2^k, k = 3..10; `s18_mid.py`): no gain (best 75 and
  78 on the two runs tried).
- **Orders** (C+P!+H!, H+P!+C!, P+H!+C!, C+H+P!, C+P+H!): all at or below C+H!+P!.
- **The power term through LN r, LN mx, (a-1)/(b+1) products, or P kept extended** (Pe), in every position:
  below C+H!+P!.
- **P as 2·fl((a/2)·LN x):** 84-92 on (80,80,s1,r0.06). Superseded by the h/o pair (finding 6).

## 5. Open (most promising first)
1. **The constant's exact phase inside the first stored sum.** Excel's first sum lands on exact ties (finding 1),
   so the constant is a hi + lo pair whose lo has dyadic bits that matter.
   - Next: per shape, read the free-C best phase (`r5lib.scan(..., ret=True)` gives d) at r ≈ 0.3, m = n, where
     the structure is exact.
   - Compare it with Loader pieces as stored doubles summed in some order: fl((h - 1/2)·log1p(w/o)),
     fl(1/2·LN(w/2π)), and Excel's stirlerr doubles.
   - That is a 1-D exact problem per shape and would give the K_hi + K_lo that every stored-sum model needs.
2. **m ≠ n and small r.** The h/o pair form (finding 6) fixes one m = n small-r run almost entirely.
   - Test it on m = n runs at r 0.04-0.25 with longer runs.
   - Look for the m ≠ n analogue: the ratio term a·ln(m/n) may enter as a stored product with LN x, for example
     fl(h·LN(m x)) - fl(o·LN x) - fl(h·LN m), on a two-constant scan.
3. Above r = 0.375 the old HKP11 already fits (round 4). Check whether the h/o pair form or the single P holds there
   at m ≠ n, before changing the engine.

## 6. Scripts
| script | what |
|---|---|
| `r5lib.py` | the directory's one library: loaders (`own:`, `pw:`, `r3:`, `br:` sets), `base` (side, r, stored P), run arrays, unit helpers |
| (r5lib, chain part) | chain grammar ('C+H!+P!', '!' = stored), term library (H, P, L variants, split pieces, LN-x products, pairs), free-C scans `scan` (exact 1/4-unit lattice) and `scan2` (two constants), `loader_split` |
| (r5lib, log1p part) | `log1p_var`: variants of Excel's positive-argument log1p (series orders, Horner, TOMS alnrel, term counts) |
| `mkprobe5.py` | the two probes and `rows()` (reuses r4-f-power probe4's builder and reader) |
| `final_model.py` | **frozen variants** msd / dd / xs / xs1 and the scorer |
| `s0_steps.py`, `s16_steps.py`, `s2_blips.py` | step views of ln(Excel·x) - P against H, P and LN changes |
| `s1_scan.py`, `s3_forms.py`, `s5_orders.py`, `s6_runsum.py` | first scans (old non-dyadic lattice; superseded by the r5lib chain scans) |
| `s4_jitter.py`, `s10_need.py`, `s11_regneed.py`, `s14_regs.py`, `s15_corr.py` | needed-shift intervals and regressor searches (negatives) |
| `s7_stepq.py` | step quanta after subtracting candidate power terms |
| `s8_chains.py`, `s9_res.py` | score chains per run, residual sequences |
| `s12_two.py`, `s13_grid.py`, `s18_mid.py` | two-constant chains: Loader split, first-sum binade, a constant between H and P |
| `s17_alpha.py` | how much of H rides inside the roundings |
| `s19_pairs.py`, `s20_pairgrid.py` | the power term as two stored LN products |

Outputs: `s6_own_a.out` (old lattice), `s8_own_a.out`, `s8_own_hold.out`, `s13_own_a.out`, `s20_own_a.out`,
`final_insample.out`, `final_hold.out`.

## 7. For the integrator
- Nothing here replaces M-sd. The best frozen model on this hold-out within 15 digits is `final_model.value(x, d1,
  d2, "xs1")`: E = fl(K + H + P) with M-sd's K. It gets 986 of 1,536, against msd's 851, but only 14 bit-exact
  against msd's 65.
- Its gain is not confirmed on r4-f-power's `a` (1,634 against dd's 1,802 within 15 digits at r <= 0.375). Do not
  integrate it without a second hold-out.
- The reusable pieces are `r5lib.scan` (free constant on an exact lattice) and the term library, for the
  constant-phase work in section 5.1.
