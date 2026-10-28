# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-10-27, 18:47 UTC run (M2 is current)

This is the scheduled 18:47 UTC run. It follows #2007, this morning's 05:47 UTC run, and covers the 64 commits merged to `main` from ed183978 (#1811) through a84da30f (#2052), the head when this run started.

`make eval` ran on a84da30f with a clean tree.
- All 17 suites were scored. calipers, corpus-bench and poi were not fetched in this container, so I ran `rustsheet-eval fetch` for them and ran `make eval` again. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowers the visible-parity lock from 3,337 to 3,254 (decision 0093).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).

CI on `main` is green: every push run since #2007 succeeded or was cancelled by the next push, and none failed. The run on a84da30f succeeded. The nightly fuzz run (03:51 UTC, on 27f704eb) passed.

**M2 is still current: M0–M2 probes (decision 0051) have 9 known failures, 2 of them visibly off.** `make exit-owners MILESTONE=M2` now runs (T-2822 added the M2 exit row) and reports 0 unowned. It also reports 0 for M5 and M6.
- `probes/t2772-unary-plus-in-range-rows` S!J2 and S!J5 are the 2 visibly-off cells. Owner: T-2885 (engine). T-2785 closed the earlier two.
- `probes/t2756-roundup-smallest-normal-threshold` Sheet1!A52–A58 are within 15 digits but not bit-exact. Owner: T-2771 (research).
- The frozen M1 list has 0 known failures (89/89 cases).

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,931, equal to the committed baseline, which #2007's run measured at 2,951 and a later PR ratcheted down;
- oversized files: 9 (= last run).

Merged since #2007's eval (7bd9090e): 64 commits, none reverted, all through PRs: #1811, #1906, #1912, #1959, #1994, #1996, #1999–#2002, #2004, #2006–#2016, #2018, #2019, #2022–#2030, #2032, #2033, #2035, #2036, #2039, #2040, #2044, #2045, #2048, #2050–#2054, #2056–#2061 and #2063–#2065. #2007 is the last steward run.
- Eight were squash-merged without the `(#n)` suffix in the title: b6b5ac36 (T-2366), ac1f85d8 (T-2552), c3d2bc1f (T-2516), cfa61d2b (T-2719), 9969ef5f (T-2299), 9786b0a6 (T-2403), 42ad9f86 (T-2358) and 401a121c (T-2344).
- Task ids merged (55): T-0083, T-0465, T-0470, T-0471, T-0476, T-0485, T-1737, T-2173, T-2299, T-2338, T-2339, T-2344, T-2358, T-2366, T-2403, T-2434, T-2506, T-2516, T-2522, T-2527, T-2548, T-2552, T-2572, T-2635, T-2681, T-2719, T-2729, T-2753, T-2755, T-2762, T-2764, T-2772, T-2785, T-2799, T-2813, T-2814, T-2820, T-2822, T-2829, T-2837, T-2840, T-2841, T-2849, T-2866, T-2867, T-2868, T-2879, T-2882, T-2884, T-2912, T-2918, T-2920, T-2921, T-2923, T-2927.
- Task-only PRs: #1811, #2011, #2016, #2028, #2030, #2035, #2054 and #2056.

