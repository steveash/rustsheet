---
id: T-2055
title: "POISSON.DIST: fit Excel's cdf and pmf (ironcalc-calc POISSON.xlsx and probes-numeric/t1048-poisson together)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
POISSON.DIST and POISSON match Excel on ironcalc-calc STATISTICAL/POISSON.xlsx without losing any excel-graph cell of
probes-numeric/t1048-poisson. The even-df CHISQ.DIST.RT identity of T-1778 must keep holding.

## Context
T-0927 tried two Poisson models; docs/excel-semantics/statistics.md "Discrete distributions: within an ulp of the exact
sum" has the detail.
- The exact value rounded matches 6 of 11 small ironcalc cdfs.
- `fl(fl(S)*exp(-mean))`, with S the partial exponential series, matches 8 of 11 small ironcalc cdfs and all 5 pmfs.
  It breaks 8 t1048-poisson cells that statrs gets right, for example `POISSON.DIST(10,4,FALSE)`.
- Routed through the identity `POISSON.DIST(k,m,TRUE)` = `CHISQ.DIST.RT(2m,2(k+1))`, that model also breaks
  m9-legacy-chidist A15/A16, where Excel's 1.0 became 1.0000000000000002.
POISSON.xlsx keeps 32 baseline lines: D31 and D55 have counts past 1,024, and the rest are near-ulp cells.
`functions/statistical/discrete.rs` has the double-double walk and the scaled exponent to reuse; the Poisson walk from
T-0927 is in that PR's history.
The likely root is Excel's incomplete gamma (T-0971, T-1778 rank 7).
Approach: fit against every Excel-computed POISSON, POISSON.DIST, CHISQ.DIST.RT and CHIDIST cell at once: ironcalc,
t1048-poisson, m9-legacy-chidist, m9-chisq-dist-rt and t1049-chisqdistrt. Compare cell by cell with main, ungated
suites included.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): POISSON and POISSON.DIST values off Excel's, pmf and cdf (`POISSON(9,100,FALSE)` Excel 1.025153212086874E-31, ours 1.0251532120868825E-31; `POISSON.DIST(100,100,TRUE)` 0.5265621985299984 against 0.5265621985300686). A negative mean is T-2474's.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 133 known-failure lines in eval/baseline/gen-m9.txt, 50 visibly off.
Examples:
- gen-m9/a2262ea466fcc142 Calc!A227 `=POISSON(8,0.5,0)`: Excel 5.876141839043822e-08, ours 5.8761418390438375e-08
- gen-m9/d37313406b9f6e55 Calc!A315 `=POISSON.DIST(2,2,2)`: Excel 0.6766764161830634, ours 0.6766764161830646
Cells:
- gen-m9/0072265d385d8ab2.case.json (2): Calc!A61, A185
- gen-m9/097af1ca33638670.case.json (3): Calc!A152, A172, A214
- gen-m9/09b47629f54b9817.case.json (4): Calc!A7, A224, A349, A376
- gen-m9/0bbab6206b5c9a07.case.json (3): Calc!A91, A95, A339
- gen-m9/11eb3ab4f4f8104c.case.json (1): Calc!A252
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A65, A69
- gen-m9/17c110a53924beb8.case.json (2): Calc!A147, A235
- gen-m9/1b31092a0d79e5f1.case.json (2): Calc!A110, A358
- gen-m9/1db18890916ee643.case.json (3): Calc!A12, A64, A216
- gen-m9/201d67f1d59436ed.case.json (2): Calc!A49, A173
- gen-m9/2138b909d8f810ea.case.json (4): Calc!A62, A176, A214, A366
- gen-m9/31f09e6c48ca245c.case.json (4): Calc!A127, A188, A279, A355
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A99, A306
- gen-m9/334f3dc9ca937f10.case.json (4): Calc!A93, A133, A195, A257
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A335
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A192
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A185
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A108
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A65, A183
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A240
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A394
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A19
- gen-m9/583ca2baea0bb30a.case.json (3): Calc!A27, A151, A399
- gen-m9/5919660b7bb4d195.case.json (5): Calc!A50, A80, A240, A316, A392
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A28, A362
- gen-m9/775d053de498b8c7.case.json (1): Calc!A118
- gen-m9/787ba8d801105a60.case.json (2): Calc!A83, A393
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A289
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A298
- gen-m9/7fd0a98183c6babf.case.json (3): Calc!A31, A69, A266
- gen-m9/873f30d2bf76f8db.case.json (5): Calc!A10, A44, A48, A124, A162
- gen-m9/91e167cd8d7eb652.case.json (2): Calc!A57, A384
- gen-m9/928d08e3e1eecb5f.case.json (4): Calc!A173, A235, A266, A328
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A186
- gen-m9/941c409532603d32.case.json (2): Calc!A63, A125
- gen-m9/9735c5ab72fa5ecf.case.json (3): Calc!A10, A48, A328
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A137
- gen-m9/9e2225ead8a6a07b.case.json (2): Calc!A20, A116
- gen-m9/a2262ea466fcc142.case.json (2): Calc!A202, A227
- gen-m9/a4090567758520df.case.json (5): Calc!A6, A122, A192, A246, A378
- gen-m9/a5342a7e24fd803e.case.json (4): Calc!A47, A49, A275, A351
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A268
- gen-m9/afc00c8b70e78baf.case.json (2): Calc!A193, A304
- gen-m9/b9a32ce9a1fd0d71.case.json (4): Calc!A69, A107, A183, A356
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A79
- gen-m9/c2684066d8df263e.case.json (1): Calc!A77
- gen-m9/c4635963fc926284.case.json (3): Calc!A49, A125, A216
- gen-m9/c7664e494e418395.case.json (2): Calc!A294, A370
- gen-m9/cb61169f4c033d4b.case.json (3): Calc!A29, A91, A153
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A62
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A169
- gen-m9/d37313406b9f6e55.case.json (5): Calc!A235, A277, A311, A315, A349
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A30, A366
- gen-m9/d8f44d556674c2fd.case.json (3): Calc!A112, A153, A343
- gen-m9/eccd03df42e40006.case.json (3): Calc!A3, A34, A155
- gen-m9/ede0323c389fc703.case.json (3): Calc!A67, A158, A371
- gen-m9/f82473275380882e.case.json (2): Calc!A151, A213
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/3c06f9298f65a3ba.case.json: Calc!A91

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 133 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline. Partly met: every
      owned pmf cell is within 15 digits and no baseline line was added; the deep-upper-tail cdf cells remain, carried
      by T-3241.
