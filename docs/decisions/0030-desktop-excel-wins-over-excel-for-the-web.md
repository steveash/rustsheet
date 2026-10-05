# 0030 — Desktop Excel wins over Excel for the web when evidence disagrees

- **Status:** accepted (2026-10-05, T-0025)
- **Context:**
  - Our golden source is the excel-graph oracle, which is Excel for the web. The other evidence
    is values that desktop Excel cached in the workbooks it saved (ironcalc-calc, poi), plus a
    future excel-desktop backend. Both are "Microsoft 365 Excel", but they disagree in places.
  - Calibration (docs/research/oracle-calibration.md, T-0004: 71,765 cells, 99.96% bit-exact)
    found two kinds of disagreement:
    1. **Behavior.** INDIRECT to a missing sheet is #REF! on desktop but a reference to an empty
       cell on the web (docs/excel-semantics/lookup.md). POI adds lowercase-`e` TEXT formats,
       elapsed tokens and `CELL("width")` (decision 0019).
    2. **Last-digit floating point.** In ATAN2, IM*, ERFC, BESSEL, MIRR, IRR and SIN, results
       differ by up to 4 ULPs or by a near-zero residue (docs/excel-semantics/numbers.md).
  - Calibration also showed that some desktop evidence is not en-US. The ironcalc workbooks were
    saved by a day-first Excel, so their cached values for date text are wrong for en-US.
  - Decision 0019 already leaves cells pending where the web contradicts a cached desktop value
    by more than 1e-9 relative. It explicitly left two questions open: who wins in general, and
    what happens to differences below 1e-9.
  - Until now the docs simply preferred desktop wherever the engine already followed it, with
    nothing recorded.
- **Options considered:**
  - *Desktop wins.* Users mostly run desktop Excel, and workbooks are saved by it. Our only
    automated oracle is the web, so desktop evidence is scarce: it is limited to cached values
    and probes on a backend we do not have yet.
  - *Web wins.* This is simple and fully automatable. But it would make the engine copy web-only
    behavior, such as INDIRECT to a missing sheet, that desktop users never see. It would also
    turn the 4 INDIRECT cells (lookup.md) and the 238 lowercase-`e` TEXT cells (decision 0019,
    number-formats.md), where the engine follows desktop, into targets for "fixing".
  - *Accept either* for the listed divergences. This needs a two-golden scoring mode in the
    harness. For behavior it is wrong: one formula cannot be both #REF! and 0.
- **Decision:**
  1. **Desktop Microsoft 365 Excel (en-US, current channel) is the reference.** When desktop
     and web evidence for the same formula disagree, desktop decides. That applies to the
     engine, to the semantics entries, and to which golden a cell is scored against.
  2. **Desktop evidence counts only for what it is.** Cached values from a workbook saved under a
     non-en-US locale are not evidence for locale-dependent behavior: date or number text, list
     separators, month and day names. There the en-US web oracle wins. An older desktop build
     does not outrank current web behavior that a current excel-desktop probe has not checked:
     record such a conflict as `CONFLICTING` (decision 0019).
  3. **Web-only evidence is golden.** If no desktop evidence exists, oracle values are goldens
     and agents cite them as Excel for the web. Exception: for a behavior listed as a known
     divergence (docs/architecture/oracle.md, "Known divergences"), web values are never
     evidence. That covers references to missing sheets, lowercase-`e` formats, elapsed tokens
     after `s`, and `CELL("width")`.
  4. **Last-digit divergences.** For the functions in numbers.md "Excel for the web and desktop
     Excel differ in the last digits", an engine fit targets desktop cached values where they
     exist. A web golden for those functions that the fitted engine misses by at most 4 ULPs, or
     only by a near-zero residue below 1e-15 absolute, stays a scored known failure in the
     baseline. It is not a target for further fitting, and it is not a regression signal. The
     harness keeps scoring strictly, because accepting either value would need two goldens per
     cell; this rule only says which value to fit. The 4-ULP and 1e-15 bands are the widest
     differences calibration measured (11 cells within 4 ULPs, residues around 2.2e-16).
- **Consequences:**
  - *Sidecars.* `rustsheet-oracle fill` keeps decision 0019's rule unchanged: a cell where the
    web contradicts a cached desktop value by more than 1e-9 relative stays pending. Smaller
    differences are written as web goldens, but suites that trust cached values score the
    desktop value, so for those suites desktop already wins (rule 1). When the excel-desktop
    backend exists, its values replace web sidecar values that disagree with them, and those
    replacements are not regressions.
  - *Baselines.* A cell that becomes a known failure because the engine follows desktop instead
    of web, under rule 1 or rule 4, is recorded with `make eval-baseline` like any other known
    failure, and the PR cites this decision. That is not a regression allowance, because the
    cell never passed against desktop evidence. The PR must cite that desktop evidence (an
    Excel-authored workbook cell or an excel-desktop probe). `make eval-baseline` still refuses
    regressions: a cell that passes in the current baseline and starts failing needs a decision
    record like any other regression allowance (CLAUDE.md directive 2).
  - *Probes.* A probe whose answer could fall in a known divergence is not answered by the web
    alone. Its entry stays `UNVERIFIED` or `CONFLICTING` until desktop evidence exists: an
    Excel-authored workbook in the repo, or the excel-desktop backend.
  - *Docs.* docs/architecture/oracle.md ("Known divergences"), docs/excel-semantics/README.md,
    lookup.md (INDIRECT to a missing sheet) and numbers.md (last-digit divergences) cite this
    record. Every new known divergence is added to oracle.md's list in the PR that finds it.
  - *Not changed.* Decision 0019's pending mechanism, the eval comparison rules, and the
    priority of directive 5 deviations (decision 0014).
