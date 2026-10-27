# 0128 — Allow four table-edit probe regressions unmasked by T-1671 totals rows

- **Status:** accepted (2026-10-26, T-1671)
- **Context:** T-1671 lets `Txn::add_table_with_totals` register a table's totals row, and the eval
  adapter now loads a spec's `"totals_row": true` with it. Until now the adapter left the totals row
  out of the table, so `Sales[#Totals]` was `#REF!` and `#All` one row short in every totals case.
  With the row loaded, all 3,368 gen-m8b cells pass (the 277 `[#Totals]`/`[#All]` cells listed in
  the task) and 11 probes cells are fixed (table-totals-baseline, table-insert-row-below-totals and
  others; `eval/baseline/probes.txt`). Four probes cells passed before only by accident: the table
  had no totals row, so the structural edit did not touch it. The engine still never moves, grows
  or shrinks a table on a row insert or delete (SPEC-0016 fallback for every table, T-2297), so
  with the totals row in the table these edits now print or read the displaced plain range:
  - `probes/table-edits/table-delete-row-totals` F!A3 `ROWS(Sales[#All])`: Excel 5, ours
    `#ERROR!` (the deleted totals row is the end of the lowered `#All` range).
  - `probes/table-edits/table-delete-row-totals` F!A15 `FORMULATEXT` of `=Sales[[#Totals],[Qty]]`:
    Excel keeps the text, ours `=#REF!`.
  - `probes/table-edits/table-delete-row-last-data-totals` F!A15: Excel keeps the text, ours
    `=S!$C$7` (the totals row moved up; the table did not shrink).
  - `probes/table-edits/table-insert-row-above-totals` F!A12: Excel keeps the text, ours `=S!$C$9`
    (the totals row moved down; the table did not grow).
  Their values and texts are in docs/excel-semantics/references.md ("Structured references when
  the table changes", "Structured references under row deletes ...").
- **Options considered:**
  - *Keep leaving the totals row out in the adapter.* Contradicts the oracle on 277 gen-m8b cells
    and 11 probes cells to keep four accidental passes.
  - *Implement table move/grow/shrink in the same PR.* That is T-2297 (spec step 1b), then T-2321 and
    T-2338 on top of it: three M-sized opus tasks; one PR would be three tasks (CLAUDE.md directive 4).
  - *Record the four as known failures and give them to the tasks that own their root cause.*
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-1671 PR, for
  exactly these four cells. table-delete-row-totals F!A3 and F!A15 and
  table-delete-row-last-data-totals F!A15 join T-2338's acceptance list; table-insert-row-above-totals
  F!A12 is already in T-2321's (every cell of the table-insert-row-* cases).
- **Consequences:** `eval/baseline/probes.txt` loses 11 lines and gains these four. Reviewers
  confirm the baseline diff adds no other line. T-2338 and T-2321 must make them pass.
