# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-11-01, 19:43 UTC run (M2 is current)

This run follows #2616, the 2026-10-31 18:43 UTC run. No steward PR landed from a 05:47 UTC firing today, so it covers the 138 commits merged to `main` after fc4aeca4 (#2616's merge) up to 1f30a9b1 (T-3616, #2747), the head when this run started. Five more PRs merged during the run (#2748, #2676, #2745, #2746 and #2750, 20:01–20:04 UTC); I merged them into this branch.

`make eval` ran on 1f30a9b1. The tree was dirty only from this run's `quality/duplication.json` ratchet.
- All 17 suites were scored. calipers, corpus-bench and poi were not fetched in this container again, so the first `make eval` skipped them; I ran `rustsheet-eval fetch` for the three and ran `make eval` again. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock (decision 0093) from 3,003 allowed in 296 cases to 2,342 in-scope visibly-off cells in 251 cases (0 in new cases).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`). Its every-suite Phase 1 count fell by 6 while the Gate B count rose. That comes from new probe evidence (+76,640 attributed cells), not from a regression.

CI on `main` is green. Every `ci.yml` push run on `main` since the last run either succeeded or was cancelled by the next push; none failed.

Merge train: `make merge-mode` reads `train` again. #2623 re-flipped it by owner ruling (decision 0208), and T-3488 (#2750, merged during this run) found the causes of the 2026-10-31 stall and raised the train's CI wait to 15 min (decision 0242).
- **Stranded (daily threshold):** `make merge-train-stats STRANDED=1` at 19:47 UTC listed **31 stranded PRs**: #2778, #2777, #2775, #2774, #2773, #2772, #2771, #2770, #2769, #2768, #2766, #2765, #2764, #2762, #2761, #2760, #2759, #2758, #2757, #2753, #2752, #2751, #2750, #2748, #2746, #2745, #2742, #2733, #2730, #2676 and #2668. The train is moving but too slowly: batches of five merged at about 17:30, 18:26 and 20:01 UTC, while about 40 PRs were opened between 13:30 and 19:40 UTC. The lock was held by a train the whole time, with the next train queued.
- Per decision 0224 that escalates, with no flip. It already had an owner, so I raised **T-3738** (throughput against arrivals, SPEC-0027's "Revisit") to **P0** with this evidence rather than file a duplicate. I also added to **T-3737** a second green-but-unmerged hold: train #2748 … #2750 went green at 19:25 UTC and squashed only at 20:01.
- The full `make merge-train-stats` (p90 latency, red-train rate, bisection runs) stopped on a GitHub read reset (`connection reset by peer`), as it did last run, so those numbers are missing from this run.
- Ejected: #2534, #2638, #2657, #2684, #2687, #2749, #2755, #2763 and #2767 (`merge-train-ejected`); the janitor owns the mechanical ones.

M2 is still current. M0–M2 probes (decision 0051) have 264 known failures, all 264 visibly off (200 last run), in three cases:
- t3298-case-mapping-sweep (130, new coverage), owned by T-3445, T-3583 and T-3584.
- t3276-collation-other-scripts (101), owned by T-3324 and T-3559.
- t3334-criteria-operand-forms (33), owned by T-3404, open as #2638 (ejected).
- The t3266 nesting-depth cases now pass (T-3338, #2725). 0 M0–M2 cases did not run. The frozen M1 list has 0 known failures (89/89 cases).
- `make exit-owners MILESTONE=M2` reports 0 unowned cells.

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,893 (2,917 committed baseline; this PR ratchets `quality/duplication.json` down);
- oversized files: 8 (= last run).

Merged since #2616 (fc4aeca4): 138 commits up to 1f30a9b1, plus the 5 during the run. None were reverted. All were committed by GitHub, each with its PR's `(#n)` suffix, from #2104 to #2747. Among them:
- the re-flip to `train` (#2623) and the merge janitor (#2699);
- the long-ready #2104 and #2437.

Blocked: 39 on `main` (38 last run). No task on `main` is `Blocked because: review-round limit`.

Open PRs: 73 at 19:45 UTC, 15 of them drafts.
- Orphaned-PR check (`train` mode: idle 6 h or more, not draft, not marked on its current head):
  - #2574 (task-only gap triage, idle since 2026-10-31 14:49): I ran the panel on its head. Security, architecture and quality approve, but Excel fidelity requests changes. `main` has since filed T-3565, T-3618 and T-3619 for the same cells, so merging it would give every cell two owners. Not marked; the blocker is commented on the PR.
  - #2646 (T-3496): ready was removed for a conflict with `main` that lay only in `eval/baseline/` (gen-m6.txt, the retired visible-parity.json). `make catch-up PR=2646` merged `main` in, re-recorded the lists and pushed dcdeee95. CI went green on that head, and I marked it ready with `CARRY=1` at 20:24 UTC.
  - #2700 (T-3530, `claude/*`): conflicts with `main` in `docs/excel-semantics/statistics.md`, a prose conflict. Commented; it needs its worker or an opus run.
- Drafts older than 24 h: #1885, #2529 and #2573 stay drafts.
- Duplicate check: no two open PRs carry the same task id.

Queues:
- `make ready-tasks LANE=functions` printed 0 ready, and engine, harness and research printed 0, all because their lanes have agent branches in flight. Ignoring claims, functions holds 288 todo tasks, and the M2 queues hold 14 (engine), 20 (harness) and 32 (research) todo tasks. Six gap-triage PRs are also open (#2574, #2624, #2640, #2749, #2778 and #2784). So I ran neither the gap-triager nor the planner (judgment: new tasks would only queue behind the claims and the train).
- api printed 9, perf 17, quality 9, and `TIER=sonnet` 35 (≥ 10).

Upstream IronCalc: `scripts/upstream-log.sh` listed no new commits, so no port tasks were filed.

Issues: the one open issue, #2277, already has its task (T-0496).

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 4 pending cases. `fill -b excel-graph --suite probes` wrote 0 sidecars: Excel for the web refused t0481-curly-quote-names-c and typed-if-one-argument again (`unsupportedWorkbook`), and the other two are known unfillable (T-1075).
- As before, ironcalc-calc's 1 pending case is Excel-authored and poi's 205 are web-vs-desktop conflicts (decision 0019), so I refilled neither. calipers (283 pending) and corpus-bench (592) are scored from their own cached values.

Spot-audit (2026-11-01; the last was 2026-10-25). I sampled 3 code PRs merged since then (seeded random draw from 484) and re-reviewed each squash diff with every reviewer on `model: opus`. At merge time, security, architecture and quality ran on Sonnet and Excel fidelity on Opus (decision 0015).
- #1980 (T-2836, the `$` cost in the format-code complexity limit): 4/4 APPROVE.
  - Minor miss (Excel fidelity, Opus): number-formats.md says other date or time tokens "get `d`'s 2". The code charges 3 for `AM/PM`, `A/P` and elapsed tokens. Already in T-2842's probe list.
  - Minor misses (architecture and quality, Sonnet): two stale `Section` fields, a `$` match written twice, and a test helper that could cover one more test.
- #2647 (T-2824, STDEV.S/VAR.S/SUMSQ/AVERAGEA/MAXA over a lifted OFFSET): 4/4 APPROVE.
  - Minor miss (Excel fidelity, Opus): legacy VAR and MAXA's lifted text are unprobed readings. They are labelled UNVERIFIED and owned by T-3533. lookup.md and SPEC-0014 each have one sentence that says more than the evidence.
  - Minor misses (quality, architecture, Sonnet): stale doc comments and a helper name.
- #2565 (T-3378, General inside a date section): 4/4 APPROVE.
  - Minor misses (quality and architecture, Sonnet): `is_date_or_clock` is a second negative list of `TextToken` variants beside `is_calendar_or_clock`, so a new variant would silently count as a date token. The fraction scale is also re-derived.
  - Excel fidelity found only extrapolations, all already in T-3419.
- No real defect reached `main` in the sample, so no fix task was filed. Next audit on or after 2026-11-08.

Step 9 (`train` mode): I ran one `make merge-train TRAIN=1` pass after the rest of this run. It queued behind the train holding the lock, so its result is in this PR's description, not here.

**Critical lane:** engine, harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: the 264 visibly-off M0–M2 cells: T-3445 (case fold), T-3324 (collating other scripts) and, in functions-text, T-3583 and T-3404.
- harness: T-3738 (P0, train throughput) and T-3736/T-3737 (merge-train holds), plus Gate A (SPEC-0029).
- research: the M2 probe queue (T-3584, T-3559).
- api and quality have no M2 work on the critical path. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs, and so did the 81 since #1736. Update 2026-10-26 (18:47 UTC run): still none; all 52 commits since #1896 came through PRs. Update 2026-10-27: still none; all 41 commits since #1954 came through PRs. Update 2026-10-27 (18:47 UTC run): still none; all 64 commits since #2007's eval were merged through GitHub. Update 2026-10-28 (18:47 UTC run): still none; all 65 commits since #2152's eval were committed by GitHub. Update 2026-10-29: still none; all 53 commits since #2216's eval match a merged PR. Update 2026-10-29 (18:47 UTC run): still none; all 68 commits since #2268's eval match a merged PR. Update 2026-10-30: still none; all 50 commits since #2343's eval match a merged PR. Update 2026-10-30 (18:47 UTC run): still none; all 67 commits since #2390's eval match a merged PR. Update 2026-10-31: still none; all 66 commits since #2456's eval match a merged PR. Update 2026-10-31 (18:43 UTC run): still none; all 36 commits since #2523 were committed by GitHub with a PR suffix. Update 2026-11-01: still none; all 138 commits since #2616 were committed by GitHub with a PR suffix.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days. Update (second 2026-10-25 run): a scheduled steward firing ran today at 18:47 UTC. SPEC-0022 notes that #1736 was probably not a scheduled run. Update 2026-10-26: the scheduled run fired again at 05:47 UTC. A second steward firing ran at 18:47 UTC. Update 2026-10-27: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-28: both fired (05:47 and 18:47 UTC). Update 2026-10-29: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-30: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-31: the 05:47 UTC run fired. The 18:43 UTC run fired too. Update 2026-11-01: no steward PR landed from a 05:47 UTC firing today; this run (fired about 19:40 UTC) covers the whole day since #2616.
- **Steward routine out of sync with routines.md** (2026-10-26). This 18:47 UTC run's stored prompt is an older text than "Prompt — steward (daily)" in docs/process/routines.md. It lacks the progress-dashboard step (`make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and asks for per-suite numbers in docs/status.md, which decision 0118 moved to the dashboard. I followed routines.md and the `steward` skill. Also, routines.md lists one daily steward run at 05:47 UTC, but steward firings arrive at both 05:47 and 18:47. Please update the routine prompt to routines.md's text, and either delete the second schedule or add it to routines.md (agents never edit routines). Update 2026-10-27: today's 05:47 UTC firing carries the current routines.md text word for word, so only the 18:47 firing is out of date. Update (18:47 UTC run): this firing now carries the routines.md text word for word as well. What remains is that routines.md lists one daily steward run, but the steward fires at 05:47 and 18:47 UTC: delete the second schedule or add it to routines.md. Update 2026-10-28 (18:47 UTC run): this firing's prompt again matches routines.md word for word; the second schedule is still unlisted there. Update 2026-10-29 (05:47 UTC run): #2252 (T-2996) added the merge-train clauses to the steward prompt in routines.md, so this firing's prompt is now the older text. That is expected until owner step (3) of the merge-train item below, which waits on T-2989 (todo); the mode is still `lock`, so nothing is missed. Update 2026-10-29 (18:47 UTC run): this firing also carries the older text (no issue-sweep step 8, no `train`-mode clauses or step 9). T-2989 merged as #2332 during this run, so owner step (3) below is now due; until then I followed the `steward` skill on `main`, which covers both. Update 2026-10-30 (05:47 UTC run): this firing also carries the older text (no issue-sweep step 8, no `train`-mode clauses or step 9). I followed the `steward` skill on `main`, which covers both, and did the issue sweep (nothing to do). Update 2026-10-30 (18:47 UTC run): this firing carries the current routines.md steward prompt word for word (issue sweep step 8, `train`-mode clauses, step 9). So step (3) of the merge-train item below is done for this schedule, if not for the 05:47 one. routines.md still lists only the 05:47 schedule: delete the 18:47 one or add it there. Update 2026-11-01: this firing's prompt is again slightly older than routines.md: its step (5) lacks the `train`-mode orphan clause (`claude/*` PRs idle 6 h, decision 0224), and its step (9) lacks `make janitor STEP=1`. I followed the `steward` skill, which has both.
- **Merge-train owner steps** (2026-10-29, T-2996, SPEC-0027 "Owner steps"). Nothing merges through the train until T-2991 sets `train` mode; `scripts/merge_train.toml` says `shadow` since T-2989 (decision 0164). (1) Create the routine **rustsheet merger** in claude.ai → Code → Routines: repository `steveash/rustsheet` attached with GitHub access, "new session each run", Sonnet 5.5, schedule `5,35 * * * *` (UTC), notifications off, prompt "Prompt — merger" in docs/process/routines.md. In `lock` mode it stops at once. (2) In the `main` ruleset, add the required checks `changes (docs-only?)`, `hygiene (ratchets, deny, machete, typos, governance)`, `rust (fmt, clippy, test)`, `eval (Excel-compatibility gate)` and `python (tools + bindings)`; leave "Require branches to be up to date before merging" and "Allow auto-merge" off. (3) **After T-2989 merges**: update the two worker routines and the steward routine to their prompts in docs/process/routines.md. (4) Optional: a ruleset that restricts `merge-train/*` branches to the owner account. Update 2026-10-29 (18:47 UTC run): **T-2989 merged (#2332), so step (3) is due now**: update the two worker routines and the steward routine to their prompts in docs/process/routines.md. `scripts/merge_train.toml` on `main` now says `shadow` (trains are built and tested on `merge-train/shadow`; nothing merges through them). Update 2026-10-30: step (3) is still due (the 05:47 UTC steward firing still has the older prompt); the mode is `shadow`, so nothing breaks until `train` mode (T-2991). `make merge-train-stats` shows 0 shadow trains in the last 24 h, so step (1), the merger routine, is what T-2991's evidence now waits on. This run also removed stale duplicate copies of these four items that an earlier merge had left below them. Update 2026-10-30 (18:47 UTC run): **step (1) looks done**: `merge-train/shadow` now has 33 CI runs, the latest ones green. The 18:47 steward firing has the new prompt; check the two worker routines and the 05:47 steward firing for step (3). Also: `gh` cannot authenticate in the steward's container (GH_TOKEN is rejected), so `make merge-train-stats` and other `gh api` scripts cannot run there. Please refresh or remove that token in the environment's secrets. Update 2026-11-01: `gh api` authenticates in the steward's container again (`make merge-train-stats STRANDED=1`, `make mark-ready` and `make catch-up` ran), so the token note above looks resolved.

excel-desktop needs are not owner items (decision 0039): T-1016, T-1051 and the other desktop tasks wait in M14 (Phase 4), tracked by T-1074 and T-1075.

## Log
- 2026-10-02 — Owner set `main` as the GitHub default branch and enabled the routines (repo attached).
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0002: pinned external suite fetcher (`rustsheet-eval fetch`); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0201: criteria/wildcard matching consolidated in functions/criteria.rs; fixed COUNTIF "<>text" on non-text cells; eval 0 regressions, 0 baseline fixes (+2 new spec cases pass).
- 2026-10-02 — T-0003: suite `poi` (Apache POI, fetched, pinned); 334 cases, 3,526 cells, 82.25%; baseline +633 known failures (new suite, 0 regressions); filed T-0111 (timeouts), T-0112 (imports).
- 2026-10-02 — T-0204: coercing range walk shared by VARA/STDEVA family and GCD/LCM; duplicated lines 8829→8610; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0007: pending cases now run crash-only (panic/timeout/error reported, per-suite crashes column); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0205: D-function criteria routed through shared criteria grammar (CriteriaFlavor::Database); eval 0 regressions, 0 fixed (+1 new smoke case passes).
- 2026-10-02 — T-0005: transitive volatile skipping + METADATA-pinned clock (NOW/TODAY now scored); NOW() keeps sub-second precision; ironcalc-calc +2 scored cells pass; eval 0 regressions, baseline unchanged.
- 2026-10-02 — T-0008: `rustsheet-gen --policy numeric-rules` → suite `probes-numeric` (2,738 hypothesis-tagged probe cells, pending oracle); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0202: catalog extended with 18 M2 functions + pair-repeat groups in generator; eval 0 regressions, 0 fixed (tools-only).
- 2026-10-03 — T-0902: eval snapshot reader decodes _xHHHH_ escapes (CHAR 29 cells fixed; ironcalc-calc known failures 1909 -> 1880)
- 2026-10-03 — T-0100: SPEC-0001 public API v0 (facade crate) accepted; filed T-0113; docs only, eval unchanged.
- 2026-10-03 — T-0900: BESSELJ/BESSELY bit-exact for |x|<8 via NR approximations (ironcalc-calc known failures 1880 -> 1761); |x|>=8 filed as T-0908
- 2026-10-03 — T-0113: new `rustsheet` facade crate with typed ErrorKind/XlError/RefError and single ERROR.TYPE table; eval 0 regressions, 0 fixed.
- 2026-10-03 — steward: health snapshot; filed T-0114..T-0117 (api/harness ready queues), T-0910/T-0911 (IronCalc ports); UPSTREAM.md triaged through 9b1ae3fd. Open PRs #11, #13 are blocked docs-only drafts from other sessions (left untouched).
- 2026-10-03 — T-0910 (partial): TIMEVALUE of "date time" text returns serial fraction as Excel does (ironcalc-calc known failures 1761 -> 1760); fractional seconds still need oracle evidence
- 2026-10-03 — T-0114: API v0 `refs.rs` (SheetId/CellRef/RangeRef/Target, pure A1 parse/print); eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0912: MOD uses exact remainder (fmod) as Excel; ironcalc-calc known failures 1760 -> 1756 (fixed 4, 0 regressions)
- 2026-10-03 — T-0115: API v0 typed Value/Array and engine cell conversion; eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0914 (partial): SINH/COSH/TANH use Excel's exp formulation (ironcalc-calc known failures 1756 -> 1731, fixed 25, 0 regressions); T-0916 (IRR/RATE/XIRR) found not fittable without the oracle; filed T-0917 for the rest
- 2026-10-03 — T-0111: VLOOKUP search column bounded by last used row; 2 POI timeout cases re-included (poi known failures 633 -> 655 from 24 newly scored cells, 0 regressions); TANH 0.9<=|x|<1 uses e^2x form (fixes main's poi G1360 regression); baseline also records ~26 crash-only POI cases (filed T-0118, T-0119)
- 2026-10-03 — T-0911 (ROW/COLUMN part): ROW/COLUMN of multi-cell references return vectors (ported ironcalc b6d34abc); ironcalc-calc 21 newly checked cells, 0 failures, 0 regressions; rest filed as T-0918
- 2026-10-03 — T-0919: @ROW/@COLUMN skip materialising whole-column vectors (165 ms -> 0.17 ms); eval delta none
- 2026-10-03 — T-0918: ported ironcalc INDIRECT (R1C1 + INDIRECT.xlsx) and text-on-arrays fixes; ironcalc-calc +311 newly checked cells with 0 failures, 0 regressions; remaining ports filed as T-0920
- 2026-10-04 — T-0920 (partial): ported POWER-accepts-ranges (daf5f6c6) and implicit-intersection export (9212095d); ironcalc-calc +51 newly checked cells (47 pass, 4 NumericUlp baselined), 0 regressions; finite-number parse and --A1:B4 remain
- 2026-10-04 — T-0118: importer no longer panics on POI workbooks (typed errors, optional style sections); poi known failures 655 -> 647, 0 regressions beyond 4 crash-kind relabels (decision 0012); duplicated lines 8610->8420.
- 2026-10-04 — T-0913: eval snapshot reader decodes rich-value `vm` errors (#CALC!/#SPILL! cached as #VALUE!); ironcalc-calc known failures 1735 -> 1727 (8 fixed), 0 regressions; engine unchanged
- 2026-10-04 — T-0101: removed lifetime from Model/UserModel/Parser/Lexer and the Python binding's leak helper; eval delta none (0 fixed, 0 regressions)
- 2026-10-04 — steward: health snapshot; queues healthy, no new upstream commits, no oracle credentials.
- 2026-10-04 — T-0923: TIME divides before dropping whole days (+7 cells, ironcalc-calc 97.60%); triaged T-0922..T-0929 (no oracle needed)
- 2026-10-04 — T-0102: API v0 `Workbook` read path + xlsx load/save with zip-bomb limits; eval adapter on the facade; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0924: shared accurate erf/erfc + normal CDF/PDF kernel (ERF*, NORM.DIST/S.DIST, LOGNORM.DIST, GAUSS, Z.TEST); ironcalc-calc known failures 1720 -> 1632 (88 fixed), 0 regressions
- 2026-10-04 — T-0103: API v0 `Workbook::edit`/`set` with undo-log rollback and diff-based `ChangeSet`; eval adapter builds specs via `edit`; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0104: Python bindings on API v0 (`rs.Workbook`, `edit()` transactions, `XlError`, typed exceptions, GIL released); eval unaffected; filed T-0931 (parser nesting stack overflow).
- 2026-10-04 — T-0928: EXPON.DIST cumulative uses -expm1; ironcalc-calc known failures 1632 -> 1626 (6 fixed), 0 regressions
- 2026-10-04 — T-0930: API edit follow-ups (diff split into edit/diff.rs, sheet-position map, error classifier next to RefError, engine take/restore tests, evaluation counter, bench moved); eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0926: GAMMALN/GAMMALN.PRECISE via libm lgamma; ironcalc-calc known failures 1626 -> 1578 (48 fixed), 0 regressions; 12 cells need oracle
- 2026-10-04 — T-0004 (partial): excel-graph oracle live (app-only, `Sites.Selected` site); fixed site drive addressing; T-0004, T-0106..T-0110, T-0203 unblocked.
- 2026-10-04 — T-0932: eval wall time 72.5 s → 71.1 s across T-0930; edit bench median baseline (100k formulas): recalc 13.4 ms, tracked set 16.5 ms, diff overhead 3.0 ms; KeyHasher deduplicated; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0009: excel-graph sidecars for probes (3) and probes-numeric (4); probes 71.11% (13 new-coverage known failures), probes-numeric 60.19% (not gated); 0 regressions; excel-semantics: 9 entries VERIFIED (snapping corrected), 4 new; inputs with ≥1E+308 literals fixed (Excel rejects them). T-0106/7/8/10 now ready.
- 2026-10-04 — Owner updated the routine prompts to the text in `docs/process/routines.md` (verified identical). Decision 0014: Excel is always right; every quirk found is recorded in docs/excel-semantics/.
- 2026-10-04 — T-0013 (+ T-0010): every oracle workbook carries an embedded canary (poisoned cached values) and is refused unless Excel recalculated it; canary exposed that Excel-authored workbooks were never saved by the Graph session (old flow returned cached values) — fixed with a post-calculate no-trace edit; Graph calls retry dropped connections (T-0010); eval delta none
- 2026-10-04 — T-0004 (partial): smoke oracle-filled (canary-verified); Excel disproved D-function criteria trimming (engine fixed, criteria.md entry); eval now lets oracle values replace hand values keyed without sheet quotes; smoke 38/38 vs Excel, 0 regressions
- 2026-10-04 — Owner review: filed T-0016..T-0023, T-0402 (model tiers, board conflicts, task ids, milestone-order triage, POI oracle fill, evidence coverage, fuzzing, reviewer models, perf gate); T-0203 and T-0004 raised to P0; critical lane engine, harness, research, perf (API v0 done, api lane empty).
- 2026-10-04 — T-0022: security/architecture/quality reviewers on Sonnet, fidelity/performance pinned to Opus (decision 0015); eval delta none.
- 2026-10-04 — T-0017: `make new-task` assigns collision-free ids (all origin/* branches counted; decision 0016); eval delta none.
- 2026-10-04 — T-0016: docs/tasks/BOARD.md no longer committed (git-ignored, `make board` renders locally; decision 0017); eval delta none.
- 2026-10-04 — T-0023: tasks carry `tier: sonnet | opus`; sonnet (hourly) and opus (2-hourly) workers replace the lane workers; stuck sonnet runs hand off (decision 0018). Needs owner: re-apply docs/process/routines.md. Eval delta none.
- 2026-10-04 — T-0004 done: oracle calibration over ironcalc-calc (71,765 cells, 99.96% bit-exact; 7 day-first-locale cached cells, 4 web-vs-desktop INDIRECT, 18 last-digit float); canary pins en-US; probes/date-text-locale added (probes known failures 13 -> 18, new coverage); filed T-0024
- 2026-10-04 — T-0108: Excel 1900 leap-year serials in DATE/DAY/MONTH/YEAR/formatting; probes known failures 18 -> 15 (A1-A3 fixed), ironcalc-calc 1578 -> 1578, 0 regressions; filed T-0125
- 2026-10-04 — T-0108 follow-up: added probes/dates-early-1900 (oracle-filled, 16 cells, 4 known failures -> T-0125); fixed EOMONTH early serials and DATE(1900,1,0)
- 2026-10-04 — T-0019: excel-graph sidecars for 40 POI workbooks (`oracle_dir`, committed in eval/suites/poi/oracle); poi pending 237 -> 198, known failures 647 -> 959 (new coverage, 0 regressions; 257 web-vs-desktop conflicts left pending, decision 0019); harness now parses JSON floats exactly (serde_json `float_roundtrip`; oracle values had read 1 ulp off, probes-numeric 60.19% -> 68.08%); quirk: SIN/COS at multiples of 90°.
- 2026-10-04 — T-0206: probes/criteria-operands oracle-filled (25 cells); fixed <,>,<=,>= with boolean/error operands and D-function explicit '=' exactness; 0 regressions, probes 25/25 new cells pass
- 2026-10-05 — T-0028: agents check PR mergeability after every push, event and merge to main, and resolve conflicts at once (decision 0020); `make check` runs typos; eval delta none.
- 2026-10-05 — T-0015: spec2xlsx output deterministic (pinned core.xml creation time); eval delta none.
- 2026-10-05 — T-0027: shared-formula members shown shifted in eval show/snapshot (engine `shift_formula_text`); eval delta none.
- 2026-10-05 — T-0203: gen-m2 oracle-filled and gated (30 cases, 12,000 cells, 85.62% bit-exact, 1,726 known failures, new coverage); IF arity fixed in the catalog; T-0208 fixed (oracle `#SPILL!` read by key name; 74 cells); 14 gap tasks T-0209..T-0222; unknown rich errorTypes read as `#UNDECODED!` and stay unscored (poi 951 -> 950, T-0030).
- 2026-10-05 — T-0123: names may contain `\` (en-US lexer, identifier validity); poi known failures 950 -> 935 (15 fixed, 0 regressions).
- 2026-10-05 — T-0031: salvaged stale draft PRs #11/#13/#38 (T-0905, T-0907 restored; financial.md; T-0901/T-0925 findings) and closed them; eval delta none.
- 2026-10-05 — Log frozen here (decision 0024, T-0033); later work: `git log --first-parent main` and the steward's daily summary.
- 2026-10-05 — Owner re-applied the T-0023 routines (sonnet hourly, opus every 2 h); T-0032: sonnet queue buffer (≥10 ready sonnet tasks; `make ready-tasks`, decision 0021).
