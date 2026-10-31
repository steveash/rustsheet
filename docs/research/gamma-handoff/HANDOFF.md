# Gamma family: handoff (2026-10-31)

Handoff from one coordinating session to the next. The previous session's link is
https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a.

**Scope:** the gamma family in RustSheet's statistical functions:
- GAMMA, GAMMALN(.PRECISE);
- GAMMA.DIST/GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT/CHIDIST;
- GAMMA.INV/GAMMAINV, CHISQ.INV, CHISQ.INV.RT/CHIINV;
- POISSON/POISSON.DIST.

The code lives in `crates/rustsheet-engine/src/functions/statistical/`: `gamma.rs`, `chisq.rs`, `poisson.rs`, `gammaln.rs`, `loader.rs`, and `incomplete_gamma/`. That last directory holds `mod.rs`, `closed_form.rs`, `density.rs`, `huge_shape.rs`, `fraction.rs`, `half_integer.rs` and `excel_tail.rs`.

## Read first
- `CLAUDE.md`. Prime directive 1 now carries the decision 0188 R3 exception.
- `docs/decisions/0188-*.md`. Phase 1 exits on Gate A (accuracy) and Gate B (user-representative suites). R4 is the binding anti-overfitting rule for any fit:
  - a hold-out split;
  - a fresh sweep, run after the fit is frozen;
  - a `FITTED(T-XXXX, n cells, range)` tag on every fitted constant;
  - no fitted piece used outside the range it was measured on.
- `docs/excel-semantics/statistics.md`, the canonical quirk record for these functions.
- `.claude/skills/work-loop` and `.claude/skills/steward`. **Merge mode is `train`** (owner instruction): once a PR has all four reviewer approvals and green CI, run `make mark-ready PR=<n> REVIEW="security=APPROVE architecture=APPROVE quality=APPROVE excel-fidelity=APPROVE"` and move on. Never take the merge lock or merge yourself. If the train ejects a PR (label `merge-train-ejected`), read the merger's comment, fix the cause, push, wait for green, and mark it again.

## Owner rulings given in the previous session (2026-10-31)
- **0188 R3/R4 apply.** Excel is matched except where decision 0188 R3 lists a MATH-OVER-EXCEL case.
- **"Allow, with records."**
  - T-3311 ships its 1E-6 cut. Gated gen-m9 c18dbf21f11093f3 A107 goes +1 ulp, recorded in decision **0210**, which was renumbered from 0197 after a collision.
  - T-3285 ships the accurate kernel for every x ≥ a+1. The cells this costs are recorded in decision **0202**.
- **"Ship, extend ruling" (twice).** T-3269's far-tail fit ships. Decision **0203** names 9 cells plus 19 ungated probe cells under "Second extension".
- **Standing rule** (the earlier "net gain, listed losses" ruling): no cell may become visibly off, or change type or error, unless Excel agrees or the owner rules on the named cells. Any new loss outside a ruling goes to the owner as a question. Never claim a ruling the owner did not give.

## In the merge train (green, marked ready, waiting for the merger)