- [x] Fewer POISSON.xlsx baseline lines; no Excel-computed cell in the suites above goes from pass to fail.
- [x] The per-call cost stays bounded, and the lifting weight covers it.
- [ ] `make check` green; reviewers approve. `make check` is green; review round 1: security and quality approved,
      architecture and fidelity requested changes, addressed in round 2; this box is ticked once that round approves.

## Out of scope
- BINOM.DIST and NEGBINOM.DIST (T-0927).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): it now owns 50 visibly-off gen-m9 cells (Context), so the task is in the
campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria
still unmet then move to a follow-up task without the key.

## Stage (decision 0114)
`campaign: visible` (retagged from `accuracy` in review round 1). The `accuracy` premise, that Excel is close to correctly
rounded here so the fix is an accurate kernel never fitted to Excel, is false for the pmf: Excel is up to 91 eps off the
exact value, and the correctly rounded pmf (60-digit Python Decimal) is visibly off at 51 of the 337 t2055 probe cells
and 19 of the 150 gated pmf cells. The pmf is Excel's own algorithm, reproduced, which is decision 0114's `visible`
stage under decision 0056's time-box. The original text was: "`campaign: accuracy`: a 50-digit reference shows Excel is
close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an
mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a
`defer: phase-1b` follow-up."

