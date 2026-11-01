---
id: T-3462
title: "rustsheet-gen --policy typed-input: draw typed number, currency, percent, fraction and E-notation entries (entered cells) with value, type and format readers for gen-m2"
milestone: M2
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen --policy typed-input --seed <n>` writes gen-m2 cases whose input cells are **typed entries**
(`{"entered": "..."}`, the way a user types into a cell) drawn at random from a grammar of the number shapes
Excel's input parser reads: signs, parentheses, currency, thousands separators, decimals, E-notation, percent,
fractions and mixed fractions, with spacing and near-miss variants. Each entry has readers beside it, so an
oracle-filled batch measures whether the engine's typed-input parser generalizes beyond the hand-built probes
it was fitted on.

## Context
- What users type is the first thing an everyday spreadsheet does, and the engine's typed-input rules
  (`Model::set_user_input`, crates/rustsheet-engine/src/model.rs; `parse_formatted_number`,
  crates/rustsheet-engine/src/formatter/parse_formatted.rs; `parse_number_text`, crates/rustsheet-engine/src/cast.rs)
  were built probe by probe: docs/excel-semantics/coercion.md "Typed currency and percent formats",
  syntax.md "`**` in a typed number is `E`", and docs/research/typed-input-formats-2026-10-07.md,
  typed-currency-percent-input-2026-10-11.md, typed-mixed-fraction-number-format-2026-10-30.md. Every one of those
  is a hand-picked shape. No generated case has an `entered` cell (`rg -l '"entered"' eval/suites/gen-*` is empty),
  so nothing measures the rules on shapes nobody picked. The probes still find new typed-input gaps one at a time
  (probes/typed-mixed-fraction-number-format B15, owned by T-3251, found by T-3237).
- The case format already carries typed entries: `{"entered": "(1,000)"}` (docs/architecture/eval-harness.md;
  excel-graph PATCHes the text before calculating and logs the `numberFormat` Excel chose; the engine adapter
  enters it as typed input). Text Excel for the web refuses (PATCH 400) leaves the cell blank, which is itself
  Excel's answer.
- Readers per entry (cell `A<n>` holds the entry): `=A<n>+0` (the number, or `#VALUE!` when it stayed text),
  `=ISTEXT(A<n>)`, and `=CELL("format",A<n>)` (the format code class Excel gave the entry: typed
  probes already use it, probes/typed-paren-number-input, probes/typed-input-sign-space-mixed-fraction). The
  entry cell's own value is scored too.
- Grammar strata this task settles (record the choice and the shares in docs/architecture/synthetic-generator.md,
  next to the other policies):
  - plain numbers: integers, decimals, leading zeros, 15 to 20 significant digits, a leading or trailing `.`;
  - sign forms: `+`, `-`, `(..)`, trailing `-`, sign before and after `$`, a sign with spaces;
  - currency `$` and thousands separators, both valid grouping (`1,234,567`) and invalid (`1,23`, `12,34.5`, `,1`);
  - percent: trailing `%`, `%` with spaces, `%` combined with `$`, E-notation and fractions;
  - E-notation: `1E3`, `1e+3`, `1.5E-7`, `E` with no digits, `**` (syntax.md), exponents near 307 and past 308;
  - fractions and mixed fractions: `1/2`, `3 1/2`, `0 1/2`, denominators over 99 and with leading zeros, `1 3/2`;
  - spacing: leading and trailing spaces, CHAR(160), a tab, doubled inner spaces;
  - near misses that must stay text: `1.2.3`, `1,,2`, `$`, `-`, `1e`, `1/0`, `1 /2`, letters glued to digits.
  Dates, clocks and month names are T-3463, not this task.
- Never draw: text that starts with `=` (a formula), a year-less date such as `1/2` (Excel reads it as a date in
  the oracle's current year, which the pinned clock cannot reproduce: dates.md "The en-US date-text grammar"), or an entry
  longer than 255 characters. A shape the grammar can produce that Excel reads as a date (`1-2-3`) is allowed
  only when it carries a four-digit year; otherwise the policy redraws it.
- Determinism and naming follow §12 of docs/architecture/synthetic-generator.md (same seed, same bytes;
  content-hash names). The policy draws text; it never predicts Excel's value (no expectations in the case),
  and it never re-implements xlsx value parsing (CLAUDE.md conventions).
- Code: tools/src/rustsheet_tools/gen/ (one new module, registered in cli.py beside `numeric-fp` and
  `arg-pairs`; reuse `common.py`'s draw helpers and `quoted`).

## Acceptance criteria
- [ ] `rustsheet-gen --policy typed-input --seed 0` writes at least 2 cases of at least 150 entries each under
      gen-m2, every entry with the three readers above, each cell tagged in `hypotheses` with its stratum
      (`typed-input/<stratum>/...`). No case is committed with oracle sidecars here (T-3464 does that).
- [ ] tools/tests: a test per stratum that its drawn texts match the stratum's shape, a test that no entry
      starts with `=`, none is a year-less date and none is over 255 characters, and a determinism test
      (two runs with one seed give identical files).
- [ ] Shown in the PR: `rustsheet-eval run` with the engine standing in for Excel scores the generated
      cases with no crash, panic or timeout (an `EngineError` case is a bug to fix before merging).
- [ ] docs/architecture/synthetic-generator.md documents the policy, its strata and shares, and the exclusions.
- [ ] `make py-check` green; `make check` green; reviewers approve.

## Out of scope
- Typed dates, clocks and month names (T-3463).
- Committing an oracle-filled batch and owning its failures (T-3464).
- Displayed text of typed entries (gen-m10's `number-formats` policy) and R1C1 typed formulas.
- Any engine change.
