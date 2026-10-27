---
id: T-0489
title: "Does Excel for the web open the poi workbooks we refuse (60825 empty sheet r:id, 47889 no cellXfs, style-alternate-content, invalid_cell_reference, 56278, 59021)?"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Each poi workbook the importer refuses with a typed error has Excel evidence of whether Excel opens it, so the
refusal is either confirmed as Excel's behavior (or an accepted malformed-input refusal) or becomes an engine task
with the values Excel shows.

## Context
Gap triage 2026-10-27 (`rustsheet-eval fetch`, then `make eval` on origin/main 6087413b): these poi cases end in
`engine_error` before any cell is scored. Decision 0012 and T-2208's Out of scope keep them failing because there
is "no Excel evidence that they load"; nobody has looked for that evidence since.

| case | importer message |
|---|---|
| poi/60825.xlsx | Missing relationship "" for worksheet (the report gives its source as excel-cached, Microsoft Excel) |
| poi/47889.xlsx | XML Error: Missing `<cellXfs>` section in styles.xml |
| poi/style-alternate-content.xlsx | XML Error: Missing "name" XML attribute |
| poi/invalid_cell_reference.xlsx | XML Error: Invalid cell reference: 'B' |
| poi/56278.xlsx, poi/59021.xlsx | Zip Error: specified file not found in archive |

The XML bombs (54764, 54764-2, poc-xmlbomb, poc-xmlbomb-empty), the 32,767-unit shared string and the three
zip files with no end-of-central-directory record are directive-5 refusals or not zip files, and stay out of
scope.

Method: open each file with the excel-graph backend (`cd tools && uv run rustsheet-oracle status` first; the
`recalc` subcommand or `rustsheet-eval compare`, whichever the tools document for an Excel-authored xlsx). Record
whether Excel for the web opens it, refuses it (for example `FileCorruptTryRepair`, as seen for
t0481-curly-quote-names-c), or repairs it, and for a file it opens, the values of its formula cells.

## Acceptance criteria
- [ ] docs/excel-semantics/xlsx-storage.md has one entry per listed file (or one shared entry) stating what Excel
      for the web does with it, with the date and backend; a refusal entry says which part Excel rejects if the
      error tells.
- [ ] For each file Excel opens, an engine task is filed with `make new-task` (lane engine, milestone M11, with the
      cell values Excel shows and a `## Visible-parity stage`, decision 0083: a workbook Excel opens that we refuse
      is wrong in every cell); for each file Excel refuses, the Outcome says the poi baseline line stays.
- [ ] No change to eval/baseline/ or the importer in this task; `make check` green; reviewers approve.

## Out of scope
- Changing the importer (the filed engine tasks do that).
- Desktop Excel's repair dialog (M14, T-1075).
