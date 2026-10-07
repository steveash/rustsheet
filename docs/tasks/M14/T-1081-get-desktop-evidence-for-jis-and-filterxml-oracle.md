---
id: T-1081
title: "Get desktop evidence for JIS and FILTERXML (oracle: none)"
milestone: M14
lane: functions-text
status: blocked
priority: P3
size: S
tier: opus
depends_on: [T-0040, T-1074]
---

## Goal
JIS and FILTERXML, the two in-scope catalog functions Excel for the web cannot evaluate, have
desktop-Excel evidence, so they can get oracle-backed cases.

## Context
T-0040 probed every in-scope catalog function on excel-graph. Two are marked `oracle: none` in
eval/catalog/functions.yaml: `JIS` is `#NAME?` on the web (probes/catalog/availability-22 A8, and as
`_xlfn.JIS` in a scratch probe), and `FILTERXML("<a>1</a>","//a")` is `#VALUE!`
(probes/catalog/function-name-forms A7) for valid XML and XPath; its page says it relies on
Windows. See
docs/architecture/oracle.md, "Functions Excel for the web lacks". Decision 0039: desktop-only
questions are M14 work and never block Phase 1.

## Acceptance criteria
- [ ] An excel-desktop run evaluates the two cells above (and a few argument variants) and the
  results are recorded in docs/excel-semantics/ (text.md for JIS; FILTERXML wherever web functions live).
- [ ] The catalog drops `oracle: none` for each function that gets desktop evidence, or says why not.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing JIS or FILTERXML.

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit.
Unblock when `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` and T-1074 is done.
