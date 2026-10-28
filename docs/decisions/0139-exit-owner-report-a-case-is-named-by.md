# 0139 — Exit-owner report: a case is named by its directory path

- **Status:** accepted (2026-10-27, T-2909). Amends decision 0096's "Ownership rule" (as amended
  by 0136 and 0138).
- **Context:**
  - Under 0138 a case whose file stem no other case has is named only by that stem. corpus-bench
    case ids are `corpus-bench/<n>/1_<n>_input.xlsx`, but tasks name the case by its directory,
    `corpus-bench/<n>` (T-2636, T-0478, T-0484, T-1089, T-0950). So T-2636's
    `corpus-bench/53062` owned none of that case's cells, and `make unowned-off` listed 220 of
    them as unowned (T-2909).
  - T-0484 ends its sentence with the name (`corpus-bench/55049.`), and a `.` is a file-name
    character, so that name was not bounded either.
- **Decision:**
  - A case whose stem no other case of the report has, and whose id has three segments or more
    (less `.case.json`), is also named by its directory path, the id less its last segment, suite
    included (`corpus-bench/53062`), when no other case of the report is under that path and no
    case has it as its id. A shorter path (`corpus-bench/5306`) names nothing, since names stay
    bounded like file names. Shared-stem cases keep 0138's names (its bare directory already
    matches inside such a path).
  - A name may end a sentence: a `.` that is not followed by a file-name character does not
    extend it, for every case name (`corpus-bench/55049.`, `TRIGONOMETRIC.xlsx.`).
    `<hash>.case.json.bak` and `<stem>.xlsx` still do not name the shorter stem.
  - The new names take part in every other part of the rule as the stem does: an A1 name in the
    task, a colon span on a line that names the case, and a line naming the case with no A1 token.
- **Consequences:**
  - Tasks that write `corpus-bench/<n>` own that case; those that write the full id keep owning
    it. Before/after counts are in T-2909's Outcome. Tests: `scripts/test_exit_owners.py`
    (`DirectoryPaths`).
