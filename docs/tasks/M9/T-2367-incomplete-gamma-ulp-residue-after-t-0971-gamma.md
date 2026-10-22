---
id: T-2367
title: "Incomplete gamma ulp residue after T-0971: GAMMA.INV F38, POISSON.DIST D16 and the inversion cap shape"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0971]
---

## Goal
The two cells T-0971's accurate incomplete gamma moved by one ulp are Excel's again, and the largest shape an
incomplete-gamma inversion finishes within the 2^19-term cap is measured and recorded.

## Context
T-0971 (`statistical/incomplete_gamma.rs` `incomplete_gamma_pair`) fixed 61 baseline cells and regressed two by one
ulp, under decision 0083's standing ulp allowance:
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV!F38` (GAMMA.INV): Excel 1, ours 0.9999999999999999.
- `ironcalc-calc/STATISTICAL/POISSON.xlsx` `POISSON.DIST!D16`: Excel 0.9863047314016171, ours 0.986304731401617.
The series now runs to 2^-54 instead of statrs' 1E-15 (about 8% more terms near x = a), so an inversion's shared
budget runs out at a smaller shape than statrs' 3.5E8 (T-1872's 3-million-sample sweep): `GAMMA.INV(1E-5,7.9E8,1E-6)`
is now `#NUM!` (530,000 terms over its cdfs), shape 3E8 still finishes. docs/excel-semantics/statistics.md
"Incomplete gamma: accurate kernel (T-0971)" and "Incomplete gamma: per-call cap and lifting (T-1872)".

## Acceptance criteria
- [ ] GAMMA.xlsx F38 and POISSON.xlsx D16 pass; no new baseline lines.
- [ ] T-1872's inversion sweep re-run on the new kernel; the smallest shape past the cap is recorded in the cap
      section (and `MAX_TERMS`'s doc), or a faster large-shape method (Temme's expansion) keeps 3.5E8.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
