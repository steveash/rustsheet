---
id: T-3816
title: "XIRR: probe and match how values and dates elements are read (fractional, negative and out-of-range dates, text and booleans in ranges and arrays, shapes) and which error wins"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
XIRR gives Excel's value or error for every kind of element in its values and dates, read from a range, an array
constant, a computed array or a single cell, so the element reading that follows the evidenced size checks is
evidenced too, with the order in which its errors win.

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

Not swept for XIRR (each as a range element, an array-constant element, a computed-array element and a one-cell
argument; guess omitted): a fractional date (truncated before the "no later date" check? `{-1000,1100}` on
`{45000,45000.5}`), a negative date, a date past 2958465, date text ("1/1/2025") in a cell and in an array, numeric
text in an array constant against the same in a cell (cells read as numbers since T-2045's probe, xirr-back-dated-stop
A31; owned by T-3267), non-numeric text and booleans in a range against in an array, an empty cell in the middle of a
dates range (read as date 0), all values of one sign, a values row against a dates column, 2-D ranges of the same
count, and an error element in values against one in dates (which wins, and against a negative date earlier in the
list). The open, already-owned points stay with their tasks: T-3312 (the 18 gen-m9 cells, numeric text inside a
values range at 31474c20fc1dafd6 A351, the text and boolean guess order), T-3267 (numeric-text cells), T-2118 (lifted
guess), T-3287 and T-3484 (the solver's stops), T-3126 and T-3184 (argument-kind coverage cells).
Code: `fn_xirr`, `xirr_numbers` and `xirr_size_error` in crates/rustsheet-engine/src/functions/financial/mod.rs,
`xirr_dates` and `whole_day_dates` there, `compute_xirr` in financial_util.rs. XNPV's reader (T-3815) is a sibling:
where both functions agree, fix the shared helper, not each copy (directive 3).

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) covers the element kinds and shapes above.
- [ ] The engine matches every probed cell; cells owned by T-3312, T-3267 or T-2118 are left to them and named.
- [ ] financial.md gains an XIRR element-reading entry (rule, probe cells, VERIFIED) in the same PR.
- [ ] `make check` green; eval known failures only shrink (baseline updated, no new lines); reviewers approve.

## Out of scope
- XNPV (T-3815); the guess argument (T-3312); the solver's root, stops and digits (T-3287, T-3484).
