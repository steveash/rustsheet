---
id: T-1270
title: "Probe BAHTTEXT carry (9.995, 999999.999), negative baht+satang and argument count"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BAHTTEXT's satang carry into the baht digits, negative amounts with baht and satang, and a wrong argument count are backed by oracle probes.

## Context
T-1214 implemented BAHTTEXT (functions/text/bahttext.rs). Not probed: 9.995 and 999999.999 (satang round to 100 and carry through 9s), -1234.56, and BAHTTEXT() / BAHTTEXT(1,2). Record findings in docs/excel-semantics/text.md#bahttext.

## Acceptance criteria
- [ ] A probe case with oracle values covers the three questions; the implementation matches, or a fix lands with them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-bahttext-carry` (excel-graph) covers the satang carry (9.995, 99.995, 999999.999, -9.995), -1234.56 and 1000001000000; the implementation already matches (6/6 cells pass, no code change). The argument count (0 or 2 arguments) cannot be probed: excel-graph rejects a workbook with a wrong-arity formula, so it is recorded as UNVERIFIED in docs/excel-semantics/text.md for the M14 desktop oracle. Eval delta: +6 passing cells in probes, no regressions.