| PR | Task | What it does |
|---|---|---|
| [#2599](https://github.com/steveash/rustsheet/pull/2599) | T-3311 | Deep lower-tail gamma quantiles: Newton steps from Excel's leading-factor root on Excel's staircase cdf. Includes T-3313's move to `closed_form.rs`. Decision 0210. |
| [#2612](https://github.com/steveash/rustsheet/pull/2612) | T-3285 | Half-integer shapes: accurate double-double Q for x ≥ a+1 (`half_integer.rs`, `fraction.rs`). Decision 0202. |
| [#2629](https://github.com/steveash/rustsheet/pull/2629) | T-3269 | Stacked on #2612. Far-tail form (Stirling-ratio prefactor × fraction) at half-integer shapes 3/2..11, x > 30 (`excel_tail.rs`). Decision 0203. |

**What to do with them:**
- Watch them, and re-mark any the train ejects.
- Common causes of ejection:
  - **Task or decision id collisions with main.** Renumber with `make new-task` / `make new-decision` and update every reference.
  - **A stale `quality/duplication.json` after main moves.** Merge origin/main, run `python3 scripts/check_duplication.py --update`, and commit. This is legitimate only when the merged tree measures no more than main's own lock.

## In progress when the session ended
All three branches are pushed. Read each task file's "Handoff" section first.

1. **T-3509: the permanent gamma-family held-out ("blind") probe.**
   - Branch `claude/hopeful-meitner-5oxf0g-blind-probe`, head e0cb68119.
   - **Done; `make check` is green.** Next: run the 4 reviewers, open the PR, and run mark-ready.
   - What it holds:
     - 4,100 cells over 6 cases: `eval/suites/probes-numeric/cases/t3509-blind-*`, each with case, oracle and math sidecars.
     - A generator: `docs/research/t3509/sweep.py`, seed 20261031. `USED_SEEDS` refuses a seed that was already filled.
     - A per-region report in `docs/research/t3509/score.py`, with its README.
     - Gate A definitions for the gamma family in `tools/src/rustsheet_tools/accuracy/defs/gamma.py`.
   - **Never fit on it.** Score every fit change on it before and after. Refresh it with a new seed.
   - Tasks already filed from its results:
     - T-3519: GAMMA.INV at p = 0.5, shape < 0.5.
     - T-3520: GAMMA.INV deep lower tail at shape 0.5–20; it depends on T-3311.
     - T-3521: CHISQ.DIST.RT at shape ≥ 20, x/df 1.4–10.
     - T-3522: POISSON pmf at k > 1000 near the mean.
     - The GAMMA.DIST shape 2.5–20 evidence was added to T-3219.
   - Regions with no owning task yet: GAMMA.DIST at shapes 0.5–2.5 and 20–1E3 with low x/a (37 cells), and the POISSON cdf near the mean (5 cells). File tasks for them.
2. **T-3505: GAMMA.INV returns #NUM! at shapes about 7.9E6 to 8.9E7.** P0, `campaign: semantics`.
   - Branch `claude/hopeful-meitner-5oxf0g-T-3505`, head 714012f93, WIP.
   - **Root cause:** the shared directive-5 term budget runs out. About 25 power-series cdfs, each about 7.3·sqrt(a) terms, exceed 2^19.
   - The probe `probes/t3505-gamma-inv-budget-band` is filled (525 cells; 120 are #NUM! on main).
   - **Duplicate of T-3350** (PR #2597, already in the merge train). Plan: once #2597 merges, merge origin/main and take main's side in `incomplete_gamma/mod.rs` and `special.rs`, dropping this branch's fix. Then keep the probe, run `make eval-baseline`, add the statistics.md note and a unit test for `GAMMA.INV(0.5,7990464.81598859,1)` = 7990464.482655259, review, and open the PR.
   - If #2597 does not land, finish this branch's own fix instead: a Temme start plus a 2^-50 first step. It changes no other cell.
   - Also noted: six p = 1E-300 cells where Excel, not we, is off from the math (the T-3294 area).
3. **T-3530: audit of the 12 thinly evidenced FITTED constants.**
   - Branch `claude/hopeful-meitner-5oxf0g-fit-audit`, head 47785ed06, WIP. It has no engine changes.
   - The report is in `docs/research/fit-audit/README.md`. There are 12 probe cases, `probes-numeric/fit-audit-*`, 579 cells.
   - `fit-audit-t3271-cdf-cut` is still pending an oracle fill, because createSession failed.
   - **Refuted: four `#NUM!`-boundary rules, which return the wrong type today.**
     - `HUGE_SHAPE_DENSITY_CUT`: Excel returns 0, not #NUM!, at shapes 1E10–1E15. Its cut is on x/shape, somewhere between 2.17E-309 and 1E-308.
     - `HUGE_SHAPE_MIN`: Excel is already #NUM! at shapes 6E8–9E8 at tiny p.
     - `BAND_UPPER_EDGE` and `BAND_LOWER_EDGE`: these misclassify cells both ways in the huge-shape GAMMA.INV band.
     These are Gate B/semantic errors. Next: file follow-ups for the density cut and for re-fitting the huge-shape GAMMA.INV band, run second-round probes, and fix them.
   - **Confirmed:** the huge-shape closed-form density cut is Excel's own boundary (verdict d). Its tag should become 61 cells.
   - **Provisional, not yet scored on both sides of the cut:** `LOG_FORM_SHAPE_MAX`, `LOG_FORM_SHAPE_MIN`, `RATIO_FORM_MIN`, `CLOSED_FORM_QUANTILE_MAX`, `CLOSED_FORM_STEP_MIN` and `SMALL_RATIO_SHAPE_MIN`. Scoring them needs a scratch build that reads its cut-points from the environment; never commit that build.

## Next work, in priority order
1. Get the three train PRs merged. Review the T-3509 blind-probe branch and open its PR. Resolve T-3505 against T-3350 (#2597).
1b. Wrong-type results found by the fit audit (T-3530): the huge-shape GAMMA.DIST density #NUM! vs 0, and the huge-shape GAMMA.INV band edges. These are semantic and count against Gate B, so they are high priority.
2. **Gate B blockers in the gamma family.** These are visibly-off cells in user-representative suites (gen-*, ironcalc-calc). Recompute them with `make eval`, then attribute cells by formula.
   - CHIINV/CHISQ.INV at p = 0.975 (T-3146 is blocked on the T-3175 re-probe; T-3336 → T-3337 covers df = 1).
   - The leftover CHISQ.INV deep-tail cells (T-3403).
   - The small GAMMADIST/CHISQ.DIST tiny-x cdf cells (T-3219).
   - `CHIDIST(F.INV.RT(..),10)`: #NUM! where Excel gives 0 (semantic).
   - About 20 gen-m9 cells blamed on gamma functions are really FORECAST.ETS #N/A (T-2425). They are not gamma's to fix, but they block Gate B attribution.
3. **Blind regions** from the held-out sweep: T-3519, T-3520, T-3521, T-3522, and T-3412 (GAMMA.DIST lower tail at shape > 1 is Excel's leading-factor form).
4. **T-3364**: GAMMA.INV's huge-shape closed-form branch must return the math (decision 0188 R3, MATH-OVER-EXCEL). Fitted pieces extrapolated past their measured range fall back to accurate math.
5. **T-3469**: T-3269's remainder. It covers the shape-6 misses, the x/a = 10 misses, and Excel's switch above shape 11. It also carries the review-round-3 notes:
   - holdout.py's mpmath reference uses the decimal x, not the double Excel receives;
   - the evidence for the shape-11 cut is thin;
   - the tag range should be widened.
6. **Phase 1b** (deferred) items include T-3242, T-3247, T-3303, T-3224, T-2367 and T-3461.

## Hard-won process lessons
- **The shared `target/` is unsafe across worktrees.** Other sessions overwrite `target/release/rustsheet-eval`, and cargo can report "fresh" while holding another worktree's build. For every eval:
  1. take `flock /tmp/rs-eval.lock`;
  2. run `cargo clean --release -p rustsheet_engine -p rustsheet_eval`;
  3. build;
  4. copy the binary to a private `/tmp/rsx-*` path, and score only from the copy.
  The oracle tools take `RUSTSHEET_EVAL_BIN=<copy>`.
- **Oracle.** `cd tools && RUSTSHEET_EVAL_BIN=<copy> uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric --case <id>` works in this environment. A case needs `"expected": {}`, and an oracle sidecar needs a `source` block.
- **Excel truncates formula literals to 15 significant digits.** Write probe literals at 15 digits or fewer, and fit at the 15-digit inputs. The 50-digit references must use the *double* Excel receives (`mpmath.mpf(float(s))`), not the decimal string.
- **R4 discipline that reviewers enforce:**
  - fix the hold-out split before choosing the model;
  - commit the frozen code *before* filling the fresh sweep, so the order can be proven from git;
  - never place a cut to keep one previously matching cell matched (decisions 0056 and 0082);
  - commit the mpmath reference values and the scripts.
- **Id collisions with main are frequent.** Push a new task or decision file the moment `make new-task` / `make new-decision` creates it. Run `python3 scripts/tasks.py validate` and `python3 scripts/check_decisions.py` after merging origin/main, before every push.
- **Disk is tight** (a few GB shared). Delete scratch output. Clear old incremental build dirs only while holding the lock.
- **Reviews.** Run 4 reviewer agents in parallel: security, architecture, quality, excel-fidelity. The fidelity reviewer should rebuild and check numbers itself. Several rounds are normal.
- **Commit format.** `T-XXXX: summary`, no model names in commits or PRs, and never rebase, amend or force-push.

## Where the evidence is
- Per-task research: `docs/research/t3285/`, `t3269/`, `t3311/`, `t3146/` (merged), and the `fit-audit/` branch.
- Coverage analysis from the previous session, against main 3003a898b: a 4,100-cell random sweep, Excel-filled.
  - Worst error against 50-digit math was 4.6e-13. That is Gate A-safe everywhere.
  - About 5% of random inputs were visibly off against Excel, clustered in the blind regions above. User-representative suites were at least 98% within 15 digits per function.
  - T-3509 makes this sweep permanent.
