---
id: T-1014
title: "Export rich-value errors as #VALUE! plus vm metadata if Excel rejects stored literals (SPEC-0003 X2)"
milestone: M1
lane: harness
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-0999, T-1011]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Workbooks we save hold `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` in the form Excel writes them: `t="e"` with `<v>#VALUE!</v>` and a `vm` index into `xl/metadata.xml` and `xl/richData/*`. **Do this only if T-0999 shows that Excel does not read the stored literal form.** If Excel does read it, close this task as not needed, citing T-0999.

## Context
SPEC-0003 "xlsx" (rollout X2). The format is in arrays.md "Cached rich errors" (metadataTypes, futureMetadata `XLRICHVALUE`, `rvb`, `rdrichvalue.xml`, the `_error` structure with `errorType`/`subType`). The reader is `crates/rustsheet-xlsx/src/import/rich_errors.rs`.

## Acceptance criteria
- [ ] Pick up only after T-0999. If T-0999 shows that Excel loads stored literals unchanged, set `status: done` with an Outcome citing it, and make no code change.
- [ ] Otherwise: the exporter writes the rich-value parts. A round trip through our importer keeps every kind, and an oracle recalc of an exported workbook loads with the expected ERROR.TYPE codes (excel-probe skill).
- [ ] `make check` green; reviewers approve.

## Out of scope
- New error kinds.
