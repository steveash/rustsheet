---
id: T-1081
title: "Confirm JIS and FILTERXML web results on desktop Excel"
milestone: M14
lane: functions-text
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-0040, T-1074]
---

## Goal
The web results for JIS (`#NAME?`) and FILTERXML (`#VALUE!`), goldens under decision 0039, are
confirmed or contradicted by desktop Excel.

## Context
T-0040 probed every in-scope catalog function on excel-graph. Two behave unlike their
documentation there: `JIS` is `#NAME?` (probes/catalog/availability-22 A8; function-name-forms-2
A1/A2 with and without `_xlfn.`), and `FILTERXML("<a>1</a>","//a")` is `#VALUE!`
(probes/catalog/function-name-forms A7) for valid XML and XPath; its page says it relies on
Windows. See
docs/architecture/oracle.md, "JIS unknown, FILTERXML failing on en-US Excel for the web" (and docs/excel-semantics text.md, web.md). Decision 0039: desktop-only
questions are M14 work and never block Phase 1.

## Acceptance criteria
- [ ] An excel-desktop run evaluates the two cells above (and a few argument variants) and the
  results are recorded in docs/excel-semantics/ (text.md for JIS; FILTERXML wherever web functions live).
- [ ] Where desktop differs, the excel-semantics entries become CONFLICTING (decision 0039 rule 4)
  and the catalog comments on JIS/FILTERXML cite the desktop evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing JIS or FILTERXML.

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit.
Unblock when `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` and T-1074 is done.
