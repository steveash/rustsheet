---
id: T-2450
title: "SUMPRODUCT: an error element beats a shape mismatch in any argument order"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
SUMPRODUCT returns the error element of any argument even when the argument shapes disagree, as Excel does.

## Context
Rule and evidence (excel-graph, 2026-10-23, T-2296): `eval/suites/probes/cases/t2296-sumproduct-error-element` and
docs/excel-semantics/coercion.md "SUMPRODUCT: a blank one-cell argument beside other arguments is `#VALUE!`". An error element in
any argument beats a shape mismatch, in either order (`SUMPRODUCT(X2x1,R2x2-with-#DIV/0!)` is `#DIV/0!`, `SUMPRODUCT(X2x1,{1,#N/A;2,3})`
is `#N/A`); a whole-argument error beats an error element (`SUMPRODUCT(R,X,NA())` is `#N/A`); a mismatch with no error anywhere stays `#VALUE!`.
Code: `fn_sumproduct` in crates/rustsheet-engine/src/functions/math_and_trigonometry/sumproduct.rs (the `TODO(T-2450)` comment; replace it) and
the unit test in crates/rustsheet-engine/src/test/mathematical/test_sumproduct.rs:138. Which element error wins when several
arguments hold one is not probed beyond the cells above; keep the first in argument order, then element order.
- gen-m7/f7e94a817e5385ac.case.json A50 `=BYROW({1,#N/A,3},LAMBDA(acc,SUMPRODUCT(Data!B1:C4,acc)))`: A50 Excel #N/A, ours #VALUE!.

## Acceptance criteria
- [ ] Every cell of probes/t2296-sumproduct-error-element passes; its known-failure lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7 f7e94a817e5385ac A50 passes and its line leaves eval/baseline/gen-m7.txt.
- [ ] The `TODO(T-2450)` comment and the test at test_sumproduct.rs:138 are updated to the probed rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMX2MY2 / SUMX2PY2 / SUMXMY2 (unprobed).

## Visible-parity stage
Decision 0083: the gen-m7 cell above has the wrong error, so this task is in the campaign; done when it has Excel's error.

## Outcome
SUMPRODUCT now reads every argument before comparing shapes, so an error element beats a shape mismatch in any order and a whole-argument error beats an element error. probes/t2296 (6 cells) and gen-m7 f7e94a817e5385ac A50 now pass; known failures probes 1903 -> 1896, gen-m7 -1.
