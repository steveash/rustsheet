# T-2506: the engine's L140 bup+bpser on r2-bratio's scoring sets

`score_l140.py` runs every row of r2-bratio's `score_lib.load` sets through the engine's K(a, b, x, y) (closed forms,
then the ported branches, else today's kernel) with the research harness op `BRATIO`
(`functions/statistical/t2464_harness.rs`), and scores the rows bratio's dispatch sends to L140 bup+bpser.
Scores are bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15); CR is the correctly
rounded value. Output: `score_l140.out`.

## Data
`score_lib.load` reads derived files that are not committed. Rebuild them from the repo root:
```
cd tools && uv run python ../docs/research/t2464/extract.py t2464-w2f w2-forensics/cells_w2f.tsv && cd ..
cd docs/research/t2464/w2-forensics && python3 exact_w2f.py && cd ../w4-fractions && python3 prep.py
cd ../w5-triangulation && T2464_HARNESS=<engine test binary> python3 build_evidence.py
```
(python3 with mpmath and numpy). The w4p1 and x29 sets have no L140 bup+bpser row under the engine's dispatch.

## Before T-2506's resume (branch at 2b2dfae1)
| set | n | engine ex/u4/w15 | CR ex/u4/w15 |
|---|---|---|---|
| be | 123 | 78/104/110 | 48/93/103 |
| w2f | 10,236 | 7,716/10,229/10,236 | 3,293/9,077/10,191 |
| ix | 1,383 | 828/1,202/1,257 | 615/1,209/1,279 |
| all | 11,742 | 8,622/11,535/11,603 | 3,956/10,379/11,573 |

548 of the cells (45 be, 503 ix) were not ported: at a whole b0 (b0' = 1) L140 calls bpser(a0, 1, x0), and with
a0 > 8 bpser had no ln B form for both shapes >= 1, so they fell back to today's kernel (139 of them visibly off).

## ln B(a0, 1) in bpser
| form of L at min(a, b) = 1 | all ex/u4/w15 | ix ex |
|---|---|---|
| none (today's kernel for a0 > 8) | 8,622/11,535/11,603 | 828 |
| -algdiv(1, a0) for a0 > 8 (TOMS betaln with GL(1) = 0) | 8,794/11,696/11,721 | 989 |
| fl(GL(1) + algdiv(1, a0)) for a0 > 8 | 8,794/11,696/11,721 | 989 |
| **-LN(a0) for a0 > 8, GAMMALN's sum below** (kept) | **8,800/11,696/11,721** | **995** |
| -LN(a0) at every a0 | 8,872/11,697/11,721 | 1,025 |

-LN(a0) is B(1, b) = 1/b, the form the density takes at a shape of 1 (statistics.md "Beta density for
min(a,b) <= 1"). Taking it at a0 <= 8 too moves no cell of beta_exact.tsv's BETA.DIST cells off bit-exact and adds
m9-beta-dist-a5 A21 and t1745-beta-inv-roundtrip G178, G181, but the gated suite ironcalc-calc regresses:
BETA.DIST(0.5,2,2,TRUE) (BETADIST_BETAINV.xlsx G15, Excel 0.4999999999999999) goes from 0.49999999999999994 (1 ulp,
GAMMALN's sum) to 0.5 (2 ulp), and the BETA.INV chained on it (H15, Excel 0.7) from bit-exact to 0.6999999999999997.
So a0 <= 8 keeps GAMMALN's sum; the 72 cells are left for whoever finds a form that fits both.
(T-2847 correction: the workbook's G15 x is (0.7-0.2)/(1.2-0.2) = 0.49999999999999994, not the literal 0.5 measured
here; at that x -LN(a0) gives Excel's G15 bit-exact, and H15's 2 ulp is BETA.INV's own. T-2847 takes -LN(a0) at every
a0.)

## Against the acceptance criteria
- Criterion 1 (>= 7,505 bit-exact and >= 11,713 w15 of 11,742): 8,800 and 11,721.
- Criterion 2 (>= 1,011 bit-exact of the model's 1,620 ix L140 cells): on the rebuilt ix set the ported dispatch puts
  1,383 rows on L140 bup+bpser (the difference from the model's 1,620 is not traced); 995 of them are bit-exact
  (72%, against the model's 62%).
- Still off by more than 15 digits: 21 cells, all ix (F.DIST, F.DIST.RT, FDIST, NEGBINOM.DIST mapped arguments).
