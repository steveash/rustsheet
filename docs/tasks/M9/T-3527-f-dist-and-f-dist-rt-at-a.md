---
id: T-3527
title: "F.DIST, F.DIST.RT and BETA.DIST at a tiny x or a huge second df or shape: the bit-level residue within 15 digits that T-3479 leaves out (t3354 and t1538 probes, 304 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3479]
defer: phase-1b
---

## Goal
The F.DIST, F.DIST.RT and BETA.DIST cells of T-3354's two cases and of the T-1538 probe that are already within half a
unit of Excel's 15th significant digit, but more than 4 ulp from Excel's double, take Excel's double. Cells
within 4 ulp are not targets unless the same model fixes them.

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main 73241cb4c, excel-graph; `scripts/exit_owners.py`'s `owned` rule
  over every failing cell and the task files of origin/main and every open task-filing branch): 304 cells, none
  visibly off, that no open task names. T-3479 owns the visibly-off cells of the same two t3354 cases and says
  "Out of scope: bit parity on cells already within 15 digits", so these are its residue. T-1538 and T-1864 (huge
  df) are done; T-2221 owns only t1538 A15 and leaves df of 1E7 and above out.
- Same arguments as T-3479: d1 1 to 90 against d2 1,000 to 1E10 at x from 1E-15 to 1E-3, plus T-3354's random sweep.
  Examples, Excel vs ours:
  - t3354-f-dist-tiny-x-huge-d2 B1 `F.DIST(1E-15,1,1000,FALSE)` 12612509.089180425 vs 12612509.089180406 (density at
    d1 = 1, the T-3383 "F at a df of 1" item)
  - t3354-f-dist-tiny-x-huge-d2 A100 `F.DIST(2.5E-12,4,1000,TRUE)` 1.2524999999958078E-23 vs 1.252499999995809E-23
  - t3354-f-dist-sweep A1 `F.DIST(1.74516065918274E-13,1,8822326,TRUE)` 3.3331722407164116E-07 vs 3.333172240716409E-07
  - t3354-f-dist-sweep C197 `F.DIST.RT(501.668789941444,3,1857)` 1.0171732746367757E-238 vs 1.0171732746367762E-238
  - t1538-beta-dist-at-excel-inv A1 `BETA.DIST(4.351460182692426e-10,2.5,5E9,TRUE)` 0.49999999906766107 vs
    0.4999999990676606; A2 `F.DIST(0.8702920369171896,5,1E10,TRUE)` 0.4999999990676617 vs 0.49999999906766135
- Suspected root cause: the incomplete beta at a huge second shape (bratio's bgrat/basym orientation with b0 in the
  millions, the x = d1·X/(d1·X + d2) mapping where x is tiny) and the F density prefactor (T-3364's extrapolated
  `STIRLING_ERROR` past df 56, T-2642's exponent at a huge df, T-2647 where density·x is subnormal).
- Code: `fn_f_dist` in crates/rustsheet-engine/src/functions/statistical/fisher.rs, f_density.rs, bratio/.
- Scorer: docs/research/t3354/score.py.
- Cells (ulp counts are against Excel's double):
  - probes/t3354-f-dist-tiny-x-huge-d2.case.json (205; 94 beyond 4 ulp; median 4, max 26 ulp): S!A3, A6, A8:A10, A12:A14, A16:A18, A20:A22, A24:A26, A28:A30, A32:A34, A36:A38, A40:A42, A44:A46, A48, A50, A57, A62, A64, A68:A70, A82, A90, A98:A111, A113:A135, A140, A143:A144, A147, A150, A152, A167, A174:A176, A183:A184, A192, B1:B5, B8:B14, B16:B23, B25:B49, B53, B56, B63:B64, B66, B71:B73, B75, B79, B81, B84:B85, B87:B89, B91, B96, B100:B104, B108:B109, B111:B112, B116:B117, B119:B120, B123:B125, B127:B128, B132, B136, B138, B140, B142, B144, B146:B147, B149, B151:B152, B159:B161, B165:B169, B173:B174, B176, B178, B181, B184, B187, B190, B192, B200, B240, C40, C48
  - probes-numeric/t3354-f-dist-sweep.case.json (90; 42 beyond 4 ulp; median 4, max 38 ulp): S!A1, A5, A18, A22, A25, A36, A39, A42:A44, A51, A53, A61, A63:A64, A67:A69, A71:A74, A83, A86:A88, A90, A94, A100, A104, A111:A114, A120, A122:A124, A130, A132, A158, A190, B2:B3, B5:B6, B14, B17, B20, B28, B34, B37, B39, B43:B44, B47, B51:B52, B56, B58:B59, B61:B64, B67, B71, B73, B75, B83:B84, B91:B92, B97, B99:B100, B104:B105, B108, B112, B117:B118, B122, B124, B133, B138, B159, B168, B182, C197
  - probes-numeric/t1538-beta-dist-at-excel-inv.case.json (9; 4 beyond 4 ulp; median 4, max 8 ulp): Calc!A1:A3, A5:A6, A13:A14, A21:A22

## Acceptance criteria
- [ ] Of the 140 listed cells beyond 4 ulp, each is within 4 ulp of Excel's double, or the Outcome lists it with Excel,
      new and 50-digit values and the reason (for example Excel itself more than 4 ulp off the math where no
      structural model reproduces it).
- [ ] None of the 304 listed cells and none of T-3479's cells moves from within 15 digits to visibly off; the
      probes known-failure lines for these cells only disappear from eval/baseline/probes.txt (`make eval-baseline`),
      with no new line in any baseline.
- [ ] Any fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome:
      hold-out counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, chosen candidate against the
      correctly rounded one, margin stated before scoring); a fresh log-uniform sweep oracle-filled after the freeze;
      the Gate A envelope result on that sweep with every point outside listed; each fitted piece tagged
      `FITTED(T-3527, n_cells, range)` and applied only inside its range; no step fits differences below the
      reproducibility floor (4 ulp web against desktop, docs/research/oracle-calibration.md); no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of the same cases (T-3479).
- Cells within 4 ulp that the chosen model does not reach (below the reproducibility floor).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
