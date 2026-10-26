# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-10-26, second run (M2 is current)

This is the scheduled run at 18:47 UTC. It follows #1896, this morning's 05:47 UTC run, and covers the 52 commits merged to `main` since then, through 96449aaf, including the two this run merged.

`make eval` ran on 5dc93dd8, this branch merged with `main` at 96449aaf, so it includes everything this run merged.
- All 17 suites were fetched and scored. On the first pass (on 7ffb09ee), calipers, corpus-bench and poi had not been fetched; I fetched them and re-ran. Timeouts: 0 in every suite.
- `make eval-lower-visible-lock` shows 0 regressions in every gated suite and lowers the visible-parity lock from 4,448 to 3,581 (decision 0093).
- I first lowered the lock to 3,541 on 7ffb09ee. reviewer-excel-fidelity pointed out that #1826, merged by this run, adds 43 visibly-off probe cells, so that lock would fail on the merged head. CI confirmed it. I merged `main` in under the merge lock and re-ran the eval.
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).
- Since the last complete run, Phase 1 functions rose from 324 to 378 of 506. Visibly-off type, error or text cells fell from 20,039 to 1,484, and visibly-off numbers from 131,347 to 105,122. VLOOKUP is now Exact.

CI on `main` is green: every push run since #1896 succeeded or was cancelled by the next push, and none failed.

