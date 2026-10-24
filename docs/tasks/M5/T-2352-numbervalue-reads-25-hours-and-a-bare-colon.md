---
id: T-2352
title: "NUMBERVALUE reads \"25:\" (hours and a bare colon left by the group separator) as 25 hours"
milestone: M5
lane: functions-text
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
When NUMBERVALUE's group separator "0" removes every zero from `"25:00"`, the text `"25:"` that is left reads
as 25 hours (1.0416666666666667), as it does in Excel. The last unowned failing cell of M5's gen-m5 exit then passes.

## Context
- `make exit-owners MILESTONE=M5` (steward run 2026-10-22) reports this cell as the only unowned M5 exit failure.
  It is the 24th cell of T-2151 (done). T-2151 fixed the other 23 cells but not this one. Its Outcome says it "had already gone
  with T-2160", but the cell is still in eval/baseline/gen-m5.txt (line `F gen-m5/eb942f4a59b1c004.case.json Calc!A115`).

  | case / cell | formula | Excel | ours |
  |---|---|---|---|
  | gen-m5/eb942f4a59b1c004 Calc!A115 | `=NUMBERVALUE("25:00",IF(FALSE(),((-0.0001)-Data!C7)),Data!E1)` | 1.0416666666666667 | `#VALUE!` |

  Data!E1 is 0 and the decimal separator is `FALSE`, so the separators are "F" and "0".
- docs/excel-semantics/text.md "NUMBERVALUE reads text as VALUE does, with its own separators" already records the
  Excel result: with the group separator "0", "25:00" still reads 1.0416666666666667 ("25:"). The other digit-group cells
  in that entry (group "5" gives "2:00", group "2" gives "5:00") pass. So the only gap is reading `"25:"`, an hour count
  followed by a colon with no minutes.
- `numbervalue_of` (crates/rustsheet-engine/src/functions/text/string_format.rs) removes the group character and then
  calls `parse_excel_number` and `Model::parse_number_text` (cast.rs). The time path is
  `functions/date_and_time/time_text.rs` `parse_time_serial` and `formatter/date_clock.rs` `parse_clock_text`. Neither
  accepts a trailing bare colon.
- Open question that decides where the fix goes: does VALUE (and `+0` coercion) also read `"25:"` and `"12:"`, or only
  NUMBERVALUE? There is no evidence for VALUE yet. Probe it with the `excel-probe` skill (new `probes` case, excel-graph). Check
  `VALUE("25:")`, `VALUE("12:")`, `"12:"+0`, `TIMEVALUE("12:")`, `NUMBERVALUE("25:")`, `NUMBERVALUE("12:30:")`,
  `VALUE("12:30:")`, `VALUE(":30")` and `NUMBERVALUE("1:","F","0")`. Put the rule in the shared parser only if VALUE agrees.
  Otherwise put it in NUMBERVALUE alone. Reuse the existing clock parser either way. Do not add a parser.

## Acceptance criteria
- [x] The line `F gen-m5/eb942f4a59b1c004.case.json Calc!A115` disappears from eval/baseline/gen-m5.txt
      (`make eval-baseline`), with no new known failure in any suite.
- [x] A new oracle-filled probes case (excel-graph, canary-verified) holds the formulas listed in Context, and
      every cell of it passes or has an owner task named in the Outcome.
- [x] docs/excel-semantics/text.md (NUMBERVALUE entry, or the VALUE time-text entry if VALUE agrees) states the
      trailing-colon rule with the probe cells as evidence, marked implemented.
- [x] An engine unit test covers `NUMBERVALUE("25:00",".","0")` = 25/24 and one VALUE cell from the probe.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the owned cell has the wrong type (`#VALUE!` against a number), so the task is in the campaign.
Its acceptance criteria above are the whole visible-parity stage.

## Out of scope
- The other unprobed NUMBERVALUE forms listed in text.md (date text with spaces, `%` with custom separators,
  separators outside the BMP). File them as M5 probe tasks if needed.
- Array lifting (T-2160).

## Outcome
- Probe probes/t2352-time-text-trailing-colon (26 cells, excel-graph, 2026-10-22): VALUE agrees with NUMBERVALUE, so
  the rule went into the shared clock parser `parse_clock_text` (formatter/date_clock.rs): a bare hour followed by one
  `:` is that hour (`"12:"` = 0.5, `"25:"` = 25/24), unless an AM/PM suffix follows (`"12: PM"` is #VALUE!). No new
  parser. Recorded in dates.md "Time text" (trailing-colon bullet) and the text.md NUMBERVALUE entry.
- Eval delta: gen-m5/eb942f4a59b1c004 Calc!A115 leaves the baseline; the new probe case passes 26/26; no new known failures.
- Unit tests: `NUMBERVALUE("25:00",".","0")` = 25 hours and `value_reads_an_hour_with_a_trailing_colon`
  (test_fn_dollar_fixed_numbervalue_lift.rs).
