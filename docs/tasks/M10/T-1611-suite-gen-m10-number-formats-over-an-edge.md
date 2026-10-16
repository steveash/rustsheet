---
id: T-1611
title: "Suite gen-m10: number formats over an edge-value pool, at least 10,000 oracle-filled formatted-text cells"
milestone: M10
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1610]
---

## Goal
A committed, seeded suite `gen-m10` with ≥ 10,000 oracle-filled formatted-text cells, the size
M10's exit criterion names, so number-format parity is measured.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- Depends on T-1610 (formatted-text expectations).
- spec2xlsx already sets per-cell number formats (`"number_formats": {"A1": "[h]:mm"}`).
- Cover:
  - all built-in formats;
  - custom formats with 1–4 sections and conditions (`[<100]`), colors, literal text and
    escapes, and `@`;
  - dates and times with elapsed `[h]` and fractional seconds;
  - fractions (`# ?/?`, `# ??/??`), scientific, thousands separators and scaling commas,
    percent, and General at narrow and wide columns.
- Apply them to an edge-value pool: negatives, zero, 1E-10..1E+15, near-integers, date serials
  0/60/61, and text and booleans.
- TEXT() with the same format codes, for a cross-check against `.Text`, is useful extra coverage.
  The existing TEXT tasks in M10 (T-0933..T-0938 and successors) cite their failing cells.

## Acceptance criteria
- [x] A generator policy (for example `--policy number-formats`) produces the cases.
- [x] `eval/suites/gen-m10` is committed with ≥ 10,000 oracle-filled text cells, canary-verified,
  `milestone = ["M10"]`, with its baseline. Every failing group has a filed or cited task.
- [x] Quirks recorded in docs/excel-semantics/number-formats.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the format engine (file tasks).

## Outcome
`rustsheet-gen --policy number-formats --seed 0` (tools gen/number_formats.py) writes 20 cases: a
20-value x 26-format grid on sheet F, every cell in `text`, plus 26 TEXT() checks on sheet T. The
pool is all 32 built-ins and 103 custom codes over 44 edge values. The excel-graph fill gives
10,400 text cells and 1,690 value cells, canary-verified, in about 13 s per case: the backend now
reads a sheet's text cells in one request for their bounding block (`text_blocks`) instead of one
request per cell. The suite is gated (`gate = true`, `milestone = ["M10"]`), and
eval/baseline/gen-m10.txt holds its 117 value known failures (TEXT() cells, 93.1% of values pass).
Ours shows 6,961 of the 10,400 texts (66.9%). Every failing code is cited in number-formats.md
"Displayed text over an edge-value pool": new tasks T-1730 to T-1738, and existing T-1711, T-0982,
T-1129, T-0981, T-0934, T-0980 and T-1614. Excel for the web refuses a workbook with a lowercase
`0.00e+00` format, so the pool leaves it out.
Gating the text cells is T-1739.
Eval: gen-m10 +20 cases (117 new-coverage value failures), 0 regressions elsewhere.
