# 0085 — How M5's exit criteria are measured

- **Status:** accepted (2026-10-20, T-2116). It defines measures and weakens no gate: no baseline,
  ratchet or CI check changes.
- **Context:**
  - M5's exit (docs/milestones.md) is "catalog coverage 100% for these categories; bit-exact
    ≥ 99.9% on generated suites (≥ 20k cells); 100% on ironcalc-calc for these categories; its
    `probes` cases (decision 0051) at 100% bit-exact". M6 says "same as M5". None of the four said
    which cells or functions count. M2's criteria were settled the same way by T-1472, decision 0058
    (T-1470) and decision 0074.
  - `make eval` at origin/main f2fa1105 (2026-10-20, eval/runs/latest/report.md; poi and
    corpus-bench not fetched, which none of these rows reads):
    - "Milestone function scope (ironcalc-calc)", M5 row (T-2111): 60,129 scored, 60,115
      bit-exact, 0 deviations, 14 failing. One is the M2 cell `LOGICAL/IF_ARRAY.xlsx`
      Sheet1!C11 (decision 0074). The 13 new at M5 are IF_ARRAY C4 and E4 (desktop SIN(4)),
      `ATAN2_POWER.xlsx` D14 and D26 (desktop ATAN2, owned by T-2142, M5), `EXP_SIGN.xlsx`
      EXP_SIGN!B44 `EXP(-745)` (T-1944, M9), and eight `TRIGONOMETRIC.xlsx` SINH/TANH/COTH/CSCH
      cells, SINH_COSH_TANH!B51, D10, D40 and COTH_CSCH_SECH!B10, B40, C51, C54, C55 (T-2134, M9).
      T-2111 had counted 53 new at M5; since then T-0915, T-0917 and T-0929 merged and 40 of
      those cells pass. T-0917 handed its eight remaining cells to T-2134 (two of them unmasked,
      decision 0084).
    - Suite row gen-m5 (T-2114): 65 cases, 22,400 scored cells, all excel-graph backed,
      22,140 bit-exact, 0 deviations, 98.84%; 260 known failures, each owned by a gap task
      (T-2114 Outcome; 19 of them by T-2134).
    - "Probe milestones (decision 0051)", M5 row: 204 cases, 8,588 scored, 8,586 bit-exact,
      0 deviations, 2 known failures, both in `probes/index-reference-args` (T-2137, M5).
    - "Catalog coverage per milestone", M5 row (T-2112): 135 in-scope functions; (a) 133 with
      ≥ 20 Excel cells; (b) 131 with a scored Excel-backed gen-m5 cell; 131 both. Missing: INFO
      (a: 0 cells, b, `oracle: none`), SHEETS (a: 1 cell, b, `oracle: none`), RAND and
      RANDBETWEEN (b, volatile). The catalog's other 9 M5 names (CALL, COPILOT, DETECTLANGUAGE,
      EUROCONVERT, PY, REGISTER.ID, STOCKHISTORY, TRANSLATE, WEBSERVICE) are `scope: out`.
- **Decision:**
  1. **Which ironcalc-calc cells count.** The cells of the cumulative M5 row of "Milestone
     function scope": every function the formula calls (an array member: its anchor's; plus the
     functions of called defined names) has catalog milestone ≤ M5, the rule T-1472 used for M2.
     The catalog milestone decides, not Microsoft's category, so LAMBDA, LET, SEQUENCE,
     RANDARRAY, REGEX*, TEXTSPLIT and the other M7 functions of the Logical, Math and Text
     categories are out, and every M2 cell is in. "100%" means the row's failing count is 0 once
     the exceptions of item 2 and item 3's time-box are left out.
  2. **IF_ARRAY C4 and E4.** Decision 0074's exception extends to them. They hold the same Mac
     desktop SIN(4) as C11, which no fitted formulation reaches without losing web cells, and
     their only open question is desktop evidence, item 11 of the M14 task T-1075. Under
     decision 0039 rule 3 that item is their filed task, so they do not block M5. They stay F
     lines in eval/baseline/ironcalc-calc.txt. ATAN2_POWER D14 and D26 get no exception here:
     T-2142 (M5) either fits them or records its own decision.
  3. **Fit tasks for M5 functions.** A failing cell of an M5 criterion is owned by a task with
     milestone M5, so the pick order (decisions 0023, 0026) reaches it while M5 is current.
     T-0915, T-0917 and T-0929 are done. T-1944 (EXP edges, B44) and T-2134 (the SINH/TANH
     kernel, eight cells here and 19 in gen-m5) move from M9 to M5 in this PR. Decision 0056
     still governs a fit: if T-2134 or another M5 fit task is blocked under its time-box and
     every cell it owns is within 4 ulp of Excel, those cells stop blocking the ironcalc-calc
     criterion. The PR that blocks the task names each such cell, and its ulp distance, under
     the criterion in docs/milestones.md, as C4, E4 and C11 are named. A cell further than 4 ulp
     off, or with a non-numeric mismatch, always blocks. Such a cell is a known failure, never a
     DEVIATION (decision 0052).
  4. **Catalog coverage.** "100%" is per function, over the M5 row of "Catalog coverage per
     milestone": every in-scope M5 function has (a) at least 20 Excel cells (decision 0062's
     floor) and (b) at least one scored Excel-backed cell in gen-m5. `scope: out` functions are
     not counted (decision 0027). Two classes satisfy (b) without a generated cell:
     - volatile functions (RAND, RANDBETWEEN), whose value cannot be scored in a generated
       suite. Their (a) cells are shape probes, such as
       `probes/volatile-and-info-rand-true-false-sheet`, and they need (a) only.
     - `oracle: none` functions (INFO, SHEETS), which still need (a) from excel-graph cells on
       the forms the canary accepts (T-2113, filled by T-2115). The bare forms `SHEETS()` and
       `INFO("numfile")` cannot be probed on the web; whatever they need goes to an M14 item,
       which counts as their filed task (decision 0039 rule 3). The function-level count does
       not wait for them.
  5. **Generated suites.** The criterion counts gen-m5 alone, read from its row in `make eval`'s
     suite table: at least 20,000 scored Excel-backed cells, and passed ÷ (scored − deviations)
     ≥ 99.9% with accepted deviations excluded (decision 0036). Today that means at most 22
     known failures among 22,400 cells. No cell is exempted: the 0.1% is the allowance.
- **Consequences:**
  - docs/milestones.md's M5 exit text cites this decision and names IF_ARRAY C4, E4 and C11 as
    the ironcalc-calc exception. The steward checks M5's exit from the four report rows above.
  - T-1944 and T-2134 are M5 tasks in `docs/tasks/M5/`; their ids and content are unchanged.
  - Open M5 exit gaps on 2026-10-20 and their owners: ironcalc-calc D14/D26 T-2142, B44 T-1944,
    the eight SINH/TANH/COTH/CSCH cells T-2134; gen-m5's 260 failures the T-2114 gap tasks (the
    criterion needs at most 22 left); probes `index-reference-args` T-2137; coverage INFO and
    SHEETS T-2115, RAND and RANDBETWEEN met by item 4.
  - M6's "same as M5" inherits these definitions with M6 in place of M5: the cumulative M6 row,
    gen-m6, the M6 probe and coverage rows. M6's own exceptions need their own record.
