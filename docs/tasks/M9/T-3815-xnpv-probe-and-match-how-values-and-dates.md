---
id: T-3815
title: "XNPV: probe and match how values and dates elements are read (fractional, negative and out-of-range dates, text and booleans in ranges and arrays, shapes) and which error wins"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XNPV gives Excel's value or error for every kind of element in its values and dates (and for its rate), read from a
range, an array constant, a computed array or a single cell, so the element reading that follows the evidenced size
checks is evidenced too, with the order in which its errors win.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, theme 3: "XIRR and XNPV coerce
their date and value arguments differently from Excel, including which error comes out"). It names a class of input,
never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe
skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel
across the class, record that in the Outcome and close the task.

Gap triage 2026-11-02 (report of `make eval` on 2026-11-02, excel-graph evidence): every visibly-off XIRR/XNPV cell
already has an owner, so no task covers the class as a sweep; this is an evidence gap. What is evidenced is in
docs/excel-semantics/financial.md: "XIRR reads empty cells as 0 and accepts earlier dates; XNPV is #NUM! on empty
cells", "XNPV reads one empty values cell as a single 0 cash flow", "XIRR and XNPV under the gen-m9 statistical cells:
size and type checks (T-2255)" (the size checks and the argument-level error order, from gen-m9 cells, much of it
inferred through an enclosing call) and "XNPV discounts with Excel's `^`".

Not swept for XNPV (each as a range element, an array-constant element, a computed-array element and a one-cell
argument): a fractional date (truncated? before or after the "precedes the first date" check: a later date of
first + 0.5), a negative date, date 0, a date past 2958465, a later date equal to or before the first, unsorted later
dates; numeric text, date text ("1/1/2025"), non-numeric text, TRUE/FALSE and an empty cell in the middle of a
multi-cell values range and of a dates range (ours: `#NUM!` for a non-number value, `#VALUE!` for a non-number date);
a first value of 0; all values 0; an error element in values against one in dates (which wins, and against a bad
date earlier in the list); a rate of -1, below -1, numeric text, an empty cell; a values row against a dates column
and 2-D ranges of the same count. The open, already-owned points stay with their tasks: T-3312 (XNPV element order,
1b60bf99d71424e7 A171 and 5ded7594e597e31e A12: start from its findings if it lands first), T-2603 (all-empty
multi-cell values `#VALUE!`), T-2118 (lifted rate), T-3126 and T-3184 (argument-kind coverage cells).
Code: `fn_xnpv`, `xnpv_numbers` and `xnpv_size_error` in crates/rustsheet-engine/src/functions/financial/mod.rs,
`xnpv_dates` and `whole_day_dates` there, `compute_xnpv` in financial_util.rs. XIRR's reader (T-3816) is a sibling:
where both functions agree, fix the shared helper, not each copy (directive 3).

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) covers the element kinds and shapes above.
- [ ] The engine matches every probed cell; cells owned by T-3312, T-2603 or T-2118 are left to them and named.
- [ ] financial.md gains an XNPV element-reading entry (rule, probe cells, VERIFIED) in the same PR, and the "Excel docs
      claim ... #VALUE!" comment in `xnpv_dates` is replaced by the evidenced rule.
- [ ] `make check` green; eval known failures only shrink (baseline updated, no new lines); reviewers approve.

## Out of scope
- XIRR (T-3816), the solver and its digits, and XNPV's `^` (evidenced).