## Outcome
POISSON.DIST's and POISSON's pmf are now Excel's two closed forms (`statistical/poisson.rs` `excel_poisson_pmf`,
statistics.md "POISSON.DIST pmf: two closed forms"), replacing statrs' `exp(k ln mean - mean - ln k!)`:
- mean <= 1.1 or k <= 1: EXP(fl(k LN(mean)) - GAMMALN(k+1), held in a register) * EXP(-mean);
- otherwise Loader's saddle point with the exponent stored, on `loader.rs` `excel_bd0` (from T-2507) and
  `stirlerr_exact`, with a POISSON-only fitted proxy for stirlerr at k = 2..5 (`POISSON_STIRLERR_2_TO_5`,
  `TODO(T-3247)`); k = 6 takes the exact value (review round 1: its offset did not hold up).
The switch, which lies in [1.1, 1.100001), and the shape of both forms come from two new excel-graph probes,
probes-numeric/t2055-poisson-pmf-sweep (204 cells) and t2055-poisson-pmf-switch (133 cells). The cdf is unchanged
(`gamma_ur`): `incomplete_gamma` is untouched (T-2638 and T-3060 are unaffected), and the even-df CHISQ.DIST.RT identity
of T-1778 still holds. A huge k over a tiny mean, where `excel_bd0` is not finite, is #NUM!.
Decision 0114's premise for POISSON (Excel close to correctly rounded: build an accurate kernel, do not fit) is false
for the pmf. Excel is up to 91 eps off the exact value, and the correctly rounded pmf is visibly off at 51 of the 337
probe cells and 19 of the 150 gated pmf cells, against 5 and 0 for these forms. So this task and T-3247 are now
`campaign: visible`, and T-3241 too (in the deep upper tail Excel is the inaccurate side).

Main-vs-HEAD, every suite, per cell. Main is d2d4a59e3 (the merge parent); main and HEAD were built in separate target
dirs, and both evaluate the two t2055 probes:
- fixed (fail -> pass): 309 (gen-m9 49, ironcalc-calc 7, probes-numeric 253);
- lost bit-exact (pass -> fail): 6, all 1 ulp, all in the new ungated probe t2055-poisson-pmf-sweep at the closed form's
  small-k edge: `POISSON.DIST(1,0.3,FALSE)` A139 +1, `(3,0.9)` A153 +1, `(3,1.0)` A154 +1, `(3,1.1)` A156 +1,
  `(5,0.9)` A159 +1, `(5,1.05)` A161 +1. None is in a gated suite (T-3247);
- newly visibly off: 0; type or error changes: 0;
- visibly fixed: 128 (101 cells now pass, 27 still fail but are within 15 digits).
Top-level POISSON / POISSON.DIST cells in the suites that existed before this task: 140 failing and 42 visibly off on
main, against 79 and 11 now. All 11 are cdfs: the deep upper tail (mean well above k, e.g. `POISSON(2,100,TRUE)`)
and ironcalc POISSON.xlsx D55 (k = 46040). No pmf cell is visibly off there.
Fit scores (exact / within 4 ulp / visibly off): the 121 gated pmf cells with resolvable arguments 106/121/0 (statrs
47/80/33); t2055-poisson-pmf-sweep 154/198/5; t2055-poisson-pmf-switch 127/133/0.
Baselines: gen-m9 -49 lines, ironcalc-calc -7 (POISSON.xlsx 15 -> 8), none added (`make eval-baseline`; round 2 moved no
gated cell).

Criteria:
- Visible parity: partly met. Every owned pmf cell is within 15 digits; the deep-upper-tail cdf cells remain, carried
  by T-3241.
- Fewer POISSON.xlsx baseline lines (15 -> 8), and no Excel-computed gated cell goes from pass to fail: met.
- Per-call cost: both forms are O(1) apart from `excel_bd0`'s capped series; the cdf, which the lifting weight charges,
  is unchanged: met.
- `make check` green: met. Review: round 1 asked for changes; round 2 addresses them.
Follow-ups: T-3241 (the cdf's deep upper tail, on the shared incomplete-gamma kernel, `campaign: visible`) and T-3247
(`defer: phase-1b`, `campaign: visible`: the pmf's bit-level residue and the real cause behind the stirlerr proxy).
