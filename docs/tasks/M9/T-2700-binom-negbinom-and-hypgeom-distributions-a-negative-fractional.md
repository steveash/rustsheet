---
id: T-2700
title: "BINOM, NEGBINOM and HYPGEOM distributions: a negative fractional count is #NUM! before truncation (gen-m9, 9 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
BINOM.DIST and BINOMDIST (number_s), NEGBINOM.DIST and NEGBINOMDIST (number_f), and HYPGEOM.DIST and HYPGEOMDIST
(number_sample, population_s) return `#NUM!` for a count in (-1, 0), as Excel does. The sign is checked before the count is
truncated. Today ours truncates first, so -0.5 becomes 0 and passes the check.

## Context
One root cause, cut on 2026-10-25 (gap triage, sonnet refill) from three opus tasks whose remaining parts need probes:
- T-2435 gave the BINOM cells.
- T-2474 gave the NEGBINOM cells.
- T-2429 gave the HYPGEOM cells.

Rules and evidence in docs/excel-semantics/statistics.md:
- "Scalar distributions at extreme arguments (T-2255)": a slightly negative number_s is `#NUM!`.
- "Scalar distributions at extreme arguments, part 2 (T-2256)": NEGBINOMDIST's number_f in (-1, 0) is `#NUM!` before
  truncation.
- "HYPGEOMDIST: a sample below the possible range is 0; a negative or huge count is #NUM! (T-2255)": any negative count,
  however small, is `#NUM!`.

HYPGEOM.DIST's sample_s already follows this rule (probes-numeric/t1049-hypgeomdist A34 `HYPGEOM.DIST(-0.5,4,8,20,TRUE)`,
the `sample_s < 0.0` check in hypegeom.rs). This task applies the same order to the other counts.

Code, all in crates/rustsheet-engine/src/functions/statistical/:
- binom.rs `binom_dist_of`: `number_s` is truncated before `number_s < 0.0`.
- binom.rs `negbinom_dist_of`: `number_f` is truncated before `number_f < 0.0`.
- hypegeom.rs `hyp_geom_dist_of`: `number_sample` and `population_s` are truncated before the `valid` check.

Check the sign on the untruncated value, then truncate. A negative number_pop and NEGBINOM's number_s already fail
after truncation, so they need no change.

T-2429 (2026-10-25) landed the HYPGEOM part: hypegeom.rs now checks every count's sign before truncation (and a count of 2^31 - 1 or more), so the three HYPGEOM cells below (7674c89c824aca43 A386, 589f42e67f4b4b8d A2, cde551ea787f713a A72) pass and left the baseline. What is left is BINOM and NEGBINOM.

Excel cells (gen-m9, excel-graph, 2026-10-22/23; known failures in eval/baseline/gen-m9.txt). The resolved count is from
statistics.md or the case's Data sheet:

| cell | formula (count as resolved) | Excel | ours |
|---|---|---|---|
| gen-m9/85f48d5cdfb24555 Calc!A35 | `BINOM.DIST((Data!E5%*2.5),10,1E-16,..)` (number_s -0.0125) | #NUM! | 0.999999999999999 |
| gen-m9/caa1eed48b1883fe Calc!A384 | `BINOMDIST((-0.0001),9,0.999999,Data!B2)` | #NUM! | 1.0000000002588124E-54 |
| gen-m9/7bd219ec0c0c5e22 Calc!A320 | `BINOMDIST((-0.0001),6,..,..)` | #NUM! | 1 |
| gen-m9/d208bbafc3bc1997 Calc!A231 | `NEGBINOMDIST(Data!E12,8,0.5)` (number_f -0.5) | #NUM! | 0.00390625 |
| gen-m9/3b7990bd4e3307ba Calc!A312 | `NEGBINOM.DIST(BETA.INV(..)%,3,0.975,TRUE)` (number_f -1E-06) | #NUM! | 0.9268593749999999 |
| gen-m9/1db18890916ee643 Calc!A124 | `NEGBINOMDIST((AVERAGEA((-0.5))+((-0.5)^Data!$C11)),9,1E-300)` (C11 42, number_f -0.5+2^-42) | #NUM! | 0 |
| gen-m9/7674c89c824aca43 Calc!A386 | `HYPGEOMDIST(3,(Data!E6+Data!$B$3)%,5,9)` (number_sample -0.025) | #NUM! | 0 |
| gen-m9/589f42e67f4b4b8d Calc!A2 | `HYPGEOMDIST(5,1,QUARTILE(SUMPRODUCT((-0.0001),0.5,1e-15),4),2)` (population_s -5E-20) | #NUM! | 0 |
| gen-m9/cde551ea787f713a Calc!A72 | `HYPGEOM.DIST(1,(-0.0001),CHIDIST(0.1,9),..,"FALSE")` | #NUM! | 0 |

Cells:
- gen-m9/1db18890916ee643.case.json (1): Calc!A124
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A312
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A2
- gen-m9/7674c89c824aca43.case.json (1): Calc!A386
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A320
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A35
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A384
- gen-m9/cde551ea787f713a.case.json (1): Calc!A72
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A231

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 9 known-failure lines (6 now; the 3 HYPGEOM lines went with T-2429) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline. In particular the probes-numeric t1048/t1049
      binomdist, negbinomdist and hypgeomdist cells keep passing.
- [x] Unit tests: `BINOM.DIST(-0.0125,10,1E-16,FALSE)`, `NEGBINOMDIST(-0.5,8,0.5)` and `HYPGEOMDIST(3,-0.025,5,9)` are
      `#NUM!`. A positive fraction still truncates: the passing probe cell `BINOM.DIST(2.9,3.9,0.5,FALSE)` keeps
      passing.
- [ ] Each of the three statistics.md entries gets an "Ours:" note naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST.RANGE, BINOM.INV, CRITBINOM and PERMUTATIONA. Their counts are not in this evidence. PERMUTATIONA does the
  opposite and truncates before the sign check (T-2701).
- NEGBINOMDIST's underflowing probability (0, not `#NUM!`) and the other cells of T-2474.
- HYPGEOM's impossible sample (T-2699) and huge counts (T-2429).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type (none is a last-digit difference), so the task is in the
campaign at the `semantics` stage. Its target is that every cell it owns has Excel's type and error; the visible-parity
criterion under Acceptance criteria states it.

## Outcome
`binom_dist_of` (number_s) and `negbinom_dist_of` (number_f) now check the sign of the untruncated count, so a count in
(-1, 0) is `#NUM!` (binom.rs). HYPGEOM was already done by T-2429. Eval delta: 7 known-failure lines left
eval/baseline/gen-m9.txt, no new ones. One bonus cell left the baseline: b9a32ce9a1fd0d71 A195 (`NEGBINOMDIST(Data!$D$6,7,..)`, D6 = -0.5, Excel `#NUM!`). The rules were already in statistics.md (T-2255, T-2256); their "Ours:" notes now name T-2700.
