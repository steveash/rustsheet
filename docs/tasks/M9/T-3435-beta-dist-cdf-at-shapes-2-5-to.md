---
id: T-3435
title: "BETA.DIST cdf at shapes 2.5 to 100 on the BETA.INV round trip: 13 cells where Excel is 4 to 126 ulp off the math and ours is visibly off Excel"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
The 13 BETA.DIST cdf cells below agree with Excel to 15 significant digits. Today ours is 5 to 16 ulp from Excel there,
visibly off at these values near a power of ten. No cell that agrees today is lost.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Each cell is a dense probes-numeric cell inside
  Gate A's envelope (SPEC-0029: ours is within 1.3E-14 of the 50-digit math on all 13), and Excel itself is off the math
  on every one, so this is reproducing Excel's last digits, not accuracy. A point that Gate A's tables show outside the
  envelope gets its own task without the key.
- Gap triage 2026-10-31 (`make eval` on origin/main 4ff408243, excel-graph). `scripts/exit_owners.py --all-off` lists
  these cells with no open owner. probes-numeric/t1745-beta-inv-roundtrip feeds Excel's own BETA.INV result x
  (column E) and its neighbouring doubles (D, F) to `BETA.DIST(.,a,b,TRUE)` (G, H, I) as constants, so the input is
  exact and the miss is BETA.DIST's. Signed ulp of Excel and of ours from mpmath at 50 digits, then ours minus Excel:
  - (a, b) = (2.5, 100): G96, H96 (p = 1E-10): Excel +11, ours -5, diff -16 (both).
  - (10, 100): G130 (1E-10): Excel -17, ours -12, diff +5.
  - (100, 2.5): G151 (0.001): Excel -6, ours +1, diff +7.
  - (100, 10): H158 (0.001): Excel -5, ours 0, diff +5. I159 (0.05): Excel +13, ours 0, diff -13.
  - (100, 100): G164 (1E-10): Excel -97, ours -87, diff +10. H164 (1E-10): Excel -126, ours -119, diff +7.
    H165 (0.001): Excel -31, ours -22, diff +9. G166 (0.05): Excel +4, ours +12, diff +8. I166 (0.05): Excel -15,
    ours -6, diff +9. H167 (0.5, x = 0.5): Excel -1, ours +4, diff +5. I167 (0.5): Excel +4, ours -1, diff -5.
- At (100, 100) Excel and ours share most of a 90 to 126 ulp error at p = 1E-10. That points at a common factor, such
  as the prefactor or ln B at large shapes (statistics.md "Incomplete beta L140 and bfrac": brcomp = Excel's
  density·x·y). The remaining 5 to 10 ulp are the branch's own arithmetic. Check which bratio branch each cell takes
  (`bratio::dispatch`; a0 = b0 = 100 is bfrac's b0 >= 40, (2.5, 100) and (100, 2.5) may be bgrat or bup) before
  starting. If the cells belong to an open branch task (T-3323 bfrac, deferred; T-3322 basym needs shapes above 100),
  move them there and close this one.
- (2.5, 100), (10, 100), (100, 2.5) and (100, 10) swap to a0 = 100 with b0 from 2.5 to 10. That is the region of the
  T-3400 sweep and T-3401 (L140 bup+bgrat in the bulk; both on branch agent/functions/tasks-refill-opus when this was
  filed). T-3401 is the accuracy fix where Excel is close to correctly rounded, and it moves the part where Excel is
  as far off as the engine to a `visible` follow-up. The cells here are of that second kind (Excel 5 to 17 ulp off
  the math), so this task is that follow-up for them. Read T-3400's sweep and T-3401's Outcome first.
- Not in this task: G1 of the same case, `BETA.DIST(2.4674011002723394E-20,0.5,0.5,TRUE)`. There Excel is the
  correctly rounded 9.999999999999999E-11 and ours is 12 ulp below, an accuracy miss in bpser at a = b = 0.5. T-3199
  owns G150, H150, G158 and G159 (half-integer shapes), and T-3192 owns G145.

Owned cells (one line, all BETA.DIST cdf):
- probes-numeric/t1745-beta-inv-roundtrip.case.json (13): Calc!G96, H96, G130, G151, H158, I159, G164, H164, H165, G166, I166, H167, I167

## Visible-parity stage
Visible (decision 0114): each of the 13 cells is within half a unit of the 15th significant digit of Excel's value.
Bit-level criteria still unmet then move to a follow-up.

## Acceptance criteria
- [ ] Each owned cell is within half a unit of Excel's 15th digit, or the Outcome names it with the (non-deferred or
      Phase 1b) task that owns its branch.
- [ ] The reviewer-excel-fidelity check 7 guards (decision 0188 R4) are in the Outcome for any fitted form:
      - train and hold-out counts from a deterministic ~30% split by cell hash over the branch's cells in probes-numeric
        (t1745-beta-inv-roundtrip, m9-beta-dist-a100, t3249-beta-large-shape-fit and -hold);
      - a fresh log-uniform sweep filled after the model is frozen, with its result;
      - the Gate A envelope result;
      - each fitted constant tagged `/// FITTED(T-3435, <n> cells, <range>)` and not applied outside that range;
      - no fitting below the reproducibility floor (decision 0112: web against desktop up to 4 ulp);
      - no branch that only keeps already-matching cells.
- [ ] No BETA.DIST, F.DIST, T.DIST or BINOM.DIST cell of a gated suite goes from bit-exact or within 15 digits to farther
      off. No new known-failure line in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- G1 (accuracy, bpser; filed with T-3192). The half-integer tail (T-3199). bfrac's bit-level scores (T-3323).
- The BETA.INV column J of the same case (T-3398).
