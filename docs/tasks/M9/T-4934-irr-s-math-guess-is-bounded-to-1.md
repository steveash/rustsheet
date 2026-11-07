---
id: T-4934
title: "IRR's math guess is bounded to (-1, 2^53): 17 irr-guess-edges cells Excel gives as #VALUE! have math values"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
IRR's Gate A math definition has no value where Excel's IRR is `#VALUE!` because of the guess. Outside
(-1, 2^53) the call is a domain error.

## Context
T-4778 (#3865, merged 2026-11-07) gave IRR an optional `guess` call parameter
(`tools/src/rustsheet_tools/accuracy/defs/cashflow.py`, `IRR = register(...)`). The parameter has no domain, so it
defaults to the unbounded `Interval()`. docs/excel-semantics/financial.md "IRR's guess range" (VERIFIED) says that
a guess at or below -1, or at or above 2^53, is `#VALUE!` whatever the flows. As things stand,
eval/suites/probes/cases/irr-guess-edges.math.json gives the number 0.1306623862918... to 17 cells whose oracle
value is `#VALUE!`: P!A1:A6 and A9:A19 (`=IRR({-100,60,60},G)` for G = 1E+300, 1E+30, 2.5E+29, 1E+29, 1E+20,
1E+16, -1E+300, -1, -2, 2^53, 2^60 ... 2^100). This rule is deterministic, so the RATE/T-3923 "Excel cannot
converge" precedent does not apply. Decision 0188 item 2 makes it a `DomainError`. The reviewer-excel-fidelity
round on #3865's T-4778 head found it (BLOCKING); the PR merged before the fix landed.

The fix was written and checked on the T-4778 branch, but that branch had merged. It is three parts:
- give the parameter its domain: `Param("guess", "real", Interval(-1.0, float(2**53)),
  source="financial.md#irrs-guess-range", optional=True)` (open ends; import `Interval` from
  `rustsheet_tools.accuracy.model`), with a comment that Excel is `#VALUE!` outside it;
- add a test next to `test_irr_guess_does_not_change_the_math_value` in tools/tests/test_accuracy_cashflow.py:
  -1, -2, `mp(2)**53` and 1E+300 raise `DomainError`, and `mp(2)**53 - 1` gives the guessless value;
- regenerate with `cd tools && uv run rustsheet-accuracy math --suite probes --function IRR --function XIRR
  --function XNPV --function NPV --function MIRR --function FVSCHEDULE` and the same for `--suite
  probes-numeric`. The definition fingerprint covers the cashflow module, so all six go stale. This takes about
  2 minutes; the full regeneration without `--function` runs past 30 minutes. Then run `math --check --suite
  probes --suite probes-numeric`.
On that branch the 17 cells came out `"t": "e"` with `why: "guess outside (-1, 9.0072e+15)"`, and the tests,
ruff and mypy passed.

## Acceptance criteria
- [ ] irr-guess-edges P!A1:A6 and A9:A19 have no math value: they are domain errors in the regenerated sidecar.
- [ ] A unit test covers both bounds and 2^53-1.
- [ ] `math --check` is current for probes and probes-numeric; Gate A (`make eval-gates`) does not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes: the engine already gives `#VALUE!` there.
