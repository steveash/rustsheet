---
id: T-3587
title: "Math sidecars for the probes suite and the probes-numeric cases filed since T-3380 (233 probes cells become Phase 1b evidence)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Every spec case of `probes` and `probes-numeric` with a one-call cell of a function that has a Gate A definition
(today F.DIST, F.INV, RATE, T.DIST, T.INV, T.INV.2T, TINV) has its committed `<case>.math.json`, so decision 0188
R1(c) reads those cells against their 50-digit math instead of counting them as "no math value".

## Context
- T-3380 wrote sidecars for 16 probes-numeric cases only. `rustsheet-accuracy math` has never been run on `probes`
  (0 sidecars there), and probes-numeric cases filed after T-3380 have none. A visibly-off number with no math value
  counts in the R1(c) rows (crates/rustsheet-eval/src/r1c.rs), so these cells inflate the M9 probe row and the
  `make unowned-off ALL=1` list, and gap triage keeps finding them as "unowned".
- Gap triage 2026-11-01 (origin/main 408444f20) ran both suites in a scratch worktree:
  `cd tools && uv run rustsheet-accuracy math --suite probes-numeric` then `--suite probes` (about 2 min 20 s). That
  wrote 3 new probes-numeric sidecars (T-3348's two Gate A RATE sweeps and T-3478's F density sweep) and 14 probes
  sidecars (RATE, statistical-argument and F/T probes, among them T-3354's tiny-x F.DIST case with 222 of the cells).
  The case ids are not repeated here on purpose: a task line naming a case with no cell makes the owner check
  (scripts/exit_owners.py) count this task as the owner of all its cells, and this task fixes no cell value.
- `rustsheet-eval run --suite probes --suite probes-numeric` with them (no `--desktop-owned`): probes 904 counted and
  233 Phase 1b (was 1137 counted and 0 Phase 1b without the M14 split); probes-numeric 38687 counted and 32121 Phase 1b
  (was 38736 and 32072; 45 F density cells and 4 RATE cells move).
- T-3478's three r4-wide sweeps carry a hand-made `.math.tsv` that the runner does not read (it reads only
  `.math.json`, `accuracy::math::math_sidecar`). Only the F density one gets a `.math.json` from the tool; IRR has no
  definition yet (T-3535). Leave the `.tsv` files alone (the fit lab reads them).
- Tool: tools/src/rustsheet_tools/accuracy/eval_math.py (deterministic output, T-3380).

## Acceptance criteria
- [x] `uv run rustsheet-accuracy math --suite probes-numeric` and `--suite probes` are run on current main and their
      new sidecars committed; running both again changes no byte.
- [x] `make eval` report: the line "Visibly-off cells under decision 0188 R1(c)" shows the probes Phase 1b count up by
      about 233 and the probes-numeric one by about 49 against main, and the PR quotes the before and after lines.
- [x] No verdict changes: `make eval-baseline` records nothing (sidecars add a field only).
- [x] `make check` green; reviewers approve.

## Out of scope
- A check that keeps sidecars current as new cases land (T-3588).
- New Gate A definitions (T-3529, T-3535, T-3549, T-3553, T-3555 regenerate their own functions' sidecars).

## Outcome
`rustsheet-accuracy math --suite probes-numeric`, then `--suite probes`, on main 4e2f5f3df (2 min 26 s) wrote 16
sidecars:
- 15 in probes, among them T-3354's tiny-x F.DIST case (2,413 lines);
- 1 in probes-numeric (T-3567's F density hold-out).

T-3348's two RATE sweeps and T-3478's F density sweep already had theirs on main. Running both again changed no byte
(sha1 of all 16 files checked). `make eval-baseline` recorded nothing.

The R1(c) line of `make eval`, before and after:
- before: `probes 1086 counted, 0 Phase 1b evidence, 0 R3 passing, 51 M14-owned; probes-numeric 39222 counted, 32121
  Phase 1b evidence`;
- after: `probes 853 counted, 233 Phase 1b evidence, 0 R3 passing, 51 M14-owned; probes-numeric 39103 counted, 32240
  Phase 1b evidence`.

That is +233 in probes, as predicted, and +119 in probes-numeric (about 49 predicted; the T-3567 case filed since
adds the rest).

Effect on the exit: `make exit-owners MILESTONE=M9` went from 346 unowned exit cells to 215, and the M9 probe row lost
the 131 t3354 cells. The 215 left are rate cells, which decision 0211 counts whatever the math says:
- F.DIST 211: t3354-f-dist-tiny-x-huge-d2 131, probes-numeric t3354-f-dist-sweep 42, t2220-f-dist-sweep-d2-1 38;
- DSUM 4.

Their owners are reported separately.

