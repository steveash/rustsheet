# 0047 — Refuse workbooks with formulas over 8,192 characters on import

- **Status:** accepted (2026-10-11, T-1093)
- **Context:**
  - Excel for the web will not open a workbook with a formula of more than 8,192 stored characters
    (the `<f>` text, without the leading `=`). It fails with createSession 501 `FileCorruptTryRepair`.
    It does open 8,192. This is Microsoft's published limit. The evidence is T-1091 and probe
    `operator-chain-limit` (T-1060), in docs/excel-semantics/syntax.md.
  - RustSheet had no limit on formula text length.
  - Other shapes Excel refuses on load become a cell-level parse error (`#ERROR!`): non-reference
    SUBTOTAL and AREAS arguments (decision 0045), array-constant signs (T-0147) and `(1)(2)` (T-0149).
    Those checks are structural, so they come back each time the formula is parsed again.
  - A length check in the parser would not come back:
    - The importer stores each formula as R1C1 text (`from_a1_to_rc`), and the model parses that text
      again in R1C1 mode at load, on sheet copy and on rename.
    - A parse-error node is stored as its raw text, and a long `1+1+...` chain parses cleanly in
      R1C1 mode.
    - R1C1 text is longer than A1 text, so a check there would refuse formulas Excel accepts.
  - The xlsx importer sees the stored `<f>` text before any conversion (`sheet_data.rs`), and it
    already refuses whole workbooks (`XlsxError::Limit`, `XlsxError::Workbook`).
- **Decision:**
  - The xlsx importer refuses the whole workbook (`XlsxError::Workbook`) when a stored formula has more
    than 8,192 characters. This is the observable behavior of Excel itself, so it is not a deviation.
  - Characters are counted as Unicode scalar values. Whether Excel counts UTF-16 code units is not
    probed, and counting scalars is never stricter.
  - The cell-level parse-error mechanism is not used for this check.
- **Consequences:**
  - A workbook Excel cannot open no longer loads in RustSheet. No upstream test workbook or eval
    suite holds a formula over 393 characters, so nothing that used to load is lost.
  - Formulas typed through the API and defined names are not limited. Their Excel limits are not
    probed. The probe is follow-up T-1386; implement any API limit only once that evidence exists.
  - Desktop Excel's behavior on load is an M14 question (decision 0039).
