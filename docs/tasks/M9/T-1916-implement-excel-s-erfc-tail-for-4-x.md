---
id: T-1916
title: "Implement Excel's ERFC tail for 4 < x <= 26.55 from the T-1778 rational"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
ERFC and ERFC.PRECISE for 4 < x <= 26.55 reproduce Excel's bits, from the tail model T-1778 found:
ERFC(x) = fl53(fl64(E*R)), with E = Excel's EXP(-fl53(fl64(x*x))) and R = fl53(fl64(N/D)), where N
(degree 8) and D (degree 9) are Horner polynomials in x with every multiply and add stored to double.

## Context
- The model is in docs/research/t1778-erf-kernel-research-plan.md, research updates 4 and 7, and the
  status table in update 8. It is bit-exact on 1,540,475 of 1,540,477 probed points. The two misses are
  inputs Excel parsed one ulp off from the xlsx text.
- E is Excel's own EXP, not glibc's. Model V6 (update 8) matches 2,139,591 of 2,139,684 EXP cells. The
  emulation is in docs/research/t1778/round7/exp/ (`exp_ref.py`, README) on the research branch.
- The coefficients are only on the research branch `origin/agent/research/T-1778-kernel-map`, in
  `docs/research/t1778/round7/erfc_tail_coef.txt`, so they have to land on main first.
- The engine today is `erfc_cf` in crates/rustsheet-engine/src/functions/statistical/special.rs, a
  bottom-up continued fraction with a compensated exp(-x*x). That family is ruled out
  (statistics.md, "ERFC core factor: candidates that do not reproduce Excel (T-1187)"). Its cutoff
  `ERFC_UNDERFLOW` (27.3) and the subnormal flush (T-1180) stay as they are.
- Cells: probes/subnormal-function-boundaries A5 (`ERFC(26.4)`, T-1186), the x > 4 points of
  probes-numeric/t1040-erfc and t1040-erfc-precise, and the tail lines of T-0968.
- Mistake in the research plan: update 8 names this engine work "T-1780". T-1780 on main is "Bound
  HSTACK and VSTACK results by the shared array limit" (done), so that reference should point here.
  The plan is owned by the T-1778 session. Fix the id there or through the steward, not in this task.
- Do not change [0.5, 4]: its (9,10) rational's exact doubles are still open (update 9).
- Kernel map rank 2 (docs/research/numeric-kernel-map.md, decision 0056).

## Acceptance criteria
- [ ] The tail coefficients (`erfc_tail_coef.txt`) and the EXP V6 description are on main under
  docs/research/t1778/, copied from `origin/agent/research/T-1778-kernel-map` with their provenance. If the
  T-1778 session has already merged those files, use main's copy instead of copying.
- [ ] `erfc` uses the T-1778 tail for 4 < x <= 26.55, with an Excel-EXP helper for E (reuse an
  existing x87 or fit-lab helper if one exists in the engine; search first). Every constant has a
  comment citing the research plan.
- [ ] probes/subnormal-function-boundaries A5 passes, and the t1040-erfc and t1040-erfc-precise points
  with x > 4 are bit-exact. `make eval-baseline` shows no regression in ERFs.xlsx, NORM_DIST.xlsx,
  GAUSS.xlsx, LOGNORM.xlsx or the probes suites. T-1186 is then done.
- [ ] Each cell that still misses is listed with its ulp error, and the finding is in
  docs/excel-semantics/statistics.md.
- [ ] Excel's EXP (T-1778 model V6, x87 F2XM1, with its evidence) has its own docs/excel-semantics entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ERFC on [0.5, 4] and ERF (T-1778's open pieces).
- NORM.S.DIST and other callers beyond what the shared `erfc` gives them (T-0968).
