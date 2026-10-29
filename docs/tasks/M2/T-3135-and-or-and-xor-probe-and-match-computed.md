---
id: T-3135
title: "AND, OR and XOR: probe and match computed, array and padded TRUE/FALSE text and a FALSE text beside a logical"
milestone: M2
lane: functions-logic
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
AND, OR and XOR read the text "TRUE" and "FALSE" the way Excel does in every form the current evidence leaves open:
- computed by a formula (`""&"FALSE"`, `IF(TRUE,"TRUE")`, `T("TRUE")`, `LOWER("TRUE")`);
- inside an array constant or an array result (`{"TRUE"}`, `IF({1},"TRUE")`);
- padded (`" TRUE"`, `"TRUE "`);
- `"FALSE"` next to a logical.
The answer comes from a committed oracle case, and coercion.md drops the "unprobed" clauses.

## Context
- docs/excel-semantics/coercion.md "AND/OR/XOR text arguments": a literal "TRUE" argument counts, case-insensitively
  (probes/and-or-xor-text-arguments H12, H13, H40, H41, H68, H69; all with `"TRUE"` or `"true"`). The section also
  says: "computed and array "TRUE" text are unprobed; "FALSE" next to a logical is unprobed, while `AND("FALSE")`
  alone is FALSE in ironcalc E48".
- Other text arguments are skipped, literal or computed: `AND(""&"1",TRUE)`, `AND(T("1"),TRUE)` and
  `AND(IFERROR("1",0),TRUE)` are TRUE.
- Picked text cells are `#VALUE!` (probes/and-or-xor-picked-text, T-1685).
- The engine (`logical_nary` in crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs) counts a string
  only from an `ItemSource::Direct` item, through `cast_to_bool`. On main 97318e68a:
  - `OR(IF(TRUE,"TRUE"),FALSE)` and `OR(T("TRUE"))` are TRUE, so computed text counts;
  - `AND(" TRUE",TRUE)` is TRUE but `OR(" TRUE",FALSE)` is FALSE, so padded text is skipped;
  - `AND(""&"FALSE",TRUE)` and `AND(LOWER("TRUE"),FALSE)` are FALSE;
  - `OR({"TRUE"},FALSE)` is FALSE and `OR({"TRUE"})` is `#VALUE!`, so array text is skipped.
  Each of these is an inference from the literal cells.
- Why opus: the probe has to separate readings that the literal cells cannot. Excel may count "TRUE"/"FALSE" text
  by the argument's syntax (a string literal only), by its source (any non-reference value, the engine's model),
  or by its value kind (array elements too). Its boolean parse may also trim or not. Each candidate fits all 70+
  cells already probed. Choose cells where they disagree, and pair each with a control that is TRUE under one
  reading and `#VALUE!` or FALSE under another. Do not use `X` alone, because a skipped argument and a TRUE look
  the same next to TRUE. The fix sits in the SPEC-0009 walker consumer for AND/OR/XOR (`LOGICAL_ARGS` and
  `ItemSource`), which every case of the three existing probes depends on.
- Method: the `excel-probe` skill, excel-graph, one case `probes/t3135-and-or-xor-computed-bool-text` (milestone M2).
  Each form goes into AND, OR and XOR, both beside FALSE or TRUE and alone (alone, a skipped argument is
  `#VALUE!`).

## Acceptance criteria
- [x] `probes/t3135-and-or-xor-computed-bool-text` is committed with its excel-graph oracle sidecar. It covers, for
      AND, OR and XOR:
      - computed "TRUE" and "FALSE" (`&`, IF, T, LOWER, a LET-bound string);
      - array-constant and array-result elements;
      - leading and trailing spaces, and a CHAR(160) pad;
      - "FALSE" beside TRUE and beside FALSE;
      - each form alone.
- [x] Every cell of that case passes, and probes/and-or-xor-text-arguments, -picked-text and -error-order keep all
      their passing cells (`make eval-check`: 0 regressions; `make eval-baseline` run).
- [x] coercion.md "AND/OR/XOR text arguments" states the probed rule as VERIFIED with the case id and cells. The
      "unprobed" clauses are removed or narrowed to what is still open. `logical_nary`'s doc comment matches.
- [x] If the probe shows cells no single rule fits within size S, the fitted cells pass, the rest are recorded as
      known failures by `make eval-baseline` (never hand-edited), and a follow-up task names them.
- [x] `make check` green; reviewers approve.

## Out of scope
- NOT (already probed: probes/not-array-cast, ironcalc-calc LOGICAL/NOT.xlsx).
- Text "TRUE" in referenced or picked cells (already probed, T-1685).
- IF's own condition coercion.

## Outcome
probes/t3135-and-or-xor-computed-bool-text (21 forms × AND/OR/XOR beside TRUE, beside FALSE and alone; 189 cells,
excel-graph) settles the rule. A direct scalar text that is exactly "TRUE" or "FALSE" (any case) counts, however it
was computed (`&`, IF, T, LOWER, LET). Padded text (spaces, CHAR(160)) is skipped, because the parse does not trim.
Array elements are skipped, in constants and in computed arrays. This is the engine's existing "by source" model
(`ItemSource::Direct` through `cast_to_bool`), so the engine already passed all 189 cells and needed no code change.
The fallback criterion did not apply: every cell fits the one rule. coercion.md drops the "unprobed" clauses and
states the rule as VERIFIED, and the doc comment on `logical_nary` matches it.
Eval delta: +189 passing probe cells (new case), 0 regressions.
