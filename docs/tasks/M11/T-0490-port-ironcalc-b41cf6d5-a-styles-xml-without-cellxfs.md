---
id: T-0490
title: "Port ironcalc b41cf6d5: a styles.xml without cellXfs (or with empty fonts, fills, borders) loads with default styles"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An xlsx whose styles.xml has no `<cellXfs>` section, or has no `<fonts>`, `<fills>` or `<borders>`
entries, loads, and every cell's style resolves (to the default style where the list was missing),
as long as Excel opens such a file.

## Context
Upstream IronCalc b41cf6d5 ("Don't panic when optional style lists are missing from styles.xml",
2026-09-30, reached upstream `main` after 141a41d0) makes every style list optional in
`xlsx/src/import/styles.rs`: a missing list becomes empty and an empty `fonts`, `fills`, `borders`,
`cellStyleXfs`, `cellXfs` or `cellStyles` list takes the default model's list. Its test
`xlsx/tests/missing_style_lists_import.rs` (adjusted by upstream 7e9304a4) removes each list in turn
and then all of them, and checks that every cell's style resolves.

Ours already reads `fonts`, `fills`, `borders`, `cellStyleXfs` and `cellStyles` as optional
(`optional_section_children` in `crates/rustsheet-xlsx/src/import/styles.rs`, T-0118), but still
refuses a file without `<cellXfs>` (`required_section`, a typed `XlsxError::Xml`, no panic), and an
empty list is kept empty, so a cell's `s` index may point at nothing. Directive 5 is already met;
this task is about loading what Excel loads.

Evidence first (directive 1): before changing what we refuse, build the six workbooks the upstream
test builds (one list missing each) plus the all-missing one and confirm with the oracle that Excel
opens them (`excel-probe` skill). If Excel refuses a variant, keep refusing it and record that in
docs/excel-semantics/xlsx-storage.md. Triaged by the 2026-10-27 18:47 UTC steward run.

## Acceptance criteria
- [ ] Each variant Excel opens loads in RustSheet and every cell's style resolves; each variant Excel
      refuses is refused with a typed error. Port upstream's test as a unit test
      (`Ported-From: ironcalc@b41cf6d5`).
- [ ] The finding is recorded in docs/excel-semantics/xlsx-storage.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making the defaulted styles render like Excel's (number formats, fonts) beyond resolving.
