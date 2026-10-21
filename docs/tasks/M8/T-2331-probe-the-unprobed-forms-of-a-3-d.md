---
id: T-2331
title: "Probe the unprobed forms of a 3-D span of a cached external workbook"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence (excel-graph) for the forms of a 3-D span into a cached linked workbook that T-1654 left UNVERIFIED, recorded in docs/excel-semantics/references.md "Lookups, names and spans over the cache".

## Context
T-1654 made `[1]Uses:Defines!…` read the cached sheets (fixtures/external-link-cached-3d-span Refs!B2:B10). Its engine choices without evidence: a span with an end the cache does not hold (`SUM([1]Uses:Nope!$B$2)`, kept `#BLOCKED!`; a span of this workbook with a missing end is `#NAME?`); blanks for readers that keep them (`TEXTJOIN(",",FALSE,[1]Uses:Defines!$A$1:$B$2)`, `CONCAT`, skipped here); a one-sheet span `[1]Uses:Uses!$A$1` as a value and in ROWS (read as a span here, `#REF!`/`#VALUE!`); `@[1]Uses:Defines!$A$1`; and the reversed span `[1]Defines:Uses!…` in COUNT, COUNTA, MAX. Code: crates/rustsheet-engine/src/external_links.rs `cached_span`, `cached_span_cells`. Build a sibling fixture of fixtures/external-link-cached-3d-span (same externalLink1.xml, raw XML; see eval/suites/fixtures/README.md) and oracle-fill it.

## Acceptance criteria
- [ ] A fixtures case with each form above, oracle-filled (excel-graph); its known failures recorded with `make eval-baseline`.
- [ ] references.md states each form's Excel result with its cell, replacing the UNVERIFIED list; an engine follow-up task is filed for each mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (follow-up tasks).
