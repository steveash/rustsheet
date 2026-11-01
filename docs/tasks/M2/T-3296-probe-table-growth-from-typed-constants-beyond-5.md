---
id: T-3296
title: "Probe: table growth from typed constants beyond 5 and hello (totals cell, an existing column name, TRUE, dates, percents, leading space, 255+ chars)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3218]
---

## Goal
Excel evidence for the typed-constant table growth cases T-3218 implemented without a probe. Today the engine
answers each of them by an engine choice (docs/excel-semantics/references.md "Calculated columns", rule 7 "Ours").

## Context
- T-3003's probes (probes/t3003-*, excel-graph) typed only `5` and `hello` beside table `T` (A1:C4), and only a
  formula into the totals cell of a table with a totals row. T-3218 grows the table for any non-empty constant,
  including one typed beside the totals row. It writes a constant typed into the header cell as the header text,
  as typed, without a leading `'` (`constant_header`, crates/rustsheet-engine/src/calculated_columns.rs). A name the
  table already has falls back to `ColumnN` (`table_edits::append_column`).
- T-3218's reviewers (PR #2453, excel-fidelity and security) flagged these as unprobed: a constant typed into the
  totals cell D5; an existing column name typed into D1 (`Col`, and `col` for case); the constants `TRUE`,
  `1/2/2026`, `5%`, `1e3` and ` x` (leading space) and `'5` typed into D1; and a header text longer than 255
  characters (Excel's limit on a column name).
- Use the `excel-probe` skill and the same probe shape as probes/t3003-*: readouts `COLUMNS/ROWS(T[#All])`,
  `D1&""`, `TYPE(D1)`, `ISFORMULA/ISTEXT(D1)`.

## Acceptance criteria
- [ ] One probe case per question above, filled by excel-graph (`rustsheet-oracle fill`), tagged M2.
- [ ] docs/excel-semantics/references.md rule 7 records each answer. Every place the engine differs gets a
      follow-up implementation task, and its cells stay in eval/baseline only through `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any difference the probes find (follow-up tasks).
- Row growth below the table (T-3273) and an unqualified `[@Col]` beside it (T-3274).