Blocked: 34 on `main` (33 last run). T-2548 is newly blocked (on the BESSELJ probe sweep, T-2881); none was unblocked. T-2803 (#1938), T-2778 (#1917), T-2657 (#1885), T-0972 (#1767), T-2131 (#1658) and T-2398 (#1583) are blocked on their draft branches. No task is `Blocked because: review-round limit`.

Open PRs: 29 at 18:50 UTC. This run closed three duplicates (#2046, #1950, #1817), merged one orphan (#1949) and merged `main` into four conflicted PRs (#1718, #1727, #1928; #1949 under the lock). Non-draft PRs older than 24 h:
- **#1949 (T-2469, T.TEST text samples)**: met every orphan-merge condition. Green CI, idle since 05:45 UTC, task done, and all four reviewers APPROVE on its current diff (MINOR only). It was waiting on the merge lock, which other sessions held all run. I merged `main` in under the merge lock and squash-merged it once CI was green on that head (cc64bd91).
- **#1817 (T-2469)**: the older duplicate of #1949, idle since 2026-10-25 and conflicting with `main`. Closed after #1949 merged (decision 0050).
- **#1718 (T-2427, FREQUENCY)**: conflicted with `main` on baselines only. I merged `main` in and re-ran `make eval-baseline` (a2ffb8c7: gen-m9 −155 and probes −5 known failures, 0 regressions; `make check` green). All four reviewers APPROVE the current diff (MINOR only), and yesterday's REQUEST_CHANGES is answered by its 06:14 UTC commits. CI was green on that head. When I took the lock, #1949 had just changed the same T.TEST entry in statistics.md, so I released the lock, merged `main` in again (af6f354e, `make check` green) and re-ran the reviewers. reviewer-excel-fidelity now requests doc changes: the T.TEST notes contradict `main`'s order, and three unprobed FREQUENCY skips have no owning probe task. Not merged; the findings are on the PR.
- **#1928 (T-2786, whitespace probes)**: conflicted in syntax.md and the probes baseline. I merged `main` in (ca95a297), kept both sides' findings, and renumbered its T-2799 to **T-2944** (`main` holds T-2799). `make check` is green; it needs CI and a review pass.
- **#1727 (T-2622)**: conflicted in `sumproduct.rs` (`main`'s new factor walk against the branch's rename of `lambda_as_number_error` to `lambda_value_error`). I merged `main` in (a044bd1d), keeping `main`'s walk with the rename; the baseline did not change and `make check` is green. It needs CI and a review pass.
- **#1950 (T-2072)**: closed. T-2072 was finished on `main` by T-2522 (#2040), and the branch conflicted. Its unmerged probe task T-2818 lived only on that branch; T-2522's comma factor covers most of it.
- **#2046 (T-1737)**: closed as a duplicate of #1912, which merged.
- #1583, #1658, #1767, #1885, #1917, #1938 and #2055 are drafts. The rest were opened in the last 24 h.

Queues:
- `make ready-tasks LANE=functions` printed 0 ready, so the gap-triager ran. Stat, text, engineering, financial and lookup are busy with live branches. The idle lanes (math, criteria, date, logic) hold only `defer: phase-1b` tasks or tasks waiting on dependencies. Every visibly-off root cause it found already has an open task except one, so it filed **T-2945** (TEXT of text with an empty fourth section, `;;;`, functions-text, sonnet). It made **T-2636** own corpus-bench/53062's 334 visibly-off cells (FREQUENCY bins), and filed **T-0084** (harness: `make new-task` breaks on a TITLE containing `"`). It found that T-2779 owns 147 of TEXT's 185 visibly-off cells.
- The gap-triager confirms that `scripts/exit_owners.py` overstates unowned cells (`.case.json` stems and A1 spans: T-2731, open as #2069; calipers `golden.xlsx`: T-2880; corpus-bench directories: T-2909).
- `TIER=sonnet` printed 15 (≥ 10).
- engine, harness and research print 0 ready only because their lanes are busy. Their M2 queues hold 3, 3 and 6 todo tasks, so the planner did not run. api printed 9, quality 6 and perf 0 (Phase 2).

Upstream IronCalc: 3 new commits in base, xlsx or bindings since 141a41d0. I triaged through e757b4f1 (UPSTREAM.md).
- Filed **T-0490** (M11, engine): port b41cf6d5 and its test fix 7e9304a4. An xlsx whose styles.xml lacks `<cellXfs>`, or has empty font, fill or border lists, should load with default styles if Excel opens such files (probe first). Ours already reads the other lists as optional (T-0118) but refuses a missing `<cellXfs>` with a typed error.
- e757b4f1 (trailing tokens after a formula are a parse error) is already our behaviour (`trailing_input_is_a_parse_error`). Upstream's test also rejects `A1 A2`, which Excel reads as an intersection and we keep. No port.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 7 pending, 0 written, 5 failures, the same as #2007. `t2065-lift-large`, `-median` and `-small` are T-1075 item 18 (M14). `typed-if-one-argument` (T-1990) and `t0481-curly-quote-names-c` (T-0481) are refused by Excel for the web. The other 2 are the unfillable M10 locale cases.
- ironcalc-calc's 1 pending case is Excel-authored, and poi's 205 are web-vs-desktop conflicts (decision 0019), so neither was refilled. calipers (283 pending) and corpus-bench (592) are scored from their own cached values.

Spot-audit: the last was 2026-10-25 (#1736), so none is due. The next is on or after 2026-11-01.

**Critical lane:** engine, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine: T-2885 owns the 2 visibly-off M0–M2 probe cells.
- research: T-2771 owns the other 7 M0–M2 probe cells (ROUNDUP's subnormal cut). T-2910 audits M2's structural exit criterion (#2067 is open).
- harness: T-2731 (#2069) and T-2909, which make the exit-owner report trustworthy.
- api and quality have no M2 work. perf stays out: Phase 2.)

## Needs owner

- **Direct pushes to `main`** (2026-10-18, continuing 2026-10-19). Session `01Xbmds2…` has now pushed fifteen T-1778 commits straight to `main` with no PR (6f85cc9 … 892a53ca reported last run; fc3f993d, c00142bb, 0b5ce7a4, 36217c63, 756f849f, 37d303dd, 32ac084d and c763916b since), so the reviewer agents never saw them. CI passed on each and nothing needs reverting. New this run: **32ac084d edits a protected path, `.claude/skills/excel-probe/SKILL.md`**, without the decision record the governance gate demands of a PR. It is a two-line pointer to the new `docs/research/excel-kernel-playbook.md`, and both read as benign research guidance. Still, agent instructions changed with no review. Recommended fix: a branch ruleset on `main` that requires a pull request (the steward and workers already merge through PRs, so nothing changes for them). If these pushes were your own interactive work, remove this item. Update 2026-10-20: no direct push to `main` since the last run; all 109 commits came through PRs. Update 2026-10-25: still none; all 221 commits since 2026-10-22 came through PRs, and so did the 81 since #1736. Update 2026-10-26 (18:47 UTC run): still none; all 52 commits since #1896 came through PRs. Update 2026-10-27: still none; all 41 commits since #1954 came through PRs. Update 2026-10-27 (18:47 UTC run): still none; all 64 commits since #2007's eval were merged through GitHub.
- **Missed steward runs** (2026-10-25). There is no steward snapshot or PR for 2026-10-23 or 2026-10-24, so this snapshot covers three days. Please check that the daily steward routine fired on those days. Update (second 2026-10-25 run): a scheduled steward firing ran today at 18:47 UTC. SPEC-0022 notes that #1736 was probably not a scheduled run. Update 2026-10-26: the scheduled run fired again at 05:47 UTC. A second steward firing ran at 18:47 UTC. Update 2026-10-27: the 05:47 UTC run fired. The 18:47 UTC run fired too.
- **Steward routine out of sync with routines.md** (2026-10-26). This 18:47 UTC run's stored prompt is an older text than "Prompt — steward (daily)" in docs/process/routines.md. It lacks the progress-dashboard step (`make eval-lower-visible-lock`, `make progress-report`, the `progress-report` skill) and asks for per-suite numbers in docs/status.md, which decision 0118 moved to the dashboard. I followed routines.md and the `steward` skill. Also, routines.md lists one daily steward run at 05:47 UTC, but steward firings arrive at both 05:47 and 18:47. Please update the routine prompt to routines.md's text, and either delete the second schedule or add it to routines.md (agents never edit routines). Update 2026-10-27: today's 05:47 UTC firing carries the current routines.md text word for word, so only the 18:47 firing is out of date. Update (18:47 UTC run): this firing now carries the routines.md text word for word as well. What remains is that routines.md lists one daily steward run, but the steward fires at 05:47 and 18:47 UTC: delete the second schedule or add it to routines.md.

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
