---
id: T-1213
title: "Implement DBCS and PHONETIC as the web oracle evaluates them"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
DBCS and PHONETIC are implemented as excel-graph evaluates them: DBCS of ASCII text returns it unchanged, PHONETIC of a reference is `#N/A`.

## Context
From T-1087's probes: probes/text-modes-dbcs C3 (`DBCS("abc")` = "abc") and probes/text-modes-phonetic, -range, -number C1 (`#N/A` for a cell, a range and a number cell) are `#NAME?` in the engine. A literal argument (`PHONETIC("abc")`) makes excel-graph refuse the workbook, so it is unobservable (docs/excel-semantics/text.md "Text function argument modes"). Check DBCS for full-width conversion of katakana/ASCII before implementing more than the ASCII case (excel-probe).

## Acceptance criteria
- [ ] The four probe cells pass; any DBCS conversion rule beyond them comes with a probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Implemented and pushed on branch `agent/functions-text/T-1213-dbcs-phonetic`: `dbcs` and `phonetic` fields in `Language` (language/mod.rs) and in tools/upstream/generate_language (languages.json for all 5 locales as the English names, main.rs), `language.bin` regenerated with `cargo run` there, `Function::Dbcs`/`Function::Phonetic` (functions/mod.rs, static_analysis.rs), `fn_dbcs`/`fn_phonetic` in functions/text/char_code.rs, a unit test in test/text_functions/mod.rs, text.md updated, baseline updated (6 cells fixed: the four probes plus probes/catalog/availability-10 A4 and availability-30 A1). The four probe cells pass.
Blocked: `make check` fails the oversized-file ratchet because `crates/rustsheet-engine/src/functions/mod.rs` grew 2388 -> 2397 lines (enum +2, name table +2, name fn +2, xlfn +1, dispatch +2); `static_analysis.rs` is back to 2156 (arms merged). Raising the ratchet needs `quality/oversized-files.json` (protected path, decision record) or shrinking 9 lines elsewhere in functions/mod.rs. Next: decide which (a decision record, or a small refactor of that file), then `make check`, review, PR, and set the Outcome and `status: done`. Draft PR opened for the branch.
