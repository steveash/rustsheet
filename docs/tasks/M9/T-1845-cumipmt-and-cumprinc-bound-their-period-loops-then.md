---
id: T-1845
title: "CUMIPMT and CUMPRINC bound their period loops, then lift over arrays (SPEC-0011)"
milestone: M9
lane: functions-financial
status: done
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
- [x] `CUMIPMT(1E-9,1E9,1,1,1E9,0)` finishes in milliseconds; any capped result is a DEVIATION entry in financial.md.
- [x] The CUMIPMT and CUMPRINC lift cells of probes/fin-argument-modes pass (baseline lines removed), with no new
  failures, and K1-K3, K11 of fin-remaining-later-argument-ranges pass or are listed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The base-value ulp misses of H72 and H80 (T-1374).

## Outcome
- Probed first: probes/fin-cumipmt-period-bound (30 cells, excel-graph, 2026-10-18; the two 1E9-period cells of T-1822
  back in the suite as A1, A2). Excel computes any period count in constant time, also where `(1+rate)^nper`
  overflows (1E308 periods, a rate of 1E10, nper 1E6 over 10 periods), an end past nper is `#NUM!` (also below
  nper + 1, which main computed), and its form is close to `log1p`/`expm1` (financial.md "CUMIPMT and CUMPRINC over
  many periods").
- functions/financial/cumulative.rs: one kernel, `cumulative_of`, for both (moved out of financial/mod.rs). Up to 32
  periods it is main's per-period IPMT/PPMT sum, bit for bit (it matches CUMPRINC_CUMIPMT.xlsx G3, G9, G15, G26,
  which every closed form tried misses); past 32 periods, or when a period's power overflows, an O(1) closed form,
  which passes 19 of the 28 probe cells over many periods bit for bit and the rest within 6 ulps, but for 48-64 ulps
  near period 1E15. Periods are now `f64` (main's `as i32` saturated 1E15 to 2^31-1). `CUMIPMT(1E-9,1E9,1,1,1E9,0)`
  takes microseconds (main: 10^9 steps).
- `LiftArrays` with `Weighted(2^8)` (kinds.rs, `// --- Financial ATP (T-1845) ---`). The weight is the derived bound,
  32 PPMT steps and the closed form, measured in release at its worst (a last step that overflows:
  `CUMPRINC(..,1024,1E307*15.98,1,32,0)`): 2^19 elements in 1.53 s, about 2.9 us an element; 32 full steps of type 1:
  1.51 s; past 32 periods 0.3 us. Both sides of the cap tested (524,288 elements run, 524,289 are `#ERROR!`). The cap
  is financial.md "CUMIPMT and CUMPRINC work cap", a DEVIATION.
- Eval: probes 24 known failures fixed (fin-argument-modes H73-H87 lift cells, fin-remaining-later-argument-ranges
  H65-H83 and K1, K2), 0 regressions; ironcalc-calc, probes-numeric and gen-m9 unchanged. probes and probes-numeric
  compared cell by cell against main: K3 (`#NUM!` → 64 ulps off) and K11 (`#N/IMPL!` → an ulp off) changed value
  and still fail; no other cell changed. The new case adds 8 known failures (A3, A5, A7, A8, A12, A18, A19, A23), all
  listed in T-1890; K3 is T-1890's, K11 T-1374's.
- Ratchets: duplicated lines 4248 → 4152, static_analysis.rs 1181 → 1170 (decision 0010).
- Follow-ups: T-1890 (fit the closed form's last digits; one formulation for every period count).

### Review fix (security, performance)
The per-period sum advanced `period += 1.0`, which stops moving at 2^53, so a sum over a few periods
near 1E16-1E17 with a rate small enough to keep `(1+r)^nper` finite never ended
(`CUMIPMT(1E-15,2E16,1,1E16,1E16,0)`). The loop now runs a bounded integer count (at most 32
steps), the sum is taken only below 2^53 (the closed form otherwise), and the argument checks are
written so a NaN fails them; `periods_past_two_to_the_53_take_the_closed_form_and_end` pins it.
