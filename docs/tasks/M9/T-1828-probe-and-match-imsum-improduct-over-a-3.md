---
id: T-1828
title: "Probe and match IMSUM/IMPRODUCT over a 3-D reference, a real next to a j number, and no numbers at all"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSUM and IMPRODUCT give Excel's result for the argument forms T-1384 left unprobed.

## Context
T-1384 folds IMSUM's and IMPRODUCT's arguments whole (`fold_complex` in
functions/engineering/complex.rs; docs/excel-semantics/engineering.md "IMSUM and IMPRODUCT take their
arguments whole"). Three forms have no oracle evidence, and ours are guesses:
- A 3-D reference, `IMSUM(Sheet1:Sheet2!A1:A2)`: ours `#VALUE!`, as a union is (probed: `IMSUM((M1,M6))`).
- A real number next to a `j` number, `IMSUM("3+4j","3")` or `IMSUM("3+4j",M2)`: ours `#VALUE!`, because a
  real number parses as an `i` number. IMSUB and IMDIV (`complex_pair_of`) have the same rule.
- A call whose arguments hold no number, `IMSUM(Q1:Q3)`, `IMPRODUCT(Q1)`, `IMSUM(,)`: ours "0".

Probe them in one case with the excel-probe skill, record the answers in engineering.md and match them.

## Acceptance criteria
- [x] The probe also pins the boolean refusal T-1384 inferred for unprobed arguments: IMDIV's first,
  IMAGINARY, IMARGUMENT, IMCOSH, IMCOT, IMCSC, IMCSCH, IMEXP, IMLN, IMLOG10, IMLOG2, IMSEC, IMSECH,
  IMSIN, IMSINH, IMTAN, and COMPLEX's `i_num` (`COMPLEX(1,TRUE)`, a TRUE cell); engineering.md
  drops its "Unprobed" line for whatever passes.
- [x] The new probe case passes, with no new known failures in eval/baseline/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Mixed suffixes in IMSUB and IMDIV (`#NUM!` in Excel, T-1766).

## Outcome
Probe `probes/eng-imsum-improduct-3d-j-empty` (61 cells, excel-graph 2026-10-20) answers the open forms and
the boolean refusals (all `#VALUE!`, as inferred). Findings in docs/excel-semantics/engineering.md:
3-D reference `#VALUE!` (already matched); a suffixless real number takes the other operand's suffix in
IMSUM, IMPRODUCT, IMSUB and IMDIV (new `Suffix::Real`); `IMSUM(,)`/`IMPRODUCT(,)` are `#N/A` (was "0").
Eval delta: probes +1 clean case, 61 cells, no new known failures.
