# 0179 — R1C1 typed entry for spec cells

- **Status:** accepted (2026-10-30, T-3113)
- **Context:** A probe could type text into a cell only in A1 mode (`{"entered": ...}`, a Graph
  `values` PATCH, T-0280), so how Excel reads a formula typed in R1C1 mode (T-3102: whitespace
  after `!` in `=S! R1C2`) could not be asked. Excel also refuses some typed text outright (Graph
  PATCH 400 `InvalidArgument`, as `=S! R1C2` gets in A1 mode, T-2869); that refusal is an answer,
  but the backend raised it as a fill failure, so no case could keep it.
- **Decision:** A spec cell `{"entered_r1c1": "=R1C2+1"}` is typed in R1C1 mode. spec2xlsx
  records it like an `entered` cell, in `rustsheet.entered.<n>` with a fourth JSON element
  `"r1c1"` (A1 entries keep their three elements, so existing workbooks stay byte-identical);
  excel-graph writes it with a range `formulasR1C1` PATCH, excel-desktop with
  `Range.FormulaR1C1` (UNVERIFIED: no desktop run), libreoffice refuses it. The engine adapter converts the text to A1 relative
  to its cell with the engine's R1C1 parser (`rustsheet_engine::r1c1_formula_to_a1`) and enters
  that with `Input::Parse`. A typed entry (either mode) that Excel for the web refuses with
  400 `InvalidArgument` leaves the cell with its prior content, blank in a spec case (only the Graph
  refusal is observed; what the desktop UI keeps is UNVERIFIED, and excel-desktop still fails the
  fill on a refusal's COM error), and
  R1C1 text the engine's R1C1 reader cannot parse is left blank by the engine adapter. A probe
  observes a refusal through a dependant such as `=ISBLANK(A1)`. A typed entry the engine itself
  refuses (`EditError::InvalidInput`) still fails the case, as before: the case-level refusal
  deviations (nesting and length limits, directive 5) are matched on that error.
- **Consequences:** Probes of R1C1-mode parsing (T-3102) can be filled and scored. A refusal is
  logged as `REFUSED` on the fill's stderr instead of failing the fill; any other Graph error still
  fails it. A typed formula Excel refuses that the engine stores as a parse error scores as a
  mismatch (T-3213). R1C1 entry cannot hold a structured reference (the converter has no tables),
  so such a cell is left blank by the engine adapter; so is R1C1 text past the parser's nesting or
  length limits and the bare `R`/`C` offsets the engine does not read yet, which Excel may accept
  (T-3212 probes them and separates "refused" from "unsupported").
