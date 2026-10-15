---
id: T-1609
title: "Probe cases for external workbook references read from cached externalLink values"
milestone: M8
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Probe cases record how Excel evaluates formulas that reference another workbook
(`[Book2.xlsx]Sheet1!A1`) when only the cached externalLink values are available.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- M8 scope: "External workbook references (read cached values)". Done work already covers part of
  it; read those cases first and probe only what they leave open:
  - T-0124: an unresolvable reference is `#VALUE!`;
  - T-1381: an uncached reference is a reference;
  - T-1382: `[n]Sheet!ref` reads the externalLink cache;
  - T-1454: external references through INDEX, OFFSET, union and spill;
  - T-1526, T-1550: aggregates over cached external ranges.
- Build the xlsx with an `xl/externalLinks/externalLink1.xml` part holding cached cell values,
  extending spec2xlsx if needed. Check what excel-graph does on upload: whether it keeps,
  refreshes or drops the cache. Record the result whatever it is.

## Acceptance criteria
- [ ] ≥ 10 new probe cases cover what those tasks did not: external defined names, 3-D external
  spans, lookups over external ranges (VLOOKUP, XLOOKUP, MATCH), and partially cached ranges. They are oracle-filled or, if
  excel-graph cannot evaluate them, recorded as such with the reason.
- [ ] Findings in docs/excel-semantics/references.md; failing cells have filed tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Opening or updating the linked workbook.

## Outcome
Two excel-graph fixtures, 34 probe cells (raw-XML externalLink cases are fixtures, one workbook per
upload-risk group): fixtures/external-link-cached-lookups (24 cells: lookups over partly cached
ranges, name and sheet case, uncached names and sheets, name scopes, all pass in ours) and
fixtures/external-link-cached-3d-span (10 cells: 3-D spans of cached sheets; 9 fail, ours does not
parse the span, filed as T-1654). Excel for the web kept the cache on upload. Findings in
references.md "Cached values of a linked workbook". Eval: fixtures +34 cells, 25 pass, 9 new known
failures (T-1654); 0 regressions.
