# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-10-27 (M2 is current)

This is the scheduled run at 05:47 UTC. It follows #1954, yesterday's 18:47 UTC run, and covers the 41 commits merged to `main` from 52a6ac4d (#1953) through 7bd9090e (#1997).

`make eval` ran on 7bd9090e, the head of `main` when this run started, with a clean tree. Five PRs merged after it (#1811, #1996 T-2338, #2001 T-2753, #2000 T-2755, #2004 T-0476); the next run covers them. The gate still holds on this branch merged with `main`.
- All 17 suites were fetched and scored (I ran `rustsheet-eval fetch` first, so calipers, corpus-bench and poi were present). Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowers the visible-parity lock from 3,581 to 3,337 (decision 0093).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).

CI on `main` is green: every push run since #1954 succeeded or was cancelled by the next push, and none failed. The run on 7bd9090e was still queued at 05:55 UTC. The nightly fuzz run (03:51 UTC, on 27f704eb) passed.

**M2 is still current: M0–M2 probes (decision 0051) have 9 known failures, 2 of them visibly off.** Every one has an open M2 owner, checked by hand.
- `probes/t2776-ws-after-bang` S!A1 and S!C1 are the 2 visibly-off cells. Owner: T-2785 (engine).
- `probes/t2756-roundup-smallest-normal-threshold` Sheet1!A52–A58 are within 15 digits but not bit-exact. Owner: T-2771 (research).
- T-2782, T-2783 and T-2784 closed the other SPEC-0023 whitespace cells since the last run.
- The frozen M1 list has 0 known failures (89/89 cases).
- `make exit-owners MILESTONE=M2` still cannot run (no M2 exit row); T-2822 (harness) adds it. `make exit-owners` reports 0 unowned for M5 and M6.

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,951, equal to the committed baseline. The last run measured 2,931 on its merged head; neither run ratcheted it;
- oversized files: 9, the committed list (unchanged since the last run, which reported 8).

