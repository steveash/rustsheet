---
id: T-3443
title: "UPPER and LOWER map only the characters in Excel's old case table (probes/t3298-case-mapping-sweep)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
UPPER and LOWER give Excel's result for every character of probes/t3298-case-mapping-sweep: they leave the
characters Excel's case table lacks unchanged and use Excel's own targets where they differ from Rust's.

## Context
- text.md "UPPER and LOWER use an old per-character case table (T-3298)" lists, as code point ranges, every character
  UPPER and LOWER leave unchanged, and the targets that differ from Rust (Greek tonos dropped by UPPER: ά έ ή ί ό ύ ώ to
  Α Ε Η Ι Ο Υ Ω; LOWER of Georgian Asomtavruli U+10A0-U+10C5 to Mkhedruli U+10D0-U+10F5).
- Failing cells: probes/t3298-case-mapping-sweep Calc!A1, A4, A5, A7:A10, A15:A18, A21:A24, A26:A50 (UPPER) and
  B8, B9, B14:B21, B23:B47 (LOWER). Each cell is 32 characters of Data!A or Data!B; `make eval-show
  CASE=probes/t3298-case-mapping-sweep.case.json` shows the expected strings.
- Code: `simple_case`, `lower_case`, `upper_case` (crates/rustsheet-engine/src/functions/text/string_format.rs). A
  sorted static table of unmapped ranges (or of Excel's mapped pairs) checked before the Rust mapping is enough; search
  for an existing range-table helper first (decision: one definition per concept). PROPER shares `simple_case`:
  its word-start letters follow the same table (probes/clean-proper-chars and probes/case-mapping-expanding must keep
  passing).
- The tonos rule needs no context: it is a per-character target.

## Acceptance criteria
- [ ] Every cell of probes/t3298-case-mapping-sweep column A and B passes; their lines leave eval/baseline/probes.txt
      (`make eval-baseline`), 0 regressions.
- [ ] Unit tests pin one character per unmapped block and the Greek and Georgian targets.
- [ ] text.md's section says "Implemented in" and its Status names this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MATCH's exact fold (T-3444, T-3445) and the collation (`fold` in collation.rs).
