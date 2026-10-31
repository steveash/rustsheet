# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-10-31, 05:47 UTC run (M2 is current)

This is the scheduled 05:47 UTC run (it started at 05:52 UTC). It follows #2456, yesterday's 18:47 UTC run. It covers the 66 commits merged to `main` after 36cf3e4d (#2456's eval head) up to 6c7c6438 (T-3321, #2511), the head when this run started.

`make eval` ran on 6c7c6438 with a clean tree.
- All 17 suites were scored. calipers, corpus-bench and poi were again not fetched in this container, so I ran `rustsheet-eval fetch` for them first (a first `make eval` started before the fetch was stopped and rerun). Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock from 3,029 to 2,418 (decision 0093).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`). Its visibly-off number count rose, from new probes-numeric oracle sweeps (T-3249's BETA.DIST and F.DIST at large shapes, owned by T-3250 and T-3173); probes-numeric is not gated.

CI on `main` is green. Of the 66 push runs created since 18:50 UTC yesterday, 52 succeeded and 13 were cancelled by the next push. None failed. The run on 6c7c6438 was still in progress when this snapshot was taken.

Nightly fuzz: run 28 (2026-10-31 03:41 UTC) **passed**.

**M2 is still current: M0–M2 probes (decision 0051) have 162 known failures, all visibly off (56 last run).** These are new coverage, not regressions: last run's table-growth, typed-formula and whitespace cells were fixed (T-3218, T-3273, T-3274, T-3210, T-3283 and T-3241 merged), and two new collation probes landed. `make exit-owners MILESTONE=M2` reports 0 unowned.
- `probes/t3276-collation-other-scripts`: 115 cells. Owners: T-3324 and T-3325 (engine).
- `probes/t3304-collation-multi-letter-accents`: 47 cells. Owners: T-3329 (engine) and T-3324.
- The frozen M1 list has 0 known failures (89/89 cases).

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,931 (`check_duplication.py` = committed baseline);
- oversized files: 8 (= last run; `quality/` is unchanged since 36cf3e4d).

Merged since #2456's eval (36cf3e4d): 66 commits up to 6c7c6438. None were reverted. All were committed by GitHub, and each carries its PR's `(#n)` suffix: #2428, #2446, #2449, #2451–#2453, #2455–#2473, #2475–#2494, #2496–#2500, #2502, #2504, #2506, #2508–#2517 and #2519–#2521 (#2456 is the last steward run).
- Task-only PRs among them: #2446, #2461, #2467, #2468, #2498, #2500, #2508, #2512, #2517, #2519 and #2521.

Blocked: 41 on `main` (40 last run). New: T-3146 (gamma quantiles at p = 0.999), `Blocked because: time-boxed (decision 0056)` after two attempts. No other blocked task's blocker has cleared. No task on `main` is `Blocked because: review-round limit`.

Open PRs: 21 at 05:55 UTC (before this PR). No non-draft PR is older than 24 h (the oldest, #2406, was opened 2026-10-30 09:44 UTC), so no orphan merge or duplicate close was due. The drafts #1658, #1885, #1917 and #2104 are older than 24 h and stay drafts (their tasks are blocked). Ten of the open PRs are marked `ready-to-merge`; seven of those were marked more than 2 h ago (#2406, #2412, #2430, #2436, #2450, #2454 and #2474).

Queues:
- `make ready-tasks LANE=functions` printed 4 ready (≥ 3), so the gap-triager did not run. `TIER=sonnet` printed 20 (≥ 10).
- engine, harness, research and perf printed 0 ready, but only because each lane has an agent branch in flight. Ignoring claims, their M2 queues hold 9 (engine), 8 (harness) and 22 (research) ready tasks, so the planner did not run. api printed 9 and quality 9.

Upstream IronCalc: `scripts/upstream-log.sh` listed no commits after 2128db12, so no port tasks were filed.

Issues: the one open issue, #2277, already has its task (T-0496, done in #2293).

Merge mode: `shadow` on `main`. `make merge-train-stats` again could not run in this container (`gh` cannot authenticate; I stopped it after 90 s). From the GitHub MCP tools instead:
- `merge-train/shadow` had 17 CI runs since 18:50 UTC yesterday: 16 succeeded and 1 was in progress. The agreement score and latency p90 need `merge-train-stats`.
- **Stranded (judgment, from labels and timestamps):** #2406, #2412, #2430, #2436, #2450, #2454 and #2474 have been marked `ready-to-merge` for more than 2 h, unmerged. In `shadow` mode their workers merge them by the per-PR lock flow; T-3344 (filed in #2508) would let the steward merge them. In `train` mode this would trigger the emergency flip to `lock`.
- `main`: 0 red `ci.yml` push runs.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 4 pending cases, the same as last run. `rustsheet-oracle fill -b excel-graph --suite probes` wrote 0 sidecars: Excel for the web refused t0481-curly-quote-names-c and typed-if-one-argument again (`unsupportedWorkbook`), and the other two are known unfillable (T-1075).
- As before, ironcalc-calc's 1 pending case is Excel-authored, and poi's 205 are web-vs-desktop conflicts (decision 0019), so I refilled neither. calipers (283 pending) and corpus-bench (592) are scored from their own cached values.

Spot-audit: the last was 2026-10-25 (#1736), six days ago, so none is due. The next is on or after 2026-11-01.

**Critical lane:** engine, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: all 162 M0–M2 cells: T-3324 (collating other scripts), T-3325 (other scripts' digits) and T-3329 (combining marks after a hyphen).
- research: T-3240 (how Excel prints a this-row reference) and the M2 probe queue.
- harness: Gate A (SPEC-0029, T-3355 open as #2522, and its slices T-3356 and T-3358), the merge-train rollout (SPEC-0027) and the M2 generator work.
- api and quality have no M2 work on the critical path. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs, and so did the 81 since #1736. Update 2026-10-26 (18:47 UTC run): still none; all 52 commits since #1896 came through PRs. Update 2026-10-27: still none; all 41 commits since #1954 came through PRs. Update 2026-10-27 (18:47 UTC run): still none; all 64 commits since #2007's eval were merged through GitHub. Update 2026-10-28 (18:47 UTC run): still none; all 65 commits since #2152's eval were committed by GitHub. Update 2026-10-29: still none; all 53 commits since #2216's eval match a merged PR. Update 2026-10-29 (18:47 UTC run): still none; all 68 commits since #2268's eval match a merged PR. Update 2026-10-30: still none; all 50 commits since #2343's eval match a merged PR. Update 2026-10-30 (18:47 UTC run): still none; all 67 commits since #2390's eval match a merged PR. Update 2026-10-31: still none; all 66 commits since #2456's eval match a merged PR.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days. Update (second 2026-10-25 run): a scheduled steward firing ran today at 18:47 UTC. SPEC-0022 notes that #1736 was probably not a scheduled run. Update 2026-10-26: the scheduled run fired again at 05:47 UTC. A second steward firing ran at 18:47 UTC. Update 2026-10-27: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-28: both fired (05:47 and 18:47 UTC). Update 2026-10-29: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-30: the 05:47 UTC run fired. The 18:47 UTC run fired too. Update 2026-10-31: the 05:47 UTC run fired.
- **Steward routine out of sync with routines.md** (2026-10-26). This 18:47 UTC run's stored prompt is an older text than "Prompt — steward (daily)" in docs/process/routines.md. It lacks the progress-dashboard step (`make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and asks for per-suite numbers in docs/status.md, which decision 0118 moved to the dashboard. I followed routines.md and the `steward` skill. Also, routines.md lists one daily steward run at 05:47 UTC, but steward firings arrive at both 05:47 and 18:47. Please update the routine prompt to routines.md's text, and either delete the second schedule or add it to routines.md (agents never edit routines). Update 2026-10-27: today's 05:47 UTC firing carries the current routines.md text word for word, so only the 18:47 firing is out of date. Update (18:47 UTC run): this firing now carries the routines.md text word for word as well. What remains is that routines.md lists one daily steward run, but the steward fires at 05:47 and 18:47 UTC: delete the second schedule or add it to routines.md. Update 2026-10-28 (18:47 UTC run): this firing's prompt again matches routines.md word for word; the second schedule is still unlisted there. Update 2026-10-29 (05:47 UTC run): #2252 (T-2996) added the merge-train clauses to the steward prompt in routines.md, so this firing's prompt is now the older text. That is expected until owner step (3) of the merge-train item below, which waits on T-2989 (todo); the mode is still `lock`, so nothing is missed. Update 2026-10-29 (18:47 UTC run): this firing also carries the older text (no issue-sweep step 8, no `train`-mode clauses or step 9). T-2989 merged as #2332 during this run, so owner step (3) below is now due; until then I followed the `steward` skill on `main`, which covers both. Update 2026-10-30 (05:47 UTC run): this firing also carries the older text (no issue-sweep step 8, no `train`-mode clauses or step 9). I followed the `steward` skill on `main`, which covers both, and did the issue sweep (nothing to do). Update 2026-10-30 (18:47 UTC run): this firing carries the current routines.md steward prompt word for word (issue sweep step 8, `train`-mode clauses, step 9). So step (3) of the merge-train item below is done for this schedule, if not for the 05:47 one. routines.md still lists only the 05:47 schedule: delete the 18:47 one or add it there.
- **Merge-train owner steps** (2026-10-29, T-2996, SPEC-0027 "Owner steps"). Nothing merges through the train until T-2991 sets `train` mode; `scripts/merge_train.toml` says `shadow` since T-2989 (decision 0164). (1) Create the routine **rustsheet merger** in claude.ai → Code → Routines: repository `steveash/rustsheet` attached with GitHub access, "new session each run", Sonnet 5.5, schedule `5,35 * * * *` (UTC), notifications off, prompt "Prompt — merger" in docs/process/routines.md. In `lock` mode it stops at once. (2) In the `main` ruleset, add the required checks `changes (docs-only?)`, `hygiene (ratchets, deny, machete, typos, governance)`, `rust (fmt, clippy, test)`, `eval (Excel-compatibility gate)` and `python (tools + bindings)`; leave "Require branches to be up to date before merging" and "Allow auto-merge" off. (3) **After T-2989 merges**: update the two worker routines and the steward routine to their prompts in docs/process/routines.md. (4) Optional: a ruleset that restricts `merge-train/*` branches to the owner account. Update 2026-10-29 (18:47 UTC run): **T-2989 merged (#2332), so step (3) is due now**: update the two worker routines and the steward routine to their prompts in docs/process/routines.md. `scripts/merge_train.toml` on `main` now says `shadow` (trains are built and tested on `merge-train/shadow`; nothing merges through them). Update 2026-10-30: step (3) is still due (the 05:47 UTC steward firing still has the older prompt); the mode is `shadow`, so nothing breaks until `train` mode (T-2991). `make merge-train-stats` shows 0 shadow trains in the last 24 h, so step (1), the merger routine, is what T-2991's evidence now waits on. This run also removed stale duplicate copies of these four items that an earlier merge had left below them. Update 2026-10-30 (18:47 UTC run): **step (1) looks done**: `merge-train/shadow` now has 33 CI runs, the latest ones green. The 18:47 steward firing has the new prompt; check the two worker routines and the 05:47 steward firing for step (3). Also: `gh` cannot authenticate in the steward's container (GH_TOKEN is rejected), so `make merge-train-stats` and other `gh api` scripts cannot run there. Please refresh or remove that token in the environment's secrets.

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