**M2 is still current: M0–M2 probes (decision 0051) have 18 known failures, 11 of them visibly off.** Every one has an open M2 owner, checked by hand by the planner and me.
- `probes/t2776-formula-text-whitespace-shapes` has 8 cells and `t2769-formula-text-whitespace` S!C17 has 1. Owners: T-2782 (draft #1953, handed off to opus), then T-2783 and T-2784.
- `probes/t2776-ws-after-bang` S!A1 and S!C1. Owner: T-2785.
- `probes/t2756-roundup-smallest-normal-threshold` Sheet1!A52–A58 are within 15 digits but not bit-exact. Owner: T-2771. The planner now names each cell in it, so the exit-owner rule can see them.
- The new failures come from the SPEC-0023 whitespace probes (T-2776). T-2781 (#1925) fixed last run's two visibly-off cells, t2723 S!C24 and S!C25.
- The frozen M1 list has 0 known failures (89/89 cases).
- `make exit-owners MILESTONE=M2` still cannot run, so the planner filed **T-2822** (harness, M2) to add an M2 exit row. `make exit-owners MILESTONE=M5` reports 0 unowned. #1826's 43 new FORMULATEXT cells are M5 cells, owned by T-2726.
- `make exit-owners MILESTONE=M6` reported 11 unowned cells: the probes/t2065-lift-* cells. T-2065 had handed them to T-2063 and T-2064, which were already done. I filed **T-2823** (COUNTIFS, AVERAGEIF, MAXIFS: 6 cells) and **T-2824** (STDEV.S, VAR.S, SUMSQ, AVERAGEA, MAXA: 5 cells, after T-2816). It now reports 0.

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,931 (was 2,951);
- oversized files: 8 (= last run).
- `check_duplication.py` and `check_file_size.py` only note that they could be ratcheted. I didn't commit the `--update`: quality/ is a protected path, and the governance gate wants a decision record for it.

Merged since #1896: 52 commits through 96449aaf. None was reverted, and all came through PRs.
- Three were squash-merged without the `(#n)` suffix in the title: a855178f (#1936, T-1883), 6340da61 (#1935, T-2687) and f6f9865c (#1951, T-2691).
- Task ids merged (50): T-0474, T-0480–T-0482, T-0954, T-1366, T-1592, T-1671, T-1883, T-1923, T-1924, T-2065, T-2391, T-2432, T-2433, T-2435, T-2472, T-2503, T-2523, T-2529, T-2535, T-2537, T-2575, T-2584, T-2618, T-2621, T-2629, T-2634, T-2664, T-2671, T-2674–T-2677, T-2687, T-2689, T-2691, T-2696, T-2700, T-2712, T-2727, T-2728, T-2746, T-2757, T-2769, T-2776, T-2781, T-2793, T-2795, T-2810.
- Task-only PRs: #1913, #1934.

Blocked: 33 (= last run, counted by front matter). None was newly blocked or unblocked on `main`. T-2803 (#1938) and T-2778 (#1917) are blocked on their draft branches.

Open PRs: 17 at 19:42 UTC. Seven non-draft PRs were older than 24 h.
- **#1834 (T-2618, close as already fixed by T-2493)**: a task-only PR. I checked its claim: the six gen-m9 and six probe cells are absent from the baselines. reviewer-security, reviewer-architecture and reviewer-quality all returned APPROVE. I merged `main` in under the merge lock; CI was green on 842a86c3 and the branch was up to date, so I **merged** it as 33e9f9ca.
- **#1826 (T-2529, FORMULATEXT probe)**: all four reviewers returned APPROVE, but the first merge of `main` turned the eval gate red. Calc!O1, `FORMULATEXT(INDIRECT({"Data!A2"}))`, regressed. Since T-2577 (#1894), a 1x1-array INDIRECT is a one-element array of references, as Excel has it, and FORMULATEXT does not yet read it per element. That is T-2726's gap. I released the lock, rebuilt the probes baseline from `main`'s (43 new-coverage, 0 regressions) and corrected the miss counts in references.md, T-2529 and T-2726 (reviewer-excel-fidelity caught my first breakdown). All four reviewers re-approved a8776648. I took the lock again, merged `main` in, and CI was green on that head, so I **merged** it as 96449aaf.
- **#1817 (T-2469)**: not merged. It conflicts with `main` (statistical sources and two baselines), and #1949 now implements the same task from a fresh, live branch. Per decision 0050, whichever merges first survives, and the other is closed after that. I commented on the PR.
- **#1811 (tasks: refill opus queue)**: not merged. It conflicts in statistics.md, and its `tasks/*` branch is outside the orphan rules. I commented on the PR.
- **#1769 (T-2504)**: a `claude/*` branch from an interactive session, so outside the orphan rules. It has had no activity since 2026-10-25.
- **#1727 (T-2622)** and **#1718 (T-2427)**: unchanged since this morning's REQUEST_CHANGES verdicts and blocker comments. Neither branch has moved.
- #1583, #1658, #1767, #1885, #1917 and #1938 are drafts whose tasks are blocked. #1953 (T-2782) is a draft handed off to opus.
- No task is `Blocked because: review-round limit`.

Queues:
- `make ready-tasks LANE=functions` printed 26 ready, so the gap-triager did not run. `TIER=sonnet` printed 47 (≥ 10).
- engine and research print 0 ready, because their lanes are busy (T-2782 and T-1943 branches moved within 2 h). Their M2 work is queued: T-2783, T-2784, T-2785 and T-2771. api printed 2, harness 10, quality 4 and perf 21.
- The planner ran for M2. It filed T-2822 and named T-2771's cells.

Upstream IronCalc: 5 new commits since 9cfbdfce, triaged through de6545ab (UPSTREAM.md).
- Filed **T-2819**: take the Excel-authored BASE_DECIMAL.xlsx (b716e550; f2f0b089 only reformats its test).
- Filed **T-2820**: take the Excel-authored NOT.xlsx (de6545ab).
- 7ec21c74 (NOT lifts over arrays) is already ours (T-0225, T-0260).
- ce346b2f (displace rows and columns in one go) is a structural-edit refactor for speed with no behaviour change, so no port task (perf is Phase 2).

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 6 pending, 0 written, 4 failures. `t2065-lift-large`, `-median` and `-small` never open a session on Excel for the web (retries exhausted), as T-2065 found; they are T-1075 item 18 (M14). `typed-if-one-argument` (T-1990) still gets 501 `FileCorruptTryRepair`. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc's 1 pending case is Excel-authored, and poi's 205 are web-vs-desktop conflicts (decision 0019), so neither was refilled. calipers (283 pending) and corpus-bench (592) are Excel-saved workbooks scored from their own cached values (`use_cached_values`), so an oracle fill does not apply to them.

Spot-audit: the last was 2026-10-25 (#1736), so none is due. The next is on or after 2026-11-01.

**Critical lane:** engine, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-2782 (handed off to opus, #1953), T-2783, T-2784 and T-2785 own 11 of the 18 M0–M2 known failures, all visibly off.
- research: T-2771 owns the last 7 (A52–A58, bit-level).
- harness: T-2822, the M2 exit row that lets `make exit-owners` check M2.
- api and quality have no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs, and so did the 81 since #1736. Update 2026-10-26 (18:47 UTC run): still none; all 52 commits since #1896 came through PRs.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days. Update (second 2026-10-25 run): a scheduled steward firing ran today at 18:47 UTC. SPEC-0022 notes that #1736 was probably not a scheduled run. Update 2026-10-26: the scheduled run fired again at 05:47 UTC. A second steward firing ran at 18:47 UTC (this run).
- **Steward routine out of sync with routines.md** (2026-10-26). This 18:47 UTC run's stored prompt is an older text than "Prompt — steward (daily)" in docs/process/routines.md. It lacks the progress-dashboard step (`make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and asks for per-suite numbers in docs/status.md, which decision 0118 moved to the dashboard. I followed routines.md and the `steward` skill. Also, routines.md lists one daily steward run at 05:47 UTC, but steward firings arrive at both 05:47 and 18:47. Please update the routine prompt to routines.md's text, and either delete the second schedule or add it to routines.md (agents never edit routines).

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
