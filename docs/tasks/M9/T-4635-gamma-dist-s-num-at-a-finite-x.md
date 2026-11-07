---
id: T-4635
title: "GAMMA.DIST's #NUM! at a finite x / beta of 1E308 at shapes 1.5 and 2: bracket it and match"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine gives Excel's `#NUM!` where GAMMA.DIST (both forms) is `#NUM!` at a huge finite x / beta, by a rule that
oracle evidence brackets, and the Gate A definition gives those points a value or a domain error by that rule.

## Context
- T-4625 found that `GAMMA.DIST(1E300,a,1E-8,..)` (x / beta = 1E308, finite) is `#NUM!` for the cdf and the density
  at shapes 1.5 and 2, and a number (1 / 0) at shapes 0.5, 1, 2.5, 2.9, 3, 10 and 1E5; at shape 2 it is a number
  from x / beta = 1E200 down. Neither "y^a overflows" (2.5 is fine) nor "a * y overflows" (1.5 fails) fits.
- Off cells (the engine gives 1 and 0): probes/t4625-gamma-dist-ratio-overflow B9, F9, B10, B11.
- statistics.md, "GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)" (the T-4625 paragraph); the Gate A
  definition gives no value there (tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py `HUGE_RATIO_*`).
- Engine: crates/rustsheet-engine/src/functions/statistical/gamma.rs `gamma_dist_of`.

- Probe 1 (2026-11-07, excel-graph). Predictions were committed before the fill (d17d413e8). The case and oracle are
  on branch research/t4635-probe-data at 3a5d849c6 (`eval/suites/probes/cases/
  t4635-gamma-dist-huge-ratio.*`, `docs/research/t4635`). They were not landed in eval: their 106 `#NUM!` cells would
  be new known failures that no rule found so far can pass. The cells are GAMMA.DIST(x, a, 1E-8, ..) at
  x / beta = 1E216, 1E258, 1E298, 1E306, 1E307, 1E308, 1.5E308 and 1.7E308.
  - The cdf and the density are `#NUM!` in exactly the same cells.
  - Shape 1 is never `#NUM!`.
  - 1.01 is `#NUM!` from 1E306, and 1.1 from 1E298 (not monotone in the shape).
  - 1.25 to 2.1 (1.25, 1.4, 1.5, 1.6, 1.75, 1.9, 1.99, 2, 2.01, 2.1) are `#NUM!` from 1E308.
  - 2.25 to 2.51 (2.25, 2.4, 2.45, 2.49, 2.5, 2.51) are `#NUM!` from 1.5E308.
  - Every other cell is a number (cdf 1, density 0).
  - CHISQ.DIST and CHIDIST at df 2 to 5 and x 1E200 and 1E290 are numbers. Excel for the web refuses a workbook
    holding x literals of 1E307 or 1.7E308.
  - The hypothesis "`#NUM!` exactly for 1 < a < 2.5 at 1E308" fails: 1.01 and 1.1 cut far lower, and 2.25 to 2.51
    also fail from 1.5E308. Neither y^a, a·y nor y/(a - 1) overflowing fits the cuts. A per-shape threshold table
    would be a fit, so the next step is to find the expression that overflows (Excel's incomplete gamma at a huge
    x: series against continued fraction, its prefactor, or lnΓ terms), and to bisect the 1.01 and 1.1 cuts.

## Acceptance criteria
- [x] A probe brackets the region: shapes between 1 and 2.5 (e.g. 1.1, 1.25, 1.75, 2.25, 2.4) and x / beta between
      1E200 and 1.7E308 at shape 2 and 1.5, both forms; the rule is recorded in statistics.md.
