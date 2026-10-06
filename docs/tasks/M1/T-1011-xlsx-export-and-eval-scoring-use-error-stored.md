---
id: T-1011
title: "xlsx export and eval scoring use Error::stored_literal; import keeps rich-value kinds (SPEC-0003 X1)"
milestone: M1
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1010]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Error cells are saved and scored the way Excel stores them: `#GETTING_DATA` as `#N/A`, and the rich kinds by name. Cached `#BLOCKED!`, `#CONNECT!` and `#FIELD!` import as those kinds instead of `#VALUE!`.

## Context
SPEC-0003 "Design", sections "xlsx" and "Harness" (rollout X1). Evidence: probes/error-literals-new S!A1 stores `#N/A` (syntax.md "Newer error literals").
Code: `crates/rustsheet-xlsx/src/export/worksheets.rs` (the three `t="e"` writers), `import/worksheets.rs` (the rich-error guard), and the error comparison in `crates/rustsheet-eval`.

## Acceptance criteria
- [ ] The exporter writes every error cell with `Error::stored_literal()`. A round-trip test saves all four kinds, and `#GETTING_DATA` reads back as `#N/A`.
- [ ] The eval harness compares engine errors through `Error::stored_literal()`. No second mapping exists, as checked with `rg`.
- [ ] The importer reads the committed rich-errors-* oracle workbooks as `#BLOCKED!`/`#CONNECT!`/`#FIELD!`, with tests. The "stay `#VALUE!`" wording in arrays.md and in `rich_errors.rs` now says "unmapped errorTypes".
- [ ] The remaining `probes/error-literals-new` and `-new2` cells pass and are recorded with `make eval-baseline`, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing `vm` rich-value metadata (T-1014).
