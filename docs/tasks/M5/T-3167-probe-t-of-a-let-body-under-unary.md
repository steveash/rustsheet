---
id: T-3167
title: "Probe T of a LET body under unary plus or a spill operator over long cell text (T(LET(x,D1,+x)), T(LET(x,F1,x#)))"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's T of a LET whose body reads a 256-unit cell through a unary plus or a spill operator is known from oracle
evidence, and RustSheet matches it or has a task that makes it.

## Context
- T-3157 left `T(LET(x,D1,+x))` reading "" (D1 typed, 256 x): the cell-text test of a LET body as written is the
  reference-context reading `is_reference` gave a LET (`t_let_argument` in `functions/text/t_function.rs`), while
  `T(+D1)` is `#VALUE!` (probes/text-255-units-reference-shapes A13). Unprobed; Excel may give `#VALUE!`.
- Shapes to probe: `T(LET(x,D1,+x))`, `T(LET(x,D1,(x)))`, `T(LET(x,F1,x#))` with F1 `=D1:D2`,
  `T(LET(x,D1,@x))`, `ISREF(LET(x,D1,+x))`.

## Acceptance criteria
- [ ] A probe case with oracle values for each shape above lands in `eval/suites/probes` (excel-probe skill).
- [ ] Any shape RustSheet gets wrong is fixed here (size S) or has a follow-up task; the finding is in
      `docs/excel-semantics/text.md`, and the `TODO(T-3167)` in `t_function.rs` is resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' cell-text or reference tests.
