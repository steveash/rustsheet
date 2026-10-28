---
id: T-2787
title: "Probe IMCOT inputs where the two denominator groupings round differently"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence confirming that IMCOT's denominator is grouped `(cosh 2y + 1) - 2cos²x`, as T-2537 fitted, and not
`cosh 2y - (2cos²x - 1)` (T-2044).

## Context
- T-2537 fitted the grouping on 64 cells, the 63 IMCOT cells of probes-numeric im-last-digit-sweep plus
  `IMCOT("-1.5+0.5i")` (gen-m9/060b18586fcff6e6 Calc!A72). Only that one input tells the two groupings apart
  (docs/excel-semantics/engineering.md, IMCOT entry).
- Find 10 or more inputs where the two groupings print different 15-digit text: a scratch scan over random x+iy with
  `trig_sin_cos` and `f64::cosh` (complex/trig.rs `imcot`). Write them as a probes-numeric case and fill it with
  `cd tools && uv run rustsheet-oracle fill`.

## Acceptance criteria
- [ ] The probe has oracle values for at least 10 such inputs. engineering.md records how many each grouping
      matches. If the T-2537 grouping is not the better fit, a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMCOT of a real number (TAN's kernel).
