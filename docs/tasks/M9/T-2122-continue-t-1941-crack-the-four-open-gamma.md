---
id: T-2122
title: "Continue T-1941: crack the four open gamma kernels to the bit"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1941]
defer: phase-1b
---

## Goal
Bit-exact models (structure, op order, x87 rounding, coefficient doubles) for the four kernels T-1941 left open:
K1 = GAMMA's G(y) on [1,2), K2 = GAMMALN on (2,3], K3 = GAMMALN near 1 on [0.7,1.5), and GAMMALN on (1.5,2].
Every other piece of GAMMA, GAMMALN and LN is already bit-exact or reduces exactly to these four.

## Context
- Restart file: `docs/research/t1941/HANDOFF2.md` (read §12 first: final worker states and pickup steps) on branch `agent/research/T-1941-crack-gammaln-gamma-ln`
  (self-contained: solved pieces, best models, the method that took K2 from 0.74 to 0.95, traps, dead ends,
  ranked hypotheses, data inventory). The chronological log is `HANDOFF.md` (sections 1-12.20) and `README.md`.
- Best models at handoff (exact / within 1 ulp vs Excel):
  - K2: `k2_ref.py`, 0.950 / 0.978. Reviewer 3's stored cand9 is 0.993 near 2 (HANDOFF2 §12.1). Structure: stored
    is favoured by the near-2 ladders; the far-only transfer control is still to run.
  - K1: `w4/k1r/plain1.txt`, 0.776 / 0.878. The [1.8,2) dip is 0.50 and looks like a coefficient problem.
  - K3: `w5/p5` / `w5/p4/ss_fc.txt`, 0.33. Ladder probes `kmyst-gam-k3l*` are filling, with an unwrap tool ready.
  - (1.5,2]: W6's chain in `w6/data/`, 42.7% exact on R.
- Engine shipping of the current best models was T-2123, split into T-2128 (GAMMALN) and T-2129 (GAMMA); `gamma_kernels.rs` holds the coefficient arrays to swap.

## How to run (restart prompt)
Paste this to a fresh coordinator session (Opus, auto mode), on the research branch above:

> You coordinate T-2122 on branch `agent/research/T-1941-crack-gammaln-gamma-ln` (research only; never
> force-push; commits `T-2122: ...` with the Co-Authored-By / Claude-Session trailers; never print oracle
> credentials). First read `docs/research/t1941/HANDOFF2.md` completely, then skim `HANDOFF.md` §12.11-12.20.
> Spawn five sessions with `create_session` on the same branch:
> - W-K2 (Opus): HANDOFF2 §3.2 / §7.1 / §12.1. Reviewer 3 argues K2 is plain stored and the ~5% residual is a
>   per-run offset c3 at N.s3. Have the coordinator fill the §12.1 micro-ladders. Unwrap c3(x) and identify
>   its law. Run the far-only-refit transfer control. Then refit (near-2 constraint, cluster bit-match plus
>   bias rms) so that one set beats both cand9 near 2 and k2_ref far from 2.
> - W-K3 (Opus): §3.4 / §7.2. Unwrap the k3l ladders with `w5/p5/ladder.py`, solve per-stage slopes for
>   coefficient moves (c·g head-move shape), peel, refit. Request 6 more anchors if the N slopes reach ~1e6 cycles/x.
> - W-K1 (Opus): §3.3 / §7.3. Dip-weighted null-lattice search from plain1 and the dip-only set. Then the D P2
>   line unwrap in [1.8,2); specify `kmyst-gam-k1l*` ladder probes if the dense runs don't unwrap.
> - W-15 (Opus): §3.5 / §7.4. Run the noise fingerprint (`w5/fp/`) on (1.5,2]. Then alias scans at N.P3 and D.P1
>   with the correct periods, then per-stage phase laws and the c·g head-move recipe.
> - Reviewer (Fable/strongest): critique each worker's claims against §5 (traps). Demand planted controls and
>   holdouts. Propose structural forms when a worker stalls.
> Rules for everyone: planted control before any real-data claim. Hold out (alternate clusters, long runs,
> near-2 ladders). Never treat a mask gain as structure evidence. Never normalise by a non-power-of-two D0.
> Compensate head moves at the next constant only (late chain fixed). Select oracle columns by formula.
> Coordinator duties:
> - fill probe requests with the oracle (`cd tools && uv run rustsheet-oracle fill -b excel-graph --suite
>   probes-numeric --case probes-numeric/<id>.case.json ...`, ≤ 16,384 rows per case, commit landed oracles per
>   batch);
> - commit worker scripts and coefficient files under `docs/research/t1941/`;
> - append dated sections to HANDOFF2.md;
> - route results between workers;
> - verify every claimed model yourself on the 14 long runs / dense clusters before committing it;
> - keep a ~40-minute self check-in;
> - when a kernel goes bit-exact (≥ 99.9% with only documented x87 near-ties), update `gamma_kernels.rs` (a new engine task) and
>   `docs/excel-semantics/statistics.md`.

