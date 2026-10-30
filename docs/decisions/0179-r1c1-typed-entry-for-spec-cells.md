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
  `Range.FormulaR1C1`, libreoffice refuses it. The engine adapter converts the text to A1 relative
  to its cell with the engine's R1C1 parser (`rustsheet_engine::r1c1_formula_to_a1`) and enters
  that with `Input::Parse`. A typed entry (either mode) that Excel refuses with 400
  `InvalidArgument` leaves the cell blank, as Excel's UI leaves it after the error dialog; the
  engine adapter leaves a typed entry the engine refuses (`EditError::InvalidInput`, or R1C1 text
  that does not parse) blank too. A probe observes a refusal through a dependant such as
  `=ISBLANK(A1)`.
- **Consequences:** Probes of R1C1-mode parsing (T-3102) can be filled and scored. A refusal is
  logged as `REFUSED` on the fill's stderr instead of failing the fill; any other Graph error still
  fails it. A typed entry the engine refuses but Excel accepts now scores as a mismatch on its
  cells instead of failing the whole case. R1C1 entry cannot hold a structured reference (the
  converter has no tables), so such a cell is left blank by the engine adapter.
