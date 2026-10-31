---
id: T-1925
title: "Numeral-system digits in locale tags print native digits"
milestone: M10
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
The numeral-system digits of a locale tag (bits 24-31, `[$-8000411]`) print the native digits Excel prints.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
`TEXT(45000,"[$-8000411]g|gg|ggg|e|ee|yyyy|mmmm")` is "R|令|令和|୫|୦୫|୦୫|3月", Odia digits for the era year (probes/text-locale-calendars A26). The resolver returns `Silent` for any numeral digit. Probe which numbers they change: date fields, plain numbers (`[$-D000409]0.00`), and digits 1-0x1F. Code: formatter/locale_tag.rs `resolve_lcid`.

From T-1924 (2026-10-26): `[$-fr_FR-x]` reads as 0x10E00AF1, numeral byte 0x10, and prints Myanmar digits ("၂၀၂၃"):
probes/t1924-locale-tag-extended-hex A26, B26.

## Acceptance criteria
- [x] A probe covers the numeral digits; its cells pass or are filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/t1925-numeral-digits (32 numeral bytes x en-US/ja-JP dates and a plain number), filled from excel-graph; findings in number-formats.md. Implementation filed as T-3467 (cells are known failures). Eval delta: new probe cells added as known failures, no regressions.