## Progress
- Run 1 (2026-10-22, one opus worker without sub-sessions): HANDOFF2 section 13. It filled every probe left from the
  pause and added K2 micro-ladders (20 cases) and K3 rungs 27-30 (48 cases). K3's true offsets are ~1e10 ulps: the
  slopes were aliased twice, the 12-anchor KMAX=30 ladder is validated by a 1e10 control, and the real joint solve
  still needs the N6-first conditional solve (13.4). K2's per-run c3 is a sawtooth, and every single-op linear law is
  refuted (13.3). K1 and (1.5,2] were not reached. Pick up from HANDOFF2 13.4 / 13.3 on the research branch.
- Run 2 (2026-10-22, one opus worker, ~40 min, K3 only): HANDOFF2 section 14, `t2122/k3_n6first.py`. N6 is one
  smooth law at 8 of 9 anchors (0.9517 aliased by about 2^28). Anchors a-f alone are fully consistent under a joint
  N6+N7 fit: rms 2.4e4 cycles/x, head move N7 +4.0e8, N8 -4.61e10, N9 +3.50e12 ulps. The g-l anchors pull N7 by
  about 2e8 ulps. The joint fit fails its planted control (no alias freedom on N6 rows), so fix it first (14.2 step
  1), then score the a-f head move on rows6 (14.2 step 2). No oracle cases were added.
- T-2513 evidence (2026-10-24, no run): the beta density M2 (T-2503) needs Excel's GAMMALN exact at its shape
  arguments, and below 1 the engine model is not. Probe probes-numeric t2513-r3-m1-residual-gl holds Excel's GAMMALN at
  569 shape arguments (`docs/research/t2464/r3-m1-residual/mkprobe_gl.py`, `probe_gl.args.json`): the engine differs on
  14 of 47 arguments below 1 (0.71: -2 ulps, 0.77: +2, 0.99: +2). With t2513-verify-r3-m1-residual-hold's 54 more, the
  verifier counts 24 of 66 below 1 and 19 of 541 from 1 on (verify-r3-m1-residual `vchecks.out`). Below 0.7 that is
  fl(GAMMALN(fl(x + 1)) - LN(x)), so these are K3 and (1.5, 2] points; use them as extra test points. Cost today: the
  M2 density's bit-exact hold-out counts drop from 3,106 to 2,804 of 4,096 and from 649 to 562 of 864.

Worksheet cells K3 owns (moved from T-0973, 2026-10-23; known failures in `eval/baseline/gen-m9.txt`, ulp-level): gen-m9
`GAMMALN.PRECISE` at 40425341c5034eed Calc!A115 (x = 1.0014973824124909), 8f2ac25945af87da A97 (0.8172501166257935),
b6593d7dd13f2dac A124 (0.9999943310657596), b8ee55934c5b824c A153 (0.8708129562778496), f2bd5f380ec002bf A241
(0.8447659700885523). K1 owns probes-numeric/t1042-gamma-negatives-edges A5 `GAMMA(-0.1)` (Excel -10.686287021193195, ours
-10.686287021193193) and A7 `GAMMA(-1.1)` (Excel 9.714806382902898, ours 9.714806382902896); that case is ungated.
They are test points for the engine swap of `gamma_kernels.rs`.

## Acceptance criteria
- [ ] Each of K1, K2, K3 and (1.5,2] has either a bit-exact model (≥ 99.9% of all probed points, misses
      only at documented x87 near-ties) committed with its verifier, or an updated best model plus a measured
      ceiling and the remaining hypotheses in HANDOFF2.md.
- [ ] Every new probe case is filled and committed. Coefficient files and scripts are under `docs/research/t1941/`.
- [ ] Quirks recorded in `docs/excel-semantics/statistics.md` ("GAMMA, GAMMALN and LN kernels"). Lessons added
      to `docs/research/excel-kernel-playbook.md`. An engine task is filed to swap `gamma_kernels.rs`'s arrays.
- [ ] The K3 model's score includes the two gen-m9 GAMMALN.PRECISE cells added by gap triage 2026-10-30.
- [ ] `make check` green; reviewers approve. Land notes, coefficient files and verifiers on main in a slim PR, as the T-1941
      close-out did. Bulk probe data (`kmyst-gam-*`, gigabytes) stays on the research branch.

## Out of scope
- Engine implementation (T-2123, T-2128, T-2129 and follow-ups).
- Incomplete gamma / beta kernels (T-0971, T-0970).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: two gen-m9 GAMMALN.PRECISE cells, 1 to 3 ulp off and none visibly off, that no open task names. Both arguments fall in K3's range: gen-m9/b8ee55934c5b824c A153 `GAMMALN.PRECISE((SQRTPI(Data!C3)-0.1))` with Data!C3 = 0.3 (argument about 0.871), Excel 0.08924278597232677, ours 0.08924278597232672; gen-m9/f2bd5f380ec002bf A241 `GAMMALN.PRECISE(ERF(Data!C4))` with Data!C4 = 1.005 (argument about 0.845), Excel 0.11110042471843157, ours 0.11110042471843155. Score K3 candidates on them too; the engine task this one files to swap `gamma_kernels.rs`'s arrays closes them.
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A153
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A241
