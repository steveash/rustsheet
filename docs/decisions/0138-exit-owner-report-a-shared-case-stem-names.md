# 0138 — Exit-owner report: a shared case stem names no case

- **Status:** accepted (2026-10-27, T-2880). Amends decision 0096's "Ownership rule" (as amended
  by 0136).
- **Context:**
  - 0096 identifies a case by its file stem, the last segment of its id. On a report with
    calipers fetched (origin/main 5cadaf05), 70 stems are shared by 603 cases: all 456 calipers
    cases are `calipers/<set>/<dir>/golden.xlsx`, and 69 gen-m9 hashes are rerun as
    `probes/t2429/<hash>` or `probes/t2436/<hash>`.
  - A task that names one of those cases therefore also owned the same A1 cell (or, on a line
    with no A1 token, every failing cell) of all the others, so `make exit-owners` and
    `--unattributed` hid unowned failures (T-2880 measured 6 such visibly-off cells; T-2704 and
    T-2867 owned pv26_011's and pv26_2_011's cells by naming other workbooks).
- **Decision:**
  - A stem that only one case of the report has names that case, as before. The report's cases
    are its `cases` list plus the cells being checked.
  - A stem that several cases of the report share names none of them, bare or with
    `.case.json`; a bare gen-m9 `<hash>` that a probe copy shares no longer names the gen-m9
    case. Such a case is named by:
    - any suffix of its id of two segments or more (less `.case.json`) that no other case of the
      report has: `pv26_011/golden.xlsx`, `t2436/<hash>`, `gen-m9/<hash>`, or the full id; and
    - its directory, the second-to-last segment, when it is below the suite (the id has three
      segments or more) and no other case of the report is in a directory of that name: the
      calipers form `pv26_011`. A suite name never names a case.
  - These names take the stem's place in every other part of the rule: the A1 name anywhere in
    the task, a colon span on a line that names the case, and a line that names the case with no
    A1 token owning all of it. Names are bounded like file names, as before.
- **Consequences:**
  - Gap-triage `Cells:` lines already write `gen-m9/<hash>.case.json` and calipers tasks write
    the directory or full id, so both keep their ownership; a task that writes only a shared
    file name or hash owns nothing and should write the directory or suite too.
  - Before/after counts are in T-2880's Outcome. Tests: `scripts/test_exit_owners.py`
    (`SharedStems`).