Merged since #1954: 41 commits, 52a6ac4d through 7bd9090e. None was reverted, and all came through PRs: #1953–#1958, #1960–#1983, #1985–#1993, #1995 and #1997 (#1954 is the last steward run).
- Seven were squash-merged without the `(#n)` suffix in the title: 9f27a7d4 (#1955, T-2792), 5f9cac4a (#1962, T-2295), 37de1469 (#1970, T-2699), 28617e6d (#1976, T-2096), 7fa39e82 (#1988, T-2184), 27f704eb (#1989, T-2784) and 81cba2b9 (#1995, T-2227).
- Task ids merged (39): T-0077, T-0079, T-0082, T-0483, T-1701, T-1790, T-1895, T-2074, T-2096, T-2184, T-2196, T-2227, T-2295, T-2297, T-2321, T-2337, T-2404, T-2495, T-2504, T-2505, T-2562, T-2699, T-2726, T-2732, T-2760, T-2777, T-2782, T-2783, T-2784, T-2792, T-2796, T-2812, T-2815, T-2817, T-2819, T-2823, T-2825, T-2826, T-2836.
- Task-only PR: #1997 (lands T-2813 and T-2814).

Blocked: 33 on `main` (= last run, counted by front matter); none newly blocked or unblocked there. On branches: I marked **T-2427** blocked on #1718's branch (below). T-2803 (#1938), T-2778 (#1917) and T-2506 (#1994) are blocked on their draft branches.

Open PRs: 22 at 05:55 UTC. Four non-draft PRs were older than 24 h; none met every orphan-merge condition.
- **#1718 (T-2427, FREQUENCY)**: the branch has been idle since 2026-10-25 and yesterday's reviewer-excel-fidelity REQUEST_CHANGES (two unprobed rules) is unanswered. I marked T-2427 `status: blocked` on the branch (d354c0a5), listing the probes it needs, and commented on the PR.
- **#1727 (T-2622)**: its session pushed review fixes at 05:54 UTC, so the branch is live; not touched.
- **#1811 (tasks: refill opus queue)**: another session merged `main` in and re-ran the reviewers at 05:42 UTC, then merged it (ed183978) after this run's eval. Not touched.
- **#1817 (T-2469)**: still conflicts with `main`. #1949 implements the same task from a live branch (pushed 05:54 UTC). Per decision 0050, the one that merges first survives and the other is closed after that.
- #1583, #1658, #1767, #1885, #1917, #1938, #1959 and #1994 are drafts. #1906, #1912, #1928 and #1950 are under 24 h old or were opened by live sessions; the rest were opened today.
- No task is `Blocked because: review-round limit`.

Queues:
- `make ready-tasks LANE=functions` printed 1 ready (T-2820), so the gap-triager ran. Six of the nine functions lanes are busy with a live branch and hold about 161 tasks that are ready once their lanes free up. The three free lanes (math, criteria, date) have no visibly-off cell without an owner. It filed seven tasks: **T-2861** (M2 argument-kind probes), **T-2862**–**T-2865** (M5 and M6 argument-kind probes, research), **T-2866** (generator: empty text and error literals for value parameters, harness) and **T-2867** (calipers anonymisation artifacts, the only visibly-off attributed cells of MIN, MAX and AVERAGE, research, opus). It also named unowned cells in T-0950 and T-2481.
- The gap-triager reports that `scripts/exit_owners.py` treats `.` as part of a word, so a task line like `gen-m9/<hash>.case.json (1): Calc!A7` never owns its cell (T-2731 covers this). It also reports that every calipers case shares the stem `golden.xlsx`, so a task naming `golden.xlsx` "owns" every calipers cell. No task covers that yet.
- `TIER=sonnet` printed 20 (≥ 10).
- engine, harness and research print 0 ready only because their lanes are busy (T-2338, T-0083/T-2755 and T-2681 branches moved within 2 h). Their M2 queues hold 2, 5 and 6 open tasks, so the planner did not run. api printed 2, quality 4 and perf 22.

Upstream IronCalc: 2 new commits since de6545ab, triaged through 141a41d0 (UPSTREAM.md).
- Filed **T-2868**: take the Excel-authored BASE_DECIMAL_issue_1617.xlsx (63ff3c35). Its two DECIMAL cells cache 0 for empty text.
- 141a41d0 (DECIMAL of empty text is 0) is already our rule (engineering.md, "DECIMAL of unreadable text"), so it needs no code port.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 7 pending, 0 written, 5 failures. `t2065-lift-large`, `-median` and `-small` never open a session (retries exhausted); they are T-1075 item 18 (M14). `typed-if-one-argument` (T-1990) still gets 501 `FileCorruptTryRepair`. New this run: `t0481-curly-quote-names-c` (T-0481, committed pending after a timeout) now gets the same 501, so Excel for the web refuses that workbook. The other 2 pending cases are the unfillable M10 locale cases.
- ironcalc-calc's 1 pending case is Excel-authored, and poi's 205 are web-vs-desktop conflicts (decision 0019), so neither was refilled. calipers (283 pending) and corpus-bench (592) are Excel-saved workbooks scored from their own cached values, so an oracle fill does not apply to them.

Spot-audit: the last was 2026-10-25 (#1736), so none is due. The next is on or after 2026-11-01.

**Critical lane:** engine, harness, research

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-2785 owns the 2 visibly-off M0–M2 probe cells.
- harness: T-2822, the M2 exit row that lets `make exit-owners` check M2, and T-2866.
- research: T-2771 (the 7 bit-level ROUNDUP cells), T-2867 (MIN, MAX and AVERAGE) and T-2861.
- api and quality have no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs, and so did the 81 since #1736. Update 2026-10-26 (18:47 UTC run): still none; all 52 commits since #1896 came through PRs. Update 2026-10-27: still none; all 41 commits since #1954 came through PRs.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days. Update (second 2026-10-25 run): a scheduled steward firing ran today at 18:47 UTC. SPEC-0022 notes that #1736 was probably not a scheduled run. Update 2026-10-26: the scheduled run fired again at 05:47 UTC. A second steward firing ran at 18:47 UTC. Update 2026-10-27: the 05:47 UTC run fired.
- **Steward routine out of sync with routines.md** (2026-10-26). This 18:47 UTC run's stored prompt is an older text than "Prompt — steward (daily)" in docs/process/routines.md. It lacks the progress-dashboard step (`make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and asks for per-suite numbers in docs/status.md, which decision 0118 moved to the dashboard. I followed routines.md and the `steward` skill. Also, routines.md lists one daily steward run at 05:47 UTC, but steward firings arrive at both 05:47 and 18:47. Please update the routine prompt to routines.md's text, and either delete the second schedule or add it to routines.md (agents never edit routines). Update 2026-10-27: today's 05:47 UTC firing carries the current routines.md text word for word, so only the 18:47 firing is out of date.

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
