# 0103 — Typed formulas refuse cell-shaped LET and LAMBDA names; Input::StoredFormula keeps the stored reading

- **Status:** accepted (2026-10-23, T-2497)
- **Context:** Excel for the web refuses a typed formula whose LET name or LAMBDA parameter is
  spelled like a cell (`=LET(f1,1,f1+1)`, `=LAMBDA(f1,f1*2)(3)`; Graph PATCH 400, T-2040), yet
  binds the same names stored as `_xlpm.f1` (probes/let-cell-shaped-name-stored). The engine's
  parser binds the bare spelling too, because the eval harness sets case formulas unprefixed
  through `Input::Formula` while spec2xlsx stores them with `_xlpm.` names, and the parser also
  re-parses stored formulas after structural edits. A refusal in the parser itself would break
  both; porting spec2xlsx's `prefix_bound_names` into the harness would duplicate it.
- **Decision:** The parser keeps binding a cell-shaped name everywhere, and reports that it did.
  Only typed entry refuses it: `Input::Formula` and `Input::Parse` through the API (engine
  `Model::set_typed_user_input`) fail with `EditError::InvalidInput`, the API's refusal. A new
  `Input::StoredFormula` reads a formula as an xlsx file stores it (cell-shaped names bind, as
  import does), and the harness sets case formulas with it; cases that model typing use
  `{"entered": ...}` (`Input::Parse`).
- **Consequences:** API users typing such a formula get an error, as Excel's users do. Tools that
  replay stored formulas through the API use `Input::StoredFormula` (or `LegacyFormula`). The Python
  bindings' formula input stays typed (`Input::Formula`); a stored-formula path is not mirrored there
  until a caller needs it. Other typed-only refusals (T-1856) can reuse the same "typed" path.
