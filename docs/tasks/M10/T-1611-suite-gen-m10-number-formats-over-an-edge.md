---
id: T-1611
title: "Suite gen-m10: number formats over an edge-value pool, at least 10,000 oracle-filled formatted-text cells"
milestone: M10
lane: harness
status: todo
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
- [ ] A generator policy (for example `--policy number-formats`) produces the cases.
- [ ] `eval/suites/gen-m10` is committed with ≥ 10,000 oracle-filled text cells, canary-verified,
  `milestone = ["M10"]`, with its baseline. Every failing group has a filed or cited task.
- [ ] Quirks recorded in docs/excel-semantics/number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the format engine (file tasks).
