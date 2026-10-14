---
id: T-1257
title: "FIXED of a negative number that rounds to zero prints 0, not -0"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FIXED of a negative number that rounds to zero at the requested decimals prints "0" (no minus sign), as Excel does.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09): gen-m6/f94b5b8236742618 Calc!A24
`=FIXED(EVEN((-0.0001)),-1,("a*"<>MINUTE(2.5)))`, i.e. `FIXED(-2,-1,TRUE)`: Excel "0", engine "-0".
`fn_fixed` (crates/rustsheet-engine/src/functions/text/string_format.rs) prefixes "-" whenever the unrounded value is
negative; it should test the rounded value (also `FIXED(-0.4,0)` gives "-0" today). Semantics: docs/excel-semantics/text.md
"FIXED drops the sign of a value that rounds to zero" (added by T-1239's triage; only `FIXED(-2,-1)` is evidenced, so
mark other shapes UNVERIFIED unless probed).

## Acceptance criteria
- [ ] gen-m6/f94b5b8236742618 A24 passes in `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell
      and no new known failures in eval/baseline/.
- [ ] Unit tests: `FIXED(-2,-1)` = "0", `FIXED(-2,-1,TRUE)` = "0", `FIXED(-6,-1)` = "-10".
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXT's sign for values rounding to zero (T-1129).

## Additional evidence (T-1281)
`probes/dollar-fixed-unichar-edges` A49 `FIXED(-0.004,2)` = "0.00" and A50 `FIXED(-0.4,0)` = "0" confirm the rule for
more shapes; both are known failures in eval/baseline/probes.txt until this lands. DOLLAR keeps its parentheses
(`DOLLAR(-0.004)` = "($0.00)").

## Outcome
`fn_fixed` drops the minus sign when the rounded text has no non-zero digit (string_format.rs); unit tests in test/text_functions/fn_fixed_negative_zero.rs; text.md updated. Eval: probes known failures 5622 -> 5620 (dollar-fixed-unichar-edges A49, A50 fixed, 0 regressions). gen-m6 is not baselined; gen-m6/f94b5b8236742618 A24 `FIXED(-2,-1,TRUE)` is covered by the unit test.
