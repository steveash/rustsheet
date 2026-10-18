---
id: T-1845
title: "CUMIPMT and CUMPRINC bound their period loops, then lift over arrays (SPEC-0011)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1822]
---

## Goal
CUMIPMT and CUMPRINC lift over arrays in every argument and refuse a multi-cell range there, as Excel does, without one call being able to loop for an unbounded time.

## Context
Split from T-1822. Their kernels (`fn_cumipmt`, `fn_cumprinc` in functions/financial/mod.rs) sum IPMT/PPMT from
start_period to end_period with no cap of their own: `CUMIPMT(1E-9,1E9,1,1,1E9,0)` loops 10^9 times today, and a lifted
call would multiply that (CLAUDE.md directive 5; SPEC-0011 "Security rule for kernel work"). Excel computes it:
-0.5819767073296636, and `CUMPRINC(1E-9,1E9,1,1,1E9,0)` is -1 (an excel-graph probe during T-1822, left out of the suite
because the engine's loop makes the case time out; add them back as a probe case here), `CUMIPMT(0.01,1E6,100000,1,1E6,0)` is -999900000, and
`CUMIPMT(0.01,1E15,100000,1E15-10,1E15,0)` is -632.3717517805198 (probes/fin-remaining-later-argument-ranges K1-K3,
T-1822), so a cap that refuses them is a DEVIATION; a closed form (total payments minus principal, or the balance
difference) that matches the loop on probes/fin-argument-modes H72/H80 and K1-K3 avoids one. Every argument is probed
over an array (fin-argument-modes H73-H87) and over a range (H74, H82; fin-remaining-later-argument-ranges H65-H84, all
`#VALUE!`), so once bounded both enter functions/lift/kinds.rs as `LiftArrays` in the `// --- Financial ATP` block,
`Weighted` if a value-dependent loop remains. K11 is a lifted element value.

## Acceptance criteria
- [ ] `CUMIPMT(1E-9,1E9,1,1,1E9,0)` finishes in milliseconds; any capped result is a DEVIATION entry in financial.md.
- [ ] The CUMIPMT and CUMPRINC lift cells of probes/fin-argument-modes pass (baseline lines removed), with no new
  failures, and K1-K3, K11 of fin-remaining-later-argument-ranges pass or are listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The base-value ulp misses of H72 and H80 (T-1374).
