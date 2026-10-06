---
id: T-0164
title: "Shared strings: leave phonetic rPh runs out of the cell text"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A shared string with phonetic (furigana) runs imports as its base text only, as the eval snapshot reader already reads it.

## Context
`read_shared_strings_from_xml` (crates/rustsheet-xlsx/src/import/shared_strings.rs) joins every
`<t>` below `<si>`, including those inside `<rPh>` (phonetic runs, ECMA-376 §18.4.6), so the cell
value gains the furigana text. `crates/rustsheet-eval/src/xlsx_snapshot.rs` skips `rPh` already.
Found by the T-0121 Excel-fidelity review: the new 32,767-character limit (T-0121) is also counted
on the joined text, so a long Japanese string with furigana could be rejected.

## Acceptance criteria
- [ ] `<rPh>` subtrees are excluded from the shared-string text (and so from the length count); unit test with an `<si>` holding `<r>` runs plus an `<rPh>`.
- [ ] If no Excel-authored workbook in the suites has phonetic runs, an `excel-probe` case (`=LEN(A1)` over such a cell) is added, pending if the oracle is down.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PHONETIC() itself.