- [x] The bracket also finds the ratio threshold at shape 1.5 and at the other shapes listed (the 1E200 bound of
      `HUGE_RATIO_MIN` is evidenced only at shape 2), and covers CHISQ.DIST and CHIDIST at df 3 and 4 (shapes 1.5 and
      2 at x / 2), which may share the `#NUM!` if it comes from the shared incomplete gamma (T-4625 excel-fidelity
      review, PR #3672).
- [x] The engine matches every cell of probes/t4625-gamma-dist-ratio-overflow and the new probe (B9, F9, B10, B11
      included), and the Gate A definition's `HUGE_RATIO_*` no-value region narrows to what the rule cannot cover.
- [x] `make check` green; reviewers approve.

## Out of scope
- The overflowing x / beta rule (T-4625; the engine already matches it).
- The overflowing x / beta rule's low end: below shape 0.01 only the cdf is evidenced (GAMMADIST/s1 A182); a density
  cell there would settle it, but that is not this task's.

## Progress
- 2026-11-07: resumed by an opus run (branch idle 4 h, no PR); first fill in, second bracket probe next.
- 2026-11-07: PR #3896 adopted as an orphan (idle 4 h after its landing notes, which this commit replaces); round 2's
  open excel-fidelity findings (R4.1 hash hold-out, R4.2 Gate A batch) are answered in the Outcome.

## Outcome
- Four probes (excel-graph 2026-11-07; generators, predictions committed before each fill and results in
  docs/research/t4635): t4635-gamma-dist-huge-ratio (320 cells), -cut (768), -fresh (104, drawn after the rule froze)
  and t4635-chisq-huge-x (38). Rule R (statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"): above shape 1 a
  finite y = x / beta is `#NUM!` where y / a * 4 overflows, and to shape 1.1 also where y^a does; at shape 1 and below
  never. The cut is on y (beta 1E-8, 1E-100, 1E-200 agree), both forms, and CHISQ.DIST, CHISQ.DIST.RT and CHIDIST
  share it at x / 2 (only df 3 reaches it at a finite x).
- Engine: gamma.rs `huge_ratio_num_error`, called by `gamma_dist_of` and chisq.rs's cdf/density and right tail.
  Every cell of the four probes and of probes/t4625-gamma-dist-ratio-overflow (B9, F9, B10, B11 included) passes.
- Gate A: `_huge_ratio_domain` replaces the `HUGE_RATIO_MIN` / `HUGE_RATIO_SHAPES` no-value box with rule R (GAMMA.DIST,
  CHISQ.DIST, and a new CHISQ.DIST.RT domain); no value only in the unprobed edges, shapes in (1.1, 1.11) where y^a
  overflows and shapes below 1.01 where only y / a * 4 does. Sidecars refreshed for the changed definitions.
- R4 guards (decision 0188). The 4 / a quotient is a structural model chosen from ranked candidates, so it owes the
  hold-out and the fresh sweep; `HUGE_RATIO_POWER_SHAPE_MAX` = 1.1 is a cut inside its evidence bracket [1.1, 1.11),
  tagged FITTED (fitted lock 96 -> 97, decision 0339). The first fill shaped the candidate list.
  - R4.1 hold-out: the fit lab's hash split (`rustsheet-fitlab huge-ratio-gamma|huge-ratio-chisq|huge-ratio-chisq-rt
    --holdout-only`, fitlab/kernels/gamma_huge_ratio.py; default 5-point margin) over every cell of the five probes at
    a finite y from 1E200 (GAMMA.DIST/GAMMADIST 1,198 cells, CHISQ.DIST 22, CHISQ.DIST.RT/CHIDIST 22; the outcome is
    `#NUM!` or 1 / 0, so the bit-exact, 4-ulp and 15-digit counts are equal). Train / test exact:

    | candidate | gamma | chisq | chisq-rt |
    |---|---|---|---|
    | 4/a (rule R) | 845/845, 353/353 | 13/13, 9/9 | 17/17, 5/5 |
    | 4/a no power (cut 1.1 against no power branch) | 811/845, 335/353 | 13/13, 9/9 | 17/17, 5/5 |
    | exp(k(4-a)), k = ln 2 / 2 | 811/845, 342/353 | 12/13, 8/9 | 16/17, 4/5 (refused) |
    | a/(a-1) | 801/845, 338/353 | 12/13, 8/9 | 15/17, 5/5 |
    | 4-a | 790/845, 329/353 | 12/13, 8/9 | 16/17, 4/5 (refused) |
    | correctly rounded (no `#NUM!` at a finite y) | 555/845, 244/353 | 12/13, 6/9 | 16/17, 2/5 |

    Rule R is the only candidate exact on both sides of every fit; the power branch's 52 GAMMA.DIST cells (shapes
    1.01 to 1.1) are what "no power" misses (the chi-squared cells have no shape in that range).
  - R4.2 fresh sweep: Gate A batch GAMMA.DIST/r4-T-4635, 200 points, 100% log-uniform over the whole domain
    (`generate --stratum log-uniform`, SPEC-0029), drawn and committed after the freeze, before its excel-graph fill
    (2026-11-07): 200 of 200 in the envelope (164 numbers, 36 domain errors: 18 an overflowing x / beta, 18 a density
    at a subnormal x / beta). As a full-domain sweep it lands no point in rule R's region (y near 1E307 is a
    vanishing share of the log-uniform domain); it checks that the change moves nothing elsewhere.
  - Rule R's region gets an investigation batch, GAMMA.DIST/e-T-4635 (120 points, `--stratum aimed`, the definition's
    `_huge_ratio_point`: y log-uniform within 10% of the cut on either side at shapes 1.01 to 1.1 and 1.11 to 4, and
    shapes 0.3 to 1 and 4 to 10 from y 1E300 up; beta 1E-100 to 1E-8, so x stays below 1E301), also drawn after the
    freeze and committed before its fill: 120 of 120 as predicted (39 rule R `#NUM!`, 4 overflowing x / beta, 77
    numbers). `rustsheet-eval accuracy`: 45,043 points, 0 failing, every point of both batches in the envelope.
  - Known gap: the chi-squared functions get no batch. Their x = 2y would be a literal of 1E307 or more, which Excel
    for the web refuses (first fill); they share GAMMA.DIST's kernel at y = x / 2, and probes/t4635-chisq-huge-x
    (x written as a product) covers them. T-4877 holds the gap.
  - The earlier probe ladder stays: the first fill (320 cells), the gen3 sweep after the freeze (104/104: 84 cells at
    0.97 and 1.03 of the cut, 4 at shapes 0.3 and 0.7, and the Cut sheet's 16, shapes 1.11 to 1.14, all numbers where
    y^a overflows).
- Follow-up: T-4877 (a probe below shape 1.01, wording of the sweep counts; filed in #3912).
- Eval delta: +1,230 passing probe cells in the four new cases; probes/t4625-gamma-dist-ratio-overflow 58 -> 62 of 62.
