---
id: T-0921
title: "One decoder for value-metadata (vm) errors shared by importer and eval snapshot"
milestone: M9
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: [T-0913]
---

## Goal
`rustsheet-xlsx` importer (`import/worksheets.rs`) guesses `vm="1"`->#CALC!, `vm="2"`->#SPILL!, while
`rustsheet-eval` `read_rich_errors` decodes vm -> metadata.xml -> rdrichvalue errorType (13/8). Keep one
definition (in rustsheet-xlsx, exposed to eval), and resolve through futureMetadata `rvb i` rather than
using `rc v` as the rv index directly.

## Acceptance criteria
- [x] Single decoder used by importer and eval reader; importer no longer hardcodes vm numbers.
- [x] Unit test for the pure decoding function with inline metadata.xml/rdrichvalue.xml.
- [x] `make check` green; no eval regressions.

## Outcome
`rustsheet_xlsx::import::rich_errors` is the one decoder: `vm` → `valueMetadata` block →
`rc t` (XLRICHVALUE in `metadataTypes`) / `rc v` → `futureMetadata` `rvb i` → `rdrichvalue` `errorType`
(by key name). The importer uses it instead of hardcoding `vm="1"`/`"2"` (an error the engine has no
kind for stays `#VALUE!`; broken rich data leaves the cached value); the eval snapshot reader calls it
and keeps mapping unresolved entries to `#UNDECODED!`. Unit tests for the pure decoder (inline XML)
and an import test on ERROR.TYPE.xlsx. Eval: 0 regressions, 0 fixes in every suite (all evidence
workbooks have `rc v` = `rvb i`).
