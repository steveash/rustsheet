---
id: T-1014
title: "Export rich-value errors as #VALUE! plus vm metadata if Excel rejects stored literals (SPEC-0003 X2)"
milestone: M1
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1011]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Workbooks we save hold `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` in the form Excel writes them: `t="e"` with `<v>#VALUE!</v>` and a `vm` index into `xl/metadata.xml` and `xl/richData/*`. T-0999 showed that Excel for the web rejects a workbook storing `#BLOCKED!`, `#CONNECT!`, `#FIELD!` or `#CALC!` as a literal (syntax.md), so this is required. It also fixes today's `#CALC!` export.

## Context
SPEC-0003 "xlsx" (rollout X2). The format is in arrays.md "Cached rich errors" (metadataTypes, futureMetadata `XLRICHVALUE`, `rvb`, `rdrichvalue.xml`, the `_error` structure with `errorType`/`subType`). The reader is `crates/rustsheet-xlsx/src/import/rich_errors.rs`.

## Acceptance criteria
- [ ] The exporter writes the rich-value parts. A round trip through our importer keeps every kind, and an oracle recalc of an exported workbook loads with the expected ERROR.TYPE codes (excel-probe skill).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New error kinds.
