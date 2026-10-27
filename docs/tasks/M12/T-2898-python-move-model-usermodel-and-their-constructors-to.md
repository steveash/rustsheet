---
id: T-2898
title: "Python: move Model, UserModel and their constructors to rustsheet.legacy with a DeprecationWarning; README on the facade (SPEC-0026)"
milestone: M12
lane: api
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2897]
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
`import rustsheet` exports only the facade (`Workbook`, `Transaction`, `Formula`, `XlError`, the errors, `open`) and
the pure helpers. `Model`, `UserModel`, `CellType`, `create`, `create_user_model`, `load_from_xlsx` and
`create_user_model_from_xlsx` move to `rustsheet.legacy`; importing it raises a `DeprecationWarning`, which says they
go at the API v1 freeze and skip the facade's load limits. The README describes the facade.

## Context
SPEC-0026 decisions 2, 6 and 7. The facade lacks what most legacy tests use (styles, undo/redo, CF, clipboard, links,
named styles, frozen panes, row and column sizes), so these classes stay until each feature is moved to the facade or
dropped (SPEC-0026's feature table). Code: bindings/python/python/rustsheet/__init__.py, a new
bindings/python/python/rustsheet/legacy.py that re-exports from `rustsheet._rustsheet`, and _ironcalc.pyi (rename it
to match the module, `_rustsheet.pyi`). Tests: conftest.py's `um`/`rm` fixtures and every test using them import from
`rustsheet.legacy` and filter the warning (pytest `filterwarnings`). The README (bindings/python/README.md) is still
IronCalc's ("pip install ironcalc"): rewrite it around `rustsheet.Workbook`, with one short section on the legacy module.
The legacy classes keep two entry points that take structured, caller-shaped data rather than indices:
`UserModel.paste_from_clipboard` and `paste_csv_string`. They go with the module at v1; until then the legacy section of
the README says the module is not hardened for untrusted input.

## Acceptance criteria
- [ ] `rustsheet.__all__` lists only the facade and the helpers (`column_name_from_number`, `column_number_from_name`,
      `quote_name`, `get_all_timezones`, `get_supported_locales`, `__version__`); a test asserts it.
- [ ] `import rustsheet.legacy` raises a `DeprecationWarning`, and every moved symbol is importable from it.
- [ ] The README names neither IronCalc nor `pip install ironcalc`, and its quick start is the facade.
- [ ] `make check` green (python job included); reviewers approve.

## Out of scope
- Removing the legacy classes (at the API v1 freeze, after SPEC-0026's feature table is cleared).
- New facade features (each is its own M12 task).
