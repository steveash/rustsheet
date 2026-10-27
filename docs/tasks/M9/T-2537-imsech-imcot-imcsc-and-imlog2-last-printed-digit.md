---
id: T-2537
title: "IMSECH, IMCOT, IMCSC and IMLOG2 last printed digit in the gen-m9 engineering sweep, after T-2044 (154 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2044, T-2535]
campaign: visible
---

## Goal
After T-2044's kernel fit, the gen-m9 sweep's IMSECH, IMCOT, IMCSC and IMLOG2 cells print Excel's last digit.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. T-2044 (in progress on origin/agent/functions-engineering/T-2044-im-residue-fit) fits these kernels on the T-2043 sweep (probes-numeric `im-last-digit-sweep`); these cells are new excel-graph evidence on 7 inputs: `IMSECH("3+4i")` with `" 3+4i"`, `"3 + 4i"`, `"3+4j"` (57 cells, the input of T-2044's own gen-m9 cell), `IMSECH("0.1")` 0.995020748953227 (ours ...226), `IMCOT("-1.5+0.5i")` -0.0557109884865415-0.463942860183828i (ours ...416; 45), `IMCSC("-1.5+0.5i")` -0.888092201609111-0.029103639571751i (ours ...112; 43), `IMCSC("1000+1i")` imaginary -0.320079291513745 (ours ...744), and IMLOG2 of 100, "420.31", 0.02, -0.0001 and "1000+1i" (7). Candidates checked by the triage in a scratch script (UNVERIFIED; docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"): IMLOG2's real part ln|z| / 0.693147180559945 (ln 2 to 15 digits) matches all 5 real parts and COMPLEXs.xlsx AH19, the exact ln 2 1 of 5; its imaginary part ATAN2 / LN(2) matches both (ours multiplies by log2(e)); IMCSC = conj(s)/|s|² of sin z from SIN, COS and the exp-based sinh/cosh (T-2535) matches both IMCSC inputs; IMSECH("3+4i") is reproduced by every simple candidate, so ours computes it some other way; IMCOT("-1.5+0.5i") is not the double-angle form with Excel's kernels. Visibly off: each is a wrong text (the last printed digit).

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 154 known-failure lines in eval/baseline/gen-m9.txt, 14 formulas, 154 visibly off.
Examples:
- gen-m9/01810fc89c5ec2c0 Calc!A192 `=IMCSC("-1.5+0.5i")`: Excel "-0.888092201609111-0.029103639571751i", ours "-0.888092201609112-0.029103639571751i"
- gen-m9/01810fc89c5ec2c0 Calc!A218 `=IMLOG2("1000+1i")`: Excel "9.96578500600925+0.00144269455999091i", ours "9.96578500600925+0.0014426945599909i"
- gen-m9/060b18586fcff6e6 Calc!A72 `=IMCOT("-1.5+0.5i")`: Excel "-0.0557109884865415-0.463942860183828i", ours "-0.0557109884865416-0.463942860183828i"
- gen-m9/060b18586fcff6e6 Calc!A236 `=IMSECH("3+4i")`: Excel "-0.065294027857947+0.0752249603027732i", ours "-0.0652940278579471+0.0752249603027732i"
- gen-m9/060b18586fcff6e6 Calc!A377 `=IMSECH("3+4j")`: Excel "-0.065294027857947+0.0752249603027732j", ours "-0.0652940278579471+0.0752249603027732j"
- gen-m9/1068c44afcd56d53 Calc!A292 `=IMSECH("3 + 4i")`: Excel "-0.065294027857947+0.0752249603027732i", ours "-0.0652940278579471+0.0752249603027732i"
Cells:
- gen-m9/01810fc89c5ec2c0.case.json (2): Calc!A192, A218
- gen-m9/060b18586fcff6e6.case.json (5): Calc!A72, A92, A213, A236, A377
- gen-m9/09e6a8028f5acfae.case.json (2): Calc!A47, A341
- gen-m9/1068c44afcd56d53.case.json (2): Calc!A194, A292
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A61
- gen-m9/2b0324e2de2e7b74.case.json (4): Calc!A48, A290, A337, A397
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A148
- gen-m9/3cf48801617a72ad.case.json (2): Calc!A85, A285
- gen-m9/408aa11aa923a4d9.case.json (2): Calc!A212, A259
- gen-m9/40fc68fccfaf9b76.case.json (2): Calc!A167, A182
- gen-m9/493bb194c8e0b577.case.json (3): Calc!A1, A245, A339
- gen-m9/4e180f54ff140abc.case.json (7): Calc!A11, A105, A107, A152, A154, A387, A391
- gen-m9/4fffa5addab1f8af.case.json (3): Calc!A174, A177, A268
- gen-m9/516488da78a6b08c.case.json (2): Calc!A271, A365
- gen-m9/59a16b5ab566ee5c.case.json (3): Calc!A115, A229, A350
- gen-m9/5b6cc12a94f7bf14.case.json (6): Calc!A6, A53, A170, A183, A279, A358
- gen-m9/5bdef392d16a5efb.case.json (2): Calc!A193, A381
- gen-m9/62c886e21a1042af.case.json (1): Calc!A233
- gen-m9/65dc4e4da669f98a.case.json (3): Calc!A12, A94, A188
- gen-m9/65ff9a692665eba9.case.json (1): Calc!A259
- gen-m9/6d16afd40600302b.case.json (3): Calc!A184, A307, A368
- gen-m9/7096c4cd615d43ac.case.json (4): Calc!A183, A260, A277, A371
- gen-m9/74779cff2810ccf4.case.json (5): Calc!A47, A96, A105, A145, A170
- gen-m9/756a9f7bc3a06bcc.case.json (1): Calc!A123
- gen-m9/76551d6fbafc7007.case.json (1): Calc!A74
- gen-m9/76b07b57099ccc8d.case.json (5): Calc!A95, A161, A363, A382, A396
- gen-m9/7820ebc42c7a56a6.case.json (2): Calc!A313, A382
- gen-m9/79dac8dde81afda9.case.json (1): Calc!A205
- gen-m9/7a819ffbbeff8fcc.case.json (1): Calc!A93
- gen-m9/7c7e5bff12d93365.case.json (4): Calc!A69, A128, A207, A304
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A256
- gen-m9/831f987e63e2412c.case.json (3): Calc!A107, A154, A309
- gen-m9/895a33aa8352864a.case.json (1): Calc!A277
- gen-m9/8c018ae063ac131b.case.json (3): Calc!A57, A117, A364
- gen-m9/8cbfd5e99d2e82da.case.json (6): Calc!A44, A85, A112, A138, A226, A273
- gen-m9/8fa8b572ebc1f001.case.json (6): Calc!A34, A42, A83, A132, A238, A255
- gen-m9/9fa170a9f17624fd.case.json (2): Calc!A42, A385
- gen-m9/a1b4181c04804efc.case.json (2): Calc!A45, A89
- gen-m9/a333e1febf9e93cd.case.json (4): Calc!A51, A72, A198, A247
- gen-m9/a7fda34502ed3030.case.json (2): Calc!A5, A126
- gen-m9/b5aef3645cec1b95.case.json (2): Calc!A58, A308
- gen-m9/bf979a17ab5aafcb.case.json (3): Calc!A45, A233, A392
- gen-m9/c2d778463fe008ed.case.json (2): Calc!A36, A190
- gen-m9/ca182d96dedee546.case.json (2): Calc!A193, A355
- gen-m9/cb80ca09be0e12bc.case.json (4): Calc!A124, A222, A274, A369
- gen-m9/d9d55972b50e55e8.case.json (4): Calc!A39, A53, A102, A199
- gen-m9/df083d85b436b49a.case.json (5): Calc!A31, A140, A172, A216, A219
- gen-m9/e707d43a891ba9b5.case.json (3): Calc!A21, A345, A364
- gen-m9/e85daedce5c2675e.case.json (2): Calc!A257, A398
- gen-m9/e882060cf4613302.case.json (1): Calc!A54
- gen-m9/ef33be4fd2389f3c.case.json (5): Calc!A54, A62, A148, A201, A391
- gen-m9/f280df8ba95f96b9.case.json (6): Calc!A9, A197, A208, A267, A302, A349
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A154
- gen-m9/fb73d8163ad87fc6.case.json (3): Calc!A184, A241, A380

Update (T-2231): IMLOG2's 7 cells (01810fc89c5ec2c0 A218, 3cf48801617a72ad A285, 4fffa5addab1f8af A177,
5b6cc12a94f7bf14 A183, 76b07b57099ccc8d A363, 7c7e5bff12d93365 A207, cb80ca09be0e12bc A274) now pass. They
left the baseline in T-2231, so 147 lines remain. IMLOG2 is not the ln 2 to 15 digits candidate above: the kernel
is in engineering.md.

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 154 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Or each line left is listed in engineering.md with T-2044's best candidate (time-box, decision 0056).
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
By this run, T-2044, T-2743, T-2750 and T-2231 had already fixed 109 of the 154 lines. The 45 left were all
`IMCOT("-1.5+0.5i")`, whose real part Excel prints as ...5415 where ours printed ...5416. IMCOT's denominator is now
`(cosh 2y + 1) - 2cos²x` (complex/trig.rs `imcot`). A scratch fit of 64 variants (8 denominator groupings, two numerators, libm or exp-based cosh and sinh) over
probes-numeric im-last-digit-sweep's 63 IMCOT cells plus this one found that only this grouping fits all 64.
T-2044's `cosh 2y - (2cos²x - 1)` fits the 63 but not this cell, and the libm or exp-based cosh makes no
difference (engineering.md). Unit test: `im_reciprocal_kernels_match_the_sweep`. Eval delta: gen-m9 known failures
4056 → 4011 (fixed 45), 0 regressions in any suite; none of the 154 lines remain.

Follow-up: T-2787 (probe more IMCOT inputs where the two groupings differ).
