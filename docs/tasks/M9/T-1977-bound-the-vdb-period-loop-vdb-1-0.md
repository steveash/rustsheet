---
id: T-1977
title: "Bound the VDB period loop (VDB(1,0,1e12,0,1e12) walks 1e12 periods)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VDB cannot be made to loop for an unbounded number of periods by an untrusted formula.

## Context
Found by the T-1764 security review: `vdb_total_depr` loops once per integer period up to end_period, bounded only by life. Prefer a closed form or a cap (a cap needs a DEVIATION entry, decision 0014). See T-1820.

## Acceptance criteria
- [ ] VDB(1,0,1e12,0,1e12) returns promptly, with a unit test
- [ ] `make check` green; reviewers approve.

## Out of scope
-
