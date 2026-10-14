---
id: T-0419
title: "xlsx export writes theme tints at full precision and rows that only carry a height or hidden flag (port ironcalc 306daf40, f428b2eb)"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Export keeps a theme colour's tint exactly (shortest round-trip, not 16 fixed decimals), and keeps a row that has no cells but a custom height or the hidden flag.

## Context
Two small upstream IronCalc export fixes, listed by the 2026-10-13 steward run:
- 306daf40 writes `tint="{}"` instead of `tint="{:.16}"`. Ours still uses `{tint:.16}` in
  `crates/rustsheet-xlsx/src/export/styles.rs`, `styles_util.rs` and `dxfs_styles.rs`.
  If the three sites share one formatting helper, port it there once, not three times.
- f428b2eb writes every row that has cells *or* row properties. Our row loop
  (`crates/rustsheet-xlsx/src/export/worksheets.rs`, `for row_index in worksheet.sheet_data.rows()`)
  skips rows with no cells, so their height and hidden flag are lost on export.
Port with `Ported-From: ironcalc@306daf40924b72a1b98cbda45500e9ba582777b4` and
`Ported-From: ironcalc@f428b2ebbc57fcf292f3e67027b24e1c866cb7c4`.

## Acceptance criteria
- [ ] Export → import round-trip tests: a theme tint such as -0.249977111117893 survives unchanged, and an empty hidden row and an empty row with a custom height keep those properties.
- [ ] Existing export tests and the xlsx round-trip suites stay green.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column (`<col>`) export changes.

## Outcome
Export writes a theme tint in shortest round-trip form (`{tint}`, three sites) and writes rows that carry only a height, hidden flag or style. New tests: crates/rustsheet-xlsx/tests/export_row_props_and_tints.rs (both fail before the change). Eval delta: none expected; see PR.
