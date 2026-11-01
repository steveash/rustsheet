---
id: T-3791
title: "FREQUENCY with no numeric bin: values <= 0 count in the first row, and an error cell as the bins is skipped (gen-m9, 46 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
FREQUENCY with no numeric bin returns Excel's two rows: the count of data values <= 0, then the rest; and a one-cell
reference to an error cell as the bins is a bin with no number, not that error.

## Context
- T-3781 (gen-m9 arg-pairs batch, excel-graph 2026-11-01), case d56012c9341cd0ff: with bins a text, empty text, boolean,
  blank cell or typed numeric text (`"2.5"` reads as no number), `FREQUENCY(0,"2.5")`, `FREQUENCY(-1,"")` and
  `FREQUENCY(Data!C3:D4,Data!G7)` are {1;0} while `FREQUENCY(2,"abc")` and `FREQUENCY(0.5,"")` are {0;1}: the lone missing
  bin acts as 0. Ours puts every value in the second row ({0;1} for 0 and -1). With bins Data!G9 (a cell holding `#DIV/0!`)
  Excel gives the same two rows (`FREQUENCY(Data!G2,Data!G9)` {1;0}, `FREQUENCY(Data!G4,Data!G9)` {0;1},
  `FREQUENCY(Data!G5,Data!G9)` {0;0}); ours returns `#DIV/0!`. A typed error bin is still the result
  (`FREQUENCY("2.5",#NUM!)` `#NUM!`), and an error cell as the data is that error (`FREQUENCY(Data!G9,Data!G1)` `#DIV/0!`).
- This refines statistics.md "FREQUENCY: one row per bin plus one ... (T-2255)" ("0 then every numeric data value" came
  from positive data only) and answers part of T-3046 (an error cell in the bins). Recorded in statistics.md "FREQUENCY
  with no numeric bin: a bin at 0; an error cell as the bins is skipped (T-3781)".
- Code: frequency.rs (`freq_values_of`, `fn_frequency`).

## Acceptance criteria
- [ ] The 46 cells below pass, and probes/frequency-bins still passes.
- [ ] statistics.md entry's Ours line updated; T-3046's context notes the one-cell answer.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0159: the task is done when the gen-m9 cells below (T-3781's arg-pairs batch, excel-graph, 2026-11-01) pass and
their known-failure lines leave eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure line in any baseline.
- gen-m9/d56012c9341cd0ff Calc!A794, A795, A1343, A1344, A1404, A1405, A1465, A1466, A1526, A1527, A1587, A1588, A1648, A1649, A1770, A1771, A1831, A1832, A1892, A1893, A2075, A2076, A2136, A2137, A2197, A2198, A2258, A2259, A2380, A2381, A2990, A2991, A4088, A4089, A4820, A4821, A6040, A6041, A7138, A7139, A9578, A9579, A9822, A9823, A9944, A9945

## Out of scope
- An error cell inside a multi-cell bins range, and the corpus-bench re-measure (T-3046).
