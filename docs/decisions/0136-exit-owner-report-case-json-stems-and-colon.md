# 0136 — Exit-owner report: case.json stems and colon spans own their cells

- **Status:** accepted (2026-10-27, T-2731). Amends decision 0096's "Ownership rule". Amended by decision 0159: a
  single A1 name now counts only in the block that names its case, not anywhere in the task.
- **Context:**
  - The gap triage of 2026-10-25 found two forms that tasks use to name cells and that the 0096
    rule does not read, so `scripts/exit_owners.py` lists owned cells as unowned and the planner
    files duplicate owners.
  - **The `.case.json` suffix.** The triage template writes each owned cell as a `Cells:` line,
    `- gen-m9/146b46e74f2dac3b.case.json (2): Calc!A145, A328`. 0096 bounds a stem like a file
    name (no `[\w.-]` on either side), so `146b46e74f2dac3b.case.json` does not name the stem
    `146b46e74f2dac3b`: T-2427's A328 was unowned. 74 open tasks hold 1,866 such lines.
  - **Spans.** T-2519 owns `probes/t2454-offset-one-cell-array-args A5:D5, A9:D9`, but only the
    span ends A5 and D5 were read, so its visibly-off Sheet1!B5 was unowned. Tasks also write
    hyphen spans (T-2460 `P!D49-D65`, T-2640 `C1-C21, D1-I12`).
- **Decision:**
  - A case's stem followed by `.case.json` names the case, exactly as the bare stem does; the
    pair is bounded like a file name, so `abc` still does not name `abcd.case.json`.
  - A colon span of two A1 names (`A5:D5`, `Sheet1!$A$5:$D$5`) on a line that names the case's
    stem names every cell in its rectangle, as an A1 reference does in Excel. Unlike a single A1
    name, which counts anywhere in the task, a span counts only on the stem's line: tasks quote
    formulas, and a quoted range (T-2519's `SUMXMY2(TAKE(Data!C11,-1,1),Data!B6:C10)`) is not a
    list of owned cells. A line holding a span still names cells, so it does not own the whole
    case.
  - A hyphen between two A1 names (`D49-D65`) is not a span. Hyphens join milestone and lane
    names (`M7-M9`, `M0-M2`) and case names, so reading them as spans would make tasks own cells
    they never meant; it names its two ends only, as before. The fix is to write the span with a
    colon.
  - The rest of 0096's rule is unchanged: the stem on a line with no A1-shaped token still owns
    the whole case, and a line that lists cells owns only those.
- **Consequences:**
  - 0096's "Cell ranges are not expanded" no longer holds for colon spans; `C204..C209` and
    hyphen spans are still not expanded.
  - Before/after counts are in T-2731's Outcome. Tests: `scripts/test_exit_owners.py`.
