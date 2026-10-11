# 0047 — Load formulas over 8,192 characters that Excel refuses

- **Status:** accepted (2026-10-11, T-1093)
- **Context:**
  - Excel for the web will not open a workbook with a formula of more than 8,192 stored characters
    (the text without the leading `=`). It fails with createSession 501 `FileCorruptTryRepair`. This
    is Microsoft's published limit. The evidence is T-1091 and probe `operator-chain-limit` (T-1060),
    in docs/excel-semantics/syntax.md "Formula nesting depth cap".
  - RustSheet has no limit on formula text length, so it loads such a formula and evaluates it.
  - Directive 5 does not need a text limit. `MAX_VALUE_OPERATORS` (4,096) and `MAX_FORMULA_DEPTH`
    (256) already bound the parser and evaluator, and the xlsx reader bounds the input
    (`ImportLimits`, rustsheet-xlsx `import/package.rs`).
  - Other shapes that Excel refuses on load become a parse error at parse time (`#ERROR!`):
    - non-reference SUBTOTAL and AREAS arguments (decision 0045)
    - array-constant signs (T-0147)
    - `(1)(2)` (T-0149)

    Those checks are structural, so they come back unchanged whenever a formula is parsed again.
  - A length check would not come back. The importer turns each formula into R1C1 text
    (`from_a1_to_rc`), and the model parses that text again in R1C1 mode at load, on sheet copy and
    on rename (`new_empty.rs`).
    - A parse-error node is stored as its raw A1 text. A long `1+1+...` chain parses cleanly in R1C1,
      so the refusal is lost at load.
    - R1C1 text is longer than A1 text (`R[-1]C[-1]` against `A1`), so a check in R1C1 mode would
      refuse formulas Excel accepts.
  - Making the refusal reliable would need a new parse-error state that survives the R1C1 storage,
    or the A1 length stored next to it. That is a storage design change, not an S task.
  - No Excel value can exist for such a cell, because Excel never opens the file. No suite can hold
    one, so there is nothing to score against.
- **Decision:**
  - RustSheet keeps loading and evaluating formulas longer than 8,192 characters. This is a recorded
    deviation (syntax.md, `Status: DEVIATION (decision 0047)`). Excel refuses the whole workbook.
  - The deviation covers only what the engine accepts beyond Excel's limit. Every formula Excel
    accepts still gets Excel's value.
- **Consequences:**
  - No code change. Fidelity agents do not file this gap again, and the probes that found it stay as
    the evidence.
  - Revisit this decision if the stored formula form ever keeps the A1 text or its length (for
    example an xlsx round-trip spec), or if desktop Excel turns out to load such files and give a
    value (M14, decision 0039). Either change makes the refusal cheap and observable.
  - Typing a long formula through the API is not probed, and the API accepts it as well.
