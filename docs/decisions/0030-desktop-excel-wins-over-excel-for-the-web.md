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
       differ by up to 4 ULPs (11 cells), within 1e-9 relative (4 cells) or by a near-zero
       residue around 2e-16 (3 cells) (docs/excel-semantics/numbers.md).
  - Calibration also showed that some desktop evidence is not en-US. The ironcalc workbooks were
    saved by a day-first Excel, so their cached values for date text are wrong for en-US. Much of
    the desktop evidence is Mac Excel (ironcalc: Microsoft Macintosh Excel, AppVersion 16.x;
    poi NumberFormatApproxTests: Mac Excel 2008, 12.0).
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
    turn the 4 INDIRECT cells (lookup.md), which pass today against desktop cached values, into
    targets for "fixing".
  - *Accept either* for the listed divergences. This needs a two-golden scoring mode in the
    harness. For behavior it is wrong: one formula cannot be both #REF! and 0.
- **Decision:**
  1. **Desktop Microsoft 365 Excel (en-US, current channel) is the reference.** When desktop
     and web evidence for the same formula disagree, desktop decides. That applies to the
     engine, to the semantics entries, and to which golden a cell is scored against. Windows and
     Mac count alike: "desktop" is any Microsoft Excel application whose `docProps/app.xml`
     AppVersion is 16.x (the Microsoft 365 / 2016+ line).
     *Amended by decision 0039: in Phases 1–3 a conflict that needs new desktop evidence matches
     the web value; desktop evidence already in the repo that cells pass against stays.*
     *Refined by decision 0073: in a suite with `use_cached_values = false` a cached value never
     replaces a web number; its caches are not desktop evidence.*
  2. **Desktop evidence counts only for what it is.** Cached values from a workbook saved under a
     non-en-US locale are not evidence for locale-dependent behavior: date or number text, list
     separators, month and day names. There the en-US web oracle wins. An older desktop build
     (AppVersion below 16, e.g. Mac Excel 2008 = 12.0) does not outrank current web behavior:
     record such a conflict as `CONFLICTING` (decision 0019) until an excel-desktop probe or a
     16.x workbook decides. That is the status of the lowercase-`e` TEXT formats (238 cells,
     pending and unscored; number-formats.md).
  3. **Web-only evidence is golden.** If no desktop evidence exists, oracle values are goldens
     and agents cite them as Excel for the web. Exception: for a behavior listed as a known
     divergence (docs/architecture/oracle.md, "Known divergences"), web values are never
     evidence. That covers references to missing sheets, lowercase-`e` formats, elapsed tokens
     after `s`, and `CELL("width")`.
     *Amended by decision 0039: in Phases 1–3 this exception applies only where desktop evidence
     is already in the repo; otherwise the web value is the target and the desktop check is an
     acceptance item on an M14 task (T-1075).*
  4. **Last-digit divergences.** For the functions in numbers.md "Excel for the web and desktop
     Excel differ in the last digits", an engine fit targets desktop cached values where they
     exist. A web golden for those functions that is already a known failure and that the
     desktop-fitted engine misses within 1e-9 relative, or only by a near-zero residue below
     1e-15 absolute, is not a target for further fitting. These bands cover every last-digit
     cell calibration measured (11 within 4 ULPs, 4 within 1e-9 relative, 3 residues around
     2e-16). The harness keeps scoring strictly, because accepting either value would need two
     goldens per cell; this rule only says which value to fit. It never turns a passing cell
     into an accepted failure (see Baselines).
- **Consequences:**
  - *Sidecars, today.* A sidecar replaces a workbook's cached values entirely
    (`prepare_xlsx` in crates/rustsheet-eval/src/runner.rs). `rustsheet-oracle fill` leaves a
    cell pending when the web contradicts a cached desktop value by more than 1e-9 relative
    (decision 0019), but writes the web value when the difference is smaller. So in sidecar-backed
    workbooks (poi, any `oracle_dir` suite) last-digit differences are scored against the web,
    against rule 1. Workbooks without a sidecar (ironcalc-calc) score the desktop cached value.
  - *Sidecars, to do.* T-0142 makes the fill keep the desktop cached value wherever a web number
    differs at all from it. When the excel-desktop backend exists, its values likewise replace
    web sidecar values that disagree with them. Both are changes of golden, made in their own PR
    with the evidence listed per cell.
  - *Baselines.* This record authorizes no regression allowance. `make eval-baseline` still
    refuses regressions, and `--allow-regressions` still needs its own decision record (CLAUDE.md
    directive 2). If following desktop would turn a cell that passes against a web golden into a
    failure, first replace that golden with the desktop value (T-0142 or the desktop backend),
    then fit the engine. A cell that newly enters the baseline as a known failure (new coverage)
    under rule 1 or rule 4 is recorded with `make eval-baseline` as usual, and the PR cites the
    desktop evidence (an Excel-authored workbook cell or an excel-desktop probe).
  - *Probes.* A probe whose answer could fall in a known divergence is not answered by the web
    alone. Its entry stays `UNVERIFIED` or `CONFLICTING` until desktop evidence exists: an
    Excel-authored workbook in the repo, or the excel-desktop backend.
  - *Docs.* docs/architecture/oracle.md ("Known divergences"), docs/excel-semantics/README.md,
    lookup.md (INDIRECT to a missing sheet) and numbers.md (last-digit divergences) cite this
    record. Every new known divergence is added to oracle.md's list in the PR that finds it.
  - *Not changed.* Decision 0019's pending mechanism, the eval comparison rules, and the
    priority of directive 5 deviations (decision 0014).
