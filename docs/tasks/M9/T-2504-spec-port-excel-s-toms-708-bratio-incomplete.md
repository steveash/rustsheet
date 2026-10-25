---
id: T-2504
title: "Spec: port Excel's TOMS 708 bratio incomplete beta behind BETA.DIST, F, T, BINOM and NEGBINOM (T-2464 E3a)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0019-toms-708-bratio-incomplete-beta-port.md
---

## Goal
A spec in docs/specs/ for replacing the engine's incomplete beta with a port of Excel's TOMS 708 `bratio`, landed one
branch at a time. CLAUDE.md directive 4 requires it: the change is architectural, touching five function families and
two existing kernels.

## Context
- T-2464 research, step E3a of docs/research/t2464/r2-impact/RESULTS.md section 4. The rules are in
  docs/excel-semantics/statistics.md, the T-2464 entries starting at "Incomplete beta is TOMS 708 `bratio`".
- The reference model is `docs/research/t2464/r2-bratio/bratio_excel.py`: `kernel(a, b, x, y, cfg)` returns
  (w, w1, branch). r2-bpser-small `lib.py` supplies the bpser ln B, and r2-tdist `xlk.py` supplies bgrat at b = 1/2.
- Today's engine:
  - `incomplete_beta.rs`: Cephes fractions, a switch at the mean, 1 - v;
  - `beta.rs`: the BETA.DIST cdf on `incomplete_beta::beta_reg` since T-2119, BETA.INV at its crossing;
  - `student_t.rs`: its own composite;
  - `binom.rs` and `discrete.rs`: a pmf walk;
  - `inverse_beta.rs` and `fisher.rs`: inverses on statrs;
  - `fisher.rs`: F's x = d1X/(d1X + d2).
- r2-impact finding 2: a plain TOMS port is net negative outside BETA.DIST (-103 visible on 10,602 cells), so each
  bratio branch moves only together with its Excel piece. Branches not yet ported delegate to today's kernel.
- Gated cells a full port would move (r2-impact `branches.out`):
  - 6 lost bit-exact: probes/stat-argument-modes F.DIST H215, F.DIST.RT H221, FDIST H226 (L140); gen-m9 F.DIST.RT
    0a614cfe A19, FDIST 05d44f6d A175 and FDIST c18dbf21 A151;
  - 2 that become visibly off: F.DIST.RT 60b0b6e6 A151 and FDIST 2801c1c1 A374;
  - the chained BETA.INV cells T-2119 left (decision 0111; owned by main's T-2633), which any kernel change moves.

## Acceptance criteria
- [x] A spec docs/specs/SPEC-NNNN-*.md (from `docs/specs/TEMPLATE.md`, next free number) covers:
  - the module layout and the K(a, b, x, y) → (w, w1, branch) signature;
  - each caller's argument forms (statistics.md "Incomplete beta is TOMS 708 `bratio`", "Student t tails are one
    kernel call");
  - the migration rule (a branch moves with its piece, and the rest delegates);
  - bounded loops, with no panics and no unbounded allocation (directive 5), and Excel's #NUM! and saturation edges;
  - x87 storage helpers reused, not copied (`arithmetic::extended_*`, `x87_register`);
  - test fixtures copied from docs/research (decision 0066);
  - how the gated regressions above are handled (decision 0083's allowance, or a decision record).
- [x] The spec maps each of T-2502, T-2505, T-2506, T-2508, T-2509, T-2510 and T-2511 to its module and branch.
- [ ] Reviewers approve.

## Out of scope
- Any engine code.

## Outcome
- docs/specs/SPEC-0019-toms-708-bratio-incomplete-beta-port.md (draft until reviewers approve). It contains:
  - the module layout: `incomplete_beta/` with `closed`, `bpser`, `ln_beta`, `bup`, `bgrat`, `bfrac`, `density`,
    `toms` and `legacy`; Excel's bd0 in `loader.rs`;
  - K = `bratio(a, b, x, y) -> Tails { w, w1, branch, swap }`, with `dispatch`, `evaluate` and `tails_for(Caller, ..)`;
  - each caller's argument forms and its delegate;
  - the migration rule, as a (caller, branch) port table whose untaken pairs run today's code bit for bit;
  - which task deletes each T-2502 seam;
  - directive 5 caps, and the `#NUM!` and saturation edges;
  - the gated cells per task: decision 0083, or a record for gen-m9 60b0b6e6 A151, 2801c1c1 A374 and c18dbf21 A151;
  - the port rules: probes-numeric main-vs-HEAD cell comparison, 15-digit literal truncation;
  - a test and evidence plan per branch, and the Rollout ordering.
- Task files: `spec:` line on T-2503, T-2505 to T-2511 and T-2585. Criteria amended in T-2505 (seams, harness, the
  visible gated cell, the layout and `rustsheet-eval diff`), T-2506, T-2510 and T-2511. The port-rule criterion was
  added to T-2505, T-2506, T-2508 to T-2511 and T-2585. T-2503 and T-2507 are in progress elsewhere, so they got only
  the `spec:` line.
- depends_on: T-2633 `[]` → `[T-2505]`, since its cells move with the port. Every other port edge checked and kept.
