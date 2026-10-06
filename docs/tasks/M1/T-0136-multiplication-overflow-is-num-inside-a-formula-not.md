---
id: T-0136
title: "Multiplication overflow is #NUM! inside a formula, not an infinite intermediate (4 probes-numeric cells)"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A product that overflows the double range (`1e307*100`) is the error `#NUM!` at once, so it
propagates through `&`, CONCAT, TEXT and LEN as an error. Today the engine passes an infinite
number on, and the error appears only when a formula's final result is infinite: `=10^308*10`
already gives `#NUM!` through the cell-level guard in `model.rs`.

## Context
Found by the gap triager on 2026-10-05. Evidence (probes-numeric, oracle excel-graph, T-0008/T-0009):
`probes-numeric/4f413cc257ce15c4`
- Calc!A121 `=CONCAT(1e307*100)`: expected `#NUM!`, actual "inf".
- Calc!A122 `=(1e307*100)&""`: expected `#NUM!`, actual "inf".
- Calc!A123 `=TEXT(1e307*100,"0.000000000000000000")`: expected `#NUM!`, actual "inf.N00000000000000000".
- Calc!A124 `=LEN((1e307*100)&"")`: expected `#NUM!`, actual 3.
Related, passing: `probes/precision-and-rounding` A12 `=10^308*10` is `#NUM!`, and A13
`=1E+307*10-10^308` is 0 (1e308 does not overflow).

probes-numeric is not gated (`gate = false` in eval/suites/probes-numeric/suite.toml, so there is
no baseline file). Check with `make eval-show CASE=probes-numeric/4f413cc257ce15c4.case.json`.

Code: `OpProduct::Times` in `crates/rustsheet-engine/src/model.rs` (`evaluate_node_in_context`)
returns `Ok(f1 * f2)` through `handle_arithmetic`/`arithmetic_on_values`
(`crates/rustsheet-engine/src/arithmetic.rs`). Return `Err(Error::NUM)` when the product is not
finite. That covers the array paths too, because they share the closure. `arithmetic::power`
already does the same for `^`.

Evidence exists only for `*`. Do not change `+`, `-` or `/` unless you cite an Excel cell. If you
think they share the rule, file an excel-probe follow-up (e.g. `=(1e308+1e308)&""`,
`=(1e308/1e-10)&""`) and record it as UNVERIFIED.

Semantics: docs/excel-semantics/numbers.md "Overflow in `*` is #NUM! where it happens".

## Acceptance criteria
- [ ] `make eval-show CASE=probes-numeric/4f413cc257ce15c4.case.json` shows Calc!A121-A124
      passing. probes-numeric cells that pass today still pass (compare `make eval` pass counts
      before and after).
- [ ] No new F lines in any baseline (`make eval-baseline` refuses regressions).
- [ ] Unit tests: `=(1e307*100)&""` and `=LEN((1e307*100)&"")` are `#NUM!`; `{1e307,1}*100`
      gives `#NUM!` in the first element only; `=1E+307*10-10^308` is still 0.
- [ ] The numbers.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Overflow in `+`, `-`, `/` and in functions (needs probes first).
- Gating probes-numeric (its other failures belong to T-0107/T-0110).

## Outcome
`*` now returns `#NUM!` on overflow via `arithmetic::times`; unit test added. Eval delta: the 4
probes-numeric cells (Calc!A121:A124) now pass; probes-numeric is not gated, so no baseline change.
Follow-up: probe `+`, `-`, `/` overflow (noted UNVERIFIED in numbers.md).
