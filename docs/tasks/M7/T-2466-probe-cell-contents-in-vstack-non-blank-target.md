---
id: T-2466
title: "Probe CELL('contents') in VSTACK: non-blank target, HSTACK, upper-case info type, beside an error argument"
milestone: M7
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for how far the `VSTACK(CELL("contents",ref),..)` `#VALUE!` (T-2457) reaches.

## Context
T-2457 matches the literal CELL call syntax; every probed target was blank. See docs/excel-semantics/information.md. Code: `is_cell_contents_call` in hstack_vstack.rs.

## Acceptance criteria
- [x] Probe case with oracle values for: a non-blank target, HSTACK, an upper-case info type, CELL beside an error argument, and CELL wrapped in LET/IF; findings in information.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings (file separate tasks).

## Outcome
probes/t2466-cell-contents-stack (25 questions, excel-graph 2026-10-28): the VSTACK/HSTACK `#VALUE!` follows a **blank** target,
not the call's syntax. Non-blank, text, formula and range targets stack their value; a blank target errors the whole stack for
VSTACK and HSTACK, any info-type case, also through LET, IF and an info type read from a cell; other info types (`"type"`) stack;
beside a scalar error argument the first erroring argument wins. Recorded in docs/excel-semantics/information.md. Ours has 27
known failures (new known failures in eval/baseline/probes.txt), including a separate bug: `CELL("contents", non-blank)` nested in
HSTACK, TOCOL, `+0` or LET/IF inside VSTACK is 0. Follow-up T-3030 (implement). Eval delta: +1 case, 27 known failures.
