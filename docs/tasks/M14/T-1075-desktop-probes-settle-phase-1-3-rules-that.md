---
id: T-1075
title: "Desktop probes: settle Phase 1-3 rules that rest on web-only or conflicting evidence"
milestone: M14
lane: research
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-1074]
---

## Goal
Each rule below gets a desktop Excel (current Microsoft 365, en-US) answer recorded in
docs/excel-semantics/. In Phases 1–3 these rules were implemented on excel-graph evidence, or were
left pending as CONFLICTING (decision 0039). Where desktop differs, a follow-up task exists.

## Context
Decision 0039 makes excel-graph the oracle of record for Phases 1–3. Desktop confirmation is not a
gate on those tasks; it is collected here. Probe with `rustsheet-oracle recalc -b excel-desktop` or
`fill -b excel-desktop` on the cited workbooks or on new probe cases (`excel-probe` skill).

Items (add one in the PR that implements a rule on web-only evidence, or that finds a new conflict):
1. **Unresolvable external-workbook reference** (T-0124). poi/link-external-workbook-b.xlsx
   Sheet0!A1 `=SUM('[link-external-workbook-a.xlsx]Sheet0'!A1:B1)`: excel-graph gives a vm-coded
   rich-value error that T-0030 decoded as `#BLOCKED!`. Add T-0124's cases (b) `[n]` with a cache and
   (c) no externalLink part if it probed them.
2. **POWER and `^` last digits** (T-0221). The fitted formulation, on the gen-m1 cells
   `6462bf910ef9f28f` A41 and A207 and `57fe331cb7611dbf` A54 and A104, and on T-0221's probe sweep.
3. **GEOMEAN last digit** (T-0941). ironcalc-calc/STATISTICAL/GEOMEAN.xlsx Sheet1!B27 and G27. Their
   golden is the excel-graph sidecar GEOMEAN.oracle.json, because the workbook is not
   Excel-authored.
4. **Elapsed token after a plain `s`** (T-0934 root cause 4; number-formats.md, "CONFLICTING:
   elapsed token after a plain `s`"). poi/ElapsedFormatTests.xlsx A8, A9, A18, A19, A28, A29, A38,
   A39, A48 and A49, pending today.
5. **Lowercase `e` exponent in a format** (number-formats.md, "CONFLICTING: lowercase `e` exponent
   in a format"). poi/NumberFormatApproxTests.xlsx: 238 pending cells, with cached values from Mac
   Excel 2008 (12.0).
6. **`CELL("width")`** (decision 0019; docs/architecture/oracle.md). poi/47804.xlsx: desktop cached
   10 and 11; Excel for the web returns `#VALUE!`, so the fill refuses the workbook.
7. **AND/OR/XOR text next to a logical value** (T-0258; coercion.md "AND/OR/XOR text arguments").
   poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J136 `AND("1",1)`: Excel 15 cached `#VALUE!`, excel-graph gives TRUE
   (probes/and-or-xor-text-arguments). The engine follows the web; J136 stays a known failure.
8. **Formula and defined-name length** (T-1386; syntax.md "Formulas over 8,192 characters refuse the workbook").
   Desktop Excel's limit for a formula typed into a cell and for a defined-name body, whether the web file-load
   limit (8,192 characters, ASCII; non-BMP counting unsettled, see T-1397; name bodies 8,191 ASCII characters) holds on desktop, and how `_xHHHH_` escapes inside `<f>` text count
   (the spec-to-xlsx writer cannot emit them raw, so this also needs a raw-XML fixture).
9. **`CELL("contents", ref)` reads the formula's own sheet** (T-1230; information.md "CELL(\"contents\", ref)
   on excel-graph"). probes/info-cell-contents-range: excel-graph reads `CELL("contents",Data!A2)` on Calc
   as Calc!A2 and never gives `#VALUE!` for a cross-sheet range. Desktop very likely honours the named
   sheet (a web bug?); if so, the engine goes back to reading `reference.sheet`.
10. **MODE typed text without a repeated number** (T-1291; coercion.md "MODE: a blank single-cell reference is #VALUE!", decision 0067).
   poi/FormulaEvalTestData_Copy.xlsx EverythingTests!G936 `MODE(C9,E16,"1",B8)`, H936 `MODE(B8,"1",2,3)` and K936
   `MODE("1",1,#DIV/0!,2,3)`: Excel 15 cached `#VALUE!`, excel-graph gives `#N/A`, `#N/A` and `#DIV/0!`
   (probes/mode-typed-text-no-mode A3, A4, A2). The engine follows the web; the three cells stay known failures.
11. **SIN(4) last digit** (T-1876, decision 0074; numbers.md "Excel for the web and desktop Excel differ in the last digits").
   ironcalc-calc/LOGICAL/IF_ARRAY.xlsx Sheet1!C4, E4 and C11 cache Mac 16.0300's -0.7568024953079283. The web and the engine give
   ...282, which is correctly rounded. Does Windows desktop give ...283? If it does, run a desktop SIN sweep near rounding
   midpoints to find the kernel. The engine follows the web, and the three cells stay known failures.
12. **A legacy cell after copy, move and fill** (T-1853; references.md "Legacy N and T over a reference-returning call").
   SPEC-0012 rule 6 assumes that a legacy `=N(IF(1,{2;3}))+A5` (a plain `<f>`, `#VALUE!`) stays legacy when it is copied, moved or
   filled. A row insert was confirmed on excel-graph. Graph refuses copyFrom, moveTo and autoFill
   (docs/research/t1853-legacy-cell-edits-2026-10-18.md). Do each edit on desktop, save, and check the `<f>` kind and the value.
13. **MIRR's root vs `^`** (T-0959; financial.md "MIRR's evaluation order", web vs desktop). ironcalc-calc/FINANCIAL/MIRR.xlsx
   Sheet2!N16 caches Mac 16.0300's 0.3933280241536672 for `MIRR({40,-5,-5,-5,-5,-5},0.1,0.2)`; the web gives ...674
   (probes-numeric/power-mirr-root). The engine keeps the desktop value with libm `exp` in MIRR's root while `^` follows the web's
   EXP kernel. Does Windows desktop give ...672, and does its `^` agree with its MIRR?

## Acceptance criteria
- [ ] Each item above has its desktop value(s) in docs/excel-semantics/ with the probe or workbook
      cell as evidence, and its status is VERIFIED, or DEVIATION if a decision says so.
- [ ] Where desktop agrees with the web value the engine implements, the entry says so.
- [ ] Where desktop differs, or where a pending CONFLICTING cell can now get a desktop golden, a
      task is filed (it changes the golden and the engine in its own PR; decision 0030
      "Sidecars, to do").
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing goldens or engine behavior (the follow-up tasks do that).
- Conditional formats, typed input, fill handle and structural edits (T-1051, T-1016, T-0171,
  T-0198).

## Blocked because
Phase 4 (decision 0039). It needs the excel-desktop backend (T-1074). Unblock when
`cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` and T-1074 is done.
