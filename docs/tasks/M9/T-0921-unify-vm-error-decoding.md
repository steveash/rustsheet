---
id: T-0921
title: "One decoder for value-metadata (vm) errors shared by importer and eval snapshot"
milestone: M9
lane: harness
status: in-progress
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
- [ ] Single decoder used by importer and eval reader; importer no longer hardcodes vm numbers.
- [ ] Unit test for the pure decoding function with inline metadata.xml/rdrichvalue.xml.
- [ ] `make check` green; no eval regressions.
