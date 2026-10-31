---
id: T-3368
title: "TEXT: probe what General prints after a date or time token (TEXT(5,\"d General\") is \"5 0.5\" in Excel)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence that explains what `General` prints after a date or time token in a TEXT format, so T-3369 can
implement the rule from cells instead of guessing: `TEXT(5,"d General")` is "5 0.5" in Excel, and nothing in eval/
says where the "0.5" comes from.

## Context
- Gap triage 2026-10-31 (`make eval` on 454f580b4): the only visibly-off cell no open task owned is
  probes/text-format-general-company Sheet1!A5, `=TEXT(5,"d General")`: Excel (excel-graph, 2026-10-31, T-1614)
  "5 0.5", ours `#VALUE!` (verdict TypeMismatch). T-1614 recorded it as an unexplained oddity in
  docs/excel-semantics/number-formats.md ("General" bullet: "General after a date token appends a value that is not
  the serial") and left it as a known failure with no follow-up task.
- Ours: `Parser::parse_part` (crates/rustsheet-engine/src/formatter/parser.rs, the `general_count` check) refuses
  `General` in a date section (`is_date`), hence `#VALUE!`.
- The case has only the one date cell, so the value General prints (0.5 for the serial 5: half of it? a tenth? a
  time part?) cannot be fitted from it. This task only gathers the evidence; T-3369 (opus) implements it.

Probe (excel-probe skill, excel-graph), one new case, e.g. probes/t3368-general-after-date-token, one question per
cell:
- the serial: `TEXT(n,"d General")` for n = 0, 1, 2, 5, 10, 31, 32, 100, 0.75, 5.25, 45000, 45000.5, -1, 2958465;
- the token before it: `"dd General"`, `"ddd General"`, `"m General"`, `"mmm General"`, `"yyyy General"`,
  `"h General"`, `"hh:mm General"`, `"s General"`, `"[h] General"`, `"AM/PM General"`, `"e General"` on 5 and 45000.5;
- the order and company: `"General d"`, `"d  General"` (two spaces), `"dGeneral"`, `"d General x"`, `"d General%"`,
  `"d General;@"`, a second section (`"0;d General"` on -5), and `"d.0 General"`;
- the case: `"d general"` and `"d GENERAL"`.
Commit the case with its oracle sidecar. Record the measured rule (or, if it stays unexplained, the cells) in the
"General" bullet of docs/excel-semantics/number-formats.md, citing the new cells, Status VERIFIED for what the cells
show.

## Acceptance criteria
- [ ] The probe case above is committed with its excel-graph oracle sidecar (`rustsheet-oracle fill`), and every
      bullet above has at least one cell.
- [ ] docs/excel-semantics/number-formats.md's "General" bullet cites the new cells and states the rule they show
      (or lists them as unexplained), replacing "a value that is not the serial".
- [ ] The new case's failing cells are known failures in eval/baseline/probes.txt (`make eval-baseline`, 0
      regressions), and T-3369's Context names the rule and the new case so it owns those lines (decision 0159).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the formatter: T-3369 implements the rule once these cells exist.
- `General` beside digit placeholders, `%` or a second `General` (T-1614, VERIFIED).
