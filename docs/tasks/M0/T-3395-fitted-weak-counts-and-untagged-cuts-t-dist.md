---
id: T-3395
title: "FITTED weak counts and untagged cuts: t_dist p cuts, f_density LARGE_DF_MIN and LOG1P_SERIES_LIMIT, RATE secant and IRR model"
milestone: M0
lane: quality
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Every `FITTED` tag's cell count and range describe the cells that actually sit near its cut, and the cuts the T-3361
review flagged are either tagged or documented as not fits (decision 0188 R4.3).

## Context
Left over from T-3385, which settled the five `range unknown` tags and `EXPM1_FORM_LIMIT` (method: temporary
`eprintln!` at the branch, a full `rustsheet-eval run`, the logged inputs reduced to ranges; revert before committing).
Still open from the T-3385 task context (crates/rustsheet-engine/src/functions/):
- statistical/t_dist.rs `T_TAIL_P`, `T_STATRS_MIN_P`: the cited probe t1531-t-tiny-p covers p 1E-300..1E-10, which does
  not reach the cuts at 1E-5 and 1E-8; find the cells either side or say none bracket them. `HUGE_SHAPE_MIN` counts only
  the 1E9 side; `T_CENTRE_C` lists only the c = 0.1 cells.
- statistical/f_density.rs `LARGE_DF_MIN` (16, r5-f-bigconst `BIG_MIN`): fitted cut or probe-design edge (r5-f-bigconst
  section 4.5)? Tag it if fitted. Trace the 5752 / 4232 counts on the Stirling constants to a named file
  (docs/research/t2464/r4-f-small-df gives both max(d) <= 10 and max(d) <= 30). `LOG1P_SERIES_LIMIT` (0.375): tag or
  record why it is not a fit.
- financial/financial_util.rs RATE secant tolerances and the IRR model (T-2005, T-1525, T-1533): tag or record why not.

## Acceptance criteria
- [ ] Each item above is tagged with a range read from the cells, or its doc comment says why it is not a fit;
      quality/fitted.json updated with `--update` if the count changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any fitted value (T-3364, T-3365).
